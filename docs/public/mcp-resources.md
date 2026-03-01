---
status: stable
---

# MCP Resources Reference

MCP resources expose read-only data from the AgenticComm server that clients can subscribe to and poll without making tool calls. Resources complement the tool interface by providing ambient access to store state, channel listings, and communication metadata.

## Resource Model

Resources follow the MCP specification for read-only data exposure:

- Resources are identified by URI strings.
- Clients can list available resources, read their contents, and subscribe to change notifications.
- Resource content is returned as JSON text.
- Resources never mutate state -- they are purely observational.

## Server Capabilities

The AgenticComm MCP server advertises resource support in its capabilities:

```json
{
  "capabilities": {
    "resources": {
      "subscribe": true,
      "listChanged": true
    }
  }
}
```

- `subscribe: true` -- Clients can subscribe to individual resources and receive notifications when the resource content changes.
- `listChanged: true` -- Clients are notified when the list of available resources changes (e.g., a new channel is created).

## Available Resources

### comm://store/stats

**Status:** Available (v0.1.0)

Store-wide statistics providing a high-level summary of the communication state.

**URI:** `comm://store/stats`

**Content type:** `application/json`

**Description:** Returns aggregate counts of channels, messages, subscriptions, and participants. Useful for dashboards, health checks, and monitoring.

**Response format:**

```json
{
  "channel_count": 5,
  "message_count": 1247,
  "subscription_count": 8,
  "total_participants": 12
}
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `channel_count` | integer | Total number of channels in the store |
| `message_count` | integer | Total number of messages across all channels |
| `subscription_count` | integer | Total number of active pub/sub subscriptions |
| `total_participants` | integer | Sum of participants across all channels (a participant in multiple channels is counted once per channel) |

**Change notification trigger:** Any mutating tool call (send_message, create_channel, subscribe, etc.) that changes one of the aggregate counts triggers a resource change notification.

---

### comm://channels

**Status:** Available (v0.1.0)

List of all communication channels in the store.

**URI:** `comm://channels`

**Content type:** `application/json`

**Description:** Returns a summary of every channel, including its ID, name, type, participant count, and message count. The list is sorted by channel ID ascending.

**Response format:**

```json
[
  {
    "id": 1,
    "name": "backend-team",
    "channel_type": "Group",
    "participant_count": 4,
    "message_count": 127,
    "config": {
      "max_participants": 0,
      "ttl_seconds": 0,
      "persistence": true,
      "encryption_required": false
    },
    "created_at": "2026-02-28T10:00:00Z"
  },
  {
    "id": 2,
    "name": "system-alerts",
    "channel_type": "Broadcast",
    "participant_count": 6,
    "message_count": 45,
    "config": {
      "max_participants": 0,
      "ttl_seconds": 86400,
      "persistence": true,
      "encryption_required": false
    },
    "created_at": "2026-02-28T10:05:00Z"
  }
]
```

**Change notification trigger:** Channel creation, deletion, participant join/leave, or configuration changes trigger a resource change notification.

---

### comm://channels/{id}

**Status:** Available (v0.1.0)

Detailed information about a specific channel.

**URI pattern:** `comm://channels/{id}` where `{id}` is the integer channel ID.

**Content type:** `application/json`

**Description:** Returns the full channel object including all participants, configuration, and metadata. This is equivalent to the `get_channel_info` tool but accessible as a subscribable resource.

**Response format:**

```json
{
  "id": 1,
  "name": "backend-team",
  "channel_type": "Group",
  "created_at": "2026-02-28T10:00:00Z",
  "participants": [
    "agent-planner",
    "agent-worker-01",
    "agent-worker-02",
    "agent-database"
  ],
  "config": {
    "max_participants": 10,
    "ttl_seconds": 0,
    "persistence": true,
    "encryption_required": false
  }
}
```

**Error handling:** If the channel ID does not exist, the resource returns an error with a descriptive message.

**Change notification trigger:** Any change to the channel (participant join/leave, configuration update, new message) triggers a notification.

---

### comm://channels/{id}/messages

**Status:** Available (v0.1.0)

Recent messages in a specific channel.

**URI pattern:** `comm://channels/{id}/messages` where `{id}` is the integer channel ID.

**Content type:** `application/json`

**Description:** Returns the most recent messages in the channel, sorted by timestamp descending. Limited to 100 messages by default. For more extensive queries, use the `query_history` or `search_messages` tools.

**Response format:**

