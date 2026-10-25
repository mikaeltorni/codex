#!/usr/bin/env bash
# Bridge the common component menu to the fork's independently usable Python CLI.
set -euo pipefail
if (( $# != 1 )); then printf 'Expected the resolved target home.\n' >&2; exit 2; fi
CODEX_INSTALL_HOME="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ISC_RECEIPT_DIR="$CODEX_INSTALL_HOME/.local/state/isc/receipts"
source "$SCRIPT_DIR/.installer-support/bootstrap.sh"
isc_activate_components

# Install the native fork using the same public selection as noninteractive callers.
# Parameters: none. Returns: native installer status.
codex_menu_install() {
  printf 'parameters=none\n' >&2
  local status=0
  python3 "$SCRIPT_DIR/scripts/install/install_fork.py" --home "$CODEX_INSTALL_HOME" --select codex_cli || status=$?
  printf '%s\n' "$status" >&2
  return "$status"
}

# Detect both native fork entrypoints without building or changing configuration.
# Parameters: none. Returns: zero when both are executable, one otherwise.
codex_menu_detect() {
  printf 'parameters=none\n' >&2
  local status=1
  if [[ -x "$CODEX_INSTALL_HOME/.local/bin/codex" && -x "$CODEX_INSTALL_HOME/.local/bin/codex-code-mode-host" ]]; then status=0; fi
  printf '%s\n' "$status" >&2
  return "$status"
}

# Remove only owned fork binaries through the same public removal operation.
# Parameters: none. Returns: native installer status.
codex_menu_uninstall() {
  printf 'parameters=none\n' >&2
  local status=0
  python3 "$SCRIPT_DIR/scripts/install/install_fork.py" --home "$CODEX_INSTALL_HOME" --uninstall codex_cli || status=$?
  printf '%s\n' "$status" >&2
  return "$status"
}

ISC_REPO_NAME=codex
ISC_REPO_LABEL='Codex fork'
ISC_COMPONENTS=("codex_cli|Codex CLI and Code Mode host|on|codex_menu_install|codex_menu_detect|codex_menu_uninstall")
component_main
