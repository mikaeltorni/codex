use super::*;
use codex_app_server_protocol::GetAccountRateLimitsResponse;
use pretty_assertions::assert_eq;
use serde_json::json;

fn banner_response(
    presentation: Option<&str>,
    ctas: serde_json::Value,
) -> GetAccountRateLimitsResponse {
    let mut banner = json!({
        "banner_type": "selected_model_limit", "model_slug": "test-model-a",
        "title": "Selected model usage exhausted",
        "description": "Ask your owner for credits or switch to another model.", "ctas": ctas,
    });
    if let Some(presentation) = presentation {
        banner["presentation"] = json!(presentation);
    }
    GetAccountRateLimitsResponse {
        ordinary_usage_allowed: None,
        account_id: Some("workspace-a".into()),
        rate_limit_upsell: Some(banner),
        rate_limits: snapshot(/*percent*/ 25.0),
        rate_limits_by_limit_id: None,
        rate_limit_reset_credits: None,
    }
}

#[tokio::test]
async fn backend_banner_presentation_and_cta_do_not_imply_recovery() {
    for presentation in [None, Some("inline"), Some("dismissible")] {
        let (mut chat, mut rx, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
        let mut response = banner_response(
            presentation,
            json!([{"action":"notify_owner","label":"Notify owner"}]),
        );
        if presentation == Some("inline") {
            use chrono::Timelike;
            chat.clock_format = crate::clock_format::ClockFormat::TwelveHour;
            let banner = response.rate_limit_upsell.as_mut().unwrap();
            banner["description"] =
                json!("Your usage resets at {time}. Ask your owner for credits.");
            banner["reset_at"] = json!(
                chrono::Local::now()
                    .with_hour(17)
                    .unwrap()
                    .with_minute(30)
                    .unwrap()
                    .timestamp()
            );
        }
        chat.update_backend_banner(&response);
        let initial = render_bottom_popup(&chat, /*width*/ 70);
        assert!(initial.contains("Selected model usage exhausted"));
        assert_eq!(
            initial.contains("esc to dismiss"),
            presentation == Some("dismissible")
        );
        chat.handle_key_event(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE));
        assert!(
            rx.try_recv()
                .is_ok_and(|event| matches!(event, AppEvent::SendAddCreditsNudgeEmail { .. }))
        );
        assert_eq!(render_bottom_popup(&chat, /*width*/ 70), initial);
        let request_id = chat
            .start_add_credits_nudge_email_request(AddCreditsNudgeCreditType::Credits)
            .expect("start notification");
        if presentation == Some("inline") {
            insta::assert_snapshot!(
                "backend_banner_notification",
                normalize_snapshot_paths(render_bottom_popup(&chat, /*width*/ 70))
            );
        }
        chat.handle_key_event(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE));
        assert!(
            rx.try_recv()
                .is_ok_and(|event| matches!(event, AppEvent::SendAddCreditsNudgeEmail { .. }))
        );
        assert_eq!(chat.composer_text_with_pending(), "");
        assert!(
            chat.start_add_credits_nudge_email_request(AddCreditsNudgeCreditType::Credits)
                .is_none()
        );
        chat.finish_add_credits_nudge_email_request(request_id, Err("local failure".into()));
        chat.update_backend_banner(&response);
        assert_eq!(render_bottom_popup(&chat, /*width*/ 70), initial);
        chat.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(
            render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted"),
            presentation != Some("dismissible")
        );
        chat.set_model("test-model-b");
        assert!(!chat.has_applicable_backend_banner());
        assert!(
            !render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted")
        );
        chat.set_model("test-model-a");
        assert_eq!(
            render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted"),
            presentation != Some("dismissible")
        );
    }
}

