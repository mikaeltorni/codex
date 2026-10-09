use super::AppServerSession;
use super::JSONRPC_INVALID_REQUEST;
use super::JSONRPC_METHOD_NOT_FOUND;
use codex_app_server_client::TypedRequestError;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::TurnSettingsUpdateParams;
use codex_app_server_protocol::TurnSettingsUpdateResponse;
use codex_app_server_protocol::TurnSettingsUpdateStatus;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;

impl AppServerSession {
    pub(crate) async fn turn_settings_update(
        &mut self,
        params: TurnSettingsUpdateParams,
    ) -> Result<bool> {
        let request_id = self.next_request_id();
        match self
            .client
            .request_typed::<TurnSettingsUpdateResponse>(ClientRequest::TurnSettingsUpdate {
                request_id,
                params,
            })
            .await
        {
            Ok(response) => Ok(response.status == TurnSettingsUpdateStatus::Applied),
            Err(TypedRequestError::Server { source, .. })
                if source.code == JSONRPC_METHOD_NOT_FOUND
                    || (source.code == JSONRPC_INVALID_REQUEST
                        && source.message.contains("turn/settings/update")) =>
            {
                Ok(false)
            }
            Err(err) => Err(err).wrap_err("turn/settings/update failed in TUI"),
        }
    }
}
