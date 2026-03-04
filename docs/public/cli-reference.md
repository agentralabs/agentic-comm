---
status: stable
---

# CLI Reference

The `acomm` binary provides terminal-based access to all AgenticComm features: sending messages, managing channels, pub/sub operations, querying history, and inspecting `.acomm` store files. This document covers every subcommand with usage, flags, examples, and output format.

## Binary

```
acomm — Agent communication CLI for channels, messaging, and pub/sub
```

### Global Options

```
--file <PATH>    Path to the .acomm store file (overrides all other path resolution)
-h, --help       Print help information
-V, --version    Print version
```

The `--file` flag is global and applies to all subcommands. When omitted, the store path is resolved using the standard chain described in [Configuration](configuration.md).

## Subcommands

### acomm send

Send a message to a channel.

```
Usage: acomm send [OPTIONS] <CHANNEL> <CONTENT>

Arguments:
  <CHANNEL>    Channel ID (integer)
  <CONTENT>    Message content (text body)

Options:
  --type <TYPE>      Message type [default: text]
                     Values: text, command, query, response, broadcast,
                             notification, acknowledgment, error
  --sender <NAME>    Sender identity [default: cli-user]
  --file <PATH>      Path to .acomm store file
```

**Example: Send a text message**

```bash
acomm send 1 "Build completed successfully"
```

Output:
```json
{
  "status": "sent",
  "message_id": 42,
  "channel_id": 1,
  "timestamp": "2026-02-28T10:30:00Z"
}
```

**Example: Send a command message with a custom sender**

```bash
acomm send 1 "Deploy auth-service to staging" --type command --sender agent-planner
```

Output:
```json
{
  "status": "sent",
  "message_id": 43,
  "channel_id": 1,
  "timestamp": "2026-02-28T10:31:00Z"
}
```

**Error cases:**
- Channel ID does not exist: prints error to stderr, exits with code 1.
- Invalid message type string: prints error to stderr, exits with code 1.
- Empty content: prints error to stderr, exits with code 1.

---

### acomm receive

Receive messages from a channel.

```
Usage: acomm receive [OPTIONS] <CHANNEL>

Arguments:
  <CHANNEL>    Channel ID (integer)

Options:
  --since <TIMESTAMP>      Only messages after this ISO 8601 timestamp
  --recipient <NAME>       Filter by recipient
  --file <PATH>            Path to .acomm store file
```

**Example: Receive all messages from channel 1**

```bash
acomm receive 1
```

Output:
```json
[
  {
    "id": 1,
    "channel_id": 1,
    "sender": "agent-planner",
    "recipient": null,
    "content": "Build completed successfully",
    "message_type": "Text",
    "timestamp": "2026-02-28T10:30:00Z",
    "metadata": {},
    "signature": "a1b2c3...",
    "acknowledged_by": []
  }
]
```

**Example: Receive messages since a specific time**

```bash
acomm receive 1 --since "2026-02-28T10:00:00Z"
```

**Example: Filter by recipient**

```bash
acomm receive 1 --recipient agent-worker-03
```

---

### acomm channel create

Create a new communication channel.

```
Usage: acomm channel create [OPTIONS] <NAME>

Arguments:
  <NAME>       Channel name (alphanumeric, hyphens, underscores; 1-128 chars)

Options:
  --type <TYPE>    Channel type [default: group]
                   Values: direct, group, broadcast, pubsub
  --file <PATH>    Path to .acomm store file
```

**Example: Create a group channel**

```bash
acomm channel create backend-team
```

Output:
```json
{
  "status": "created",
  "channel_id": 1,
  "name": "backend-team",
  "type": "group"
}
```

**Example: Create a broadcast channel**

```bash
acomm channel create system-alerts --type broadcast
```

**Example: Create a pub/sub channel**

```bash
acomm channel create events --type pubsub
```

**Error cases:**
- Invalid channel name (empty, too long, special characters): prints error, exits 1.
- Invalid channel type string: prints error, exits 1.

---

### acomm channel list

List all channels in the store.

```
Usage: acomm channel list [OPTIONS]

Options:
  --file <PATH>    Path to .acomm store file
```

**Example:**

```bash
acomm channel list
```

Output:
```json
[
  {
    "id": 1,
    "name": "backend-team",
    "channel_type": "Group",
    "created_at": "2026-02-28T10:00:00Z",
    "participants": ["agent-planner", "agent-worker"],
    "config": {
      "max_participants": 0,
      "ttl_seconds": 0,
      "persistence": true,
      "encryption_required": false
    }
  }
]
```

---

### acomm channel info

Show detailed information about a specific channel.

```
Usage: acomm channel info <CHANNEL_ID>

Arguments:
  <CHANNEL_ID>     Channel ID (integer)

Options:
  --file <PATH>    Path to .acomm store file
```

**Example:**

```bash
acomm channel info 1
```

Output:
```json
{
  "id": 1,
  "name": "backend-team",
  "channel_type": "Group",
  "created_at": "2026-02-28T10:00:00Z",
  "participants": ["agent-planner", "agent-worker-01", "agent-worker-02"],
  "config": {
    "max_participants": 0,
    "ttl_seconds": 0,
    "persistence": true,
    "encryption_required": false
  }
}
```

**Error cases:**
- Channel ID not found: prints error to stderr, exits 1.

---

### acomm subscribe

Subscribe to a pub/sub topic.

```
Usage: acomm subscribe <TOPIC> <SUBSCRIBER>

Arguments:
  <TOPIC>          Topic name (alphanumeric, hyphens, underscores; 1-128 chars)
  <SUBSCRIBER>     Subscriber identity name

Options:
  --file <PATH>    Path to .acomm store file
```

**Example:**

```bash
acomm subscribe build-events agent-monitor
```

