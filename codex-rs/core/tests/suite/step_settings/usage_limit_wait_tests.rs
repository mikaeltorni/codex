use super::*;
use chrono::Utc;
use pretty_assertions::assert_eq;
use wiremock::ResponseTemplate;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn model_updates_resume_quota_wait_with_the_accepted_settings() -> Result<()> {
    let server = start_mock_server().await;
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
            ResponseTemplate::new(/*s*/ 200)
                .set_body_raw(sse_completed("recovered"), "text/event-stream"),
        ],
    )
    .await;
    let test = step_settings_test()
        .with_config(|config| {
            config.auto_resume_on_usage_limit = true;
            config.model_provider.request_max_retries = Some(0);
        })
        .build_with_auto_env(&server)
        .await?;
    let submission = test
        .codex
        .start_or_steer_turn(TurnInputRequest::user_input(vec![UserInput::Text {
            text: "keep this request".into(),
            text_elements: Vec::new(),
        }]))
        .await?;
    let codex_protocol::turn_input::TurnInputSubmission::Started { turn_id } = submission else {
        panic!("expected a new turn");
    };
    for model in [MODEL_B, MODEL_C] {
        wait_for_event(&test.codex, |event| {
            matches!(event, EventMsg::UsageLimitWaitStarted(_))
        })
        .await;
        assert_eq!(
            submit_turn_settings(
                &test.codex,
                &turn_id,
                TurnSettingsUpdate {
                    model: Some(model.to_string()),
                    ..Default::default()
                },
            )
            .await?,
            TurnSettingsUpdateOutcome::Applied
        );
        tokio::time::timeout(
            std::time::Duration::from_secs(/*secs*/ 3),
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
    assert_eq!(
        requests
            .iter()
            .map(|request| request.body_json()["model"].clone())
            .collect::<Vec<_>>(),
        vec![json!(MODEL_A), json!(MODEL_B), json!(MODEL_C)]
    );
    assert_eq!(
        requests[0].message_input_texts("user"),
        requests[2].message_input_texts("user")
    );
    assert!(
        requests[2]
            .message_input_texts("user")
            .contains(&"keep this request".to_string())
    );
    Ok(())
}
