use super::*;
use pretty_assertions::assert_eq;

/// Build an independently specified goal notification for shortcut tests.
/// Parameters: thread_id - owning thread; status - observed lifecycle state. Returns: the goal.
fn shortcut_goal(thread_id: ThreadId, status: AppThreadGoalStatus) -> AppThreadGoal {
    AppThreadGoal {
        thread_id: thread_id.to_string(),
        objective: "Finish the task".into(),
        status,
        token_budget: Some(80_000),
        tokens_used: 12_500,
        time_used_seconds: 90,
        created_at: 1_776_272_400,
        updated_at: 1_776_272_460,
    }
}

/// Check both enhanced and legacy shifted letters resume through the native lifecycle event.
/// Parameters: none. Returns: None.
#[tokio::test]
async fn goal_resume_shortcut_preserves_complete_draft() {
    for status in [
        AppThreadGoalStatus::Paused,
        AppThreadGoalStatus::Blocked,
        AppThreadGoalStatus::UsageLimited,
    ] {
        for key in [
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT | KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Char('G'), KeyModifiers::ALT),
            KeyEvent::new(KeyCode::Char('G'), KeyModifiers::ALT | KeyModifiers::SHIFT),
        ] {
            let (mut chat, mut rx, mut op_rx) =
                make_chatwidget_manual(/*model_override*/ None).await;
            chat.set_feature_enabled(Feature::Goals, /*enabled*/ true);
            let thread_id = ThreadId::new();
            chat.thread_id = Some(thread_id);
            chat.on_thread_goal_updated(shortcut_goal(thread_id, status), /*turn_id*/ None);
            chat.bottom_pane.set_composer_text(
                "draft [paste]".into(),
                vec![TextElement::new((6..13).into(), Some("[paste]".into()))],
                vec![PathBuf::from("goal.png")],
            );
            chat.set_remote_image_urls(vec!["https://example.com/goal.png".into()]);
            chat.bottom_pane
                .set_composer_pending_pastes(vec![("[paste]".into(), "full text".into())]);
            let before = (
                chat.bottom_pane.composer_text(),
                chat.bottom_pane.composer_cursor(),
                chat.bottom_pane.composer_text_elements(),
                chat.bottom_pane.composer_local_images(),
                chat.remote_image_urls(),
                chat.bottom_pane.composer_pending_pastes(),
            );
            while rx.try_recv().is_ok() {}
            chat.handle_key_event(key);
            assert_matches!(rx.try_recv(), Ok(AppEvent::SetThreadGoalStatus {
                thread_id: actual, status: AppThreadGoalStatus::Active }) if actual == thread_id);
            assert!(rx.try_recv().is_err());
            assert_eq!(
                (
                    chat.bottom_pane.composer_text(),
                    chat.bottom_pane.composer_cursor(),
                    chat.bottom_pane.composer_text_elements(),
                    chat.bottom_pane.composer_local_images(),
                    chat.remote_image_urls(),
                    chat.bottom_pane.composer_pending_pastes()
                ),
                before
            );
            assert!(!chat.bottom_pane.has_active_view());
            assert_no_submit_op(&mut op_rx);
        }
    }
}