```json
{
  "channel_id": 1,
  "messages": [
    {
      "id": 42,
      "sender": "agent-planner",
      "content": "Deploy auth-service to staging",
      "message_type": "Command",
      "timestamp": "2026-02-28T10:30:00Z",
      "acknowledged_by": ["agent-worker-01"]
    },
    {
      "id": 41,
      "sender": "agent-worker-01",
      "content": "Build completed, ready for deploy",
      "message_type": "Text",
      "timestamp": "2026-02-28T10:29:30Z",
      "acknowledged_by": []
    }
  ],
  "total_count": 127,
  "showing": 100
}
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `channel_id` | integer | The channel these messages belong to |
| `messages` | array | Array of message objects (most recent first) |
| `total_count` | integer | Total number of messages in the channel |
| `showing` | integer | Number of messages included in this response |

**Change notification trigger:** Any new message sent to the channel triggers a notification.

---

### comm://dead-letters

**Status:** Available (v0.1.0)

Dead letter queue contents.

**URI:** `comm://dead-letters`

**Content type:** `application/json`

**Description:** Returns messages that failed delivery after exhausting all retry attempts. Dead letters indicate communication failures that may require manual intervention.

**Response format:**

```json
{
  "dead_letters": [
    {
      "original_message_id": 55,
      "channel_id": 3,
      "sender": "agent-monitor",
      "content": "Alert: CPU threshold exceeded",
      "message_type": "Notification",
      "original_timestamp": "2026-02-28T10:15:00Z",
      "failure_reason": "Channel closed",
      "retry_count": 3,
      "dead_lettered_at": "2026-02-28T10:15:45Z"
    }
  ],
  "count": 1
}
```

**Change notification trigger:** Any message entering the dead letter queue triggers a notification.

---

### comm://subscriptions

**Status:** Available (v0.1.0)

List of all active pub/sub subscriptions.

**URI:** `comm://subscriptions`

**Content type:** `application/json`

**Description:** Returns all active subscriptions across all topics, including the subscriber identity, topic, and creation time.

**Response format:**

```json
[
  {
    "id": 1,
    "topic": "build-events",
    "subscriber": "agent-monitor",
    "created_at": "2026-02-28T10:00:00Z"
  },
  {
    "id": 2,
    "topic": "deploy-events",
    "subscriber": "agent-deploy",
    "created_at": "2026-02-28T10:01:00Z"
  }
]
```

**Change notification trigger:** Subscribe or unsubscribe operations trigger a notification.

---

### comm://session/operations

**Status:** Available (v0.1.0)

Operation log for the current MCP session.

**URI:** `comm://session/operations`

**Content type:** `application/json`

**Description:** Returns the list of tool operations recorded during the current session. Each entry includes the tool name, timestamp, and optional related entity ID. This resource is useful for session replay and debugging.

**Response format:**

```json
{
  "session_active": true,
  "operation_count": 15,
  "operations": [
    {
      "tool_name": "create_channel",
      "timestamp": "2026-02-28T10:00:00Z",
      "related_id": 1
    },
    {
      "tool_name": "join_channel",
      "timestamp": "2026-02-28T10:00:05Z",
      "related_id": 1
    },
    {
      "tool_name": "send_message",
      "timestamp": "2026-02-28T10:00:10Z",
      "related_id": 42
    }
  ]
}
```

**Change notification trigger:** Every tool call appends to the operation log and triggers a notification.

---

### comm://session/context

**Status:** Available (v0.1.0)

Communication context log for the current session (20-Year Clock entries).

**URI:** `comm://session/context`

**Content type:** `application/json`

**Description:** Returns all `communication_log` entries recorded during the current session. Each entry captures the intent behind a communication action, enabling post-session analysis of why agents communicated.

**Response format:**

```json
{
  "entries": [
    {
      "intent": "Coordinating deployment sequence across backend agents",
      "observation": "Agent-deploy is blocked waiting for agent-build",
      "related_message_id": 42,
      "topic": "deployment-coordination",
      "timestamp": "2026-02-28T10:30:00Z"
    }
  ],
  "count": 1
}
```

**Change notification trigger:** Each `communication_log` tool call triggers a notification.

## Resource Subscription

Clients can subscribe to resources to receive notifications when the resource content changes. The subscription protocol follows the MCP specification:

1. Client sends `resources/subscribe` with the resource URI.
2. Server confirms the subscription.
3. When the resource changes, the server sends a `notifications/resources/updated` notification with the URI.
4. Client reads the updated resource content with `resources/read`.

### Example Subscription Flow

```json
// Client subscribes to store stats
{"method": "resources/subscribe", "params": {"uri": "comm://store/stats"}}

// ... time passes, a message is sent ...

// Server notifies client of change
{"method": "notifications/resources/updated", "params": {"uri": "comm://store/stats"}}

// Client reads updated stats
{"method": "resources/read", "params": {"uri": "comm://store/stats"}}
```

## Resource URIs by Category

| Category | URIs |
|----------|------|
| Store-wide | `comm://store/stats` |
| Channels | `comm://channels`, `comm://channels/{id}`, `comm://channels/{id}/messages` |
| Pub/Sub | `comm://subscriptions` |
| Dead Letters | `comm://dead-letters` |
| Session | `comm://session/operations`, `comm://session/context` |
