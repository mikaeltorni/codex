# Account-banner merge regression coverage

Run from the repository root with `just test -p codex-tui`.
The task uses `CARGO_TARGET_DIR=/home/mk/projects/.worktrees/codex/codex_chore-merge160-harnesses/codex-rs/target`
to reuse the existing test cache. Each case creates a fresh ChatWidget fixture.

| Input and state | Expected public observation | Saved executable case in backend_banners_tests.rs |
| --- | --- | --- |
| Model B; dismiss an unrelated inline notice; unchanged Model A account banner; switch to A | Account banner appears, despite unrelated dismissal | unrelated_inline_banner_dismissal_preserves_hidden_account_banner; repeats with fresh state and snapshots restored UI |
| Display dismissible account banner; Escape; change copy/model | Same occurrence stays dismissed until authoritative replacement | backend_banner_dismissal_tracks_occurrence_not_copy_or_model |
| Display account banner; start quota wait; update account banner; end wait | Countdown persists during wait; updated account banner returns afterward | usage_wait_countdown_stays_visible_and_restores_updated_account_banner |
| Reserve recovery notice; quota wait; reset picker open or closed | Countdown replaces notice while preserving reset picker | usage_wait_replaces_reserve_notice_but_preserves_reset_picker |
| New turn with shown/unshown account content | Only shown dismissible account content disappears | backend_banner_new_turn_dismisses_only_shown_dismissible_content |
| Pro plan (and ProMax in the release integration) reaches quota | Personal usage limit menu; reset action; no upgrade request | usage_wait_identifies_personal_and_workspace_accounts |

## Shared input validation

This repair changes account-banner lifecycle observation, not command dispatch
or operand parsing. Missing/extra operands, numeric conversions and unknown
commands are outside its interface and remain owned by the existing CLI/TUI
parsers. Escape and model changes are exercised through the existing key-event
and ChatWidget interfaces, with rendered UI as the state observation.
