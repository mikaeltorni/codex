# Usage-wait selection refresh regression

Authoritative ledger: `/home/mk/projects/codex/tmp/workflow/b1669b5bc8984ec4bceefe3b8f64443f.md`.
Current and only capability: “See whats wrong and keep it PR ready for sure”.

| Operation | Rejection class | Rejected public input | Validation owner | Expected rejection | Test and immediate public observations |
| --- | --- | --- | --- | --- | --- |
| Refresh selected reset | Success | Down, countdown, changed account copy, repeated wait deadline | Inline banner selection reconciliation | Reset remains highlighted and Enter opens reset picker once | usage_wait_account_refresh_preserves_selected_reset: rendered selected row/countdown and empty event queue after each refresh; snapshot and reset event |
| Refresh reordered/renamed reset | Success | Local Redeem reset replaced by backend Reset usage at new index | Semantic action identity | Selection follows reset, not row position or label | usage_wait_account_refresh_tracks_reordered_and_renamed_reset: selected row before Enter, reset event |
| Refresh selected Keep waiting | Success | Account read confirms zero reset credits | Inline banner selection reconciliation | Waiting remains highlighted despite an earlier row disappearing | usage_wait_account_refresh_preserves_waiting_when_reset_disappears: selected row/countdown then dismissal event |
| Initial menu before first frame | Success | Account response inserts recovery choices before rendering | Shown-banner provenance guard | Default remains first action until a selection has been shown | Retained backend_banners_tests.rs: usage_wait_banner_exposes_account_recovery_choices_while_task_runs and usage_wait_uses_team_recovery_with_nullable_account_banner, unchanged snapshots |
| Accept removed reset | Action unavailable | Enter after reset balance becomes zero | Inline banner activation guard | No replacement action emitted | usage_wait_account_refresh_does_not_retarget_enter, changed_destination=false: empty event queue immediately after Enter, deadline and countdown preserved, explicit navigation works |
| Accept changed URL with same label | Destination changed | Enter after Upgrade destination changes | Semantic action identity | No replacement URL emitted | Same case, changed_destination=true: empty event queue immediately after Enter, deadline/countdown preserved, explicit navigation works |

This repair owns key-event selection and background refresh, not command parsing. Operand shape, numeric conversion and textual dispatch are inapplicable. Independent cases construct fresh widgets; no real browser, owner request or reset is executed. No new public query is added. Retained deliberate-selection, key remapping, hidden-banner, draft and repeated-Enter regressions in unsolicited_upgrade_coverage.md remain cumulative.

Runner (task root, private fixtures and existing binary/cache):
`umask 077; env -u NO_COLOR TERM=xterm-256color CARGO_BIN_EXE_codex=/home/mk/.local/share/linux_codex_claude_code_setup/npm/bin/codex CARGO_TARGET_DIR=/home/mk/projects/.worktrees/codex/codex_chore-merge160-harnesses/codex-rs/target just test -p codex-tui usage_wait_account_refresh`

Baseline and final results are recorded in the authoritative plan and task tmp logs.

Observed verification: saved pre-repair runner executed 4 tests, 3 failed for losing selection and 1 existing activation-safety case passed. Final cumulative `just test -p codex-tui` executed 5,638 tests successfully, with 8 repository-profile skips. The new snapshot highlights Reset usage after coalesced account/wait refreshes; existing snapshots remain unchanged. `just clippy -p codex-tui --lib --no-deps` and `just fmt` passed. Only static invariant/lint adjustments followed the full test run; no tests were rerun after formatting, as directed by AGENTS.md.
