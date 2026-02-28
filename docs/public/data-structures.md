---
status: draft
---

# Data Structures

This document specifies every data structure used in AgenticComm's core library and persistence layer. All structures are defined in Rust and serialized using bincode for on-disk storage. Field-level constraints, defaults, and invariants are documented for each structure.

## Message

The fundamental unit of communication. Every message is immutable after creation -- once persisted, its fields cannot be modified.

### Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | `u64` | auto | auto-increment | Unique message identifier within the store. Assigned by the engine at send time. |
| `message_type` | `MessageType` | yes | -- | One of: Text, Command, Query, Response, Broadcast, Notification, Acknowledgment, Error. |
| `sender` | `String` | yes | -- | Participant ID of the sender. Max 128 characters. |
| `channel_id` | `u64` | yes | -- | ID of the channel this message belongs to. |
| `content` | `String` | yes | -- | Message body. Max 1 MB (1,048,576 bytes) as UTF-8. |
| `topic` | `Option<String>` | no | `None` | Topic string for pub/sub routing. Dot-separated hierarchy. Max 256 characters. |
| `correlation_id` | `Option<String>` | no | `None` | UUID linking related messages (query-response pairs, command-acknowledgment chains). |
| `priority` | `Priority` | no | `Normal` | Delivery priority: Critical(0), High(1), Normal(2), Low(3), Background(4). |
| `metadata` | `Option<MessageMetadata>` | no | `None` | Structured key-value metadata attached to the message. |
| `created_at` | `u64` | auto | Unix timestamp | Creation time in seconds since Unix epoch (UTC). |
| `delivered_at` | `Option<u64>` | auto | `None` | Time the message was delivered. Set by the engine. |
| `acknowledged_at` | `Option<u64>` | auto | `None` | Time the message was acknowledged. Set when an ack is received. |
| `ttl` | `Option<u64>` | no | `None` | Time-to-live in seconds. Messages past TTL are not delivered. |
| `status` | `MessageStatus` | auto | `Created` | Current lifecycle status: Created, Sent, Delivered, Acknowledged, Failed, DeadLetter, Archived. |
| `retry_count` | `u32` | auto | `0` | Number of delivery attempts. Incremented on each retry. |
| `signature` | `Option<Vec<u8>>` | no | `None` | Cryptographic signature from AgenticIdentity. Ed25519 signature bytes. |

### MessageType Enum

```rust
#[repr(u8)]
pub enum MessageType {
    Text = 0,
    Command = 1,
    Query = 2,
    Response = 3,
    Broadcast = 4,
    Notification = 5,
    Acknowledgment = 6,
    Error = 7,
}
```

Serialized as a single `u8`. Values 8-255 are reserved for future message types and must be rejected by parsers.

### MessageStatus Enum

```rust
#[repr(u8)]
pub enum MessageStatus {
    Created = 0,
    Sent = 1,
    Delivered = 2,
    Acknowledged = 3,
    Failed = 4,
    DeadLetter = 5,
    Archived = 6,
}
```

### Priority Enum

```rust
#[repr(u8)]
pub enum Priority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}
```

### Validation Rules for Message

- `sender` must be non-empty and at most 128 bytes. Must match pattern `[a-zA-Z0-9_-]+`.
- `content` must be non-empty. Maximum 1,048,576 bytes.
- `topic`, if present, must be non-empty, at most 256 bytes, and match pattern `[a-zA-Z0-9_-]+(\.[a-zA-Z0-9_*#-]+)*`.
- `correlation_id`, if present, must be a valid UUID v4 string.
- `ttl`, if present, must be > 0.
- `channel_id` must refer to an existing channel in the store.

## Channel

A named communication context. Channels own messages and define delivery semantics.

### Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | `u64` | auto | auto-increment | Unique channel identifier. |
| `name` | `String` | yes | -- | Human-readable channel name. Max 128 characters. |
| `channel_type` | `ChannelType` | yes | -- | One of: Direct, Group, Broadcast, PubSub. |
| `owner` | `String` | yes | -- | Participant ID of the channel creator/owner. |
| `participants` | `Vec<Participant>` | auto | `[owner]` | List of participants with roles. |
| `config` | `ChannelConfig` | no | `ChannelConfig::default()` | Channel configuration (delivery semantics, retention, etc.). |
| `state` | `ChannelState` | auto | `Active` | Current lifecycle state: Active, Paused, Draining, Closed. |
| `created_at` | `u64` | auto | Unix timestamp | Creation time in seconds since Unix epoch. |
| `modified_at` | `u64` | auto | Unix timestamp | Last modification time. Updated on config changes, participant changes. |
| `message_count` | `u64` | auto | `0` | Total number of messages in this channel. |
| `description` | `Option<String>` | no | `None` | Optional human-readable description. Max 1024 characters. |
| `tags` | `Vec<String>` | no | `[]` | Optional tags for categorization. Each tag max 64 characters. |

### ChannelType Enum

```rust
#[repr(u8)]
pub enum ChannelType {
    Direct = 0,
    Group = 1,
    Broadcast = 2,
    PubSub = 3,
}
```

### ChannelState Enum

```rust
#[repr(u8)]
pub enum ChannelState {
    Active = 0,
    Paused = 1,
    Draining = 2,
    Closed = 3,
}
```

### Validation Rules for Channel

- `name` must be non-empty, at most 128 bytes. Must match pattern `[a-zA-Z0-9_-]+(/[a-zA-Z0-9_-]+)*`. Slashes are allowed for hierarchical naming (e.g., `team/backend/alerts`).
- `owner` must be a valid participant ID.
- Direct channels must have exactly 2 participants.
- Broadcast channels must have exactly 1 participant with Owner role.
- Channel names must be unique within a store.
- Maximum 100 tags per channel, each tag at most 64 bytes.

## Participant

A member of a channel with an assigned role.

### Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | `String` | yes | -- | Participant identifier. Max 128 characters. |
| `role` | `ParticipantRole` | no | `Member` | Role within the channel. |
| `joined_at` | `u64` | auto | Unix timestamp | When the participant joined. |
| `identity_id` | `Option<String>` | no | `None` | AgenticIdentity `aid_...` identifier for verified participants. |

### ParticipantRole Enum

```rust
#[repr(u8)]
pub enum ParticipantRole {
    Owner = 0,
    Member = 1,
    Observer = 2,
}
```

## Subscription

A pub/sub subscription linking a participant to a topic pattern on a channel.

### Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | `u64` | auto | auto-increment | Unique subscription identifier. |
| `channel_id` | `u64` | yes | -- | PubSub channel this subscription belongs to. |
| `subscriber` | `String` | yes | -- | Participant ID of the subscriber. |
| `topic_pattern` | `String` | yes | -- | Topic pattern to match. Supports exact, wildcard (*), and multi-level wildcard (#). |
| `match_mode` | `MatchMode` | auto | inferred | Inferred from pattern: Exact, Wildcard, MultiLevel. |
| `created_at` | `u64` | auto | Unix timestamp | When the subscription was created. |
| `active` | `bool` | auto | `true` | Whether the subscription is active. |
| `filter` | `Option<MessageFilter>` | no | `None` | Optional additional filter applied to matched messages. |

### MatchMode Enum

```rust
#[repr(u8)]
pub enum MatchMode {
    Exact = 0,    // "build.frontend.complete"
    Wildcard = 1, // "build.*.complete" (single segment wildcard)
    MultiLevel = 2, // "build.#" (zero or more segments)
}
```

Match mode is inferred from the topic pattern at subscription creation time. Patterns containing `*` are Wildcard. Patterns ending with `#` are MultiLevel. All other patterns are Exact.

## ChannelConfig

Configuration controlling channel behavior.

### Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `delivery` | `DeliveryMode` | no | `AtMostOnce` | Delivery guarantee: AtMostOnce, AtLeastOnce, ExactlyOnce. |
| `max_message_size` | `u64` | no | `1048576` (1 MB) | Maximum message content size in bytes. |
| `max_participants` | `Option<u32>` | no | `None` (unlimited) | Maximum number of participants. |
| `retention` | `RetentionPolicy` | no | `Forever` | Message retention policy. |
| `ack_timeout` | `Option<u64>` | no | `None` | Seconds to wait for acknowledgment before retry/escalation. |
| `max_retries` | `u32` | no | `3` | Maximum delivery retry attempts before dead-lettering. |
| `retry_backoff_ms` | `u64` | no | `1000` | Initial retry backoff in milliseconds. Doubles on each retry. |
| `echo` | `bool` | no | `false` | Whether senders receive their own messages. |
| `sticky_messages` | `bool` | no | `false` | Whether new participants receive messages sent before they joined. |
| `priority_ordering` | `bool` | no | `true` | Whether to respect message priority for delivery ordering. |

### DeliveryMode Enum

```rust
#[repr(u8)]
pub enum DeliveryMode {
    AtMostOnce = 0,  // Fire and forget. No retry.
    AtLeastOnce = 1, // Retry until acknowledged or max_retries exhausted.
    ExactlyOnce = 2, // Deduplication + at-least-once. Requires correlation_id.
}
```

### RetentionPolicy Enum

```rust
pub enum RetentionPolicy {
    Forever,
    Duration(u64),  // seconds
    Count(u64),     // max messages
    Size(u64),      // max bytes
}
```

Serialized as: tag byte (0=Forever, 1=Duration, 2=Count, 3=Size) followed by a `u64` value for non-Forever variants.

## MessageFilter

A filter used to query messages by various criteria.

### Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `channel_ids` | `Option<Vec<u64>>` | no | `None` | Filter to messages in these channels. |
| `sender` | `Option<String>` | no | `None` | Filter to messages from this sender. |
| `message_types` | `Option<Vec<MessageType>>` | no | `None` | Filter to these message types. |
| `topic_pattern` | `Option<String>` | no | `None` | Filter by topic (supports wildcards). |
| `after` | `Option<u64>` | no | `None` | Messages created after this timestamp. |
| `before` | `Option<u64>` | no | `None` | Messages created before this timestamp. |
| `status` | `Option<Vec<MessageStatus>>` | no | `None` | Filter by message status. |
| `priority` | `Option<Vec<Priority>>` | no | `None` | Filter by priority level. |
| `correlation_id` | `Option<String>` | no | `None` | Filter by correlation ID (thread). |
| `content_pattern` | `Option<String>` | no | `None` | Regex pattern to match against message content. |
| `limit` | `u64` | no | `100` | Maximum number of results to return. |
| `offset` | `u64` | no | `0` | Skip this many results (pagination). |
| `sort_by` | `SortField` | no | `CreatedAt` | Field to sort results by. |
| `sort_order` | `SortOrder` | no | `Descending` | Sort direction. |
| `include_archived` | `bool` | no | `false` | Include archived messages in results. |

### SortField Enum

```rust
pub enum SortField {
    CreatedAt,
    Priority,
    Sender,
    MessageType,
}
```

### SortOrder Enum

```rust
pub enum SortOrder {
    Ascending,
    Descending,
}
```

## MessageMetadata

Structured key-value metadata attached to a message. Used for application-specific data that does not fit into the core message fields.

### Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `entries` | `BTreeMap<String, MetadataValue>` | no | empty | Key-value pairs. Keys are strings, max 128 characters. Max 64 entries. |

### MetadataValue Enum

```rust
pub enum MetadataValue {
    String(String),   // Max 4096 bytes
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}
```

Metadata is serialized as a length-prefixed sequence of key-value pairs. Keys are sorted (BTreeMap) for deterministic serialization.

## CommStore

The in-memory representation of a `.acomm` file. Holds all channels, messages, subscriptions, and indexes.

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `path` | `PathBuf` | File path of the `.acomm` file. |
| `channels` | `HashMap<u64, Channel>` | All channels, keyed by channel ID. |
| `messages` | `Vec<Message>` | All messages, ordered by creation time. |
| `subscriptions` | `HashMap<u64, Subscription>` | All subscriptions, keyed by subscription ID. |
| `dead_letters` | `Vec<Message>` | Messages that failed delivery and exhausted retries. |
| `next_channel_id` | `u64` | Next available channel ID. |
| `next_message_id` | `u64` | Next available message ID. |
| `next_subscription_id` | `u64` | Next available subscription ID. |
| `modified` | `bool` | Dirty flag. Set to true when any mutation occurs. |
| `created_at` | `u64` | Store creation timestamp. |
| `modified_at` | `u64` | Last modification timestamp. |

### Invariants

- `next_channel_id` is always greater than the maximum channel ID in `channels`.
- `next_message_id` is always greater than the maximum message ID in `messages`.
- Every message's `channel_id` references a channel in `channels`.
- Every subscription's `channel_id` references a PubSub channel in `channels`.
- Direct channels have exactly 2 participants.
- Broadcast channels have exactly 1 owner; other participants are observers.

## Index Structures

Indexes are maintained in-memory for fast lookup and serialized to the `.acomm` file for persistence.

### ChannelMessageIndex

Maps channel IDs to the message IDs belonging to that channel.

```rust
pub struct ChannelMessageIndex {
    // channel_id -> sorted Vec of message_ids
    index: HashMap<u64, Vec<u64>>,
}
```

### TimestampIndex

Maps time ranges to message IDs for fast temporal queries.

```rust
pub struct TimestampIndex {
    // Sorted vec of (timestamp, message_id) for binary search
    entries: Vec<(u64, u64)>,
}
```

Temporal queries use binary search on the `entries` vector. A query for messages between timestamp A and B performs two binary searches (lower bound on A, upper bound on B) and returns the slice between them.

### TopicIndex

Maps topic strings to message IDs for fast pub/sub queries.

```rust
pub struct TopicIndex {
    // exact topic -> sorted Vec of message_ids
    exact: HashMap<String, Vec<u64>>,
    // topic segment count for wildcard matching
    segment_cache: HashMap<String, Vec<String>>,
}
```

### SenderIndex

Maps sender participant IDs to their message IDs.

```rust
pub struct SenderIndex {
    // sender -> sorted Vec of message_ids
    index: HashMap<String, Vec<u64>>,
}
```

### CorrelationIndex

Maps correlation IDs to all messages in the thread.

```rust
pub struct CorrelationIndex {
    // correlation_id -> sorted Vec of message_ids
    index: HashMap<String, Vec<u64>>,
}
```

## Serialization

### Bincode

All structures are serialized to binary format using bincode (version 2). Bincode is chosen over JSON for three reasons:

1. **Compact**: Bincode produces 5-10x smaller output than JSON for the same data.
2. **Fast**: Serialization and deserialization are zero-allocation for fixed-size types.
3. **Type-safe**: Bincode preserves Rust type information, preventing deserialization of incompatible data.

### Compression

Message content is compressed using flate2 (gzip level 6). The compression is applied to the content block as a whole, not per-message, because adjacent messages often share vocabulary (agent names, technical terms) that compress better together.

Compression ratio benchmarks:

| Content Type | Ratio | Notes |
|-------------|-------|-------|
| Agent conversations | 3.2:1 | Repetitive vocabulary, structured text |
| Code snippets | 2.8:1 | Syntax keywords, indentation |
| JSON payloads | 4.1:1 | Repeated keys, structured data |
| Binary attachments | 1.1:1 | Already compressed data gains little |

### Endianness

All multi-byte integers are stored in little-endian format, matching bincode's default behavior and the native byte order of x86-64 and ARM64 platforms.