Output:
```json
{
  "status": "subscribed",
  "subscription_id": 1,
  "topic": "build-events",
  "subscriber": "agent-monitor"
}
```

**Error cases:**
- Invalid topic name: prints error, exits 1.
- Empty subscriber: prints error, exits 1.

---

### acomm publish

Publish a message to all subscribers of a pub/sub topic.

```
Usage: acomm publish [OPTIONS] <TOPIC> <CONTENT>

Arguments:
  <TOPIC>          Topic name
  <CONTENT>        Message content

Options:
  --sender <NAME>    Publisher identity [default: cli-user]
  --file <PATH>      Path to .acomm store file
```

**Example:**

```bash
acomm publish build-events "Frontend build succeeded" --sender ci-agent
```

Output:
```json
{
  "status": "published",
  "delivered_count": 3,
  "topic": "build-events"
}
```

The `delivered_count` indicates how many subscribers received the message. If no subscribers are registered for the topic, the count is 0 and no messages are created.

**Coordination tip (multi-agent teams):**
- Use `subscribe` + `publish` for routing/fan-out.
- Before sending a follow-up response in collaborative channels, run a poll/read step first so you do not post a duplicate parallel reply.
- See runnable example: `examples/pubsub-fanout-recipient-delivery.sh`.

---

### acomm history

Query message history for a channel with optional text search.

```
Usage: acomm history [OPTIONS] <CHANNEL>

Arguments:
  <CHANNEL>        Channel ID (integer)

Options:
  --query <TEXT>     Text search query (filters by content substring)
  --limit <N>        Maximum results [default: 50]
  --file <PATH>      Path to .acomm store file
```

**Example: Query all history for a channel**

```bash
acomm history 1 --limit 10
```

Output: A JSON array of message objects, sorted by timestamp ascending, limited to the specified count.

**Example: Search within a channel**

```bash
acomm history 1 --query "deploy" --limit 20
```

When `--query` is provided, the command uses full-text search across all messages (not limited to the specified channel). Without `--query`, it returns chronological history for the specified channel filtered by the `MessageFilter` defaults.

---

### acomm info

Show statistics about a `.acomm` store file.

```
Usage: acomm info <FILE>

Arguments:
  <FILE>    Path to the .acomm file to inspect
```

**Example:**

```bash
acomm info project.acomm
```

Output:
```json
{
  "file": "project.acomm",
  "channels": 5,
  "messages": 1247,
  "subscriptions": 8,
  "total_participants": 12
}
```

**Error cases:**
- File does not exist or is not a valid `.acomm` file: prints error, exits 1.

---

### acomm add

Add a communication record using a hook-compatible interface. This subcommand is designed for integration with external tools and scripts that need to insert messages into a store file programmatically.

```
Usage: acomm add [OPTIONS] <FILE> <TYPE> <CONTENT>

Arguments:
  <FILE>       Path to the .acomm file (created if it does not exist)
  <TYPE>       Message type: text, command, query, response, broadcast,
               notification, acknowledgment, error
  <CONTENT>    Message content

Options:
  --channel <NAME>       Channel name [default: default]
  --session <ID>         Session identifier (reserved for future use)
  --confidence <FLOAT>   Confidence level (reserved for hook compatibility)
```

**Example:**

```bash
acomm add project.acomm text "Agent completed task 47" --channel task-log
```

Output:
```json
{
  "status": "added",
  "message_id": 1,
  "channel": "task-log",
  "file": "project.acomm"
}
```

**Key behavior:**
- If the `.acomm` file does not exist, it is created.
- If the named channel does not exist, it is auto-created as a `Group` channel.
- The sender is always `cli-hook`.
- The `--session` and `--confidence` flags are accepted for API compatibility with other Agentra sisters but are not currently used by the communication engine.

## Output Format

All subcommands produce JSON output on stdout. Error messages are written to stderr as plain text prefixed with `Error:` or `Warning:`.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (invalid arguments, store not found, operation failed) |

### Mutation and Persistence

All mutating subcommands (`send`, `channel create`, `subscribe`, `publish`, `add`) follow this lifecycle:

1. Load the store from disk (or create a new one).
2. Perform the operation on the in-memory store.
3. Print the result to stdout.
4. Save the store back to disk.

If the save step fails (e.g., permission denied, disk full), a warning is printed to stderr but the command still exits with code 0 (the operation succeeded in memory and the result was printed). This matches the behavior of other Agentra sister CLIs.

### Read-only Subcommands

Read-only subcommands (`receive`, `channel list`, `channel info`, `history`, `info`) load the store, perform the query, and print results without saving. They never modify the store file.

## Store Path Resolution

The `acomm` CLI resolves the store path using this priority chain:

1. **`--file` flag**: If provided, use this path exactly.
2. **`ACOMM_STORE` environment variable**: If set, use this path.
3. **Local `.acomm/store.acomm`**: If this file exists in the current directory, use it.
4. **`~/.store.acomm`**: Global fallback in the user's home directory.

This enables per-project isolation (via `.acomm/` directory), per-environment configuration (via `ACOMM_STORE`), and zero-config usage (via the global fallback).

## Piping and Scripting

The CLI is designed for use in scripts and pipelines. All output is valid JSON, enabling composition with tools like `jq`:

```bash
# Get the number of messages in channel 1
acomm history 1 | jq '. | length'

# List channel names
acomm channel list | jq '.[].name'

# Send a message and capture the ID
MSG_ID=$(acomm send 1 "Hello" | jq '.message_id')

# Check store statistics
acomm info project.acomm | jq '.messages'
```

## Version Information

```bash
acomm --version
# acomm 0.1.0

acomm --help
# Agent communication CLI for channels, messaging, and pub/sub
```
