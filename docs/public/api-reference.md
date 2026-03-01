---
status: stable
---

# API Reference

Complete API reference for the AgenticComm core library. All functions are available through the Rust crate `agentic-comm`. Python and Node.js bindings expose the same API through the FFI layer.

## Store Operations

### `CommStore::create`

Create a new `.acomm` store file.

```rust
pub fn create(path: impl AsRef<Path>) -> Result<CommStore, CommError>
```

**Parameters:**
- `path` -- Path where the `.acomm` file will be created. Parent directory must exist.

**Returns:** A new `CommStore` instance with no channels or messages.

**Errors:**
- `CommError::FileExists` -- A file already exists at the given path.
- `CommError::IoError` -- The parent directory does not exist or is not writable.

**Example:**
```rust
let store = CommStore::create("project.acomm")?;
assert_eq!(store.channel_count(), 0);
assert_eq!(store.message_count(), 0);
```

### `CommStore::open`

Open an existing `.acomm` store file or create it if it does not exist.

```rust
pub fn open(path: impl AsRef<Path>) -> Result<CommStore, CommError>
```

**Parameters:**
- `path` -- Path to the `.acomm` file. Created if it does not exist.

**Returns:** A `CommStore` instance loaded with all channels, messages, and indexes.

**Errors:**
- `CommError::IntegrityError` -- The file exists but the SHA-256 checksum does not match.
- `CommError::VersionError` -- The file version is not supported by this library version.
- `CommError::IoError` -- File cannot be read.

**Example:**
```rust
let store = CommStore::open("project.acomm")?;
println!("Channels: {}, Messages: {}", store.channel_count(), store.message_count());
```

### `CommStore::flush`

Write all pending changes to disk.

```rust
pub fn flush(&mut self) -> Result<(), CommError>
```

**Returns:** `Ok(())` on success.

**Errors:**
- `CommError::IoError` -- Disk write failed.
- `CommError::StoreCapacity` -- Resulting file would exceed the 2 GB limit.

### `CommStore::stats`

Get aggregate statistics about the store.

```rust
pub fn stats(&self) -> StoreStats
```

**Returns:**
```rust
pub struct StoreStats {
    pub channel_count: u64,
    pub message_count: u64,
    pub subscription_count: u64,
    pub dead_letter_count: u64,
    pub file_size_bytes: u64,
    pub created_at: u64,
    pub modified_at: u64,
}
```

### `CommStore::compact`

Remove archived messages and dead letters older than the specified duration. Reclaims disk space.

```rust
pub fn compact(&mut self, older_than: Duration) -> Result<CompactResult, CommError>
```

**Parameters:**
- `older_than` -- Remove archived messages and dead letters older than this duration.

**Returns:**
```rust
pub struct CompactResult {
    pub messages_removed: u64,
    pub dead_letters_removed: u64,
    pub bytes_reclaimed: u64,
}
```

## Channel Operations

### `CommStore::create_channel`

Create a new communication channel.

```rust
pub fn create_channel(
    &mut self,
    name: &str,
    channel_type: ChannelType,
    owner: &str,
) -> Result<Channel, CommError>
```

**Parameters:**
- `name` -- Channel name. Max 128 characters, pattern `[a-zA-Z0-9_-]+(/[a-zA-Z0-9_-]+)*`.
- `channel_type` -- One of: `Direct`, `Group`, `Broadcast`, `PubSub`.
- `owner` -- Participant ID of the channel creator. Added as the first participant with `Owner` role.

**Returns:** The created `Channel` struct.

**Errors:**
- `CommError::ValidationFailed` -- Name or owner is invalid.
- `CommError::DuplicateChannel` -- A channel with this name already exists.
- `CommError::StoreCapacity` -- Maximum channel count reached.

**Example:**
```rust
let channel = store.create_channel("team/backend", ChannelType::Group, "agent-lead")?;
assert_eq!(channel.name, "team/backend");
assert_eq!(channel.channel_type, ChannelType::Group);
```

### `CommStore::get_channel`

Retrieve a channel by ID.

```rust
pub fn get_channel(&self, channel_id: u64) -> Result<&Channel, CommError>
```

**Errors:**
- `CommError::ChannelNotFound` -- No channel exists with the given ID.

### `CommStore::get_channel_by_name`

Retrieve a channel by name.

```rust
pub fn get_channel_by_name(&self, name: &str) -> Result<&Channel, CommError>
```

**Errors:**
- `CommError::ChannelNotFound` -- No channel exists with the given name.

### `CommStore::list_channels`

List all channels, optionally filtered by type or state.

```rust
pub fn list_channels(
    &self,
    channel_type: Option<ChannelType>,
    state: Option<ChannelState>,
) -> Vec<&Channel>
```

**Parameters:**
- `channel_type` -- Optional filter by channel type.
- `state` -- Optional filter by channel state.

**Returns:** A vector of channel references matching the filters.

### `CommStore::join_channel`

Add a participant to a channel.

