//! Exercise quota wait notifications and cancellation through the public app-server API.

use anyhow::Context;
use anyhow::Result;
use app_test_support::ChatGptAuthFixture;
use app_test_support::MockResponsesConfig;
use app_test_support::TestAppServer;
use app_test_support::write_chatgpt_auth;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::ConsumeAccountRateLimitResetCreditOutcome;
use codex_app_server_protocol::ConsumeAccountRateLimitResetCreditParams;
use codex_app_server_protocol::ConsumeAccountRateLimitResetCreditResponse;
use codex_app_server_protocol::ThreadReadParams;
use codex_app_server_protocol::ThreadReadResponse;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::TurnCompletedNotification;
use codex_app_server_protocol::TurnInterruptParams;
use codex_app_server_protocol::TurnInterruptResponse;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::TurnStatus;
use codex_app_server_protocol::UsageLimitWaitChangedNotification;
use codex_app_server_protocol::UserInput;
use codex_config::types::AuthCredentialsStoreMode;
use core_test_support::responses;
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
async fn attaching_during_quota_wait_replays_the_deadline_and_wait_end() -> Result<()> {
    use super::connection_handling_websocket::*;
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(/*s*/ 429).set_body_json(json!({
            "error": { "type": "usage_limit_reached", "resets_at": chrono::Utc::now().timestamp() + 3600 }
        }))).expect(/*r*/ 1).mount(&server).await;
    let home = TempDir::new()?;
    MockResponsesConfig::new(&server.uri())
        .with_root_config("auto_resume_on_usage_limit = true")
        .write(home.path())?;
    let (_process, address) = spawn_websocket_server(home.path()).await?;
    let mut first = connect_websocket(address).await?;
    send_request(
        &mut first,
        "initialize",
        /*id*/ 1,
        Some(json!({
            "clientInfo": {"name": "first", "version": "1"},
            "capabilities": {"experimentalApi": true}
        })),
    )
    .await?;
    read_response_for_id(&mut first, /*id*/ 1).await?;
    send_request(
        &mut first,
        "thread/start",
        /*id*/ 2,
        Some(json!({
            "cwd": home.path(), "environments": [{"environmentId": "local", "cwd": home.path()}]
        })),
    )
    .await?;
    let started: codex_app_server_protocol::ThreadStartResponse =
        serde_json::from_value(read_response_for_id(&mut first, /*id*/ 2).await?.result)?;
    let thread_id = started.thread.id;
    send_request(
        &mut first,
        "turn/start",
        /*id*/ 3,
        Some(json!({
            "threadId": thread_id, "input": [{"type": "text", "text": "keep working"}]
        })),
    )
    .await?;
    let (turn, waiting) = read_response_and_notification_for_method(
        &mut first,
        /*id*/ 3,
        "thread/usageLimitWaitChanged",
    )
    .await
    .context("initial wait and turn response")?;
    let turn: TurnStartResponse = serde_json::from_value(turn.result)?;
    let waiting: UsageLimitWaitChangedNotification =
        serde_json::from_value(waiting.params.unwrap())?;
    assert!(waiting.retry_at_ms.is_some());
    let mut second = connect_websocket(address).await?;
    send_request(
        &mut second,
        "initialize",
        /*id*/ 1,
        Some(json!({
            "clientInfo": {"name": "second", "version": "1"},
            "capabilities": {"experimentalApi": true}
        })),
    )
    .await?;
    read_response_for_id(&mut second, /*id*/ 1).await?;
    send_request(
        &mut second,
        "thread/resume",
        /*id*/ 2,
        Some(json!({"threadId": thread_id})),
    )
    .await?;
    read_response_for_id(&mut second, /*id*/ 2).await?;
    let replayed: UsageLimitWaitChangedNotification = serde_json::from_value(
        read_notification_for_method(&mut second, "thread/usageLimitWaitChanged")
            .await
            .context("replayed wait")?
            .params
            .unwrap(),
    )?;
    assert_eq!(replayed, waiting);
    send_request(
        &mut second,
        "turn/interrupt",
        /*id*/ 3,
        Some(json!({
            "threadId": thread_id, "turnId": turn.turn.id,
        })),
    )
    .await?;
    let (_, ended) = read_response_and_notification_for_method(
        &mut second,
        /*id*/ 3,
        "thread/usageLimitWaitChanged",
    )
    .await
    .context("interrupt response and wait end")?;
    let ended: UsageLimitWaitChangedNotification = serde_json::from_value(ended.params.unwrap())?;
    assert_eq!(
        ended,
        UsageLimitWaitChangedNotification {
            thread_id,
            retry_at_ms: None
        }
    );
    let original_ended: UsageLimitWaitChangedNotification = serde_json::from_value(
        read_notification_for_method(&mut first, "thread/usageLimitWaitChanged")
            .await
            .context("original client wait end")?
            .params
            .unwrap(),
    )?;
    assert_eq!(original_ended, ended);
    Ok(())
}

