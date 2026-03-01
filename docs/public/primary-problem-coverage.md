# Primary Problem Coverage (Comm)

This page tracks direct coverage for Comm primary problems:

- P08 no inter-agent messaging
- P09 no message ordering guarantees
- P10 no channel-based isolation
- P11 no subscription management
- P12 no message search/retrieval
- P35 no offline-first communication
- P36 no broadcast coordination
- P37 no message persistence

## What is implemented now

Comm already provides the core communication runtime. This phase adds an explicit regression entrypoint:

```bash
./scripts/test-primary-problems.sh
```

The script validates:

1. Channel creation and management
2. Message send/receive with ordering guarantees
3. Subscription lifecycle (subscribe, unsubscribe, list)
4. Message search and history retrieval
5. Broadcast to all channels
6. Store health checks
7. Focused Rust + MCP regression tests

## Problem-to-capability map

| Problem | Coverage primitive |
|---|---|
| P08 | `comm_send` + `comm_receive` + `.acomm` persistence |
| P09 | Monotonic message IDs + timestamp ordering |
| P10 | Named channels with independent message streams |
| P11 | `comm_subscribe` / `comm_unsubscribe` / `comm_subscriptions` |
| P12 | `comm_search` + `comm_history` with filters |
| P35 | Local `.acomm` file — no network required |
| P36 | `comm_broadcast` to all channels |
| P37 | Binary `.acomm` format with LZ4 compression |

## Notes on message ordering

Messages use monotonically increasing IDs within each channel. Cross-channel ordering uses timestamps. The `.acomm` binary format guarantees append-only writes for crash safety.

## See also

- [Initial Problem Coverage](initial-problem-coverage.md)
