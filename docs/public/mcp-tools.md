---
status: draft
---

# MCP Tools Reference

The AgenticComm MCP server exposes 17 tools for agent-to-agent communication, channel management, pub/sub messaging, message history, and context logging. This document provides the complete specification for each tool as registered in the `ToolRegistry`.

## Tool Summary

| Tool | Description |
|------|-------------|
| `send_message` | Send a message to a channel or specific recipient |
| `receive_messages` | Retrieve pending or recent messages from a channel |
| `create_channel` | Create a new communication channel |
| `list_channels` | List all available communication channels |
| `join_channel` | Join an existing communication channel |
| `leave_channel` | Leave a communication channel |
| `get_channel_info` | Get detailed information about a channel |
| `subscribe` | Subscribe to a pub/sub topic for message delivery |
| `unsubscribe` | Remove a subscription from a pub/sub topic |
| `publish` | Publish a message to all subscribers of a topic |
| `broadcast` | Send a message to all participants in a broadcast channel |
| `query_history` | Search message history with filters |
| `search_messages` | Full-text search across all messages |
| `get_message` | Retrieve a specific message by ID |
| `acknowledge_message` | Mark a message as received and acknowledged |
| `set_channel_config` | Update configuration for a communication channel |
| `communication_log` | Log intent and context behind communication actions (20-Year Clock) |

## Error Handling

Tools follow the MCP Quality Standard for error reporting:

- **Tool execution errors** (channel not found, invalid content, permission denied): The JSON-RPC response succeeds, but the tool result contains `isError: true` with a descriptive message. These are expected operational failures.

- **Protocol errors** (unknown tool name, malformed parameters): A JSON-RPC error response is returned with the appropriate error code.

| Error Code | Name | Condition |
|------------|------|-----------|
| `-32803` | `TOOL_NOT_FOUND` | Unknown tool name in `tools/call` |
| `-32602` | `INVALID_PARAMS` | Required parameter missing or wrong type |

---

## send_message

Send a message to a channel or specific recipient.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Target channel ID |
| `sender` | string | yes | -- | Sender identity |
| `content` | string | yes | -- | Message body (1 byte to 1 MB) |
| `message_type` | string | no | `"text"` | Message type: `text`, `command`, `query`, `response`, `broadcast`, `notification`, `acknowledgment`, `error` |

### Return Format

On success, returns the full message object:

```json
{
  "id": 42,
  "channel_id": 1,
  "sender": "agent-planner",
  "recipient": null,
  "content": "Deploy auth-service to staging",
  "message_type": "Command",
  "timestamp": "2026-02-28T10:30:00Z",
  "metadata": {},
  "signature": "e3b0c44298fc1c149afb...",
  "acknowledged_by": []
}
```

### Error Conditions

- `channel_id is required` -- `channel_id` parameter missing or not an integer
- `sender is required` -- `sender` parameter missing or not a string
- `content is required` -- `content` parameter missing or not a string
- `Invalid sender: Sender cannot be empty` -- sender is an empty string
- `Invalid message content: Message content cannot be empty` -- content is empty
- `Invalid message content: Message content exceeds maximum size of 1048576 bytes` -- content too large
- `Channel not found: <id>` -- channel does not exist
- `Unknown message type: <type>` -- invalid message type string

### Example Request

```json
{
  "method": "tools/call",
  "params": {
    "name": "send_message",
    "arguments": {
      "channel_id": 1,
      "sender": "agent-planner",
      "content": "Deploy auth-service to staging",
      "message_type": "command"
    }
  }
}
```

---

## receive_messages

Retrieve pending or recent messages from a channel.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID to read from |
| `recipient` | string | no | null | Filter messages by recipient |
| `since` | string | no | null | ISO 8601 timestamp -- only return messages after this time |

### Return Format

Returns an array of message objects sorted by timestamp ascending:

