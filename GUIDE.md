# GUIDE.md

## Interactive Channel Management

`acomm channels --interactive` guides you through creating and managing channels.

## Quick Start

```bash
# Create a channel
acomm create my-store channel "task-queue"

# Send a message
acomm send my-store "task-queue" "Build the frontend"

# Receive messages
acomm receive my-store "task-queue"

# Subscribe to a channel
acomm subscribe my-store "task-queue" "worker-1"

# Search messages
acomm search my-store "frontend"
```

## MCP Integration

Once connected via MCP, your LLM client gains access to 17 communication tools:

- `comm_channel_create` / `comm_channel_list` / `comm_channel_delete`
- `comm_send` / `comm_receive` / `comm_peek`
- `comm_subscribe` / `comm_unsubscribe` / `comm_subscriptions`
- `comm_search` / `comm_history`
- `comm_broadcast` / `comm_poll`
- `comm_store_info` / `comm_store_create` / `comm_health`
- `conversation_log`

See the [MCP Tools Reference](docs/public/mcp-tools.md) for full parameter details.
