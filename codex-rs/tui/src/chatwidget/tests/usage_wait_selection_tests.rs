use super::*;
use codex_app_server_protocol::GetAccountRateLimitsResponse;
use codex_app_server_protocol::RateLimitResetCreditsSummary;
use pretty_assertions::assert_eq;
use serde_json::json;

/// Build account responses with fresh copy and independently specified recovery actions.
/// Parameters: ctas - ordered backend choices; revision - replacement copy version.
/// Returns: a Plus account response for the selected model.
fn response(ctas: serde_json::Value, revision: i64) -> GetAccountRateLimitsResponse {
    println!("ctas={ctas}, revision={revision}");
    let result = serde_json::from_value(json!({
        "accountId": "personal-a",
        "rateLimits": {"planType": "plus"},
        "rateLimitUpsell": {
            "banner_type": "selected_model_limit", "model_slug": "test-model-a",
            "title": "Usage limit reached", "description": format!("Account update {revision}"),
            "ctas": ctas
        }
    }))
    .unwrap();
    println!("{result:?}");
    result
}

/// Preserve the reported Reset usage selection through countdown ticks and changed account reads.
/// Parameters: none.
/// Returns: None.
#[tokio::test]
async fn usage_wait_account_refresh_preserves_selected_reset() {
    println!("parameters=none");
    let (mut chat, mut events, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
    chat.has_chatgpt_account = true;
    chat.plan_type = Some(PlanType::Plus);
    let ctas = json!([
        {"action": "open_pricing_dialog", "label": "Upgrade"},
        {"action": "reset_usage", "label": "Reset usage"}
    ]);
    chat.update_backend_banner(&response(ctas.clone(), /*revision*/ 0));
    chat.bottom_pane.set_task_running(/*running*/ true);
    let retry_at = chrono::Utc::now().timestamp_millis() + 60_000;
    chat.update_usage_limit_wait(Some(retry_at));
    assert_matches!(events.try_recv(), Ok(AppEvent::RefreshRateLimits { .. }));
    let _ = render_bottom_popup(&chat, /*width*/ 90);
    chat.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    for revision in 1..=3 {
        chat.refresh_usage_limit_wait_for_time_tick();
        chat.update_backend_banner(&response(ctas.clone(), revision));
        chat.update_usage_limit_wait(Some(retry_at));
        let rendered = render_bottom_popup(&chat, /*width*/ 90);
        assert!(rendered.contains("› 2. Reset usage"), "{rendered}");
        assert!(rendered.contains("Resuming in "), "{rendered}");
        assert!(events.try_recv().is_err());
    }
    let rendered = render_bottom_popup(&chat, /*width*/ 90);
    let stable = rendered
        .lines()
        .filter(|line| !line.contains("Resuming in"))
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("usage_wait_reset_selection_after_account_refresh", stable);
    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_matches!(events.try_recv(), Ok(AppEvent::OpenRateLimitResetCredits));
    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(events.try_recv().is_err());
    assert_eq!(chat.usage_limit_wait_retry_at_ms, Some(retry_at));
    println!("None");
}

/// Follow the selected action when backend choices move or relabel it.
/// Parameters: none.
/// Returns: None.
#[tokio::test]
async fn usage_wait_account_refresh_tracks_reordered_and_renamed_reset() {
    println!("parameters=none");
    let (mut chat, mut events, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
    chat.has_chatgpt_account = true;
    chat.plan_type = Some(PlanType::Plus);
    chat.bottom_pane.set_task_running(/*running*/ true);
    chat.update_usage_limit_wait(Some(chrono::Utc::now().timestamp_millis() + 60_000));
    assert_matches!(events.try_recv(), Ok(AppEvent::RefreshRateLimits { .. }));
    let _ = render_bottom_popup(&chat, /*width*/ 90);
    chat.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    chat.update_backend_banner(&response(
        json!([
            {"action": "view_usage", "label": "View usage"},
            {"action": "open_pricing_dialog", "label": "Upgrade"},
            {"action": "reset_usage", "label": "Reset usage"}
        ]),
        /*revision*/ 1,
    ));
    let rendered = render_bottom_popup(&chat, /*width*/ 90);
    assert!(rendered.contains("› 3. Reset usage"), "{rendered}");
    assert!(events.try_recv().is_err());
    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_matches!(events.try_recv(), Ok(AppEvent::OpenRateLimitResetCredits));
    println!("None");
}

/// Keep the waiting action selected when reset availability removes an earlier row.
/// Parameters: none.
/// Returns: None.
#[tokio::test]
async fn usage_wait_account_refresh_preserves_waiting_when_reset_disappears() {
    println!("parameters=none");
    let (mut chat, mut events, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
    chat.has_chatgpt_account = true;
    chat.plan_type = Some(PlanType::Plus);
    chat.bottom_pane.set_task_running(/*running*/ true);
    chat.update_usage_limit_wait(Some(chrono::Utc::now().timestamp_millis() + 60_000));
    assert_matches!(events.try_recv(), Ok(AppEvent::RefreshRateLimits { .. }));
    let _ = render_bottom_popup(&chat, /*width*/ 90);
    chat.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    chat.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let mut updated = response(json!([]), /*revision*/ 1);
    updated.rate_limit_reset_credits = Some(RateLimitResetCreditsSummary {
        available_count: 0,
        credits: None,
    });
    chat.update_backend_banner(&updated);
    let rendered = render_bottom_popup(&chat, /*width*/ 90);
    assert!(rendered.contains("› 2. Keep waiting"), "{rendered}");
    assert!(!rendered.contains("Redeem reset"), "{rendered}");
    assert!(events.try_recv().is_err());
    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_matches!(
        events.try_recv(),
        Ok(AppEvent::DismissUsageLimitWaitBanner { .. })
    );
    println!("None");
}

/// Disarm Enter when the selected recovery action disappears or changes destination.
/// Parameters: none.
/// Returns: None.
#[tokio::test]
async fn usage_wait_account_refresh_does_not_retarget_enter() {
    println!("parameters=none");
    for changed_destination in [false, true] {
        let (mut chat, mut events, _ops) = make_chatwidget_manual(Some("test-model-a")).await;
        chat.has_chatgpt_account = true;
        chat.plan_type = Some(PlanType::Plus);
        chat.bottom_pane.set_task_running(/*running*/ true);
        let retry_at = chrono::Utc::now().timestamp_millis() + 60_000;
        chat.update_usage_limit_wait(Some(retry_at));
        assert_matches!(events.try_recv(), Ok(AppEvent::RefreshRateLimits { .. }));
        let _ = render_bottom_popup(&chat, /*width*/ 90);
        chat.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        let mut updated = response(json!([]), /*revision*/ 1);
        if changed_destination {
            chat.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
            updated = response(
                json!([
                    {"action": "open_plus_pricing_web", "label": "Upgrade"}
                ]),
                /*revision*/ 1,
            );
        } else {
            updated.rate_limit_reset_credits = Some(RateLimitResetCreditsSummary {
                available_count: 0,
                credits: None,
            });
        }
        chat.update_backend_banner(&updated);
        let _ = render_bottom_popup(&chat, /*width*/ 90);
        chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(events.try_recv().is_err(), "refresh retargeted Enter");
        assert_eq!(chat.usage_limit_wait_retry_at_ms, Some(retry_at));
        assert!(render_bottom_popup(&chat, /*width*/ 90).contains("Resuming in "));
        chat.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        chat.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_matches!(events.try_recv(), Ok(AppEvent::OpenUrlInBrowser { .. }));
        assert!(events.try_recv().is_err());
    }
    println!("None");
}
