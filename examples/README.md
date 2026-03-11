# AgenticComm Examples

Runnable, no-cloud examples for common coordination patterns.

## Prerequisites

```bash
acomm --version
```

The examples below use only the `acomm` CLI and a temporary local `.acomm` store.

## Examples

| File | Description |
|------|-------------|
| `pubsub-fanout-recipient-delivery.sh` | Verifies `subscribe + publish` fan-out and recipient-scoped delivery (`receive --recipient`). |

## Running

```bash
bash examples/pubsub-fanout-recipient-delivery.sh
```

Optional: pass a store path to keep artifacts for inspection.

```bash
bash examples/pubsub-fanout-recipient-delivery.sh ./scratch/pubsub-demo.acomm
```

## MCP Server

For MCP-based integration (Claude Desktop, VS Code, Cursor, Windsurf), see the [MCP server README](../crates/agentic-comm-mcp/README.md).
