---
status: draft
---

# Message Engine Guide

This document explains how AgenticComm's message engine works end-to-end: how messages are sent, routed, delivered, acknowledged, and recovered. It covers the internal pipelines, broadcast fan-out, pub/sub topic matching, retry semantics, and dead letter handling.

## Send Pipeline

Every message passes through a five-stage send pipeline before it reaches any recipient.

### Stage 1: Validate

The engine validates all message fields against the rules defined in [Data Structures](data-structures.md):

1. **Sender validation**: The sender participant ID must be non-empty, at most 128 characters, and match the allowed character pattern.
2. **Channel validation**: The `channel_id` must reference an existing channel in the store. The channel must be in `Active` state (not Paused, Draining, or Closed).
3. **Participant check**: The sender must be a participant of the channel. For broadcast channels, the sender must be the owner.
4. **Content validation**: Content must be non-empty and within the channel's `max_message_size` limit.
5. **Type-specific validation**:
   - Response messages must have a `correlation_id` matching an existing Query.
   - Acknowledgment messages must have a `correlation_id` matching an existing Command.
   - PubSub messages must have a `topic` field.
6. **Priority validation**: Priority must be a valid enum value (0-4).

If any validation fails, the engine returns an error immediately. The message is not persisted or routed. The error includes the specific validation failure and the field that caused it.

```rust
// Validation error example
CommError::ValidationFailed {
    field: "content",
    reason: "Message content exceeds channel max_message_size (1048576 bytes)",
    value_length: 1500000,
}
```

### Stage 2: Assign ID and Timestamp

The engine assigns a unique message ID (monotonically increasing `u64`) and sets the `created_at` timestamp to the current Unix time in seconds (UTC). The message status is set to `Sent`.

### Stage 3: Route

The routing engine determines recipients based on channel type:

**Direct channel**: The recipient is the other participant (not the sender).

**Group channel**: All participants except the sender are recipients. If `echo: true` is set in the channel config, the sender is also a recipient.

**Broadcast channel**: All participants with Observer or Member role are recipients (everyone except the owner, unless `echo: true`).

**PubSub channel**: The engine matches the message's `topic` against all active subscriptions on the channel. Each subscription whose `topic_pattern` matches the message topic is a recipient. See [Topic Matching](#topic-matching) below.

### Stage 4: Persist

The message is written to the in-memory `CommStore` and the dirty flag is set. The store is flushed to disk according to the flush policy:

- **Immediate flush**: Every message triggers a file write. Safest but slowest.
- **Batch flush**: Messages are accumulated and flushed every N messages or every T milliseconds, whichever comes first. Default: 100 messages or 5000 ms.
- **Manual flush**: The caller explicitly triggers flushes. Used in testing and high-throughput scenarios.

Indexes (channel-message, timestamp, topic, sender, correlation) are updated in-memory at persist time. They are written to disk during the next flush.

### Stage 5: Deliver

Delivery means making the message available to the recipient's receive pipeline. In the current in-process architecture, delivery updates the per-recipient message queue. The message status is set to `Delivered` and the `delivered_at` timestamp is recorded.

For channels with `AtLeastOnce` or `ExactlyOnce` delivery mode, the engine starts an acknowledgment timer for each recipient.

## Receive Pipeline

When a recipient queries for messages, the receive pipeline filters and returns matching messages.

### Stage 1: Query

The recipient calls `receive_messages` with a `MessageFilter` specifying the criteria: channel, time range, message types, sender, topic, status, etc. All filter fields are optional; an empty filter returns all unread messages across all channels the recipient participates in.

### Stage 2: Filter

The engine applies the filter to the message store, using indexes where available:

1. **Channel filter**: Uses the channel-message index for O(1) lookup.
2. **Timestamp filter**: Uses the timestamp index with binary search for O(log N) range queries.
3. **Topic filter**: Uses the topic index for exact matches; falls back to linear scan for wildcard matches.
4. **Sender filter**: Uses the sender index for O(1) lookup.
5. **Correlation filter**: Uses the correlation index for O(1) thread lookup.
6. **Content pattern**: Applies regex matching against message content. This is always a linear scan over the filtered result set -- apply other filters first to narrow the scan.

