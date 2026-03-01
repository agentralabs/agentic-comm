# agentic-comm-mcp

MCP server for AgenticComm — exposes 17 communication tools to any MCP-compatible LLM client.

## Install

```bash
cargo install agentic-comm-mcp
```

## Configure

Add to your MCP client config:

```json
{
  "mcpServers": {
    "agentic-comm": {
      "command": "agentic-comm-mcp",
      "args": ["serve"]
    }
  }
}
```

## Tools

| Tool | Description |
|------|-------------|
| `comm_channel_create` | Create a communication channel |
| `comm_channel_list` | List available channels |
| `comm_channel_delete` | Remove a channel |
| `comm_send` | Send a message to a channel |
| `comm_receive` | Receive messages from a channel |
| `comm_peek` | Preview messages without consuming |
| `comm_subscribe` | Subscribe an agent to a channel |
| `comm_unsubscribe` | Remove a subscription |
| `comm_subscriptions` | List subscriptions |
| `comm_search` | Search messages by content |
| `comm_history` | View channel message history |
| `comm_broadcast` | Send to all channels |
| `comm_poll` | Check all subscribed channels |
| `comm_store_info` | Get store information |
| `comm_store_create` | Create a new store |
| `comm_health` | Check store health |
| `conversation_log` | Log conversation context |

## License

MIT — see [LICENSE](LICENSE)
