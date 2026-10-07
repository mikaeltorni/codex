# Installed CX deployment checks

Run `python3 scripts/install/verify_cx_deployment.py -v` from this checkout
with Python 3.11 or newer, after publishing the test-branch CLI and Code Mode
host through the setup repository's atomic publication helper. These checks
are read-only and start no agent turns, network requests or terminal windows.
Each unittest case rereads the registry and verifies its bytes are unchanged.

| Contract | Independently expected result | Executable case |
| --- | --- | --- |
| Installed revision marker against newTest1601 | Committed ancestor with identical runtime sources | test_committed_runtime |
| Published CLI and host against final build artifacts | Executable files with matching SHA-256 for both | test_committed_runtime |
| Every registered account's direct CLI | Workspace version and accepted quota-resume/step-switch configuration | test_registered_account_launchers |
| CX aliases and codexN wrappers | Existing ACC routing; codexN reaches same CLI version with explicit account home | test_registered_account_launchers |
| Public help parsing | Missing -c operand and unknown option rejected; subsequent --version still succeeds | test_cli_rejections |
| Every case including rejection | Registry and selected account unchanged immediately after probes | tearDown and inline registry observations |

## Shared validation

The existing Codex CLI owns argument parsing. These installed-runtime checks
cover representative missing-value and unknown-option paths; they do not add
new parsing rules. CLI help/version probes require no authentication and do
not modify conversations. Account credentials are never read. Existing TUI,
protocol, and recovery suites cover quota and banner behavior.
