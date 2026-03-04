#!/usr/bin/env bash
set -euo pipefail

# Demonstrates deterministic pub/sub coordination:
# 1) two subscribers register to one topic
# 2) one publish fans out to both
# 3) each recipient reads only their own delivery entry

if ! command -v acomm >/dev/null 2>&1; then
  echo "error: acomm CLI is required but was not found in PATH" >&2
  exit 1
fi

STORE_INPUT="${1:-}"
TOPIC="${TOPIC:-updates}"
SUBSCRIBER_A="${SUBSCRIBER_A:-meera}"
SUBSCRIBER_B="${SUBSCRIBER_B:-ishika}"
SENDER="${SENDER:-ci-agent}"
CONTENT="${CONTENT:-hello-topic}"
CHANNEL_ID=1

cleanup_store=0
if [[ -n "$STORE_INPUT" ]]; then
  STORE="$STORE_INPUT"
else
  STORE="$(mktemp -t agentic-comm-pubsub-XXXXXX.acomm)"
  cleanup_store=1
fi

cleanup() {
  if [[ "$cleanup_store" -eq 1 ]]; then
    rm -f "$STORE"
  fi
}
trap cleanup EXIT

rm -f "$STORE"
acomm init --json "$STORE" >/dev/null

acomm subscribe --file "$STORE" --json "$TOPIC" "$SUBSCRIBER_A" >/dev/null
acomm subscribe --file "$STORE" --json "$TOPIC" "$SUBSCRIBER_B" >/dev/null

publish_json="$(acomm publish --file "$STORE" --json --sender "$SENDER" "$TOPIC" "$CONTENT")"
if ! grep -q '"delivered_count":[[:space:]]*2' <<<"$publish_json"; then
  echo "error: expected delivered_count=2, got:" >&2
  echo "$publish_json" >&2
  exit 1
fi

recv_a="$(acomm receive --file "$STORE" --json --recipient "$SUBSCRIBER_A" "$CHANNEL_ID")"
recv_b="$(acomm receive --file "$STORE" --json --recipient "$SUBSCRIBER_B" "$CHANNEL_ID")"

if ! grep -q "\"recipient\":[[:space:]]*\"$SUBSCRIBER_A\"" <<<"$recv_a"; then
  echo "error: expected recipient $SUBSCRIBER_A in receive output" >&2
  echo "$recv_a" >&2
  exit 1
fi

if ! grep -q "\"recipient\":[[:space:]]*\"$SUBSCRIBER_B\"" <<<"$recv_b"; then
  echo "error: expected recipient $SUBSCRIBER_B in receive output" >&2
  echo "$recv_b" >&2
  exit 1
fi

if grep -q "\"recipient\":[[:space:]]*\"$SUBSCRIBER_B\"" <<<"$recv_a"; then
  echo "error: cross-delivery detected in $SUBSCRIBER_A receive output" >&2
  echo "$recv_a" >&2
  exit 1
fi

if grep -q "\"recipient\":[[:space:]]*\"$SUBSCRIBER_A\"" <<<"$recv_b"; then
  echo "error: cross-delivery detected in $SUBSCRIBER_B receive output" >&2
  echo "$recv_b" >&2
  exit 1
fi

echo "PASS: publish fan-out delivered to two subscribers with recipient-scoped receive output."
echo "store: $STORE"