#[tokio::test]
async fn backend_banner_zero_actions_preserve_guidance_and_composer_input() {
    for presentation in [None, Some("inline"), Some("dismissible")] {
        for ctas in [
            json!([]),
            json!([{"action":"future_action","label":"Unsupported"}]),
        ] {
            let (mut chat, _rx, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
            chat.update_backend_banner(&banner_response(presentation, ctas));
            let rendered = render_bottom_popup(&chat, /*width*/ 70);
            assert!(rendered.contains("Selected model usage exhausted"));
            assert!(rendered.contains("switch to another model"));
            assert!(!rendered.to_lowercase().contains("no matches"));
            assert!(!rendered.contains("Press a number"));
            assert_eq!(
                rendered.contains("esc to dismiss"),
                presentation == Some("dismissible")
            );
            if presentation == Some("dismissible") {
                insta::assert_snapshot!(
                    "backend_banner_information_dismissible",
                    normalize_snapshot_paths(rendered)
                );
            } else {
                insta::assert_snapshot!(
                    "backend_banner_information_only",
                    normalize_snapshot_paths(rendered)
                );
            }
            chat.handle_key_event(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE));
            // An ordinary navigation key flushes the composer's pending first-character buffer.
            chat.handle_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
            assert_eq!(chat.composer_text_with_pending(), "1");
        }
    }
}

