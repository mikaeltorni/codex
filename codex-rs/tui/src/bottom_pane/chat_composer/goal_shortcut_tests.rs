use super::tests::new_test_composer;
use super::tests::snapshot_composer_state_with_width;
use super::*;
use codex_config::types::TuiKeymap;
use pretty_assertions::assert_eq;

/// Check prefix insertion and removal through native key events without submitting.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_toggles_without_submitting() {
    println!("parameters=none");
    for (text, expected) in [
        ("", "/goal "),
        ("hello\n世界", "/goal hello\n世界"),
        ("/goal hello", "hello"),
        ("/goal", ""),
        ("/goal ", ""),
        ("/goal\thello", "hello"),
        ("/goal\nhello", "hello"),
        ("/goal\u{2003}世界", "世界"),
        ("/goal  hello", " hello"),
        ("/goal resume", "resume"),
        ("/goalkeeper", "/goal /goalkeeper"),
        ("/goal/hello", "/goal /goal/hello"),
        ("ask /goal later", "/goal ask /goal later"),
        ("!echo hello", "/goal !echo hello"),
    ] {
        let (mut composer, mut events) = new_test_composer();
        composer.set_goal_command_enabled(/*enabled*/ true);
        composer.set_text_content(text.into(), Vec::new(), Vec::new());
        let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT);
        assert_eq!(composer.handle_key_event(key).0, InputResult::None);
        assert_eq!(composer.current_text(), expected);
        assert!(events.try_recv().is_err());
    }
    println!("None");
}

/// Check that atomic paste content and image attachments survive both prefix edits.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_preserves_pastes_and_attachments() {
    println!("parameters=none");
    let (mut composer, _events) = new_test_composer();
    composer.set_goal_command_enabled(/*enabled*/ true);
    composer.handle_paste("x".repeat(LARGE_PASTE_CHAR_THRESHOLD + 1));
    composer.attach_image(crate::test_support::test_path_buf("image.png"));
    composer.set_remote_image_urls(vec!["https://example.com/remote.png".into()]);
    let before = composer.snapshot_draft();
    composer.handle_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT));
    let mut expected = before.clone();
    expected.text.insert_str(/*idx*/ 0, "/goal ");
    expected.cursor += 6;
    for element in &mut expected.text_elements {
        element.byte_range.start += 6;
        element.byte_range.end += 6;
    }
    // A recognized slash command becomes atomic through normal popup synchronization.
    expected.text_elements.insert(
        /*index*/ 0,
        TextElement::new((0..5).into(), Some("/goal".into())),
    );
    assert_eq!(composer.snapshot_draft(), expected);
    composer.handle_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT));
    assert_eq!(composer.snapshot_draft(), before);
    println!("None");
}

/// Check buffered input is flushed before the prefix edit rather than lost.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_flushes_buffered_typing() {
    println!("parameters=none");
    for (text, expected) in [("", "/goal x"), ("/goal ", "x")] {
        let (mut composer, _events) = new_test_composer();
        composer.set_goal_command_enabled(/*enabled*/ true);
        composer.set_text_content(text.into(), Vec::new(), Vec::new());
        composer.set_current_cursor(text.len());
        composer.handle_key_event(KeyCode::Char('x').into());
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT));
        assert_eq!(composer.current_text(), expected);
    }
    println!("None");
}

/// Check feature/input guards and release events leave populated drafts intact.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_respects_input_guards() {
    println!("parameters=none");
    for guard in ["feature", "disabled", "parent", "plain", "release"] {
        let (mut composer, _events) = new_test_composer();
        composer.set_goal_command_enabled(guard != "feature");
        composer.set_text_content("/goal draft".into(), Vec::new(), Vec::new());
        match guard {
            "disabled" => {
                composer.set_input_enabled(/*enabled*/ false, /*placeholder*/ None)
            }
            "parent" => composer.set_parent_owned_thread(),
            "plain" => composer.config = ChatComposerConfig::plain_text(),
            _ => {}
        }
        let key = KeyEvent::new_with_kind(
            KeyCode::Char('g'),
            KeyModifiers::ALT,
            if guard == "release" {
                KeyEventKind::Release
            } else {
                KeyEventKind::Press
            },
        );
        composer.handle_key_event(key);
        assert_eq!(composer.current_text(), "/goal draft");
    }
    println!("None");
}

/// Check prefix removal keeps Unicode cursor offsets and participates in Vim undo and redo.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_removal_preserves_cursor_and_vim_history() {
    println!("parameters=none");
    for cursor in [0, 5, 6, 12, 15, 18] {
        let (mut composer, _events) = new_test_composer();
        composer.set_goal_command_enabled(/*enabled*/ true);
        composer.set_vim_enabled(/*enabled*/ true);
        composer.set_text_content("/goal hello 世界".into(), Vec::new(), Vec::new());
        composer.handle_key_event(KeyCode::Esc.into());
        composer.set_current_cursor(cursor);
        let before = composer.snapshot_draft();
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT));
        assert_eq!(composer.current_text(), "hello 世界");
        assert_eq!(composer.current_cursor(), cursor.saturating_sub(6));
        composer.handle_key_event(KeyCode::Char('u').into());
        assert_eq!(composer.snapshot_draft(), before);
        // Undo may reopen slash completion; dismiss it before its keys reach Vim redo.
        composer.handle_key_event(KeyCode::Esc.into());
        assert_eq!(composer.snapshot_draft(), before);
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert_eq!(composer.current_text(), "hello 世界");
        assert_eq!(composer.current_cursor(), cursor.saturating_sub(6));
    }
    println!("None");
}