#[tokio::test]
async fn successful_reset_redemption_resumes_an_active_quota_wait() -> Result<()> {
    for (outcome, expected) in [
        (
            "no_credit",
            ConsumeAccountRateLimitResetCreditOutcome::NoCredit,
        ),
        (
            "already_redeemed",
            ConsumeAccountRateLimitResetCreditOutcome::AlreadyRedeemed,
        ),
        (
            "nothing_to_reset",
            ConsumeAccountRateLimitResetCreditOutcome::NothingToReset,
        ),
        ("reset", ConsumeAccountRateLimitResetCreditOutcome::Reset),
    ] {
        let server = MockServer::start().await;
        let mut replies = vec![ResponseTemplate::new(/*s*/ 429).set_body_json(json!({
            "error": {
                "type": "usage_limit_reached",
                "resets_at": chrono::Utc::now().timestamp() + 3600,
                "plan_type": "pro"
            }
        }))];
        if expected == ConsumeAccountRateLimitResetCreditOutcome::Reset {
            replies.push(
                ResponseTemplate::new(/*s*/ 200)
                    .set_body_raw(responses::sse_completed("recovered"), "text/event-stream"),
            );
        }
        let model_requests = responses::mount_response_sequence(&server, replies).await;
        Mock::given(method("POST"))
            .and(path("/api/codex/rate-limit-reset-credits/consume"))
            .respond_with(ResponseTemplate::new(/*s*/ 200).set_body_json(json!({
                "code": outcome, "windows_reset": if outcome == "reset" { 1 } else { 0 }
            })))
            .expect(/*r*/ 1)
            .mount(&server)
            .await;
        let home = TempDir::new()?;
        MockResponsesConfig::new(&server.uri())
            .with_root_config(&format!(
                "auto_resume_on_usage_limit = true\nchatgpt_base_url = {:?}",
                server.uri(),
            ))
            .write(home.path())?;
        write_chatgpt_auth(
            home.path(),
            ChatGptAuthFixture::new("chatgpt-token")
                .account_id("account-123")
                .plan_type("pro"),
            AuthCredentialsStoreMode::File,
        )?;
        let mut app = TestAppServer::builder()
            .with_codex_home(home.path())
            .with_env_overrides(&[("OPENAI_API_KEY", None)])
            .build_initialized()
            .await?;
        let thread = app.start_thread(ThreadStartParams::default()).await?.thread;
        let TurnStartResponse { turn } = app
            .request(|request_id| ClientRequest::TurnStart {
                request_id,
                params: TurnStartParams {
                    thread_id: thread.id.clone(),
                    input: vec![UserInput::Text {
                        text: "keep working".to_string(),
                        text_elements: Vec::new(),
                    }],
                    ..Default::default()
                },
            })
            .await?;
        let waiting: UsageLimitWaitChangedNotification = app
            .read_notification("thread/usageLimitWaitChanged")
            .await?;
        assert!(waiting.retry_at_ms.is_some());
        let request_id = app
            .send_consume_account_rate_limit_reset_credit_request(
                ConsumeAccountRateLimitResetCreditParams {
                    idempotency_key: "redeem-once".to_string(),
                    credit_id: None,
                },
            )
            .await?;
        let response = app
            .read_stream_until_response_message(codex_app_server_protocol::RequestId::Integer(
                request_id,
            ))
            .await?;
        let redeemed: ConsumeAccountRateLimitResetCreditResponse =
            serde_json::from_value(response.result)?;
        assert_eq!(
            redeemed,
            ConsumeAccountRateLimitResetCreditResponse { outcome: expected }
        );
        if expected != ConsumeAccountRateLimitResetCreditOutcome::Reset {
            let read: ThreadReadResponse = app
                .request(|request_id| ClientRequest::ThreadRead {
                    request_id,
                    params: ThreadReadParams {
                        thread_id: thread.id.clone(),
                        include_turns: true,
                    },
                })
                .await?;
            let active = read
                .thread
                .turns
                .last()
                .expect("the waiting turn is retained");
            assert_eq!(
                (&active.id, &active.status),
                (&turn.id, &TurnStatus::InProgress)
            );
            assert!(
                timeout(
                    Duration::from_millis(/*millis*/ 200),
                    app.read_notification::<UsageLimitWaitChangedNotification>(
                        "thread/usageLimitWaitChanged"
                    )
                )
                .await
                .is_err()
            );
            assert_eq!(model_requests.requests().len(), 1);
            let _: TurnInterruptResponse = app
                .request(|request_id| ClientRequest::TurnInterrupt {
                    request_id,
                    params: TurnInterruptParams {
                        thread_id: thread.id.clone(),
                        turn_id: turn.id.clone(),
                    },
                })
                .await?;
        }
        let ended: UsageLimitWaitChangedNotification = timeout(
            Duration::from_secs(/*secs*/ 10),
            app.read_notification("thread/usageLimitWaitChanged"),
        )
        .await??;
        assert_eq!(
            ended,
            UsageLimitWaitChangedNotification {
                thread_id: thread.id.clone(),
                retry_at_ms: None
            }
        );
        let completed: TurnCompletedNotification = app.read_notification("turn/completed").await?;
        assert_eq!(
            completed.turn.status,
            if expected == ConsumeAccountRateLimitResetCreditOutcome::Reset {
                TurnStatus::Completed
            } else {
                TurnStatus::Interrupted
            }
        );
        if expected == ConsumeAccountRateLimitResetCreditOutcome::Reset {
            let requests = model_requests.requests();
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[0].input(), requests[1].input());
        }
    }
    Ok(())
}

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
