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

## MCP Tools

All tools exposed by the `agentic-comm-mcp` MCP server.

### send_message

Send a message through a communication channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | ID of the target channel |
| `sender` | string | yes | -- | Participant ID of the sender |
| `message_type` | string | yes | -- | Message type: text, command, query, response, broadcast, notification, acknowledgment, error |
| `content` | string | yes | -- | Message body (max 1 MB) |
| `topic` | string | no | null | Topic string for pub/sub routing |
| `correlation_id` | string | no | null | UUID linking related messages |
| `priority` | string | no | "normal" | Priority: critical, high, normal, low, background |
| `ttl` | integer | no | null | Time-to-live in seconds |
| `metadata` | object | no | null | Key-value metadata |

**Returns:**
```json
{
  "id": 42,
  "message_type": "command",
  "sender": "agent-planner",
  "channel_id": 1,
  "content": "Deploy auth-service to staging",
  "correlation_id": "a1b2c3d4-e5f6-4789-abcd-ef0123456789",
  "priority": "normal",
  "status": "sent",
  "created_at": 1740667812
}
```

**Errors:**
- Channel not found (channel_id does not exist)
- Channel closed (channel is in Closed or Draining state)
- Not a participant (sender is not in the channel)
- Permission denied (sender is an Observer)
- Validation failed (content empty or too large)
- Store capacity exceeded

**Example request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "send_message",
    "arguments": {
      "channel_id": 1,
      "sender": "agent-planner",
      "message_type": "command",
      "content": "Deploy auth-service to staging environment",
      "priority": "high"
    }
  }
}
```

---

### receive_messages

Receive messages from a channel for a specific participant.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel to receive from |
| `recipient` | string | yes | -- | Participant ID receiving messages |
| `message_types` | array[string] | no | null | Filter by message types |
| `after` | integer | no | null | Only messages after this Unix timestamp |
| `before` | integer | no | null | Only messages before this Unix timestamp |
| `sender` | string | no | null | Filter by sender |
| `topic_pattern` | string | no | null | Filter by topic (supports wildcards) |
| `limit` | integer | no | 100 | Maximum messages to return |
| `offset` | integer | no | 0 | Pagination offset |

**Returns:**
```json
{
  "messages": [
    {
      "id": 42,
      "message_type": "command",
      "sender": "agent-planner",
      "content": "Deploy auth-service to staging",
      "created_at": 1740667812,
      "status": "delivered"
    }
  ],
  "total_count": 1,
  "has_more": false
}
```

**Errors:**
- Channel not found
- Not a participant (recipient is not in the channel)

---

### create_channel

Create a new communication channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `name` | string | yes | -- | Channel name (max 128 chars) |
| `channel_type` | string | yes | -- | Type: direct, group, broadcast, pubsub |
| `owner` | string | yes | -- | Owner participant ID |
| `description` | string | no | null | Human-readable description |
| `tags` | array[string] | no | [] | Categorization tags |

**Returns:**
```json
{
  "id": 1,
  "name": "backend-team",
  "channel_type": "group",
  "owner": "agent-lead",
  "state": "active",
  "participants": [{"id": "agent-lead", "role": "owner"}],
  "message_count": 0,
  "created_at": 1740667800
}
```

**Errors:**
- Validation failed (name invalid or too long)
- Duplicate channel (name already exists)
- Store capacity exceeded

---

### list_channels

List all channels in the store.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_type` | string | no | null | Filter by type: direct, group, broadcast, pubsub |
| `state` | string | no | null | Filter by state: active, paused, draining, closed |

**Returns:**
```json
{
  "channels": [
    {
      "id": 1,
      "name": "backend-team",
      "channel_type": "group",
      "state": "active",
      "participant_count": 4,
      "message_count": 127,
      "created_at": 1740667800
    }
  ]
}
```

---

### join_channel

Add a participant to an existing channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel to join |
| `participant` | string | yes | -- | Participant ID |
| `role` | string | no | "member" | Role: member, observer |

**Returns:**
```json
{
  "channel_id": 1,
  "participant": "agent-worker-03",
  "role": "member",
  "joined_at": 1740668000
}
```

**Errors:**
- Channel not found
- Channel closed
- Already a participant
- Max participants reached
- Validation failed (participant ID invalid)

---

### leave_channel

Remove a participant from a channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel to leave |
| `participant` | string | yes | -- | Participant ID |

**Returns:**
```json
{
  "channel_id": 1,
  "participant": "agent-worker-03",
  "left_at": 1740669000
}
```

**Errors:**
- Channel not found
- Not a participant
- Owner cannot leave (must transfer ownership first)

---

### get_channel_info

Get detailed information about a channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID |

**Returns:**
```json
{
  "id": 1,
  "name": "backend-team",
  "channel_type": "group",
  "owner": "agent-lead",
  "state": "active",
  "participants": [
    {"id": "agent-lead", "role": "owner", "joined_at": 1740667800},
    {"id": "agent-api", "role": "member", "joined_at": 1740667810},
    {"id": "agent-db", "role": "member", "joined_at": 1740667820}
  ],
  "config": {
    "delivery": "at_most_once",
    "max_message_size": 1048576,
    "retention": "forever",
    "echo": false
  },
  "message_count": 127,
  "description": "Backend team coordination channel",
  "tags": ["backend", "coordination"],
  "created_at": 1740667800,
  "modified_at": 1740670000
}
```

---

### subscribe