```rust
pub fn join_channel(
    &mut self,
    channel_id: u64,
    participant: &str,
) -> Result<(), CommError>
```

**Parameters:**
- `channel_id` -- ID of the channel to join.
- `participant` -- Participant ID of the joining agent.

**Errors:**
- `CommError::ChannelNotFound` -- Channel does not exist.
- `CommError::ChannelClosed` -- Channel is in `Closed` state.
- `CommError::AlreadyParticipant` -- Participant is already in the channel.
- `CommError::MaxParticipants` -- Channel's participant limit reached.
- `CommError::ValidationFailed` -- Participant ID is invalid.

### `CommStore::leave_channel`

Remove a participant from a channel.

```rust
pub fn leave_channel(
    &mut self,
    channel_id: u64,
    participant: &str,
) -> Result<(), CommError>
```

**Errors:**
- `CommError::ChannelNotFound` -- Channel does not exist.
- `CommError::NotParticipant` -- Participant is not in the channel.
- `CommError::OwnerCannotLeave` -- The owner cannot leave without transferring ownership first.

### `CommStore::set_channel_config`

Update a channel's configuration.

```rust
pub fn set_channel_config(
    &mut self,
    channel_id: u64,
    config: ChannelConfig,
) -> Result<(), CommError>
```

**Errors:**
- `CommError::ChannelNotFound` -- Channel does not exist.
- `CommError::PermissionDenied` -- Caller is not the channel owner.

### `CommStore::set_channel_state`

Change a channel's lifecycle state.

```rust
pub fn set_channel_state(
    &mut self,
    channel_id: u64,
    state: ChannelState,
) -> Result<(), CommError>
```

Valid transitions: `Active -> Paused`, `Active -> Draining`, `Paused -> Active`, `Draining -> Closed`. Other transitions return `CommError::InvalidStateTransition`.

### `CommStore::delete_channel`

Delete a channel and all its messages.

```rust
pub fn delete_channel(&mut self, channel_id: u64) -> Result<DeleteResult, CommError>
```

**Returns:**
```rust
pub struct DeleteResult {
    pub messages_deleted: u64,
    pub subscriptions_deleted: u64,
}
```

**Errors:**
- `CommError::ChannelNotFound` -- Channel does not exist.
- `CommError::ChannelNotEmpty` -- Channel has pending (unacknowledged) messages. Use `force: true` or drain the channel first.

## Message Operations

### `CommStore::send_message`

Send a message through a channel.

```rust
pub fn send_message(
    &mut self,
    channel_id: u64,
    sender: &str,
    message_type: MessageType,
    content: &str,
) -> Result<Message, CommError>
```

**Parameters:**
- `channel_id` -- Target channel.
- `sender` -- Sending participant ID.
- `message_type` -- Type of message.
- `content` -- Message body (max 1 MB).

**Returns:** The sent `Message` with assigned ID, timestamp, and status `Sent`.

**Errors:**
- `CommError::ChannelNotFound` -- Channel does not exist.
- `CommError::ChannelClosed` -- Channel is closed or draining.
- `CommError::NotParticipant` -- Sender is not in the channel.
- `CommError::PermissionDenied` -- Sender is an Observer (cannot send).
- `CommError::ValidationFailed` -- Content is empty or exceeds size limit.
- `CommError::StoreFull` -- Store capacity limits reached.

### `CommStore::send_message_with_options`

Send a message with additional options (topic, priority, correlation, metadata, TTL).

```rust
pub fn send_message_with_options(
    &mut self,
    channel_id: u64,
    sender: &str,
    message_type: MessageType,
    content: &str,
    options: SendOptions,
) -> Result<Message, CommError>
```

**SendOptions:**
```rust
pub struct SendOptions {
    pub topic: Option<String>,
    pub correlation_id: Option<String>,
    pub priority: Option<Priority>,
    pub metadata: Option<MessageMetadata>,
    pub ttl: Option<u64>,
}
```

### `CommStore::send_query`

Convenience method to send a Query message with automatic correlation ID generation.

```rust
pub fn send_query(
    &mut self,
    channel_id: u64,
    sender: &str,
    content: &str,
    timeout: Option<Duration>,
) -> Result<Message, CommError>
```

**Returns:** A Query message with an auto-generated `correlation_id` (UUID v4).

### `CommStore::send_response`

Send a Response linked to a query via correlation ID.

```rust
pub fn send_response(
    &mut self,
    channel_id: u64,
    sender: &str,
    correlation_id: &str,
    content: &str,
) -> Result<Message, CommError>
```

**Errors:**
- `CommError::CorrelationNotFound` -- No Query with this correlation ID exists.

### `CommStore::acknowledge_message`

Send an acknowledgment for a Command message.

```rust
pub fn acknowledge_message(
    &mut self,
    message_id: u64,
    sender: &str,
    status: &str,
    content: Option<&str>,
) -> Result<Message, CommError>
```