/// Check no goal, terminal statuses, startup, read-only views and non-press events are inert.
/// Parameters: none. Returns: None.
#[tokio::test]
async fn goal_resume_shortcut_rejects_ineligible_state_without_editing() {
    for guard in [
        "missing", "active", "complete", "budget", "feature", "thread", "stale", "parent",
        "disabled", "repeat", "release", "modal", "popup",
    ] {
        let (mut chat, mut rx, mut op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
        chat.set_feature_enabled(Feature::Goals, /*enabled*/ true);
        let thread_id = ThreadId::new();
        chat.thread_id = Some(thread_id);
        let status = match guard {
            "active" => AppThreadGoalStatus::Active,
            "complete" => AppThreadGoalStatus::Complete,
            "budget" => AppThreadGoalStatus::BudgetLimited,
            _ => AppThreadGoalStatus::Paused,
        };
        if guard != "missing" {
            chat.on_thread_goal_updated(shortcut_goal(thread_id, status), /*turn_id*/ None);
        }
        chat.bottom_pane
            .set_composer_text("draft".into(), Vec::new(), Vec::new());
        match guard {
            "feature" => {
                chat.set_feature_enabled(Feature::Goals, /*enabled*/ false);
            }
            "thread" => chat.thread_id = None,
            "stale" => chat.thread_id = Some(ThreadId::new()),
            "parent" => chat.blocks_direct_input = true,
            "disabled" => chat
                .bottom_pane
                .set_composer_input_enabled(/*enabled*/ false, /*placeholder*/ None),
            "modal" => chat.show_resume_paused_goal_prompt(thread_id, "Finish the task".into()),
            "popup" => chat
                .bottom_pane
                .set_composer_text("/go".into(), Vec::new(), Vec::new()),
            _ => {}
        }
        while rx.try_recv().is_ok() {}
        let before = (
            chat.bottom_pane.composer_text(),
            chat.bottom_pane.composer_cursor(),
        );
        let kind = match guard {
            "repeat" => KeyEventKind::Repeat,
            "release" => KeyEventKind::Release,
            _ => KeyEventKind::Press,
        };
        chat.handle_key_event(KeyEvent::new_with_kind(
            KeyCode::Char('G'),
            KeyModifiers::ALT,
            kind,
        ));
        assert_eq!(
            (
                chat.bottom_pane.composer_text(),
                chat.bottom_pane.composer_cursor()
            ),
            before
        );
        assert!(
            !std::iter::from_fn(|| rx.try_recv().ok())
                .any(|event| matches!(event, AppEvent::SetThreadGoalStatus { .. }))
        );
        assert_no_submit_op(&mut op_rx);
    }
}

/// Check lowercase Alt+G edits the draft while resume preserves even pending buffered input.
/// Parameters: none. Returns: None.
#[tokio::test]
async fn goal_resume_shortcut_is_distinct_from_prefix_and_preserves_pending_input() {
    let (mut chat, mut rx, mut op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.set_feature_enabled(Feature::Goals, /*enabled*/ true);
    let thread_id = ThreadId::new();
    chat.thread_id = Some(thread_id);
    chat.on_thread_goal_updated(
        shortcut_goal(thread_id, AppThreadGoalStatus::Paused),
        /*turn_id*/ None,
    );
    chat.bottom_pane
        .set_composer_text("draft".into(), Vec::new(), Vec::new());
    while rx.try_recv().is_ok() {}
    chat.handle_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT));
    assert_eq!(chat.bottom_pane.composer_text(), "/goal draft");
    assert!(
        !std::iter::from_fn(|| rx.try_recv().ok())
            .any(|event| matches!(event, AppEvent::SetThreadGoalStatus { .. }))
    );
    chat.bottom_pane
        .set_composer_text(String::new(), Vec::new(), Vec::new());
    chat.handle_key_event(KeyCode::Char('x').into());
    let before = chat.bottom_pane.composer_text_with_pending();
    assert_eq!(before, "");
    assert!(chat.bottom_pane.is_in_paste_burst());
    while rx.try_recv().is_ok() {}
    chat.handle_key_event(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::ALT));
    assert_eq!(chat.bottom_pane.composer_text_with_pending(), before);
    assert!(chat.bottom_pane.is_in_paste_burst());
    assert_matches!(
        rx.try_recv(),
        Ok(AppEvent::SetThreadGoalStatus {
            status: AppThreadGoalStatus::Active,
            ..
        })
    );
    tokio::time::sleep(Duration::from_millis(/*millis*/ 100)).await;
    chat.bottom_pane.flush_paste_burst_if_due();
    assert_eq!(chat.bottom_pane.composer_text(), "x");
    assert_no_submit_op(&mut op_rx);
}

/// Check remapping and explicit unbinding route through the existing native keymap update.
/// Parameters: none. Returns: None.
#[tokio::test]
async fn goal_resume_shortcut_customization_and_running_turn() {
    for binding in ["\"f6\"", "[]"] {
        let config = toml::from_str(&format!("[chat]\nresume_goal = {binding}")).unwrap();
        let keymap = crate::keymap::RuntimeKeymap::from_config(&config).unwrap();
        let (mut chat, mut rx, mut op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
        chat.apply_keymap_update(config, &keymap);
        chat.set_feature_enabled(Feature::Goals, /*enabled*/ true);
        let thread_id = ThreadId::new();
        chat.thread_id = Some(thread_id);
        chat.on_thread_goal_updated(
            shortcut_goal(thread_id, AppThreadGoalStatus::Paused),
            /*turn_id*/ None,
        );
        chat.bottom_pane.set_task_running(/*running*/ true);
        chat.bottom_pane
            .set_composer_text("draft".into(), Vec::new(), Vec::new());
        while rx.try_recv().is_ok() {}
        chat.handle_key_event(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::ALT));
        assert_eq!(chat.bottom_pane.composer_text(), "draft");
        assert!(rx.try_recv().is_err());
        chat.handle_key_event(KeyCode::F(6).into());
        assert_eq!(chat.bottom_pane.composer_text(), "draft");
        if binding == "[]" {
            assert!(rx.try_recv().is_err());
        } else {
            assert_matches!(
                rx.try_recv(),
                Ok(AppEvent::SetThreadGoalStatus {
                    status: AppThreadGoalStatus::Active,
                    ..
                })
            );
        }
        assert_no_submit_op(&mut op_rx);
    }
}

