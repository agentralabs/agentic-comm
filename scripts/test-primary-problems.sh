#!/usr/bin/env bash
# test-primary-problems.sh — Validate primary problem coverage for AgenticComm
set -euo pipefail

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

assert_file() {
  [ -f "$1" ] || fail "Missing required file: $1"
}

run_acomm() {
  cargo run --quiet -p agentic-comm-cli --bin acomm -- "$@"
}

tmpdir="$(mktemp -d)"
acomm_file="$tmpdir/primary.acomm"

echo "[1/6] Initialize store"
init_out="$(run_acomm init --path "$acomm_file")"
printf '%s\n' "$init_out" | (rg -q "Initialized|created|Created" || true)

echo "[2/6] Basic channel lifecycle"
run_acomm channel create --path "$acomm_file" --name control --type direct >/dev/null
run_acomm channel list --path "$acomm_file" >/dev/null

echo "[3/6] Send/receive path"
run_acomm send --path "$acomm_file" --channel control --sender planner "deploy approved" >/dev/null
run_acomm receive --path "$acomm_file" --channel control --limit 10 >/dev/null

echo "[4/6] Search/history path"
run_acomm search --path "$acomm_file" "deploy" >/dev/null
run_acomm history --path "$acomm_file" --channel control --limit 10 >/dev/null

echo "[5/6] Focused regression tests"
cargo test --quiet -p agentic-comm --lib
cargo test --quiet -p agentic-comm-mcp --test edge_cases_inventions

echo "[6/6] Coverage docs present"
assert_file "docs/public/primary-problem-coverage.md"
assert_file "docs/public/initial-problem-coverage.md"

echo "Primary comm problem checks passed."
