# Reserve switch regression inventory
Runner: from codex-rs, CARGO_TARGET_DIR=$HOME/.cache/linux_codex_claude_code_setup/codex-main/target just test --release --locked --lib -p codex-features -p codex-tui -E 'test(reserve_auto_switch) or test(luna_reserve) or test(backend_banner)'. Use cached checksum-verified V8 archive and binding environment paths. All cases create fresh app/widget state.

| Interface/rule | Saved executable case | Expected observation |
| --- | --- | --- |
| TOML feature registry | features::reserve_auto_switch_toml_resolves | Recognized key; default true; false disables; true restores; invalid TOML bool rejected by features parser. |
| Account usage RPC with Reserve available | app::luna_reserve_switch_config_controls_settings_rpc | false retains gpt-5.5 and high effort with zero settings RPC and no return cache; true switches once to gpt-reserve. |
| Banner offering non-Reserve fallback | same RPC test | false still switches to ordinary fallback, preserving existing behavior. |
| Resumed/manual Reserve model picker | chatwidget::reserve_auto_switch_disabled_keeps_manual_models_available | false makes picker unrestricted and prevents ordinary-model submission deferral; true preserves restriction. |
| Existing automatic recovery and task wait | existing luna_reserve and backend_banner suites | Previous return-model, queued-turn, account identity and permission checks remain intact. |
| CLI missing/extra/unknown feature | existing CLI feature parsing and features::reserve_auto_switch_toml_resolves | Generic registry/CLI validation owner remains unchanged; no custom parser introduced. |
