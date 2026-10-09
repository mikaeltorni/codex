use super::*;

/// Build the account-3 Plus-plan pricing notice reported during background status polling.
/// Parameters: none.
/// Returns: the account response with the same Upgrade destination as the incident.
fn plus_upgrade_response() -> codex_app_server_protocol::GetAccountRateLimitsResponse {
    println!("parameters=none");
    let response = serde_json::from_value(serde_json::json!({
        "accountId": "test-account", "rateLimits": {"planType": "plus"},
        "rateLimitUpsell": {
            "banner_type": "luna_reserve", "presentation": "dismissible",
            "title": "Usage limit reached",
            "description": "Upgrade to continue using the advanced models.",
            "ctas": [{"action": "open_pricing_dialog", "label": "Upgrade"}]
        }
    }))
    .unwrap();
    println!("{response:?}");
    response
}

/// Keep an asynchronous account response from treating a status command's Enter as Upgrade.
/// Parameters: none.
/// Returns: None.
#[tokio::test]
async fn account_refresh_does_not_turn_status_input_into_upgrade() {
    println!("parameters=none");
    for command in ["/status", "/usage"] {
        for prefix_len in [0, 3, command.len()] {
            let (mut chat, mut events, _ops) = make_chatwidget_manual(Some("gpt-6.1-sol")).await;
            chat.has_chatgpt_account = true;
            chat.plan_type = Some(PlanType::Plus);
            chat.apply_external_edit(command[..prefix_len].to_string());
            chat.update_backend_banner(&plus_upgrade_response());
            let _ = render_bottom_popup(&chat, /*width*/ 90);
            for character in command[prefix_len..].chars() {
                chat.handle_key_event(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));
            }
            chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
            let queued = std::iter::from_fn(|| events.try_recv().ok()).collect::<Vec<_>>();
            assert!(
                !queued
                    .iter()
                    .any(|event| matches!(event, AppEvent::OpenUrlInBrowser { .. })),
                "Account refresh hijacked {command} at {prefix_len}: {queued:?}"
            );
        }
    }
    println!("None");
}

/// Preserve a draft and route only an explicitly chosen visible Upgrade shortcut to the browser.
/// Parameters: none.
/// Returns: None.
#[tokio::test]
async fn account_upgrade_notice_preserves_draft_and_explicit_action() {
    println!("parameters=none");
    let (mut chat, mut events, _ops) = make_chatwidget_manual(Some("gpt-6.1-sol")).await;
    chat.has_chatgpt_account = true;
    chat.plan_type = Some(PlanType::Plus);
    chat.apply_external_edit("preserved draft".to_string());
    chat.update_backend_banner(&plus_upgrade_response());
    let rendered = render_bottom_popup(&chat, /*width*/ 90);
    assert!(rendered.contains("preserved draft"), "{rendered}");
    assert!(rendered.contains("Upgrade"), "{rendered}");
    assert_eq!(chat.composer_text_with_pending(), "preserved draft");
    chat.apply_external_edit(String::new());
    let _ = render_bottom_popup(&chat, /*width*/ 90);
    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(
        !std::iter::from_fn(|| events.try_recv().ok())
            .any(|event| matches!(event, AppEvent::OpenUrlInBrowser { .. }))
    );
    assert_eq!(chat.composer_text_with_pending(), "");
    chat.handle_key_event(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE));
    assert!(
        std::iter::from_fn(|| events.try_recv().ok()).any(|event| matches!(
            event,
            AppEvent::OpenUrlInBrowser { url }
                if url == "https://chatgpt.com/?cta_tab=personal&highlight_plan=pro#pricing"
        ))
    );
    assert_eq!(chat.composer_text_with_pending(), "");
    println!("None");
}