**Parameters:**
- `message_id` -- ID of the Command message being acknowledged.
- `sender` -- The participant sending the acknowledgment.
- `status` -- One of: "received", "in_progress", "completed", "failed".
- `content` -- Optional acknowledgment body.

### `CommStore::get_message`

Retrieve a single message by ID.

```rust
pub fn get_message(&self, message_id: u64) -> Result<&Message, CommError>
```

### `CommStore::receive_messages`

Receive messages from a channel for a specific participant.

```rust
pub fn receive_messages(
    &self,
    channel_id: u64,
    recipient: &str,
    filter: Option<&MessageFilter>,
) -> Result<QueryResult, CommError>
```

### `CommStore::get_thread`

Get all messages in a correlation thread.

```rust
pub fn get_thread(&self, correlation_id: &str) -> Result<Vec<&Message>, CommError>
```

## Subscription Operations

### `CommStore::subscribe`

Subscribe to a topic pattern on a PubSub channel.

```rust
pub fn subscribe(
    &mut self,
    channel_id: u64,
    subscriber: &str,
    topic_pattern: &str,
) -> Result<Subscription, CommError>
```

**Errors:**
- `CommError::ChannelNotFound` -- Channel does not exist.
- `CommError::WrongChannelType` -- Channel is not a PubSub channel.
- `CommError::NotParticipant` -- Subscriber is not in the channel.
- `CommError::ValidationFailed` -- Topic pattern is invalid.
- `CommError::DuplicateSubscription` -- Subscriber already has this exact pattern.

### `CommStore::unsubscribe`

Remove a subscription.

```rust
pub fn unsubscribe(&mut self, subscription_id: u64) -> Result<(), CommError>
```

### `CommStore::publish`

Publish a message to a topic on a PubSub channel.

```rust
pub fn publish(
    &mut self,
    channel_id: u64,
    sender: &str,
    topic: &str,
    content: &str,
) -> Result<Message, CommError>
```

This is a convenience wrapper around `send_message_with_options` that sets the topic and message type to `Broadcast`.

### `CommStore::broadcast`

Send a broadcast message to all participants in a channel.

```rust
pub fn broadcast(
    &mut self,
    channel_id: u64,
    sender: &str,
    content: &str,
) -> Result<Message, CommError>
```

## Query Operations

### `CommStore::query_messages`

Query messages using a filter.

```rust
pub fn query_messages(&self, filter: &MessageFilter) -> Result<QueryResult, CommError>
```

See [Data Structures](data-structures.md) for `MessageFilter` fields.

### `CommStore::search_messages`

Full-text search across message content.

```rust
pub fn search_messages(
    &self,
    query: &str,
    filter: Option<&MessageFilter>,
) -> Result<QueryResult, CommError>
```

**Parameters:**
- `query` -- Search string. Supports regex if prefixed with `regex:`.
- `filter` -- Optional additional filter to narrow results.

### `CommStore::query_history`

Query the complete communication history with pagination.

```rust
pub fn query_history(
    &self,
    channel_id: Option<u64>,
    after: Option<u64>,
    before: Option<u64>,
    limit: u64,
    offset: u64,
) -> Result<QueryResult, CommError>
```

### `CommStore::dead_letters`

Get all messages in the dead letter queue.

```rust
pub fn dead_letters(&self) -> &[Message]
```

### `CommStore::replay_dead_letter`

Re-enter a dead letter message into the send pipeline.

```rust
pub fn replay_dead_letter(&mut self, message_id: u64) -> Result<Message, CommError>
```

## Error Types

```rust
pub enum CommError {
    // Store errors
    IoError(std::io::Error),
    IntegrityError { expected: String, actual: String },
    VersionError { file_version: u16, max_supported: u16 },
    FileExists(PathBuf),
    StoreFull { reason: String },
    StoreCapacity { current_bytes: u64, max_bytes: u64 },

    // Channel errors
    ChannelNotFound { id: u64 },
    DuplicateChannel { name: String },
    ChannelClosed { id: u64, state: ChannelState },
    WrongChannelType { expected: ChannelType, actual: ChannelType },
    MaxParticipants { channel_id: u64, max: u32 },
    InvalidStateTransition { from: ChannelState, to: ChannelState },
    ChannelNotEmpty { id: u64, pending: u64 },

    // Participant errors
    NotParticipant { participant: String, channel_id: u64 },
    AlreadyParticipant { participant: String, channel_id: u64 },
    OwnerCannotLeave { channel_id: u64 },
    PermissionDenied { participant: String, reason: String },

    // Message errors
    MessageNotFound { id: u64 },
    CorrelationNotFound { correlation_id: String },
    ValidationFailed { field: String, reason: String },
    DuplicateSubscription { subscriber: String, pattern: String },

    // Delivery errors
    DeliveryFailed { message_id: u64, reason: String },
    AcknowledgmentTimeout { message_id: u64, timeout_secs: u64 },
}
```

All error types implement `std::fmt::Display` with human-readable messages and `std::error::Error` for composability.
