---
status: draft
---

# Command Surface

Install commands are documented in [Installation](installation.md).

## Binaries

- `acomm` (CLI engine)
- `agentic-comm-mcp` (MCP server)

## `acomm` Top-level

```bash
acomm init
acomm send
acomm receive
acomm ack
acomm publish
acomm broadcast
acomm search
acomm history
acomm thread
acomm stats
acomm compact
acomm channel create
acomm channel list
acomm channel info
acomm channel join
acomm channel leave
acomm channel config
acomm channel pause
acomm channel resume
acomm channel drain
acomm channel close
acomm channel delete
acomm subscribe
acomm unsubscribe
acomm daemon start
acomm daemon stop
acomm daemon status
acomm daemon logs
acomm daemon config
acomm daemon health-check
acomm export
acomm import
acomm completions
```

## `agentic-comm-mcp` Commands

```bash
agentic-comm-mcp serve
agentic-comm-mcp validate
agentic-comm-mcp info
agentic-comm-mcp export
agentic-comm-mcp compact
agentic-comm-mcp stats
```

## MCP Tools (17 consolidated)

All tools use an `operation` parameter to select the specific action. This keeps the tool surface small (~1,700 tokens) while supporting 200+ operations.

### comm_channel

Manage communication channels.

| Operation | Description |
|-----------|-------------|
| `create` | Create a new channel (requires `name`) |
| `list` | List all channels |
| `join` | Add participant to channel (requires `channel_id`, `participant`) |
| `leave` | Remove participant (requires `channel_id`, `participant`) |
| `info` | Get channel details (requires `channel_id`) |
| `config` | Update channel config (requires `channel_id`) |
| `pause` | Pause a channel |
| `resume` | Resume a paused channel |
| `drain` | Drain a channel |
| `close` | Close a channel |
| `expire` | Expire old messages |
| `compact` | Compact channel storage |

### comm_message

Send, receive, and manage messages.

| Operation | Description |
|-----------|-------------|
| `send` | Send a message (requires `channel_id`, `sender`, `content`) |
| `receive` | Receive messages (requires `channel_id`) |
| `search` | Search messages (requires `query`) |
| `get` | Get message by ID (requires `message_id`) |
| `ack` | Acknowledge a message |
| `delete` | Delete a message |
| `forward` | Forward to another channel |
| `reply` | Reply to a message |
| `thread` / `replies` | Get thread or replies |
| `rich_send` / `rich_get` | Rich content messaging |
| `priority_send` | Send with priority |
| `broadcast` / `publish` / `subscribe` / `unsubscribe` | Pub/sub operations |

### comm_semantic

Semantic messaging, echo chambers, ghosts, and metamessages.

| Operation | Description |
|-----------|-------------|
| `send` / `extract` / `graft` | Core semantic operations |
| `conflicts` / `resolve` | Conflict management |
| `cluster` / `merge` / `fragment` | Semantic graph operations |
| `ghosts_*` | Ghost conversation management |
| `echo_*` | Echo chamber detection and breaking |
| `metamessage_*` | Metamessage encoding and tracing |
| `fork_*` | Conversation forks (create, explore, merge, list) |

### comm_affect

Manage emotional affect states, contagion, archaeology, and prophecy.

| Operation | Description |
|-----------|-------------|
| `set` / `get` / `history` | Core affect state |
| `resist` / `decay` | Contagion resistance and decay |
| `contagion_*` | Affect contagion simulation |
| `archaeology_*` | Emotional archaeology |
| `prophecy_*` | Affect prediction |
| `unspeakable_*` | Unspeakable content handling |
| `anticipate_*` | Anticipatory understanding (predict, prepare, calibrate, accuracy) |

### comm_hive

Form and manage hive minds and collective consciousness.

