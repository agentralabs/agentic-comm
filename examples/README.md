# AgenticComm Examples

Runnable examples demonstrating the AgenticComm Python SDK.

## Prerequisites

```bash
pip install agentic-comm
cargo install agentic-comm
```

## Examples

| File | Description |
|------|-------------|
| `01_basic_messaging.py` | Simplest possible example. Create a store, create a channel, send and receive messages. |
| `02_multi_agent.py` | Multi-agent coordination. Multiple agents sending tasks and status updates through shared channels. |
| `03_task_queue.py` | Task queue pattern. One agent posts work items, another agent picks them up and reports completion. |
| `04_code_review.py` | Code review handoff. Developer agent requests review, reviewer agent provides feedback through channels. |
| `05_broadcast.py` | Broadcasting pattern. System-wide announcements sent to all channels simultaneously. |
| `06_search_history.py` | Search and history. Finding past messages and reviewing channel history for context. |

## Running

```bash
# All examples — no API key needed
python examples/01_basic_messaging.py
python examples/02_multi_agent.py
python examples/03_task_queue.py
python examples/04_code_review.py
python examples/05_broadcast.py
python examples/06_search_history.py
```

## MCP Server

For MCP-based integration (Claude Desktop, VS Code, Cursor, Windsurf), see the [MCP server README](../crates/agentic-comm-mcp/README.md).
