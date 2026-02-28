---
status: draft
---

# Quickstart

Get started with AgenticComm in 10 minutes. This guide walks through installation, creating channels, sending messages, setting up pub/sub, querying history, and configuring the MCP server for Claude Code.

## Prerequisites

- Rust 1.75 or later (for building from source)
- OR a pre-built binary from the releases page

## 1. Install

### From crates.io (recommended)

```bash
cargo install agentic-comm-cli
```

This installs the `acomm` CLI binary.

### From source

```bash
git clone https://github.com/agentralabs/agentic-comm
cd agentic-comm
cargo install --path crates/agentic-comm-cli
```

### Verify installation

```bash
acomm --version
# agentic-comm-cli 0.1.0
```

## 2. Create Your First Channel

Initialize a new communication store and create a channel:

```bash
# Create a store (creates project.acomm in current directory)
acomm init project.acomm

# Create a direct channel between two agents
acomm channel create --name planner-executor --type direct --owner agent-planner project.acomm

# Add the other participant
acomm channel join --name planner-executor --participant agent-executor project.acomm

# Verify
acomm channel list project.acomm
```

Output:
```
ID  Name               Type    State   Participants  Messages
1   planner-executor   Direct  Active  2             0
```

## 3. Send a Message

Send a text message through the channel:

```bash
# Send a text message
acomm send --channel planner-executor --sender agent-planner --type text \
  "The authentication service needs a rate limiter before deployment" project.acomm

# Send a command (expects acknowledgment)
acomm send --channel planner-executor --sender agent-planner --type command \
  "Deploy auth-service to staging environment" project.acomm
```

Output:
```
Message sent: id=1, type=Text, channel=planner-executor
Message sent: id=2, type=Command, channel=planner-executor, correlation=q-a1b2c3d4
```

## 4. Receive Messages

Read messages from a channel as a specific participant:

```bash
# Receive all messages for agent-executor
acomm receive --channel planner-executor --as agent-executor project.acomm
```

Output:
```
ID  Type     Sender          Time                 Content
1   Text     agent-planner   2026-02-27 14:30:12  The authentication service needs...
2   Command  agent-planner   2026-02-27 14:30:15  Deploy auth-service to staging...
```

### Acknowledge a command

```bash
acomm ack --message-id 2 --sender agent-executor --status received \
  "Acknowledged. Starting deployment." project.acomm
```

### Later, acknowledge completion

```bash
acomm ack --message-id 2 --sender agent-executor --status completed \
  "Deployment successful. Health check passing." project.acomm
```

## 5. Set Up Pub/Sub

Create a pub/sub channel for event-driven communication:

```bash
# Create a pub/sub channel
acomm channel create --name ci-events --type pubsub --owner ci-agent project.acomm

# Add participants
acomm channel join --name ci-events --participant deploy-agent project.acomm
acomm channel join --name ci-events --participant monitor-agent project.acomm
acomm channel join --name ci-events --participant alert-agent project.acomm

# Subscribe to topics
acomm subscribe --channel ci-events --subscriber deploy-agent --topic "build.*.success" project.acomm
acomm subscribe --channel ci-events --subscriber monitor-agent --topic "deploy.#" project.acomm
acomm subscribe --channel ci-events --subscriber alert-agent --topic "*.failure" project.acomm
```

### Publish events

```bash
# Publish a build success event
acomm publish --channel ci-events --sender ci-agent --topic build.frontend.success \
  "Build completed: 847 tests passed, bundle size 2.1 MB" project.acomm

# Publish a deployment event
acomm publish --channel ci-events --sender ci-agent --topic deploy.staging.started \
  "Deploying auth-service v2.3.1 to staging" project.acomm

# Publish a failure event
acomm publish --channel ci-events --sender ci-agent --topic build.backend.failure \
  "Build failed: compilation error in auth/rate_limiter.rs:42" project.acomm
```

### Check who received what

```bash
# deploy-agent receives build.frontend.success (matches build.*.success)
acomm receive --channel ci-events --as deploy-agent project.acomm
# Output: 1 message (build.frontend.success)

# monitor-agent receives deploy.staging.started (matches deploy.#)
acomm receive --channel ci-events --as monitor-agent project.acomm
# Output: 1 message (deploy.staging.started)

# alert-agent receives build.backend.failure (matches *.failure)
acomm receive --channel ci-events --as alert-agent project.acomm
# Output: 1 message (build.backend.failure)
```

## 6. Query History

Search and filter communication history:

```bash
# All messages in a channel
acomm history --channel planner-executor project.acomm

# Messages from a specific sender
acomm history --sender agent-planner project.acomm

# Messages of a specific type
acomm history --type command project.acomm

# Messages in a time range
acomm history --after "2026-02-27T14:00:00Z" --before "2026-02-27T15:00:00Z" project.acomm

# Search by content
acomm search "rate limiter" project.acomm

# Full-text search with regex
acomm search --regex "deploy.*staging" project.acomm

# View a thread (all messages with same correlation ID)
acomm thread q-a1b2c3d4 project.acomm
```

### Store statistics

```bash
acomm stats project.acomm
```

