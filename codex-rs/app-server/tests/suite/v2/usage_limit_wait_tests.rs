//! Exercise quota wait notifications and cancellation through the public app-server API.

use anyhow::Result;
use app_test_support::MockResponsesConfig;
use app_test_support::TestAppServer;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::TurnCompletedNotification;
use codex_app_server_protocol::TurnInterruptParams;
use codex_app_server_protocol::TurnInterruptResponse;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::TurnStatus;
use codex_app_server_protocol::UsageLimitWaitChangedNotification;
use codex_app_server_protocol::UserInput;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

#[tokio::test]
async fn quota_wait_notifications_end_before_interrupted_turn_completes() -> Result<()> {
    let server = MockServer::start().await;
    let resets_at = chrono::Utc::now().timestamp() + 3600;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(/*s*/ 429).set_body_json(json!({
            "error": {
                "type": "usage_limit_reached",
                "resets_at": resets_at,
                "plan_type": "pro"
            }
        })))
        .expect(/*r*/ 1)
        .mount(&server)
        .await;
    let home = TempDir::new()?;
    MockResponsesConfig::new(&server.uri())
        .with_root_config("auto_resume_on_usage_limit = true")
        .write(home.path())?;
    let mut app = TestAppServer::builder()
        .with_codex_home(home.path())
        .build_initialized()
        .await?;
    let thread = app.start_thread(ThreadStartParams::default()).await?.thread;
    let TurnStartResponse { turn } = app
        .request(|request_id| ClientRequest::TurnStart {
            request_id,
            params: TurnStartParams {
                thread_id: thread.id.clone(),
                input: vec![UserInput::Text {
                    text: "Reply with hello".to_string(),
                    text_elements: Vec::new(),
                }],
                ..Default::default()
            },
        })
        .await?;
    let waiting: UsageLimitWaitChangedNotification = timeout(
        Duration::from_secs(/*secs*/ 15),
        app.read_notification("thread/usageLimitWaitChanged"),
    )
    .await??;
    assert_eq!(waiting.thread_id, thread.id);
    let retry_at_ms = waiting.retry_at_ms.expect("wait has a retry deadline");
    assert!((resets_at * 1000..=resets_at * 1000 + 6000).contains(&retry_at_ms));

    let _: TurnInterruptResponse = app
        .request(|request_id| ClientRequest::TurnInterrupt {
            request_id,
            params: TurnInterruptParams {
                thread_id: thread.id.clone(),
                turn_id: turn.id.clone(),
            },
        })
        .await?;
    // Match both terminal notifications so the helper cannot buffer an early turn/completed
    // and accidentally let an out-of-order wait-end notification pass this assertion.
    let notification = timeout(
        Duration::from_secs(/*secs*/ 15),
        app.read_stream_until_matching_notification(
            "wait end before turn completion",
            |notification| {
                matches!(
                    notification.method.as_str(),
                    "thread/usageLimitWaitChanged" | "turn/completed"
                )
            },
        ),
    )
    .await??;
    assert_eq!(notification.method, "thread/usageLimitWaitChanged");
    let ended: UsageLimitWaitChangedNotification =
        serde_json::from_value(notification.params.expect("wait-end parameters"))?;
    assert_eq!(
        ended,
        UsageLimitWaitChangedNotification {
            thread_id: thread.id.clone(),
            retry_at_ms: None,
        }
    );
    let completed: TurnCompletedNotification = timeout(
        Duration::from_secs(/*secs*/ 15),
        app.read_notification("turn/completed"),
    )
    .await??;
    assert_eq!(
        (
            completed.thread_id,
            completed.turn.id,
            completed.turn.status
        ),
        (thread.id, turn.id, TurnStatus::Interrupted)
    );
    Ok(())
}
