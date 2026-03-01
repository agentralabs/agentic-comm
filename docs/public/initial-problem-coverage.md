# Initial Problem Coverage (Comm)

This page records the **foundational problems AgenticComm already solved** before the newer primary-problem expansion.

## Reference set

| Ref | Initial problem solved | Shipped capability |
|---|---|---|
| ACOMM-I01 | No durable inter-agent messaging | `.acomm` artifact + channel persistence |
| ACOMM-I02 | No typed message model | Channels, messages, subscriptions, metadata |
| ACOMM-I03 | No message ordering | Monotonic IDs + timestamp-ordered message streams |
| ACOMM-I04 | No search/retrieval | `acomm search`, `acomm history`, MCP `comm_search` |
| ACOMM-I05 | No subscription management | `acomm subscribe` / `unsubscribe` / `subscriptions` |
| ACOMM-I06 | No broadcast coordination | `acomm broadcast`, MCP `comm_broadcast` |
| ACOMM-I07 | No offline-first communication | Local `.acomm` file, no network dependency |
| ACOMM-I08 | No universal MCP comm runtime | `agentic-comm-mcp` tools/resources |

## Status

All initial references `ACOMM-I01` to `ACOMM-I08` are implemented and actively testable from CLI/MCP surfaces.

## See also

- [Primary Problem Coverage](primary-problem-coverage.md)
- [Quickstart](quickstart.md)
