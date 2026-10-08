# Unsolicited upgrade regression

Authoritative capability ledger: `/home/mk/projects/codex/tmp/workflow/0e2527d5ef3147a39edb8207b933c05c.md`.
Current capability: “See whats wrong and fix it completely.” No deferred capabilities.

| Input/state | Expected result | Saved public-path assertions |
| --- | --- | --- |
| Plus account, quota wait; local fallback and backend Upgrade CTA | Rendering/tick emits no browser action; plain Enter leaves wait deadline/menu intact | backend_banners_tests.rs: usage_wait_requires_explicit_navigation_before_enter_opens_upgrade |
| Explicit Down/Up, tick, Enter | Pricing browser event once; wait deadline unchanged | Same case |
| Repeated Enter after deliberate selection | No duplicate browser event; deadline and visible Upgrade preserved | Same case |
| Numbered Upgrade shortcut then Enter | Shortcut opens pricing once; Enter does not reopen; deadline unchanged | Same case |
| Custom navigation/accept keys | Existing remapped selection remains functional | actionable_banner_tests.rs: running_banner_uses_remapped_navigation_and_accept |
| Countdown tick and unchanged account polling | Explicit navigation selection preserved | backend_banners_tests.rs: usage_wait_countdown_tick_preserves_selected_recovery_action |
| Hidden menu, populated draft, key release/repeat | Composer input preserved, no recovery side effects; hiding consumes prior navigation | actionable_banner_tests.rs: running_banner_requires_navigation_before_accepting_recovery_action |
| Clipped actions and existing shortcut overlay | Hidden action numbers remain composer input; Escape closes the overlay first | actionable_banner_tests.rs: partially_clipped_banner_preserves_hidden_action_shortcuts; bottom_pane/mod.rs: escape_closes_shortcut_help_before_inline_banner |
| Rendered recovery menu | Existing appearance, action order and remapped hints remain unchanged | Existing running_banner_remapped_keys and usage_wait_preserves_selected_action insta snapshots |

Validation boundary: this repair owns key-event routing, not textual command/operand parsing. Empty/unknown commands, numeric conversion, and missing/extra operands are inapplicable; the existing CLI/composer parsers are unchanged. Every independent test builds fresh widget/pane state. Browser launch is observed as an app event; tests never open a real browser.

Observed baseline: each new regression fails on the original application (unselected Enter/accept emitted a browser event). Final cumulative run: 5632 TUI tests passed, 8 skipped by the repository profile. Existing popup/paste-burst guards are preserved; this change does not alter those parsers.
