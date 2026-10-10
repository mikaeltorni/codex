//! Integration coverage for retrying a quota-limited sampling request inside its original turn.

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use chrono::Utc;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::UsageLimitWaitEvent;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_response_sequence;
use core_test_support::responses::sse;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

/// Verify accepted steering resumes repeated waits inside the original root turn.
/// Parameters: none.
/// Returns: success after all retry requests retain the accepted inputs.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn steering_resumes_repeated_quota_waits_with_the_accepted_input() -> anyhow::Result<()> {
    println!("parameters=none");
    let server = MockServer::start().await;
    let quota = ResponseTemplate::new(/*s*/ 429).set_body_json(json!({
        "error": {
            "type": "usage_limit_reached",
            "resets_at": Utc::now().timestamp() + 3600,
            "plan_type": "pro"
        }
    }));
    let responses = mount_response_sequence(
        &server,
        vec![
            quota.clone(),
            quota,
            ResponseTemplate::new(/*s*/ 200).set_body_raw(
                sse(vec![
                    ev_response_created("recovered"),
                    ev_completed("recovered"),
                ]),
                "text/event-stream",
            ),
        ],
    )
    .await;
    let test = test_codex()
        .with_config(|config| {
            config.auto_resume_on_usage_limit = true;
            config.model_provider.request_max_retries = Some(0);
        })
        .build_with_auto_env(&server)
        .await?;
    let started = test
        .codex
        .start_or_steer_turn(codex_core::TurnInputRequest::user_input(vec![
            codex_protocol::user_input::UserInput::Text {
                text: "do the task".into(),
                text_elements: Vec::new(),
            },
        ]))
        .await?;
    let codex_protocol::turn_input::TurnInputSubmission::Started {
        turn_id,
        root_turn_id,
    } = started
    else {
        panic!("expected a new turn");
    };
    for text in ["first correction", "second correction"] {
        wait_for_event(&test.codex, |event| {
            matches!(event, EventMsg::UsageLimitWaitStarted(_))
        })
        .await;
        let steered = test
            .codex
            .start_or_steer_turn(codex_core::TurnInputRequest::user_input(vec![
                codex_protocol::user_input::UserInput::Text {
                    text: text.into(),
                    text_elements: Vec::new(),
                },
            ]))
            .await?;
        assert_eq!(
            steered,
            codex_protocol::turn_input::TurnInputSubmission::Steered {
                turn_id: turn_id.clone(),
                root_turn_id: root_turn_id.clone(),
            }
        );
        tokio::time::timeout(
            std::time::Duration::from_secs(/*secs*/ 10),
            wait_for_event(&test.codex, |event| {
                matches!(event, EventMsg::UsageLimitWaitEnded)
            }),
        )
        .await?;
    }
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    let requests = responses.requests();
    assert_eq!(requests.len(), 3);
    let mut expected = requests[0].message_input_texts("user");
    assert_eq!(expected.last().map(String::as_str), Some("do the task"));
    for (request, text) in requests[1..]
        .iter()
        .zip(["first correction", "second correction"])
    {
        expected.push(text.to_string());
        assert_eq!(request.message_input_texts("user"), expected);
    }
    println!("Ok(())");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn retries_multiple_usage_limits_with_completed_tool_result() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let attempts = Arc::new(AtomicUsize::new(0));
    let response_number = Arc::clone(&attempts);
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(move |_request: &wiremock::Request| {
            match response_number.fetch_add(1, Ordering::SeqCst) {
                0 => ResponseTemplate::new(200).set_body_raw(
                    sse(vec![
                        ev_response_created("first"),
                        ev_function_call("completed-tool", "unsupported_tool", "{}"),
                        ev_completed("first"),
                    ]),
                    "text/event-stream",
                ),
                1 | 2 => ResponseTemplate::new(429).set_body_json(json!({
                    "error": {
                        "type": "usage_limit_reached",
                        "message": "limit reached",
                        "resets_at": Utc::now().timestamp() - 60,
                        "plan_type": "pro"
                    }
                })),
                _ => ResponseTemplate::new(200).set_body_raw(
                    sse(vec![ev_response_created("last"), ev_completed("last")]),
                    "text/event-stream",
                ),
            }
        })
        .expect(4)
        .mount(&server)
        .await;

    let test = test_codex()
        .with_config(|config| {
            config.auto_resume_on_usage_limit = true;
            config.model_provider.request_max_retries = Some(0);
        })
        .build_with_auto_env(&server)
        .await?;
    test.codex
        .start_or_steer_turn(codex_core::TurnInputRequest::user_input(vec![
            codex_protocol::user_input::UserInput::Text {
                text: "do the task".into(),
                text_elements: Vec::new(),
            },
        ]))
        .await?;
    let mut waits_started = 0;
    let mut waits_ended = 0;
    loop {
        match wait_for_event(&test.codex, |_| true).await {
            EventMsg::UsageLimitWaitStarted(_) => waits_started += 1,
            EventMsg::UsageLimitWaitEnded => {
                waits_ended += 1;
                assert_eq!(waits_ended, waits_started);
            }
            EventMsg::TurnComplete(_) => break,
            _ => {}
        }
    }
    assert_eq!(waits_started, 2);
    assert_eq!(waits_ended, 2);

    let requests = server.received_requests().await.unwrap_or_default();
    let requests: Vec<_> = requests
        .into_iter()
        .filter(|request| request.url.path() == "/v1/responses")
        .collect();
    assert_eq!(requests.len(), 4);
    let second: serde_json::Value = serde_json::from_slice(&requests[1].body)?;
    let last: serde_json::Value = serde_json::from_slice(&requests[3].body)?;
    assert_eq!(second["input"], last["input"]);
    assert!(
        last["input"]
            .as_array()
            .is_some_and(|input| input.iter().any(|item| {
                item["type"] == "function_call_output" && item["call_id"] == "completed-tool"
            }))
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelling_quota_wait_aborts_without_another_request() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": {
                "type": "usage_limit_reached",
                "message": "limit reached",
                "resets_at": Utc::now().timestamp() + 3600,
                "plan_type": "pro"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let test = test_codex()
        .with_config(|config| {
            config.auto_resume_on_usage_limit = true;
            config.model_provider.request_max_retries = Some(0);
        })
        .build_with_auto_env(&server)
        .await?;
    test.codex
        .start_or_steer_turn(codex_core::TurnInputRequest::user_input(vec![
            codex_protocol::user_input::UserInput::Text {
                text: "do the task".into(),
                text_elements: Vec::new(),
            },
        ]))
        .await?;
    let retry_at_ms = match wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::UsageLimitWaitStarted(UsageLimitWaitEvent { retry_at_ms: _ })
        )
    })
    .await
    {
        EventMsg::UsageLimitWaitStarted(event) => event.retry_at_ms,
        _ => unreachable!(),
    };
    assert!(retry_at_ms > Utc::now().timestamp_millis());
    test.codex.submit(Op::Interrupt).await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::UsageLimitWaitEnded)
    })
    .await;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnAborted(_))
    })
    .await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_reset_timestamp_keeps_terminal_usage_limit_error() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": {
                "type": "usage_limit_reached",
                "message": "limit reached",
                "plan_type": "pro"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let test = test_codex()
        .with_config(|config| {
            config.auto_resume_on_usage_limit = true;
            config.model_provider.request_max_retries = Some(0);
        })
        .build_with_auto_env(&server)
        .await?;
    test.codex
        .start_or_steer_turn(codex_core::TurnInputRequest::user_input(vec![
            codex_protocol::user_input::UserInput::Text {
                text: "do the task".into(),
                text_elements: Vec::new(),
            },
        ]))
        .await?;
    let error = wait_for_event(&test.codex, |event| matches!(event, EventMsg::Error(_))).await;
    let EventMsg::Error(error) = error else {
        unreachable!();
    };
    assert!(error.message.contains("limit"));
    assert_eq!(
        server
            .received_requests()
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|request| request.url.path() == "/v1/responses")
            .count(),
        1
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn disabled_auto_resume_keeps_terminal_usage_limit_error() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": {
                "type": "usage_limit_reached",
                "message": "limit reached",
                "resets_at": Utc::now().timestamp() + 3600,
                "plan_type": "pro"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let test = test_codex()
        .with_config(|config| {
            assert!(!config.auto_resume_on_usage_limit);
            config.model_provider.request_max_retries = Some(0);
        })
        .build_with_auto_env(&server)
        .await?;
    test.codex
        .start_or_steer_turn(codex_core::TurnInputRequest::user_input(vec![
            codex_protocol::user_input::UserInput::Text {
                text: "do the task".into(),
                text_elements: Vec::new(),
            },
        ]))
        .await?;
    let error = wait_for_event(&test.codex, |event| matches!(event, EventMsg::Error(_))).await;
    let EventMsg::Error(error) = error else {
        unreachable!();
    };
    assert!(error.message.contains("limit"));
    assert_eq!(
        server
            .received_requests()
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|request| request.url.path() == "/v1/responses")
            .count(),
        1
    );
    Ok(())
}
