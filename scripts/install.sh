#!/bin/bash
# AgenticComm install wrapper — delegates to installer/acomm_installer/install.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

exec bash "${REPO_ROOT}/installer/acomm_installer/install.sh" "$@"