Filters are applied conjunctively (AND). All specified criteria must match.

### Stage 3: Sort

Results are sorted by the field specified in `sort_by` (default: `CreatedAt`) and the direction specified in `sort_order` (default: `Descending`).

Within the same priority level, messages maintain FIFO order. When `priority_ordering` is enabled on the channel, the sort first groups by priority, then by the requested sort field within each group.

### Stage 4: Return

The filtered, sorted result set is truncated to `limit` entries (default: 100) after skipping `offset` entries (default: 0). The result includes the total count of matching messages before pagination, enabling the caller to determine if more results exist.

```rust
QueryResult {
    messages: Vec<Message>,  // paginated results
    total_count: u64,        // total matching messages
    has_more: bool,          // true if total_count > offset + limit
}
```

## Topic Matching

Topic matching is the core of the pub/sub routing engine. Given a message topic and a set of subscription patterns, the engine determines which subscriptions should receive the message.

### Algorithm

```
function matches(topic: &str, pattern: &str) -> bool:
    topic_segments = topic.split('.')
    pattern_segments = pattern.split('.')

    if pattern ends with '#':
        // Multi-level wildcard: match prefix
        prefix = pattern_segments[0..len-1]
        if topic_segments.len() < prefix.len():
            return false
        for i in 0..prefix.len():
            if prefix[i] != '*' and prefix[i] != topic_segments[i]:
                return false
        return true

    if topic_segments.len() != pattern_segments.len():
        return false

    for i in 0..topic_segments.len():
        if pattern_segments[i] == '*':
            continue  // wildcard matches any single segment
        if pattern_segments[i] != topic_segments[i]:
            return false

    return true
```

### Matching Examples

| Message Topic | Pattern | Matches? | Reason |
|---------------|---------|----------|--------|
| `build.frontend.complete` | `build.frontend.complete` | Yes | Exact match |
| `build.frontend.complete` | `build.*.complete` | Yes | `*` matches `frontend` |
| `build.frontend.complete` | `build.#` | Yes | `#` matches `frontend.complete` |
| `build.frontend.complete` | `build.backend.complete` | No | `backend` != `frontend` |
| `build.frontend.complete` | `build.*.start` | No | `start` != `complete` |
| `build.frontend.test.unit` | `build.*.complete` | No | Segment count mismatch |
| `build` | `build.#` | Yes | `#` matches zero segments |
| `deploy.staging` | `*.staging` | Yes | `*` matches `deploy` |

### Performance Optimization

For channels with many subscriptions, the engine groups subscriptions by match mode:

1. **Exact subscriptions** are checked first using a `HashMap<String, Vec<SubscriptionId>>` lookup (O(1)).
2. **Wildcard subscriptions** are checked by segment count first (rejecting patterns with different segment counts), then by segment comparison.
3. **Multi-level subscriptions** are checked by prefix match (the non-wildcard prefix segments).

This tiered approach avoids checking every subscription against every message.

## Broadcast Fan-out

When a message is sent to a broadcast or group channel, the engine performs fan-out:

1. Determine the recipient list from the channel participants.
2. For each recipient, create a delivery record (message_id, recipient_id, delivery_status).
3. Update the message's delivery status to `Delivered` once all delivery records are created.
4. If any delivery record fails (e.g., recipient no longer exists), the failure is logged but does not block delivery to other recipients.

Fan-out is synchronous in the current in-process architecture. The engine processes all recipients before returning from the send call. For channels with many recipients (>1000), the engine uses batch processing to avoid holding the store lock for extended periods.

## Acknowledgment Flow

For channels with `AtLeastOnce` or `ExactlyOnce` delivery mode, the engine tracks acknowledgments.

### Sender Side

