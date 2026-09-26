//! Cancellable quota waits for an active sampling turn. The caller retains the request history.

use super::session::Session;
use super::turn_context::TurnContext;
use chrono::DateTime;
use chrono::Utc;
use codex_protocol::error::CodexErr;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::UsageLimitWaitEvent;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::info;

/// Wait until the advertised reset plus a margin for delayed backend quota updates.
/// Emit a matching end event before returning, including when the turn is interrupted.
#[tracing::instrument(skip_all, fields(turn_id = %turn_context.sub_id, %resets_at))]
pub(super) async fn wait_for_usage_limit_reset(
    sess: &Session,
    turn_context: &TurnContext,
    resets_at: DateTime<Utc>,
    cancellation_token: &CancellationToken,
) -> Result<(), CodexErr> {
    // A stale reset still waits five seconds, preventing a tight loop of quota errors.
    let now = Utc::now();
    let wait_duration = resets_at
        .signed_duration_since(now)
        .to_std()
        .unwrap_or_default()
        .saturating_add(Duration::from_secs(/*secs*/ 5));
    let retry_at_ms = now
        .timestamp_millis()
        .saturating_add(i64::try_from(wait_duration.as_millis()).unwrap_or(i64::MAX));
    sess.send_event(
        turn_context,
        EventMsg::UsageLimitWaitStarted(UsageLimitWaitEvent { retry_at_ms }),
    )
    .await;
    info!(
        wait_seconds = wait_duration.as_secs(),
        "Waiting for usage limit reset"
    );
    let result = tokio::select! {
        biased;
        _ = cancellation_token.cancelled() => {
            info!("Usage limit wait cancelled");
            Err(CodexErr::TurnAborted)
        }
        _ = tokio::time::sleep(wait_duration) => {
            info!("Usage limit reset wait complete; retrying sampling request");
            Ok(())
        }
    };
    // End the countdown before retrying so the UI starts a fresh working timer.
    sess.send_event(turn_context, EventMsg::UsageLimitWaitEnded)
        .await;
    result
}
