#!/usr/bin/env bash
# check-runtime-hardening.sh — Validate runtime hardening requirements
set -euo pipefail

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

find_fixed() {
  if command -v rg >/dev/null 2>&1; then
    rg -nF "$1" "$2"
  else
    grep -R -n -F -- "$1" "$2"
  fi
}

assert_contains() {
  find_fixed "$1" "$2" >/dev/null || fail "Missing pattern '$1' in $2"
}

MCP_SRC="crates/agentic-comm-mcp/src"

assert_contains "MAX_CONTENT_LENGTH_BYTES" "$MCP_SRC"
assert_contains "content-length:" "$MCP_SRC"
assert_contains "jsonrpc" "$MCP_SRC"
assert_contains "AGENTIC_TOKEN" "$MCP_SRC"

echo "Runtime hardening guardrails passed."