/// Check remapped prefix keys remove existing prefixes while disabled bindings preserve them.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_removal_obeys_custom_binding() {
    println!("parameters=none");
    for binding in ["\"f6\"", "[]"] {
        let config: TuiKeymap =
            toml::from_str(&format!("[composer]\nprepend_goal = {binding}")).unwrap();
        let keymap = RuntimeKeymap::from_config(&config).unwrap();
        let (mut composer, _events) = new_test_composer();
        composer.set_goal_command_enabled(/*enabled*/ true);
        composer.set_keymap_bindings(&keymap);
        composer.set_text_content("/goal draft".into(), Vec::new(), Vec::new());
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT));
        assert_eq!(composer.current_text(), "/goal draft");
        composer.handle_key_event(KeyCode::F(6).into());
        assert_eq!(
            composer.current_text(),
            if binding == "[]" {
                "/goal draft"
            } else {
                "draft"
            }
        );
    }
    println!("None");
}

/// Check a native remap replaces the default binding and explicit unbinding works.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_respects_custom_binding_and_unbinding() {
    for binding in ["\"f6\"", "[]"] {
        let config: TuiKeymap =
            toml::from_str(&format!("[composer]\nprepend_goal = {binding}")).unwrap();
        let keymap = RuntimeKeymap::from_config(&config).unwrap();
        let (mut composer, _events) = new_test_composer();
        composer.set_goal_command_enabled(/*enabled*/ true);
        composer.set_keymap_bindings(&keymap);
        composer.set_text_content("draft".into(), Vec::new(), Vec::new());
        composer.handle_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT));
        assert_eq!(composer.current_text(), "draft");
        composer.handle_key_event(KeyCode::F(6).into());
        assert_eq!(
            composer.current_text(),
            if binding == "[]" {
                "draft"
            } else {
                "/goal draft"
            }
        );
    }
}

/// Snapshot the visible composer after a native goal-prefix edit.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_composer_snapshot() {
    snapshot_composer_state_with_width(
        "goal_prefix_composer",
        /*width*/ 60,
        /*enhanced_keys_supported*/ false,
        |composer| {
            composer.set_goal_command_enabled(/*enabled*/ true);
            composer.set_text_content("Improve test coverage".into(), Vec::new(), Vec::new());
            composer.handle_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT));
        },
    );
}

/// Check conflicting remaps are rejected rather than shadowing another composer action.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_rejects_conflicting_binding() {
    let config: TuiKeymap = toml::from_str("[composer]\nprepend_goal = \"enter\"").unwrap();
    let error = RuntimeKeymap::from_config(&config).unwrap_err();
    assert!(error.contains("prepend_goal"));
    assert!(error.contains("submit"));
}

/// Check a prefix edit retains the original cursor offset and supports native Vim undo.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_preserves_cursor_and_vim_undo() {
    let (mut composer, _events) = new_test_composer();
    composer.set_goal_command_enabled(/*enabled*/ true);
    composer.set_vim_enabled(/*enabled*/ true);
    composer.set_text_content("hello world".into(), Vec::new(), Vec::new());
    composer.handle_key_event(KeyCode::Esc.into());
    composer.set_current_cursor(/*cursor*/ 3);
    composer.handle_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::ALT));
    assert_eq!(
        (composer.current_text(), composer.current_cursor()),
        ("/goal hello world".into(), 9)
    );
    composer.handle_key_event(KeyCode::Char('u').into());
    assert_eq!(composer.current_text(), "hello world");
}

/// Check new defaults yield to existing customized keys and chord prefixes.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_preserves_existing_shortcuts() {
    for binding in ["alt-g", "alt-g x"] {
        let config: TuiKeymap =
            toml::from_str(&format!("[editor]\nmove_line_start = \"{binding}\"")).unwrap();
        let keymap = RuntimeKeymap::from_config(&config).unwrap();
        assert_eq!(
            keymap.primary_hint(KeymapContext::Composer, "prepend_goal"),
            None
        );
    }
}

/// Check a configured chord routes through native dispatch to the prefix operation.
/// Parameters: none. Returns: None.
#[test]
fn goal_prefix_custom_chord_dispatches() {
    let config: TuiKeymap = toml::from_str("[composer]\nprepend_goal = \"ctrl-x g\"").unwrap();
    let keymap = RuntimeKeymap::from_config(&config).unwrap();
    let (mut composer, _events) = new_test_composer();
    composer.set_goal_command_enabled(/*enabled*/ true);
    composer.set_keymap_bindings(&keymap);
    composer.set_text_content("draft".into(), Vec::new(), Vec::new());
    let mut matcher = crate::keymap::KeyChordMatcher::default();
    assert!(matches!(
        matcher.advance(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            &keymap.chords,
            composer.keymap_contexts()
        ),
        crate::keymap::KeyChordMatch::Pending(_)
    ));
    let crate::keymap::KeyChordMatch::Completed(event) = matcher.advance(
        KeyCode::Char('g').into(),
        &keymap.chords,
        composer.keymap_contexts(),
    ) else {
        panic!("goal prefix chord");
    };
    assert_eq!(composer.handle_key_event(event).0, InputResult::None);
    assert_eq!(composer.current_text(), "/goal draft");
}