```json
[
  {
    "id": 1,
    "channel_id": 1,
    "sender": "agent-planner",
    "recipient": null,
    "content": "Hello",
    "message_type": "Text",
    "timestamp": "2026-02-28T10:30:00Z",
    "metadata": {},
    "signature": "a1b2c3...",
    "acknowledged_by": []
  }
]
```

### Error Conditions

- `channel_id is required` -- missing or invalid parameter
- `Channel not found: <id>` -- channel does not exist

### Notes

When `recipient` is provided, the results include messages explicitly addressed to that recipient and messages with no specific recipient (broadcast to all channel members). When `since` is provided, only messages with a timestamp strictly after the given time are included. The `since` value must be a valid ISO 8601 / RFC 3339 timestamp; invalid timestamps are silently ignored (all messages are returned).

---

## create_channel

Create a new communication channel.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `name` | string | yes | -- | Channel name (alphanumeric, hyphens, underscores; 1-128 chars) |
| `channel_type` | string | no | `"group"` | Channel type: `direct`, `group`, `broadcast`, `pubsub` |
| `max_participants` | integer | no | `0` | Maximum participants (0 = unlimited) |
| `ttl_seconds` | integer | no | `0` | Message TTL in seconds (0 = forever) |
| `persistence` | boolean | no | `true` | Whether to persist messages |
| `encryption_required` | boolean | no | `false` | Whether encryption is required |

### Return Format

Returns the created channel object:

```json
{
  "id": 1,
  "name": "backend-team",
  "channel_type": "Group",
  "created_at": "2026-02-28T10:00:00Z",
  "participants": [],
  "config": {
    "max_participants": 0,
    "ttl_seconds": 0,
    "persistence": true,
    "encryption_required": false
  }
}
```

### Error Conditions

- `name is required` -- name parameter missing
- `Invalid channel name: Channel name must be 1-128 characters` -- name too short or too long
- `Invalid channel name: Channel name must contain only alphanumeric characters, hyphens, or underscores` -- invalid characters
- `Unknown channel type: <type>` -- invalid channel type string

---

## list_channels

List all available communication channels.

### Parameters

No parameters required. The input schema accepts an empty object.

### Return Format

Returns an array of channel objects sorted by channel ID ascending:

```json
[
  {
    "id": 1,
    "name": "backend-team",
    "channel_type": "Group",
    "created_at": "2026-02-28T10:00:00Z",
    "participants": ["agent-planner"],
    "config": { "max_participants": 0, "ttl_seconds": 0, "persistence": true, "encryption_required": false }
  }
]
```

---

## join_channel

Join an existing communication channel.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID to join |
| `participant` | string | yes | -- | Name of the participant joining |

### Return Format

```json
{
  "status": "joined",
  "channel_id": 1,
  "participant": "agent-worker-03"
}
```

### Error Conditions

- `channel_id is required` / `participant is required` -- missing parameters
- `Channel not found: <id>` -- channel does not exist
- `Participant '<name>' is already in channel <id>` -- duplicate join
- `Channel <id> has reached maximum participants` -- channel is full
- `Invalid sender: Sender cannot be empty` -- empty participant name

---

## leave_channel

Leave a communication channel.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID to leave |
| `participant` | string | yes | -- | Name of the participant leaving |

### Return Format

```json
{
  "status": "left",
  "channel_id": 1,
  "participant": "agent-worker-03"
}
```

### Error Conditions

- `channel_id is required` / `participant is required` -- missing parameters
- `Channel not found: <id>` -- channel does not exist
- `Participant '<name>' is not in channel <id>` -- participant is not a member

---

## get_channel_info

Get detailed information about a channel.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID to inspect |

### Return Format

Returns the full channel object including participants and configuration:

```json
{
  "id": 1,
  "name": "backend-team",
  "channel_type": "Group",
  "created_at": "2026-02-28T10:00:00Z",
  "participants": ["agent-planner", "agent-worker-01"],
  "config": {
    "max_participants": 10,
    "ttl_seconds": 3600,
    "persistence": true,
    "encryption_required": false
  }
}
```