Output:
```
Store: project.acomm
Created:       2026-02-27 14:30:00 UTC
Modified:      2026-02-27 14:35:12 UTC
Channels:      2
Messages:      7
Subscriptions: 3
Dead Letters:  0
File Size:     24 KB
```

## 7. MCP Server Setup for Claude Code

Configure the MCP server so Claude Code can use AgenticComm tools.

### Install the MCP server

```bash
cargo install agentic-comm-mcp
```

### Add to Claude Code MCP config

Add to your `~/.claude/mcp.json`:

```json
{
  "mcpServers": {
    "agentic-comm": {
      "command": "agentic-comm-mcp",
      "args": []
    }
  }
}
```

### Or for project-specific configuration

Add to your project's `.claude/mcp.json`:

```json
{
  "mcpServers": {
    "agentic-comm": {
      "command": "agentic-comm-mcp",
      "args": [],
      "env": {
        "ACOMM_STORE": "./project.acomm",
        "ACOMM_DEFAULT_CHANNEL": "main",
        "ACOMM_SENDER": "claude-code"
      }
    }
  }
}
```

### Verify MCP tools are available

In Claude Code, the following tools will be available:

- `send_message` -- Send a message through a channel
- `receive_messages` -- Receive messages from a channel
- `create_channel` -- Create a new communication channel
- `list_channels` -- List all channels in the store
- `join_channel` -- Join an existing channel
- `leave_channel` -- Leave a channel
- `get_channel_info` -- Get detailed info about a channel
- `subscribe` -- Subscribe to a pub/sub topic
- `unsubscribe` -- Remove a subscription
- `publish` -- Publish a message to a pub/sub topic
- `broadcast` -- Broadcast a message to all channel participants
- `query_history` -- Query communication history with filters
- `search_messages` -- Search messages by content
- `get_message` -- Get a specific message by ID
- `acknowledge_message` -- Acknowledge receipt of a command
- `set_channel_config` -- Update channel configuration
- `communication_log` -- Log communication context for memory integration

### Example MCP interaction

Claude Code can now use AgenticComm tools directly:

```
User: Create a channel for the backend team and send a message about the database migration

Claude: I'll create a group channel and send the message.

[Tool call: create_channel]
name: "backend-team"
channel_type: "group"
owner: "claude-code"

[Tool call: send_message]
channel_id: 1
sender: "claude-code"
message_type: "text"
content: "Database migration plan: 1) Add new columns with defaults. 2) Backfill in batches of 1000. 3) Add NOT NULL constraints. 4) Remove old columns."
```

## 8. Validation Rules

AgenticComm validates all inputs before processing. Here are the rules you need to know:

### Channel Names

- Must be non-empty.
- Maximum 128 characters.
- Allowed characters: letters, digits, hyphens, underscores, forward slashes.
- Pattern: `[a-zA-Z0-9_-]+(/[a-zA-Z0-9_-]+)*`
- Forward slashes create hierarchical names: `team/backend/alerts` is valid.
- Must be unique within a store.

**Valid:** `backend-team`, `ci/build-events`, `agent_01_channel`
**Invalid:** `""` (empty), `my channel` (space), `../escape` (path traversal), `a` * 129 (too long)

### Participant IDs

- Must be non-empty.
- Maximum 128 characters.
- Allowed characters: letters, digits, hyphens, underscores.
- Pattern: `[a-zA-Z0-9_-]+`

**Valid:** `agent-planner`, `worker_03`, `claude-code-session-1`
**Invalid:** `""` (empty), `agent planner` (space), `agent@home` (@ not allowed)

### Message Content

- Must be non-empty (zero-length content is rejected).
- Maximum 1,048,576 bytes (1 MB).
- Must be valid UTF-8.

### Topic Strings

- Must be non-empty (when provided).
- Maximum 256 characters.
- Dot-separated segments.
- Each segment: letters, digits, hyphens, underscores, or wildcard characters (`*`, `#`).
- `*` matches exactly one segment. `#` matches zero or more segments and must be the last segment.
- Topics starting with `_system.` are reserved.

**Valid:** `build.frontend.complete`, `deploy.*.success`, `events.#`
**Invalid:** `""` (empty), `.leading.dot`, `trailing.dot.`, `mid.#.wild` (`#` not at end)

### Correlation IDs

- Must be a valid UUID v4 string when provided.
- Format: `xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx` where x is hex and y is 8, 9, a, or b.

### Priority Values

- Integer 0-4.
- 0=Critical, 1=High, 2=Normal (default), 3=Low, 4=Background.
- Values outside 0-4 are rejected.

### Metadata

- Maximum 64 key-value entries.
- Keys: maximum 128 characters, non-empty.
- String values: maximum 4,096 bytes.
- Supported value types: String, Integer (i64), Float (f64), Boolean, Null.

## Next Steps

- [Core Concepts](concepts.md) -- Understand message types, channels, and communication protocols.
- [API Reference](api-reference.md) -- Full library API documentation.
- [Command Surface](command-surface.md) -- Complete CLI and MCP tool reference.
- [Integration Guide](integration-guide.md) -- Sister integrations and embedding.
- [Benchmarks](benchmarks.md) -- Performance characteristics.