/// Check existing remaps win over a new default and explicitly conflicting remaps reject.
/// Parameters: none. Returns: None.
#[test]
fn goal_resume_shortcut_keymap_conflicts_and_chords() {
    for binding in ["alt-shift-g", "alt-shift-g x"] {
        let config = toml::from_str(&format!("[editor]\nmove_line_start = \"{binding}\"")).unwrap();
        let keymap = crate::keymap::RuntimeKeymap::from_config(&config).unwrap();
        assert_eq!(
            keymap.primary_hint(crate::keymap::KeymapContext::Chat, "resume_goal"),
            None
        );
    }
    let config =
        toml::from_str("[chat]\nresume_goal = \"alt-g\"\n[composer]\nprepend_goal = \"alt-g\"")
            .unwrap();
    let error = crate::keymap::RuntimeKeymap::from_config(&config).unwrap_err();
    assert!(error.contains("resume_goal") && error.contains("prepend_goal"));
    let config = toml::from_str("[chat]\nresume_goal = \"ctrl-x g\"").unwrap();
    let keymap = crate::keymap::RuntimeKeymap::from_config(&config).unwrap();
    let mut matcher = crate::keymap::KeyChordMatcher::default();
    let context = crate::keymap::KeymapContextSet::new(crate::keymap::KeymapContext::Chat);
    assert_matches!(
        matcher.advance(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            &keymap.chords,
            context
        ),
        crate::keymap::KeyChordMatch::Pending(_)
    );
    let crate::keymap::KeyChordMatch::Completed(key) =
        matcher.advance(KeyCode::Char('g').into(), &keymap.chords, context)
    else {
        panic!("resume goal chord");
    };
    // Runtime action bindings include the internal token; user-facing bindings hide it.
    assert!(keymap.chat.resume_goal.is_pressed(key));
}

/// Check explicit goal remaps cannot steal existing textarea editing bindings.
/// Parameters: none. Returns: None.
#[test]
fn goal_resume_shortcut_rejects_editor_collisions() {
    for config in [
        "[chat]\nresume_goal = \"ctrl-a\"",
        "[composer]\nprepend_goal = \"ctrl-a\"",
    ] {
        let config = toml::from_str(config).unwrap();
        let error = crate::keymap::RuntimeKeymap::from_config(&config)
            .expect_err("goal action must not shadow the editor");
        assert!(error.contains("goal") && error.contains("move_line_start"));
    }
}

/// Check a remapped resume key still reaches slash completion while its popup owns input.
/// Parameters: none. Returns: None.
#[tokio::test]
async fn goal_resume_shortcut_preserves_popup_key_routing() {
    let config = toml::from_str("[chat]\nresume_goal = \"tab\"\n[composer]\nqueue = []").unwrap();
    let keymap = crate::keymap::RuntimeKeymap::from_config(&config).unwrap();
    let (mut chat, mut rx, mut op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.apply_keymap_update(config, &keymap);
    chat.set_feature_enabled(Feature::Goals, /*enabled*/ true);
    let thread_id = ThreadId::new();
    chat.thread_id = Some(thread_id);
    chat.on_thread_goal_updated(
        shortcut_goal(thread_id, AppThreadGoalStatus::Paused),
        /*turn_id*/ None,
    );
    chat.bottom_pane
        .set_composer_text("/go".into(), Vec::new(), Vec::new());
    while rx.try_recv().is_ok() {}
    chat.handle_key_event(KeyCode::Tab.into());
    assert_eq!(chat.bottom_pane.composer_text(), "/goal ");
    assert!(
        !std::iter::from_fn(|| rx.try_recv().ok())
            .any(|event| matches!(event, AppEvent::SetThreadGoalStatus { .. }))
    );
    assert_no_submit_op(&mut op_rx);
}