### Error Conditions

- `channel_id is required` -- missing parameter
- `Channel <id> not found` -- channel does not exist

---

## subscribe

Subscribe to a pub/sub topic for message delivery.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `topic` | string | yes | -- | Topic to subscribe to (alphanumeric, hyphens, underscores; 1-128 chars) |
| `subscriber` | string | yes | -- | Subscriber identity |

### Return Format

Returns the subscription object:

```json
{
  "id": 1,
  "topic": "build-events",
  "subscriber": "agent-monitor",
  "created_at": "2026-02-28T10:00:00Z"
}
```

### Error Conditions

- `topic is required` / `subscriber is required` -- missing parameters
- `Invalid channel name: ...` -- topic name fails validation (same rules as channel names)
- `Invalid sender: Sender cannot be empty` -- empty subscriber

### Notes

Subscriptions are global -- they are not tied to a specific channel. When `publish` is called with a matching topic, all subscribers for that topic receive the message regardless of which channel the topic resides in. If no pub/sub channel exists for the topic, one is auto-created on the first `publish` call.

---

## unsubscribe

Remove a subscription from a pub/sub topic.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `subscription_id` | integer | yes | -- | Subscription ID to remove |

### Return Format

```json
{
  "status": "unsubscribed",
  "subscription_id": 1
}
```

### Error Conditions

- `subscription_id is required` -- missing parameter
- `Subscription not found: <id>` -- subscription does not exist

---

## publish

Publish a message to all subscribers of a topic.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `topic` | string | yes | -- | Topic to publish to |
| `sender` | string | yes | -- | Publisher identity |
| `content` | string | yes | -- | Message content (1 byte to 1 MB) |

### Return Format

```json
{
  "status": "published",
  "delivered_count": 3,
  "topic": "build-events"
}
```

### Error Conditions

- `topic is required` / `sender is required` / `content is required` -- missing parameters
- `Invalid sender: Sender cannot be empty` -- empty sender
- `Invalid message content: ...` -- content empty or too large
- `Invalid channel name: ...` -- topic name fails validation

### Notes

The `delivered_count` is the number of subscribers who received the message. If no subscribers exist for the topic, the count is 0 and no messages are created. The publisher is excluded from delivery (they do not receive their own published message). If no pub/sub channel exists for the topic, one is automatically created.

---

## broadcast

Send a message to all participants in a broadcast channel.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Broadcast channel ID |
| `sender` | string | yes | -- | Sender identity |
| `content` | string | yes | -- | Message content |

### Return Format

```json
{
  "status": "broadcast",
  "delivered_count": 5,
  "channel_id": 1
}
```

### Error Conditions

- `channel_id is required` / `sender is required` / `content is required` -- missing parameters
- `Channel not found: <id>` -- channel does not exist
- Validation errors for sender and content

### Notes

One message is created per participant in the channel, excluding the sender. Each message copy has the `recipient` field set to the individual participant. The `delivered_count` reflects the number of recipients.

---

## query_history

Search message history with filters.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID to query |
| `since` | string | no | null | ISO 8601 timestamp -- only messages after this time |
| `before` | string | no | null | ISO 8601 timestamp -- only messages before this time |
| `sender` | string | no | null | Filter by sender |
| `message_type` | string | no | null | Filter by message type |
| `limit` | integer | no | `100` | Maximum results (range: 1-10000) |

### Return Format

Returns an array of message objects sorted by timestamp ascending, truncated to the limit:

```json
[
  {
    "id": 10,
    "channel_id": 1,
    "sender": "agent-planner",
    "content": "Deploy to staging",
    "message_type": "Command",
    "timestamp": "2026-02-28T10:30:00Z"
  }
]
```

### Error Conditions

- `channel_id is required` -- missing parameter

### Notes

