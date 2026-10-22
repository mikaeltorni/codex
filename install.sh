#!/usr/bin/env bash
# Install this fork's checked-out source independently of other setup projects.
set -euo pipefail
if [[ ${EUID} == 0 && -n ${SUDO_USER:-} && $SUDO_USER != root ]]; then
  exec sudo -H -u "$SUDO_USER" bash "$0" "$@"
fi
exec python3 "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/scripts/install/install_fork.py" "$@"