1. Send a Command or Query message.
2. Engine starts an acknowledgment timer (duration = `ack_timeout` from channel config).
3. If an Acknowledgment or Response with the matching `correlation_id` arrives before timeout: mark the original message as `Acknowledged`, record `acknowledged_at`.
4. If timeout expires: trigger retry (see [Retry Semantics](#retry-semantics)).

### Receiver Side

1. Receive a Command or Query that expects acknowledgment.
2. Process the message.
3. Send an Acknowledgment (for Commands) or Response (for Queries) with the original message's `correlation_id`.
4. The acknowledgment is routed back to the original sender through the same channel.

### Acknowledgment Statuses

| Status | Meaning |
|--------|---------|
| `received` | The receiver has received the message but has not started processing. |
| `in_progress` | The receiver is actively processing the message. |
| `completed` | Processing is complete and successful. |
| `failed` | Processing failed. The error details are in the acknowledgment content. |

## Retry Semantics

When a message is not acknowledged within the timeout period, the engine retries delivery:

1. Increment `retry_count` on the message.
2. If `retry_count < max_retries` (from channel config):
   a. Wait for `retry_backoff_ms * 2^(retry_count - 1)` milliseconds (exponential backoff).
   b. Re-deliver the message to the recipient.
   c. Restart the acknowledgment timer.
3. If `retry_count >= max_retries`:
   a. Move the message to the dead letter queue.
   b. Set message status to `DeadLetter`.
   c. Emit a notification on the `_system.message.deadletter` topic.

### Backoff Schedule (default config: 1000ms base, 3 retries)

| Retry | Wait Time | Cumulative Time |
|-------|-----------|-----------------|
| 1 | 1,000 ms | 1,000 ms |
| 2 | 2,000 ms | 3,000 ms |
| 3 | 4,000 ms | 7,000 ms |
| Dead letter | -- | 7,000 ms + ack_timeout |

## Dead Letter Handling

Dead-lettered messages are preserved for inspection and manual recovery:

```rust
// Query dead letters
let dead_letters = store.dead_letters();

// Replay a dead letter (re-enter the send pipeline)
store.replay_dead_letter(message_id)?;

// Discard a dead letter
store.discard_dead_letter(message_id)?;

// Discard all dead letters older than a duration
store.purge_dead_letters(Duration::from_secs(86400 * 7))?;
```

Dead letters retain all original message fields plus:
- `retry_count`: How many times delivery was attempted.
- `last_error`: The reason for the last delivery failure.
- `dead_lettered_at`: When the message entered the dead letter queue.

## Backpressure

When the message engine detects resource pressure, it applies backpressure to senders:

### Memory Pressure

If the in-memory message count exceeds a configurable threshold (default: 100,000 active messages), the engine:
1. Flushes the store to disk.
2. Evicts acknowledged and archived messages from memory.
3. If still above threshold, rejects new messages with a `StoreFull` error.

### Channel Pressure

Individual channels can be configured with a maximum pending message count (`max_pending`, default: unlimited). When the pending count is reached:
1. The engine pauses delivery on the channel.
2. New messages are accepted but queued without delivery.
3. When pending messages are acknowledged and the count drops below the threshold, delivery resumes.

### Disk Pressure

When the `.acomm` file approaches the 2 GB limit:
1. The engine emits a warning notification.
2. When within 100 MB of the limit, new messages are rejected with a `StoreCapacity` error.
3. The caller can compact the store (archive old messages, remove dead letters) to free space.

## Common Workflows

### One-to-one Task Delegation

```rust
use agentic_comm::{CommStore, MessageType, ChannelType};

let mut store = CommStore::open("project.acomm")?;

// Create a direct channel
let channel = store.create_channel("planner-executor", ChannelType::Direct, "agent-planner")?;
store.join_channel(channel.id, "agent-executor")?;

// Send a command
let msg = store.send_message(
    channel.id,
    "agent-planner",
    MessageType::Command,
    "Deploy the authentication service to staging",
)?;

// Executor receives and acknowledges
let messages = store.receive_messages(channel.id, "agent-executor", None)?;
store.acknowledge_message(messages[0].id, "agent-executor", "received", None)?;

// Later: acknowledge completion
store.acknowledge_message(messages[0].id, "agent-executor", "completed",
    Some("Deployed successfully. Health check passing."))?;
```

### Pub/Sub Event Notification

```rust
use agentic_comm::{CommStore, ChannelType, MessageType};

let mut store = CommStore::open("ci.acomm")?;

// Create a pub/sub channel for CI events
let channel = store.create_channel("ci-events", ChannelType::PubSub, "ci-agent")?;

// Agents subscribe to topics they care about
store.subscribe(channel.id, "deploy-agent", "build.*.success")?;
store.subscribe(channel.id, "monitor-agent", "deploy.#")?;
store.subscribe(channel.id, "alert-agent", "*.failure")?;

// CI agent publishes a build event
store.publish(
    channel.id,
    "ci-agent",
    "build.frontend.success",
    "Frontend build completed: 847 tests passed, bundle size 2.1 MB",
)?;

// deploy-agent receives (matches build.*.success)
let msgs = store.receive_messages(channel.id, "deploy-agent", None)?;
assert_eq!(msgs.len(), 1);

// alert-agent does NOT receive (build.frontend.success does not match *.failure)
let msgs = store.receive_messages(channel.id, "alert-agent", None)?;
assert_eq!(msgs.len(), 0);
```

### Threaded Conversation

```rust
use agentic_comm::{CommStore, ChannelType, MessageType};

let mut store = CommStore::open("session.acomm")?;
let channel = store.create_channel("discussion", ChannelType::Group, "agent-lead")?;
store.join_channel(channel.id, "agent-researcher")?;
store.join_channel(channel.id, "agent-reviewer")?;

// Start a thread with a query
let query = store.send_query(
    channel.id,
    "agent-lead",
    "What is the recommended connection pool size for 10K concurrent users?",
    Some(Duration::from_secs(30)),
)?;

// Researcher responds (using the query's correlation_id)
store.send_response(
    channel.id,
    "agent-researcher",
    &query.correlation_id.unwrap(),
    "Based on PostgreSQL best practices: pool_size = num_cores * 2 + disk_spindles. For your 8-core setup: 20 connections.",
)?;

// Reviewer adds to the thread
store.send_response(
    channel.id,
    "agent-reviewer",
    &query.correlation_id.unwrap(),
    "Agreed, but consider PgBouncer for connection multiplexing if you expect bursts.",
)?;

// Query the entire thread
let thread = store.get_thread(&query.correlation_id.unwrap())?;
assert_eq!(thread.len(), 3); // original query + 2 responses
```

### Querying Message History

```rust
use agentic_comm::{CommStore, MessageFilter, SortField, SortOrder};

let store = CommStore::open("project.acomm")?;

// Find all commands sent in the last hour
let filter = MessageFilter {
    message_types: Some(vec![MessageType::Command]),
    after: Some(now() - 3600),
    sort_by: SortField::CreatedAt,
    sort_order: SortOrder::Descending,
    limit: 50,
    ..Default::default()
};

let results = store.query_messages(&filter)?;
println!("Found {} commands in the last hour", results.total_count);

// Find all failed deliveries
let dead_filter = MessageFilter {
    status: Some(vec![MessageStatus::DeadLetter]),
    ..Default::default()
};
let dead = store.query_messages(&dead_filter)?;
println!("{} messages in dead letter queue", dead.total_count);
```

## ExactlyOnce Deduplication

`ExactlyOnce` delivery mode prevents duplicate processing of retried messages:

1. Every message with `ExactlyOnce` delivery must have a `correlation_id`.
2. The engine maintains a deduplication set of `(correlation_id, recipient_id)` tuples.
3. Before delivering a retried message, the engine checks if the tuple already exists in the set.
4. If the tuple exists, the message is marked as `Delivered` without re-delivering to the recipient.
5. The deduplication set is persisted in the store and survives restarts.
6. Entries in the deduplication set expire after `2 * ack_timeout` seconds.

## Flush and Durability

The relationship between in-memory state and on-disk state:

| Event | In-Memory | On-Disk |
|-------|-----------|---------|
| Message sent | Updated immediately | Written at next flush |
| Channel created | Updated immediately | Written at next flush |
| Acknowledgment received | Updated immediately | Written at next flush |
| Store opened | Loaded from disk | Unchanged |
| Store flushed | Unchanged | Written atomically |
| Process crash | Lost since last flush | Last flushed state |

To minimize data loss on crash, applications can:
1. Reduce the batch flush interval.
2. Use immediate flush mode for critical channels.
3. Call `store.flush()` explicitly after important operations.
