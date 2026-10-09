# Recovery typing regression

Feature: `Lets fix it properly`; subject `fix: keep ordinary typing out of recovery actions`.

| Operation | Case class | Public input and expected result | Validation owner | Saved test/assertion and immediate observations |
| --- | --- | --- | --- | --- |
| Async recovery | Ordinary text rejection | Fresh Plus account with Upgrade; type jk, 1, k, j on four idle/running/Reserve surfaces; no recovery events, inline text retained; Enter emits no recovery | SelectionActivation and inline key dispatcher | usage_wait_selection_tests::recovery_actions_do_not_capture_ordinary_typing; event queue and rendered Upgrade after each key; composer read before Enter |
| Recovery shortcut | Success | Alt+1 emits exactly pricing URL once on all four surfaces; subsequent Enter emits nothing | Same activation owner | Same test; exact URL, empty event queue and composer |
| Recovery navigation | Retained success/rejection | Arrows/Enter allowed once; polls, repeats, hidden views and drafts rejected; stable selections preserved across refresh | Existing shared activation and list owner | Retained backend_banners_tests, actionable_banner_tests and usage_wait_selection_tests |

No new public query or state read: older rejection blocks already observe event/render/composer state. Blank/operand/numeric conversion classes inapplicable to terminal key events. External browser, email and billing actions captured only.

The saved pre-repair regression failed on ordinary `jk` then Enter activating recovery. First post-repair text preservation used the unflushed paste buffer; the fixture now disables paste detection for deterministic keystrokes. Composer submission is allowed to emit its ordinary events; the assertion rejects browser, owner request, reset picker and wait-dismissal effects explicitly. Existing selection preservation and 81 CTA fixtures are retained.

The same regression now also checks Ctrl+j/k and Ctrl+n/p, which are composer editing/history keys on the default keymap. These must not arm recovery Enter; public render/events are observed after each input. Non-text arrow/function navigation remains the explicit selection interface.