All filter parameters are optional and can be combined. When multiple filters are provided, they are applied with AND logic (all must match). Invalid timestamp strings are silently ignored. The default limit is 100; specify a higher value to retrieve more history.

---

## search_messages

Full-text search across all messages.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | yes | -- | Search text |
| `max_results` | integer | no | `20` | Maximum results (range: 1-1000) |

### Return Format

Returns an array of matching message objects sorted by timestamp descending (most recent first):

```json
[
  {
    "id": 42,
    "channel_id": 1,
    "sender": "agent-planner",
    "content": "Deploy auth-service to staging",
    "message_type": "Command",
    "timestamp": "2026-02-28T10:30:00Z"
  }
]
```

### Error Conditions

- `query is required` -- missing parameter

### Notes

Search is case-insensitive substring matching on message content. The search covers all channels in the store. Results are sorted by timestamp descending (most recent matches first).

---

## get_message

Retrieve a specific message by ID.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `message_id` | integer | yes | -- | Message ID to retrieve |

### Return Format

Returns the full message object, or an error if not found.

### Error Conditions

- `message_id is required` -- missing parameter
- `Message <id> not found` -- message does not exist

---

## acknowledge_message

Mark a message as received and acknowledged.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `message_id` | integer | yes | -- | Message ID to acknowledge |
| `recipient` | string | yes | -- | Acknowledging participant |

### Return Format

```json
{
  "status": "acknowledged",
  "message_id": 42,
  "recipient": "agent-worker-01"
}
```

### Error Conditions

- `message_id is required` / `recipient is required` -- missing parameters
- `Message not found: <id>` -- message does not exist
- `Invalid sender: Sender cannot be empty` -- empty recipient

### Notes

Acknowledgment is idempotent. Acknowledging the same message twice with the same recipient is a no-op (the recipient is not added to `acknowledged_by` a second time). The acknowledgment is recorded by adding the recipient to the message's `acknowledged_by` array.

---

## set_channel_config

Update configuration for a communication channel.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID to update |
| `max_participants` | integer | no | (current) | Maximum participants (0 = unlimited) |
| `ttl_seconds` | integer | no | (current) | Message TTL in seconds (0 = forever) |
| `persistence` | boolean | no | (current) | Whether to persist messages |
| `encryption_required` | boolean | no | (current) | Whether encryption is required |

### Return Format

```json
{
  "status": "updated",
  "channel_id": 1
}
```

### Error Conditions

- `channel_id is required` -- missing parameter
- `Channel <id> not found` -- channel does not exist

### Notes

Only provided fields are updated. Omitted fields retain their current values. The handler reads the current channel configuration and merges the provided values.

---

## communication_log

Log intent and context behind communication actions (20-Year Clock).

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `intent` | string | yes | -- | Why the communication action is happening |
| `observation` | string | no | null | What was noticed or concluded |
| `related_message_id` | integer | no | null | Link to a related message ID |
| `topic` | string | no | null | Category or topic (e.g., `"agent-coordination"`, `"debugging"`) |

### Return Format

```json
{
  "status": "logged",
  "timestamp": "2026-02-28T10:30:00Z",
  "intent": "Coordinating deployment across backend agents"
}
```

### Error Conditions

- Deserialization error if the input JSON does not match the expected structure

### Notes

This tool implements the 20-Year Clock pattern shared across all Agentra sisters. It captures WHY communication is happening, not just WHAT is being communicated. Entries are stored in the session's context log and linked to the temporal chain via the `related_message_id` field. The `topic` field enables categorization and filtering of context entries.

### Example Request

```json
{
  "method": "tools/call",
  "params": {
    "name": "communication_log",
    "arguments": {
      "intent": "Coordinating deployment sequence across three agents",
      "observation": "Agent-deploy is waiting for agent-build to complete",
      "related_message_id": 42,
      "topic": "deployment-coordination"
    }
  }
}
```