| Operation | Description |
|-----------|-------------|
| `form` / `join` / `leave` / `dissolve` | Hive lifecycle |
| `status` / `think` | Hive operations |
| `meld_*` | Mind-meld sessions |
| `consciousness_*` | Collective consciousness |
| `silence_*` | Silent communion (enter, presence, attend, leave) |
| `mind_meld_*` | Mind meld (initiate, sync, status, separate) |
| `dream_*` | Collective dreaming (start, contribute, insights, wake) |

### comm_consent

Manage consent gates between agents.

| Operation | Description |
|-----------|-------------|
| `grant` / `revoke` | Grant or revoke consent |
| `check` | Check consent status |
| `list_gates` / `pending` / `respond` | Gate management |

### comm_trust

Manage agent trust levels.

| Operation | Description |
|-----------|-------------|
| `set` | Set trust level (requires `agent_id`, `level`) |
| `get` | Get trust level (requires `agent_id`) |
| `list` | List all trust levels |

### comm_keys

Manage encryption keys and message encryption.

| Operation | Description |
|-----------|-------------|
| `generate` / `rotate` / `list` / `export` | Key management |
| `encrypt` / `decrypt` / `verify` | Cryptographic operations |
| `public_key` | Get public key |

### comm_federation

Configure federation, zones, gateways, and routing.

| Operation | Description |
|-----------|-------------|
| `configure` / `zones_list` / `zones_policy` | Federation config |
| `gateway_*` | Gateway management |
| `route_*` | Message routing |
| `zone_*` | Zone management |
| `reality_*` | Consensus reality operations |
| `destiny_*` | Destiny channels (create, align, probability, converge) |

### comm_temporal

Schedule messages, manage dead letters, legacy, and precognition.

| Operation | Description |
|-----------|-------------|
| `schedule` / `cancel` / `list_pending` / `deliver` / `commit` | Scheduling |
| `legacy_*` | Legacy message management |
| `precognition_*` | Precognitive messaging |
| `dead_letter_*` | Dead letter resurrection |
| `consensus_*` | Temporal consensus (propose, vote, status, resolve) |

### comm_query

Query message history, relationships, conversations, and logs.

| Operation | Description |
|-----------|-------------|
| `messages` / `relationships` / `echoes` / `conversations` | Query types |
| `timeline` / `history` | Historical queries |
| `audit_log` / `comm_log` | Log access |

### comm_forensics

Forensic investigation, health diagnostics, oracle, and pattern detection.

| Operation | Description |
|-----------|-------------|
| `investigate` / `report` / `evidence` / `timeline` | Forensics |
| `health_*` | Health monitoring |
| `oracle_*` | Oracle predictions |
| `pattern_*` | Pattern detection |

### comm_collaboration

Ancestor tracking, collective intelligence, and telepathic links.

| Operation | Description |
|-----------|-------------|
| `ancestor_*` | Ancestor tracking |
| `collective_*` | Collective intelligence |
| `telepathy_*` | Telepathic links |

### comm_workspace

Create and query multi-store workspaces.

| Operation | Description |
|-----------|-------------|
| `create` / `list` / `add_context` | Workspace management |
| `query` / `compare` / `xref` | Cross-workspace queries |

### comm_session

Manage sessions and conversation logging.

| Operation | Description |
|-----------|-------------|
| `start` / `end` / `resume` | Session lifecycle |
| `log` | Conversation context logging |

### comm_agent

Register and manage communicating agents.

| Operation | Description |
|-----------|-------------|
| `register` / `get` / `list` | Agent registry |
| `update_availability` / `unregister` | Presence management |

### comm_store

Save, load, export, and manage the communication store.

| Operation | Description |
|-----------|-------------|
| `save` / `load` / `info` | Store I/O |
| `export` / `compact` | Maintenance |
| `ground` / `evidence` / `suggest` | Grounding operations |

**Example request (consolidated pattern):**
```json
{
  "method": "tools/call",
  "params": {
    "name": "comm_message",
    "arguments": {
      "operation": "send",
      "channel_id": 1,
      "sender": "agent-planner",
      "content": "Deploy auth-service to staging",
      "message_type": "command"
    }
  }
}
```