#[tokio::test]
async fn backend_banner_dismissal_tracks_occurrence_not_copy_or_model() {
    let (mut chat, _rx, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
    let mut response = banner_response(Some("dismissible"), json!([]));
    chat.update_backend_banner(&response);
    assert!(render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted"));
    chat.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    chat.set_model("test-model-b");
    response.rate_limit_upsell.as_mut().unwrap()["description"] = json!("Updated backend copy");
    chat.update_backend_banner(&response);
    chat.set_model("test-model-a");
    assert!(!render_bottom_popup(&chat, /*width*/ 70).contains("Updated backend copy"));
    response.rate_limit_upsell.as_mut().unwrap()["reset_at"] = json!(2000000000);
    chat.update_backend_banner(&response);
    assert!(render_bottom_popup(&chat, /*width*/ 70).contains("Updated backend copy"));
    chat.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    let mut absent = response.clone();
    absent.rate_limit_upsell = None;
    chat.update_backend_banner(&absent);
    chat.update_backend_banner(&response);
    assert!(render_bottom_popup(&chat, /*width*/ 70).contains("Updated backend copy"));
}

#[tokio::test]
async fn backend_banner_occurrence_uses_explicit_or_legacy_source_model() {
    for explicit_source in [false, true] {
        let (mut chat, _rx, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
        let mut response = banner_response(Some("dismissible"), json!([]));
        if explicit_source {
            response.rate_limit_upsell.as_mut().unwrap()["blocked_model_slug"] =
                json!("test-model-a");
        }
        chat.update_backend_banner(&response);
        assert!(
            render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted")
        );
        chat.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        response.rate_limit_upsell.as_mut().unwrap()["model_slug"] = json!("test-model-b");
        chat.update_backend_banner(&response);
        chat.set_model("test-model-b");
        assert_eq!(
            render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted"),
            !explicit_source
        );
    }
}

#[tokio::test]
async fn backend_banner_turn_on_another_model_preserves_hidden_occurrence() {
    let (mut chat, _rx, mut ops) = make_chatwidget_manual(Some("test-model-a")).await;
    chat.thread_id = Some(ThreadId::new());
    chat.update_backend_banner(&banner_response(Some("dismissible"), json!([])));
    assert!(render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted"));
    chat.set_model("test-model-b");
    assert!(!render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted"));
    chat.submit_user_message(UserMessage::from("hello"));
    assert!(
        ops.try_recv()
            .is_ok_and(|op| matches!(op, Op::UserTurn { .. }))
    );
    handle_turn_started(&mut chat, "turn-1");
    handle_turn_completed(&mut chat, "turn-1", /*duration_ms*/ None);
    chat.set_model("test-model-a");
    assert!(render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted"));
}

#[tokio::test]
async fn backend_banner_restores_only_programmatically_displaced_switch_prompt() {
    for user_dismissed in [false, true] {
        let (mut chat, _rx, _ops) = make_chatwidget_manual(Some("gpt-5")).await;
        chat.has_chatgpt_account = true;
        chat.on_rate_limit_snapshot(Some(snapshot(/*percent*/ 95.0)));
        chat.maybe_show_pending_rate_limit_prompt();
        assert!(render_bottom_popup(&chat, /*width*/ 90).contains("Approaching rate limits"));
        if user_dismissed {
            chat.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        }
        let mut response = banner_response(Some("inline"), json!([]));
        response.rate_limit_upsell.as_mut().unwrap()["model_slug"] = json!("gpt-5");
        chat.update_backend_banner(&response);
        assert!(
            render_bottom_popup(&chat, /*width*/ 90).contains("Selected model usage exhausted")
        );
        response.rate_limit_upsell = None;
        chat.update_backend_banner(&response);
        assert_eq!(
            render_bottom_popup(&chat, /*width*/ 90).contains("Approaching rate limits"),
            !user_dismissed
        );
    }
}

#[tokio::test]
async fn owner_notification_completion_cannot_cross_account_change() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    let previous = chat
        .start_add_credits_nudge_email_request(AddCreditsNudgeCreditType::Credits)
        .expect("start previous account request");
    chat.update_account_state(
        /*status_account_display*/ None, /*plan_type*/ None,
        /*has_chatgpt_account*/ true, /*has_codex_backend_auth*/ true,
    );
    let current = chat
        .start_add_credits_nudge_email_request(AddCreditsNudgeCreditType::UsageLimit)
        .expect("new account must be actionable");
    drain_insert_history(&mut rx);
    chat.finish_add_credits_nudge_email_request(previous, Ok(AddCreditsNudgeEmailStatus::Sent));
    assert!(drain_insert_history(&mut rx).is_empty());
    assert!(
        chat.start_add_credits_nudge_email_request(AddCreditsNudgeCreditType::UsageLimit)
            .is_none()
    );
    chat.finish_add_credits_nudge_email_request(current, Ok(AddCreditsNudgeEmailStatus::Sent));
    let rendered = drain_insert_history(&mut rx)
        .into_iter()
        .map(|lines| lines_to_single_string(&lines))
        .collect::<String>();
    insta::assert_snapshot!("owner_notification_after_account_change", rendered);
}

#[tokio::test]
async fn usage_wait_countdown_stays_visible_and_restores_updated_account_banner() {
    let (mut chat, _events, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
    let original = banner_response(Some("inline"), json!([]));
    chat.update_backend_banner(&original);
    assert!(render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted"));

    chat.bottom_pane.set_task_running(true);
    chat.update_usage_limit_wait(Some(chrono::Utc::now().timestamp_millis() + 60_000));
    let countdown = render_bottom_popup(&chat, /*width*/ 70);
    assert!(
        countdown.contains("Usage limit reached (Auto-continue is enabled)"),
        "{countdown}"
    );
    assert!(countdown.contains("Resuming in "), "{countdown}");
    assert!(
        !countdown.contains("Auto-continue is enabled. Resuming in"),
        "{countdown}"
    );
    assert!(!countdown.contains("Press Ctrl-C to stop"), "{countdown}");

    let mut updated = original;
    updated.rate_limit_upsell.as_mut().unwrap()["title"] = json!("Updated account limit");
    chat.update_backend_banner(&updated);
    let countdown_after_account_update = render_bottom_popup(&chat, /*width*/ 70);
    assert!(
        countdown_after_account_update.contains("Usage limit reached (Auto-continue is enabled)"),
        "{countdown_after_account_update}"
    );
    assert!(
        countdown_after_account_update.contains("Resuming in "),
        "{countdown_after_account_update}"
    );
    assert!(
        !countdown_after_account_update.contains("Press Ctrl-C to stop"),
        "{countdown_after_account_update}"
    );
    assert!(!countdown_after_account_update.contains("Updated account limit"));

    chat.update_usage_limit_wait(None);
    chat.bottom_pane.set_task_running(false);
    let restored = render_bottom_popup(&chat, /*width*/ 70);
    assert!(restored.contains("Updated account limit"), "{restored}");
    assert!(!restored.contains("Auto-continue is enabled"));
}

#[tokio::test]
async fn usage_wait_banner_exposes_account_recovery_choices_while_task_runs() {
    let (mut chat, mut events, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
    chat.has_chatgpt_account = true;
    chat.requires_openai_auth = true;
    let thread_id = ThreadId::new();
    chat.thread_id = Some(thread_id);

    let mut models = chat.model_catalog.try_list_models().unwrap();
    if !models
        .iter()
        .any(|model| model.model == crate::model_catalog::LUNA_RESERVE_MODEL)
    {
        let mut reserve = models[0].clone();
        reserve.model = crate::model_catalog::LUNA_RESERVE_MODEL.to_string();
        reserve.display_name = "Luna Reserve".to_string();
        models.push(reserve);
        chat.model_catalog = Arc::new(ModelCatalog::new(models));
    }

    chat.bottom_pane.set_task_running(true);
    chat.update_usage_limit_wait(Some(chrono::Utc::now().timestamp_millis() + 60_000));
    assert!(matches!(
        events.try_recv(),
        Ok(AppEvent::RefreshRateLimits {
            origin: crate::app_event::RateLimitRefreshOrigin::Recovery
        })
    ));
    let mut response: GetAccountRateLimitsResponse = serde_json::from_value(json!({
        "accountId": "account-preview", "rateLimits": {},
        "rateLimitUpsell": {
            "banner_type": "luna_reserve", "presentation": "inline",
            "title": "Usage limit reached",
            "description": "Add credits or upgrade to continue.",
            "ctas": [
                {"action": "add_credits", "label": "Add credits"},
                {"action": "open_pricing_dialog", "label": "Upgrade"}
            ]
        }
    }))
    .unwrap();
    response.rate_limits.plan_type = Some(PlanType::Plus);
    chat.update_backend_banner(&response);
    chat.refresh_usage_limit_wait_for_time_tick();

    let rendered = render_bottom_popup(&chat, /*width*/ 90);
    for action in [
        "Usage limit reached (Auto-continue is enabled)",
        "Add credits",
        "Upgrade",
        "Switch to Luna Reserve",
        "Keep waiting",
        "Resuming in ",
    ] {
        assert!(
            rendered.contains(action),
            "missing {action:?} in:\n{rendered}"
        );
    }
    let snapshot = rendered
        .lines()
        .filter(|line| !line.contains("Resuming in"))
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("usage_wait_account_recovery_choices", snapshot);

    chat.handle_key_event(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE));
    assert!(matches!(
        events.try_recv(),
        Ok(AppEvent::OpenUrlInBrowser { url })
            if url == "https://chatgpt.com/codex/settings/usage?credits_modal=true"
    ));
    chat.handle_key_event(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE));
    assert!(matches!(
        events.try_recv(),
        Ok(AppEvent::OpenUrlInBrowser { url })
            if url == "https://chatgpt.com/?cta_tab=personal&highlight_plan=pro#pricing"
    ));
    chat.handle_key_event(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE));
    assert!(matches!(
        events.try_recv(),
        Ok(AppEvent::ApplyBackendBannerFallback { thread_id: event_thread_id })
            if event_thread_id == thread_id
    ));
    chat.handle_key_event(KeyEvent::new(KeyCode::Char('4'), KeyModifiers::NONE));
    assert!(matches!(
        events.try_recv(),
        Ok(AppEvent::DismissUsageLimitWaitBanner {
            thread_id: Some(event_thread_id)
        }) if event_thread_id == thread_id
    ));

    chat.dismiss_usage_limit_wait_banner();
    let waiting = render_bottom_popup(&chat, /*width*/ 90);
    assert!(!waiting.contains("Auto-continue is enabled"), "{waiting}");
    assert!(waiting.contains("Resuming in "), "{waiting}");
}

#[tokio::test]
async fn usage_wait_uses_team_recovery_with_nullable_account_banner() {
    let (mut chat, mut events, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
    chat.has_chatgpt_account = true;
    chat.bottom_pane.set_task_running(true);
    chat.update_usage_limit_wait(Some(chrono::Utc::now().timestamp_millis() + 60_000));
    assert!(matches!(
        events.try_recv(),
        Ok(AppEvent::RefreshRateLimits { .. })
    ));

    let mut response = banner_response(
        /*presentation*/ None,
        json!([{"action": "notify_owner", "label": "Request credits"}]),
    );
    response.rate_limits.plan_type = Some(PlanType::Team);
    response.rate_limits.rate_limit_reached_type =
        Some(RateLimitReachedType::WorkspaceMemberCreditsDepleted);
    let upsell = response.rate_limit_upsell.as_mut().unwrap();
    upsell["banner_type"] = json!("workspace_member_credits_depleted");
    upsell["model_slug"] = serde_json::Value::Null;
    upsell["presentation"] = serde_json::Value::Null;
    upsell["fallback_model_slugs"] = serde_json::Value::Null;
    chat.update_backend_banner(&response);
    chat.on_rate_limit_snapshot(Some(response.rate_limits));
    chat.refresh_usage_limit_wait_for_time_tick();

    let rendered = render_bottom_popup(&chat, /*width*/ 90);
    assert!(rendered.contains("Request increase"), "{rendered}");
    assert!(rendered.contains("Keep waiting"), "{rendered}");
    assert!(rendered.contains("Resuming in "), "{rendered}");
    chat.handle_key_event(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE));
    assert!(matches!(
        events.try_recv(),
        Ok(AppEvent::SendAddCreditsNudgeEmail {
            credit_type: AddCreditsNudgeCreditType::UsageLimit
        })
    ));
    let stable = rendered
        .lines()
        .filter(|line| !line.contains("Resuming in"))
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("usage_wait_team_recovery_choices", stable);
}

#[tokio::test]
async fn hiding_usage_wait_actions_keeps_reset_countdown_running() {
    let (mut chat, _events, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
    chat.bottom_pane.set_task_running(true);
    let retry_at_ms = chrono::Utc::now().timestamp_millis() + 60_000;
    chat.update_usage_limit_wait(Some(retry_at_ms));
    let initial_tick = chat
        .usage_limit_wait_next_tick
        .expect("active wait should schedule a countdown refresh");

    chat.dismiss_usage_limit_wait_banner();
    assert_eq!(chat.usage_limit_wait_retry_at_ms, Some(retry_at_ms));
    assert_eq!(chat.usage_limit_wait_next_tick, Some(initial_tick));
    let waiting = render_bottom_popup(&chat, /*width*/ 70);
    assert!(!waiting.contains("Auto-continue is enabled"), "{waiting}");
    assert!(waiting.contains("Resuming in "), "{waiting}");

    chat.refresh_usage_limit_wait_for_time_tick();
    assert_eq!(chat.usage_limit_wait_retry_at_ms, Some(retry_at_ms));
    assert!(
        chat.usage_limit_wait_next_tick
            .is_some_and(|next_tick| next_tick > initial_tick)
    );
    let waiting_after_tick = render_bottom_popup(&chat, /*width*/ 70);
    assert!(
        !waiting_after_tick.contains("Auto-continue is enabled"),
        "{waiting_after_tick}"
    );
    assert!(
        waiting_after_tick.contains("Resuming in "),
        "{waiting_after_tick}"
    );
}

#[tokio::test]
async fn backend_banner_new_turn_dismisses_only_shown_dismissible_content() {
    for (presentation, show_before_submit) in [
        (None, true),
        (Some("inline"), true),
        (Some("dismissible"), true),
        (Some("dismissible"), false),
    ] {
        let (mut chat, _rx, mut ops) = make_chatwidget_manual(Some("test-model-a")).await;
        chat.thread_id = Some(ThreadId::new());
        chat.update_backend_banner(&banner_response(presentation, json!([])));
        if show_before_submit {
            assert!(
                render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted")
            );
        }
        chat.submit_user_message(UserMessage::from("hello"));
        assert!(
            ops.try_recv()
                .is_ok_and(|op| matches!(op, Op::UserTurn { .. }))
        );
        handle_turn_started(&mut chat, "turn-1");
        handle_turn_completed(&mut chat, "turn-1", /*duration_ms*/ None);
        assert_eq!(
            render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted"),
            !(presentation == Some("dismissible") && show_before_submit)
        );
    }
}

#[tokio::test]
async fn backend_banner_invalid_content_and_absence_restore_fallback() {
    for replacement in [
        serde_json::Value::Null,
        json!({"presentation":"future_mode"}),
        json!({"title":" "}),
    ] {
        let (mut chat, _rx, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
        let mut response = banner_response(Some("inline"), json!([]));
        chat.update_backend_banner(&response);
        assert!(
            render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted")
        );
        if replacement.is_null() {
            response.rate_limit_upsell = None;
        } else {
            for (key, value) in replacement.as_object().unwrap() {
                response.rate_limit_upsell.as_mut().unwrap()[key.as_str()] = value.clone();
            }
        }
        chat.update_backend_banner(&response);
        assert!(!chat.has_applicable_backend_banner());
        chat.codex_rate_limit_reached_type =
            Some(RateLimitReachedType::WorkspaceMemberCreditsDepleted);
        chat.on_rate_limit_error(RateLimitErrorKind::Generic, "limit".into());
        assert!(render_bottom_popup(&chat, /*width*/ 90).contains("workspace owner"));
    }
}

#[tokio::test]
async fn backend_banner_fallback_candidates_and_notice_follow_selected_model() {
    let (mut chat, _events, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
    chat.has_chatgpt_account = true;
    let template = chat.model_catalog.try_list_models().unwrap()[0].clone();
    let models = ["hidden-model", "test-model-b", "test-model-c"].map(|model| ModelPreset {
        model: model.into(),
        show_in_picker: model != "hidden-model",
        ..template.clone()
    });
    chat.model_catalog = Arc::new(ModelCatalog::new(models.to_vec()));
    let mut response = banner_response(Some("inline"), json!([]));
    response.rate_limit_upsell.as_mut().unwrap()["blocked_model_slug"] = json!("test-model-a");
    response.rate_limit_upsell.as_mut().unwrap()["fallback_model_slugs"] =
        json!(["hidden-model", "test-model-c", "test-model-b"]);
    chat.update_backend_banner(&response);
    assert_eq!(
        chat.backend_banner_fallback().map(|switch| switch.model),
        Some(models[2].clone())
    );
    // A retained ChatGPT login must not change a task whose server does not require auth.
    chat.requires_openai_auth = false;
    assert_eq!(chat.backend_banner_fallback(), None);
    chat.requires_openai_auth = true;
    let mut mode = chat.effective_collaboration_mode();
    mode.settings.model = "test-model-c".into();
    chat.finish_backend_banner_fallback(mode);
    let before = render_bottom_popup(&chat, /*width*/ 70);
    assert!(before.contains("Selected model usage exhausted"));
    chat.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted"));
    assert_eq!(chat.backend_banner_fallback(), None);
    let (mut other_task, _events, _ops) = make_chatwidget_manual(Some("test-model-c")).await;
    other_task.inherit_backend_banner_state(&mut chat);
    assert!(
        render_bottom_popup(&other_task, /*width*/ 70).contains("Selected model usage exhausted")
    );
    other_task.set_model("test-model-a");
    assert!(
        !render_bottom_popup(&other_task, /*width*/ 70).contains("Selected model usage exhausted")
    );
    other_task.set_model("unrelated-model");
    assert!(
        !render_bottom_popup(&other_task, /*width*/ 70).contains("Selected model usage exhausted")
    );
    other_task.set_model("test-model-c");
    assert!(
        render_bottom_popup(&other_task, /*width*/ 70).contains("Selected model usage exhausted")
    );
}

#[tokio::test]
async fn backend_banner_sparse_updates_preserve_visible_and_dismissed_occurrences() {
    for dismiss in [false, true] {
        let (mut chat, _events, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
        chat.update_backend_banner(&banner_response(Some("dismissible"), json!([])));
        assert!(
            render_bottom_popup(&chat, /*width*/ 70).contains("Selected model usage exhausted")
        );
        if dismiss {
            chat.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        }
        let before = render_bottom_popup(&chat, /*width*/ 70);
        chat.on_rolling_rate_limit_snapshot(snapshot(/*percent*/ 60.0));
        assert_eq!(render_bottom_popup(&chat, /*width*/ 70), before);
        // A model-scoped banner can coexist with an absent aggregate reached type.
        let mut exhausted = snapshot(/*percent*/ 100.0);
        exhausted.rate_limit_reached_type =
            Some(RateLimitReachedType::WorkspaceMemberCreditsDepleted);
        chat.on_rolling_rate_limit_snapshot(exhausted);
        assert_eq!(render_bottom_popup(&chat, /*width*/ 70), before);
    }
}

#[tokio::test]
async fn backend_banner_changed_remedy_keeps_fallback_until_applicable_replacement() {
    for rolling_update in [false, true] {
        let (mut chat, _events, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
        let mut credits = snapshot(/*percent*/ 100.0);
        credits.rate_limit_reached_type =
            Some(RateLimitReachedType::WorkspaceMemberCreditsDepleted);
        chat.on_rate_limit_snapshot(Some(credits));
        let mut response = banner_response(
            Some("inline"),
            json!([{"action":"notify_owner","label":"Notify owner"}]),
        );
        chat.update_backend_banner(&response);
        assert!(render_bottom_popup(&chat, /*width*/ 90).contains("Notify owner"));
        let mut cap = snapshot(/*percent*/ 100.0);
        cap.rate_limit_reached_type = Some(RateLimitReachedType::WorkspaceMemberUsageLimitReached);
        if rolling_update {
            chat.on_rolling_rate_limit_snapshot(cap);
        }
        chat.on_rate_limit_error(RateLimitErrorKind::UsageLimit, "Usage cap reached".into());
        assert!(
            render_bottom_popup(&chat, /*width*/ 90)
                .contains("Request a limit increase from your owner")
        );
        response.rate_limit_upsell.as_mut().unwrap()["title"] =
            json!("Workspace spending cap reached");
        response.rate_limit_upsell.as_mut().unwrap()["ctas"] =
            json!([{"action":"request_increase","label":"Request more usage"}]);
        chat.update_backend_banner(&response);
        let rendered = render_bottom_popup(&chat, /*width*/ 90);
        assert!(
            rendered.contains("Workspace spending cap reached"),
            "{rendered}"
        );
        assert!(rendered.contains("Request more usage"));
        assert!(!rendered.contains("Request a limit increase from your owner"));
        chat.set_model("test-model-b");
        chat.on_rate_limit_error(RateLimitErrorKind::UsageLimit, "Usage cap reached".into());
        assert!(
            render_bottom_popup(&chat, /*width*/ 90)
                .contains("Request a limit increase from your owner")
        );
    }
}