Subscribe to a topic pattern on a pub/sub channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | PubSub channel ID |
| `subscriber` | string | yes | -- | Subscriber participant ID |
| `topic_pattern` | string | yes | -- | Topic pattern (supports *, #) |

**Returns:**
```json
{
  "id": 1,
  "channel_id": 3,
  "subscriber": "agent-deploy",
  "topic_pattern": "build.*.success",
  "match_mode": "wildcard",
  "created_at": 1740668000
}
```

**Errors:**
- Channel not found
- Wrong channel type (not a PubSub channel)
- Not a participant
- Validation failed (invalid topic pattern)
- Duplicate subscription

---

### unsubscribe

Remove a pub/sub subscription.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `subscription_id` | integer | yes | -- | Subscription ID to remove |

**Returns:**
```json
{
  "subscription_id": 1,
  "removed": true
}
```

---

### publish

Publish a message to a pub/sub topic.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | PubSub channel ID |
| `sender` | string | yes | -- | Publisher participant ID |
| `topic` | string | yes | -- | Topic string (dot-separated) |
| `content` | string | yes | -- | Message content |
| `priority` | string | no | "normal" | Message priority |

**Returns:**
```json
{
  "id": 55,
  "topic": "build.frontend.success",
  "matched_subscriptions": 2,
  "status": "delivered",
  "created_at": 1740668500
}
```

**Errors:**
- Channel not found
- Wrong channel type (not PubSub)
- Not a participant
- Validation failed (topic or content invalid)

---

### broadcast

Broadcast a message to all participants in a channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID |
| `sender` | string | yes | -- | Sender participant ID |
| `content` | string | yes | -- | Broadcast message content |
| `priority` | string | no | "normal" | Message priority |

**Returns:**
```json
{
  "id": 60,
  "message_type": "broadcast",
  "recipients": 5,
  "status": "delivered",
  "created_at": 1740669000
}
```

---

### query_history

Query communication history with filters.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | no | null | Filter by channel |
| `sender` | string | no | null | Filter by sender |
| `message_types` | array[string] | no | null | Filter by message types |
| `after` | integer | no | null | After Unix timestamp |
| `before` | integer | no | null | Before Unix timestamp |
| `status` | array[string] | no | null | Filter by status |
| `topic_pattern` | string | no | null | Filter by topic |
| `correlation_id` | string | no | null | Filter by thread |
| `limit` | integer | no | 100 | Max results |
| `offset` | integer | no | 0 | Pagination offset |
| `sort_by` | string | no | "created_at" | Sort field |
| `sort_order` | string | no | "descending" | Sort direction |
| `include_archived` | boolean | no | false | Include archived messages |

**Returns:**
```json
{
  "messages": [...],
  "total_count": 250,
  "has_more": true
}
```

---

### search_messages

Search messages by content with optional filters.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | yes | -- | Search string (prefix with "regex:" for regex) |
| `channel_id` | integer | no | null | Limit to channel |
| `sender` | string | no | null | Limit to sender |
| `limit` | integer | no | 50 | Max results |

**Returns:**
```json
{
  "messages": [...],
  "total_count": 12,
  "has_more": false
}
```

---

### get_message

Get a specific message by ID.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `message_id` | integer | yes | -- | Message ID |

**Returns:** A single message object.

**Errors:**
- Message not found

---

### acknowledge_message

Acknowledge receipt or completion of a command message.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `message_id` | integer | yes | -- | ID of the Command message |
| `sender` | string | yes | -- | Acknowledging participant |
| `status` | string | yes | -- | Status: received, in_progress, completed, failed |
| `content` | string | no | null | Optional acknowledgment body |

**Returns:**
```json
{
  "id": 65,
  "message_type": "acknowledgment",
  "correlation_id": "a1b2c3d4-e5f6-4789-abcd-ef0123456789",
  "status": "completed",
  "created_at": 1740670000
}
```

---

### set_channel_config

Update channel configuration.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID |
| `delivery` | string | no | null | Delivery mode: at_most_once, at_least_once, exactly_once |
| `max_message_size` | integer | no | null | Max message size in bytes |
| `retention` | string | no | null | Retention: forever, or duration/count/size with value |
| `ack_timeout` | integer | no | null | Ack timeout in seconds |
| `max_retries` | integer | no | null | Max delivery retries |
| `echo` | boolean | no | null | Whether senders receive own messages |

**Returns:**
```json
{
  "channel_id": 1,
  "config": {
    "delivery": "at_least_once",
    "max_message_size": 1048576,
    "retention": "forever",
    "ack_timeout": 30,
    "max_retries": 3,
    "echo": false
  },
  "updated_at": 1740671000
}
```

---

### communication_log

Log communication context for memory integration and temporal chaining.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `user_message` | string | no | null | What the user said or asked |
| `agent_response` | string | no | null | Summary of the agent's communication action |
| `topic` | string | no | null | Topic or category |

**Returns:**
```json
{
  "logged": true,
  "chain_position": 42
}
```

## MCP Resources

The MCP server exposes read-only resources:

| URI | Description |
|-----|-------------|
| `comm://store/stats` | Store statistics (channel count, message count, etc.) |
| `comm://channels` | List of all channels |
| `comm://channels/{id}` | Channel details |
| `comm://channels/{id}/messages` | Messages in a channel (paginated) |
| `comm://dead-letters` | Dead letter queue contents |

## MCP Prompts

| Prompt | Description |
|--------|-------------|
| `comm-status` | Summarize current communication state: channels, pending messages, recent activity |
| `channel-health` | Analyze a channel's health: delivery rate, acknowledgment rate, dead letters |
| `thread-summary` | Summarize a message thread by correlation ID |

## Server Capabilities

The MCP server advertises the following capabilities:

```json
{
  "capabilities": {
    "tools": {},
    "resources": {
      "subscribe": true,
      "listChanged": true
    },
    "prompts": {
      "listChanged": true
    }
  }
}
```
