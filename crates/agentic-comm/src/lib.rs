//! AgenticComm — agent-to-agent and agent-to-human communication engine.
//!
//! Provides structured messaging, channels, pub/sub, message routing, and
//! communication history stored in `.acomm` files.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read as IoRead, Write as IoWrite};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub mod affect;
pub mod bridges;
pub mod channel;
pub mod crypto;
pub mod encryption;
pub mod format;
pub mod query;
pub mod semantic;
pub mod temporal;
pub mod types;
pub mod workspace;

pub use affect::*;
pub use bridges::*;
pub use channel::*;
pub use crypto::*;
pub use encryption::*;
pub use format::*;
pub use query::*;
pub use semantic::*;
pub use temporal::*;
pub use types::*;
pub use workspace::*;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// All errors produced by the communication engine.
#[derive(thiserror::Error, Debug)]
pub enum CommError {
    /// Invalid channel name.
    #[error("Invalid channel name: {0}")]
    InvalidChannelName(String),

    /// Invalid message content.
    #[error("Invalid message content: {0}")]
    InvalidContent(String),

    /// Invalid sender.
    #[error("Invalid sender: {0}")]
    InvalidSender(String),

    /// Channel not found.
    #[error("Channel not found: {0}")]
    ChannelNotFound(u64),

    /// Message not found.
    #[error("Message not found: {0}")]
    MessageNotFound(u64),

    /// Subscription not found.
    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(u64),

    /// Channel is full.
    #[error("Channel {0} has reached maximum participants")]
    ChannelFull(u64),

    /// Participant not in channel.
    #[error("Participant '{0}' is not in channel {1}")]
    NotInChannel(String, u64),

    /// Participant already in channel.
    #[error("Participant '{0}' is already in channel {1}")]
    AlreadyInChannel(String, u64),

    /// Channel is in a state that does not allow the operation.
    #[error("Channel {0} is {1} — operation not allowed")]
    ChannelStateViolation(u64, String),

    /// Dead letter index out of bounds.
    #[error("Dead letter index {0} out of bounds")]
    DeadLetterNotFound(usize),

    /// I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid file format.
    #[error("Invalid .acomm file: {0}")]
    InvalidFile(String),

    /// Consent error.
    #[error("Consent error: {0}")]
    ConsentError(String),

    /// Trust level error.
    #[error("Trust error: {0}")]
    TrustError(String),

    /// Temporal scheduling error.
    #[error("Temporal error: {0}")]
    TemporalError(String),

    /// Federation error.
    #[error("Federation error: {0}")]
    FederationError(String),

    /// Hive mind error.
    #[error("Hive error: {0}")]
    HiveError(String),

    /// Consent denied — the recipient has not granted the required consent.
    #[error("Consent denied: {reason}")]
    ConsentDenied { reason: String },

    /// Rate limit exceeded — the sender has exceeded the configured rate limit.
    #[error("Rate limit exceeded: {limit}")]
    RateLimitExceeded { limit: String },

    /// File locking error.
    #[error("Lock error: {0}")]
    LockError(String),

    /// Key not found.
    #[error("Key not found: {0}")]
    KeyNotFound(u64),

    /// Generic not-found error.
    #[error("Not found: {0}")]
    NotFound(String),
}

/// Convenience result type.
pub type CommResult<T> = Result<T, CommError>;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Maximum message content size: 1 MB.
pub const MAX_CONTENT_SIZE: usize = 1_048_576;

/// Magic bytes for the .acomm file format.
pub const ACOMM_MAGIC: &[u8; 8] = b"ACOMM001";

/// File format version.
pub const ACOMM_VERSION: u32 = 1;

/// The type of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Plain text message.
    Text,
    /// A command to be executed.
    Command,
    /// A query expecting a response.
    Query,
    /// A response to a query.
    Response,
    /// A broadcast to all channel members.
    Broadcast,
    /// A system notification.
    Notification,
    /// Acknowledgment of receipt.
    Acknowledgment,
    /// An error message.
    Error,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Text => write!(f, "text"),
            MessageType::Command => write!(f, "command"),
            MessageType::Query => write!(f, "query"),
            MessageType::Response => write!(f, "response"),
            MessageType::Broadcast => write!(f, "broadcast"),
            MessageType::Notification => write!(f, "notification"),
            MessageType::Acknowledgment => write!(f, "acknowledgment"),
            MessageType::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for MessageType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(MessageType::Text),
            "command" => Ok(MessageType::Command),
            "query" => Ok(MessageType::Query),
            "response" => Ok(MessageType::Response),
            "broadcast" => Ok(MessageType::Broadcast),
            "notification" => Ok(MessageType::Notification),
            "acknowledgment" | "ack" => Ok(MessageType::Acknowledgment),
            "error" => Ok(MessageType::Error),
            other => Err(format!("Unknown message type: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// MessageStatus
// ---------------------------------------------------------------------------

/// Lifecycle status of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    /// Message has been created but not yet sent.
    Created,
    /// Message has been sent.
    Sent,
    /// Message has been delivered to the recipient.
    Delivered,
    /// Message has been read by the recipient.
    Read,
    /// Message has been acknowledged by the recipient.
    Acknowledged,
    /// Message sending failed.
    Failed,
    /// Message has expired (TTL exceeded).
    Expired,
    /// Message has been moved to the dead letter queue.
    DeadLettered,
}

impl Default for MessageStatus {
    fn default() -> Self {
        MessageStatus::Created
    }
}

impl std::fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageStatus::Created => write!(f, "created"),
            MessageStatus::Sent => write!(f, "sent"),
            MessageStatus::Delivered => write!(f, "delivered"),
            MessageStatus::Read => write!(f, "read"),
            MessageStatus::Acknowledged => write!(f, "acknowledged"),
            MessageStatus::Failed => write!(f, "failed"),
            MessageStatus::Expired => write!(f, "expired"),
            MessageStatus::DeadLettered => write!(f, "dead_lettered"),
        }
    }
}

// ---------------------------------------------------------------------------
// MessagePriority
// ---------------------------------------------------------------------------

/// Priority level for a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Low priority.
    Low = 0,
    /// Normal priority (default).
    Normal = 1,
    /// High priority.
    High = 2,
    /// Urgent priority.
    Urgent = 3,
    /// Critical priority.
    Critical = 4,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

impl std::fmt::Display for MessagePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessagePriority::Low => write!(f, "low"),
            MessagePriority::Normal => write!(f, "normal"),
            MessagePriority::High => write!(f, "high"),
            MessagePriority::Urgent => write!(f, "urgent"),
            MessagePriority::Critical => write!(f, "critical"),
        }
    }
}

// ---------------------------------------------------------------------------
// ChannelState
// ---------------------------------------------------------------------------

/// Operational state of a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelState {
    /// Channel is active and fully operational.
    Active,
    /// Channel is paused — no new messages can be sent or received.
    Paused,
    /// Channel is draining — receives are allowed but sends are blocked.
    Draining,
    /// Channel is closed — all operations are blocked.
    Closed,
    /// Channel is archived — read-only but searchable.
    Archived,
    /// Shared semantic space without words.
    SilentCommunion,
    /// Merged consciousness state.
    HiveMode,
    /// Awaiting participant consent.
    PendingConsent,
}

impl Default for ChannelState {
    fn default() -> Self {
        ChannelState::Active
    }
}

impl std::fmt::Display for ChannelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelState::Active => write!(f, "active"),
            ChannelState::Paused => write!(f, "paused"),
            ChannelState::Draining => write!(f, "draining"),
            ChannelState::Closed => write!(f, "closed"),
            ChannelState::Archived => write!(f, "archived"),
            ChannelState::SilentCommunion => write!(f, "silent_communion"),
            ChannelState::HiveMode => write!(f, "hive_mode"),
            ChannelState::PendingConsent => write!(f, "pending_consent"),
        }
    }
}

// ---------------------------------------------------------------------------
// DeliveryMode
// ---------------------------------------------------------------------------

/// Message delivery semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryMode {
    /// Message may be lost (fire and forget).
    AtMostOnce,
    /// Message will be delivered at least once (may duplicate).
    AtLeastOnce,
    /// Message will be delivered exactly once.
    ExactlyOnce,
}

impl Default for DeliveryMode {
    fn default() -> Self {
        DeliveryMode::AtLeastOnce
    }
}

// ---------------------------------------------------------------------------
// RetentionPolicy
// ---------------------------------------------------------------------------

/// How long messages are retained in a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionPolicy {
    /// Messages are retained forever.
    Forever,
    /// Messages are retained for a given number of seconds.
    Duration(u64),
    /// Only the most recent N messages are retained.
    MessageCount(u64),
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        RetentionPolicy::Forever
    }
}

// ---------------------------------------------------------------------------
// DeadLetter
// ---------------------------------------------------------------------------

/// Reason a message was dead-lettered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadLetterReason {
    /// The target channel was closed.
    ChannelClosed,
    /// The target channel was not found.
    ChannelNotFound,
    /// The intended recipient was unavailable.
    RecipientUnavailable,
    /// Maximum retry attempts were exceeded.
    MaxRetriesExceeded,
    /// The message expired (TTL exceeded).
    Expired,
    /// The message failed validation.
    ValidationFailed(String),
}

impl std::fmt::Display for DeadLetterReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeadLetterReason::ChannelClosed => write!(f, "channel_closed"),
            DeadLetterReason::ChannelNotFound => write!(f, "channel_not_found"),
            DeadLetterReason::RecipientUnavailable => write!(f, "recipient_unavailable"),
            DeadLetterReason::MaxRetriesExceeded => write!(f, "max_retries_exceeded"),
            DeadLetterReason::Expired => write!(f, "expired"),
            DeadLetterReason::ValidationFailed(s) => write!(f, "validation_failed: {s}"),
        }
    }
}

/// A message that could not be delivered and was placed in the dead letter queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetter {
    /// The original message that failed delivery.
    pub original_message: Message,
    /// Why the message was dead-lettered.
    pub reason: DeadLetterReason,
    /// When the message was dead-lettered.
    pub dead_lettered_at: DateTime<Utc>,
    /// Number of delivery retries attempted.
    pub retry_count: u32,
}

// ---------------------------------------------------------------------------
// Key management
// ---------------------------------------------------------------------------

/// A key entry for channel encryption metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEntry {
    /// Unique key identifier.
    pub id: u64,
    /// Algorithm name (e.g. "aes-256-gcm", "x25519").
    pub algorithm: String,
    /// Creation timestamp (seconds since epoch).
    pub created_at: u64,
    /// Key status: "active", "rotated", or "revoked".
    pub status: String,
    /// Optional channel this key is bound to.
    pub channel_id: Option<u64>,
    /// Fingerprint of the key material.
    pub fingerprint: String,
}

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

/// A single message in the communication system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier.
    pub id: u64,
    /// Channel this message belongs to.
    pub channel_id: u64,
    /// Who sent the message.
    pub sender: String,
    /// Optional specific recipient (None = all channel participants).
    pub recipient: Option<String>,
    /// Message body.
    pub content: String,
    /// Type of message.
    pub message_type: MessageType,
    /// When the message was created (UTC).
    pub timestamp: DateTime<Utc>,
    /// Arbitrary key-value metadata.
    pub metadata: HashMap<String, String>,
    /// Optional SHA-256 content signature.
    pub signature: Option<String>,
    /// Set of participants who have acknowledged this message.
    #[serde(default)]
    pub acknowledged_by: Vec<String>,
    /// Lifecycle status of this message.
    #[serde(default)]
    pub status: MessageStatus,
    /// Priority level of this message.
    #[serde(default)]
    pub priority: MessagePriority,
    /// ID of the message this is a reply to.
    #[serde(default)]
    pub reply_to: Option<u64>,
    /// Correlation ID for request/response pairing.
    #[serde(default)]
    pub correlation_id: Option<String>,
    /// Thread grouping identifier.
    #[serde(default)]
    pub thread_id: Option<String>,
    /// Causal timestamp with Lamport/vector clocks.
    #[serde(default)]
    pub comm_timestamp: CommTimestamp,
    /// Rich message content stored as JSON (avoids bincode enum issues).
    /// Use `MessageContent::from_json_string` / `to_json_string` to convert.
    #[serde(default)]
    pub rich_content_json: Option<String>,
    /// UUID-based universal identifier (alongside legacy u64 id).
    #[serde(default)]
    pub comm_id: Option<CommId>,
}

/// The type of a communication channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    /// 1:1 direct message.
    Direct,
    /// Group conversation.
    Group,
    /// One-to-many broadcast.
    Broadcast,
    /// Publish/subscribe topic.
    PubSub,
    /// Shared state space channel.
    Telepathic,
    /// Hive mind channel.
    Hive,
    /// Time-shifted messaging.
    Temporal,
    /// Fate-linked communication.
    Destiny,
    /// Predictive messaging.
    Oracle,
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelType::Direct => write!(f, "direct"),
            ChannelType::Group => write!(f, "group"),
            ChannelType::Broadcast => write!(f, "broadcast"),
            ChannelType::PubSub => write!(f, "pubsub"),
            ChannelType::Telepathic => write!(f, "telepathic"),
            ChannelType::Hive => write!(f, "hive"),
            ChannelType::Temporal => write!(f, "temporal"),
            ChannelType::Destiny => write!(f, "destiny"),
            ChannelType::Oracle => write!(f, "oracle"),
        }
    }
}

impl std::str::FromStr for ChannelType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "direct" => Ok(ChannelType::Direct),
            "group" => Ok(ChannelType::Group),
            "broadcast" => Ok(ChannelType::Broadcast),
            "pubsub" => Ok(ChannelType::PubSub),
            "telepathic" => Ok(ChannelType::Telepathic),
            "hive" => Ok(ChannelType::Hive),
            "temporal" => Ok(ChannelType::Temporal),
            "destiny" => Ok(ChannelType::Destiny),
            "oracle" => Ok(ChannelType::Oracle),
            other => Err(format!("Unknown channel type: {other}")),
        }
    }
}

/// Configuration for a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Maximum number of participants (0 = unlimited).
    pub max_participants: u32,
    /// Message time-to-live in seconds (0 = forever).
    pub ttl_seconds: u64,
    /// Whether messages should be persisted.
    pub persistence: bool,
    /// Whether encryption is required for this channel.
    pub encryption_required: bool,
    /// Message delivery semantics.
    #[serde(default)]
    pub delivery_mode: DeliveryMode,
    /// How long messages are retained.
    #[serde(default)]
    pub retention_policy: RetentionPolicy,
    /// Minimum trust level required to send messages or join this channel.
    /// If `None`, no trust check is enforced.
    #[serde(default)]
    pub min_trust_level: Option<CommTrustLevel>,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            max_participants: 0,
            ttl_seconds: 0,
            persistence: true,
            encryption_required: false,
            delivery_mode: DeliveryMode::default(),
            retention_policy: RetentionPolicy::default(),
            min_trust_level: None,
        }
    }
}

/// A communication channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// Unique channel identifier.
    pub id: u64,
    /// Human-readable channel name.
    pub name: String,
    /// Type of channel.
    pub channel_type: ChannelType,
    /// When the channel was created.
    pub created_at: DateTime<Utc>,
    /// Current participants.
    pub participants: Vec<String>,
    /// Channel configuration.
    pub config: ChannelConfig,
    /// Operational state of the channel.
    #[serde(default)]
    pub state: ChannelState,
    /// UUID-based universal identifier (alongside legacy u64 id).
    #[serde(default)]
    pub comm_id: Option<CommId>,
}

/// A pub/sub subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Unique subscription identifier.
    pub id: u64,
    /// Topic being subscribed to.
    pub topic: String,
    /// Who is subscribed.
    pub subscriber: String,
    /// When the subscription was created.
    pub created_at: DateTime<Utc>,
}

/// Filter for querying message history.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageFilter {
    /// Only messages after this time.
    pub since: Option<DateTime<Utc>>,
    /// Only messages before this time.
    pub before: Option<DateTime<Utc>>,
    /// Only messages from this sender.
    pub sender: Option<String>,
    /// Only messages of this type.
    pub message_type: Option<MessageType>,
    /// Maximum number of results.
    pub limit: Option<usize>,
    /// Filter by message priority (numeric: 0=Low, 1=Normal, 2=High, 3=Urgent, 4=Critical).
    #[serde(default)]
    pub priority: Option<u32>,
    /// Filter by thread identifier.
    #[serde(default)]
    pub thread_id: Option<u64>,
    /// Filter by content substring (case-insensitive).
    #[serde(default)]
    pub content_contains: Option<String>,
}

/// File header for .acomm files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcommHeader {
    /// Magic bytes (always "ACOMM001").
    pub magic: [u8; 8],
    /// Format version.
    pub version: u32,
    /// Number of channels in the file.
    pub channel_count: u32,
    /// Number of messages in the file.
    pub message_count: u64,
}

// ---------------------------------------------------------------------------
// CommStore — the main store
// ---------------------------------------------------------------------------

/// Per-sender rate tracking state.
#[derive(Debug, Clone, Default)]
pub struct RateTracker {
    /// Number of messages sent in the current minute window.
    pub message_count_minute: u32,
    /// Epoch second when the minute window was last reset.
    pub last_minute_reset: u64,
    /// Number of messages sent in the current hour window.
    pub message_count_hour: u32,
    /// Epoch second when the hour window was last reset.
    pub last_hour_reset: u64,
}

// ---------------------------------------------------------------------------
// File locking
// ---------------------------------------------------------------------------

/// Advisory file lock for concurrent access to `.acomm` files.
///
/// Uses `fs2` advisory locks so that multiple processes can safely read/write
/// the same `.acomm` file without corruption.  The sidecar lock file lives at
/// `<data_path>.acomm.lock`.
pub struct CommFileLock {
    lock_file: File,
    lock_path: PathBuf,
}

impl CommFileLock {
    /// Acquire an exclusive lock on the `.acomm` file (blocks until available).
    pub fn acquire(data_path: &Path) -> CommResult<Self> {
        let lock_path = data_path.with_extension("acomm.lock");
        let lock_file = File::create(&lock_path).map_err(|e| {
            CommError::LockError(format!("Failed to create lock file: {}", e))
        })?;
        lock_file.lock_exclusive().map_err(|e| {
            CommError::LockError(format!("Failed to acquire exclusive lock: {}", e))
        })?;
        Ok(Self { lock_file, lock_path })
    }

    /// Try to acquire an exclusive lock without blocking.
    ///
    /// Returns an error immediately if the lock is already held by another
    /// process.
    pub fn try_acquire(data_path: &Path) -> CommResult<Self> {
        let lock_path = data_path.with_extension("acomm.lock");
        let lock_file = File::create(&lock_path).map_err(|e| {
            CommError::LockError(format!("Failed to create lock file: {}", e))
        })?;
        lock_file.try_lock_exclusive().map_err(|e| {
            CommError::LockError(format!("Failed to acquire lock (already held): {}", e))
        })?;
        Ok(Self { lock_file, lock_path })
    }

    /// Acquire a shared (read) lock on the `.acomm` file (blocks until
    /// available).
    ///
    /// Multiple readers can hold a shared lock simultaneously, but a shared
    /// lock blocks exclusive writers.
    pub fn acquire_shared(data_path: &Path) -> CommResult<Self> {
        let lock_path = data_path.with_extension("acomm.lock");
        let lock_file = File::create(&lock_path).map_err(|e| {
            CommError::LockError(format!("Failed to create lock file: {}", e))
        })?;
        lock_file.lock_shared().map_err(|e| {
            CommError::LockError(format!("Failed to acquire shared lock: {}", e))
        })?;
        Ok(Self { lock_file, lock_path })
    }

    /// Explicitly release the lock and attempt to clean up the lock file.
    pub fn release(self) -> CommResult<()> {
        self.lock_file.unlock().map_err(|e| {
            CommError::LockError(format!("Failed to release lock: {}", e))
        })?;
        // Best-effort cleanup of the sidecar file.
        let _ = std::fs::remove_file(&self.lock_path);
        Ok(())
    }

    /// Check if the lock file is stale (older than `max_age_secs` seconds) and
    /// remove it if so.
    ///
    /// Returns `Ok(true)` if a stale lock was recovered, `Ok(false)` otherwise.
    pub fn recover_stale(data_path: &Path, max_age_secs: u64) -> CommResult<bool> {
        let lock_path = data_path.with_extension("acomm.lock");
        if lock_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&lock_path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed.as_secs() > max_age_secs {
                            let _ = std::fs::remove_file(&lock_path);
                            return Ok(true);
                        }
                    }
                }
            }
        }
        Ok(false)
    }
}

impl Drop for CommFileLock {
    fn drop(&mut self) {
        let _ = self.lock_file.unlock();
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

// ---------------------------------------------------------------------------
// CommStore
// ---------------------------------------------------------------------------

/// The main communication store holding channels, messages, and subscriptions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommStore {
    /// All channels, keyed by channel id.
    pub channels: HashMap<u64, Channel>,
    /// All messages, keyed by message id.
    pub messages: HashMap<u64, Message>,
    /// All subscriptions, keyed by subscription id.
    pub subscriptions: HashMap<u64, Subscription>,
    /// Next channel id.
    next_channel_id: u64,
    /// Next message id.
    next_message_id: u64,
    /// Next subscription id.
    next_subscription_id: u64,
    /// Dead letter queue for undeliverable messages.
    #[serde(default)]
    pub dead_letters: Vec<DeadLetter>,

    /// Consent gates: (grantor, grantee, scope) -> ConsentGateEntry.
    #[serde(default)]
    pub consent_gates: Vec<ConsentGateEntry>,

    /// Trust level overrides: agent_id -> trust_level.
    #[serde(default)]
    pub trust_levels: HashMap<String, CommTrustLevel>,

    /// Temporal message queue.
    #[serde(default)]
    pub temporal_queue: Vec<TemporalMessage>,

    /// Next temporal message ID.
    #[serde(default = "default_one")]
    next_temporal_id: u64,

    /// Federation configuration.
    #[serde(default)]
    pub federation_config: FederationConfig,

    /// Hive minds.
    #[serde(default)]
    pub hive_minds: HashMap<u64, HiveMind>,

    /// Next hive mind ID.
    #[serde(default = "default_one")]
    next_hive_id: u64,

    /// Communication log entries.
    #[serde(default)]
    pub comm_log: Vec<CommunicationLogEntry>,

    /// Next log entry index.
    #[serde(default = "default_one")]
    next_log_index: u64,

    /// Audit log entries.
    #[serde(default)]
    pub audit_log: Vec<AuditEntry>,

    /// Rate limit configuration.
    #[serde(default)]
    pub rate_limit_config: RateLimitConfig,

    /// Semantic operations log.
    #[serde(default)]
    pub semantic_operations: Vec<SemanticOperation>,

    /// Next semantic operation ID.
    #[serde(default = "default_one")]
    next_semantic_id: u64,

    /// Semantic conflicts.
    #[serde(default)]
    pub semantic_conflicts: Vec<SemanticConflict>,

    /// Per-agent affect states.
    #[serde(default)]
    pub affect_states: HashMap<String, AffectState>,

    /// Affect contagion resistance (global default).
    #[serde(default = "default_resistance")]
    pub affect_resistance: f64,

    /// Pending consent requests.
    #[serde(default)]
    pub pending_consent_requests: Vec<ConsentRequest>,

    /// Meld sessions.
    #[serde(default)]
    pub meld_sessions: Vec<MeldSession>,

    /// Per-zone federation policies.
    #[serde(default)]
    pub zone_policies: HashMap<String, ZonePolicyConfig>,

    /// Key metadata for channel encryption.
    #[serde(default)]
    pub key_store: Vec<KeyEntry>,

    /// Next key ID.
    #[serde(default = "default_one")]
    next_key_id: u64,

    /// Global Lamport counter for causal ordering of messages.
    #[serde(default)]
    pub lamport_counter: u64,

    /// Per-sender rate tracking (not persisted — rebuilt at runtime).
    #[serde(skip)]
    pub rate_trackers: HashMap<String, RateTracker>,

    /// Optional Ed25519 key pair for cryptographic message signing.
    /// Not serialized — must be set at runtime via `set_signing_key`.
    #[serde(skip)]
    pub key_pair: Option<CommKeyPair>,

    /// Registered communicating agents (agent_id -> CommunicatingAgent).
    #[serde(default)]
    pub agents: HashMap<String, CommunicatingAgent>,

    /// Bridge configuration for sister integrations.
    /// Not serialized — must be set at runtime via `set_bridge_config`.
    #[serde(skip)]
    pub bridge_config: BridgeConfig,
}

fn default_one() -> u64 {
    1
}

fn default_resistance() -> f64 {
    0.5
}

impl Default for CommStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CommStore {
    /// Create a new empty communication store.
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            messages: HashMap::new(),
            subscriptions: HashMap::new(),
            next_channel_id: 1,
            next_message_id: 1,
            next_subscription_id: 1,
            dead_letters: Vec::new(),
            consent_gates: Vec::new(),
            trust_levels: HashMap::new(),
            temporal_queue: Vec::new(),
            next_temporal_id: 1,
            federation_config: FederationConfig::default(),
            hive_minds: HashMap::new(),
            next_hive_id: 1,
            comm_log: Vec::new(),
            next_log_index: 1,
            audit_log: Vec::new(),
            rate_limit_config: RateLimitConfig::default(),
            semantic_operations: Vec::new(),
            next_semantic_id: 1,
            semantic_conflicts: Vec::new(),
            affect_states: HashMap::new(),
            affect_resistance: 0.5,
            pending_consent_requests: Vec::new(),
            meld_sessions: Vec::new(),
            zone_policies: HashMap::new(),
            key_store: Vec::new(),
            next_key_id: 1,
            lamport_counter: 0,
            rate_trackers: HashMap::new(),
            key_pair: None,
            agents: HashMap::new(),
            bridge_config: BridgeConfig::default(),
        }
    }

    // -----------------------------------------------------------------------
    // Bridge configuration
    // -----------------------------------------------------------------------

    /// Set the bridge configuration for sister integrations.
    pub fn set_bridge_config(&mut self, config: BridgeConfig) {
        self.bridge_config = config;
    }

    // -----------------------------------------------------------------------
    // Agent registry
    // -----------------------------------------------------------------------

    /// Register a new communicating agent.
    pub fn register_agent(&mut self, agent: CommunicatingAgent) -> CommResult<()> {
        let agent_id = agent.agent_id.clone();
        self.agents.insert(agent_id.clone(), agent);
        self.log_audit(
            AuditEventType::AgentRegistered,
            &agent_id,
            &format!("Agent registered: {}", agent_id),
            Some(agent_id.clone()),
        );
        Ok(())
    }

    /// Get a registered agent by ID.
    pub fn get_agent(&self, agent_id: &str) -> Option<&CommunicatingAgent> {
        self.agents.get(agent_id)
    }

    /// List all registered agents.
    pub fn list_agents(&self) -> Vec<&CommunicatingAgent> {
        self.agents.values().collect()
    }

    /// Update agent availability/presence.
    pub fn update_agent_availability(
        &mut self,
        agent_id: &str,
        availability: Availability,
    ) -> CommResult<()> {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.availability = availability;
            Ok(())
        } else {
            Err(CommError::NotFound(format!("Agent not found: {}", agent_id)))
        }
    }

    /// Remove a registered agent.
    pub fn unregister_agent(&mut self, agent_id: &str) -> CommResult<()> {
        if self.agents.remove(agent_id).is_some() {
            self.log_audit(
                AuditEventType::AgentUnregistered,
                agent_id,
                &format!("Agent unregistered: {}", agent_id),
                Some(agent_id.to_string()),
            );
            Ok(())
        } else {
            Err(CommError::NotFound(format!("Agent not found: {}", agent_id)))
        }
    }

    // -----------------------------------------------------------------------
    // Validation helpers
    // -----------------------------------------------------------------------

    fn validate_channel_name(name: &str) -> CommResult<()> {
        if name.is_empty() || name.len() > 128 {
            return Err(CommError::InvalidChannelName(
                "Channel name must be 1-128 characters".to_string(),
            ));
        }
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(CommError::InvalidChannelName(
                "Channel name must contain only alphanumeric characters, hyphens, or underscores"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn validate_content(content: &str) -> CommResult<()> {
        if content.is_empty() {
            return Err(CommError::InvalidContent(
                "Message content cannot be empty".to_string(),
            ));
        }
        if content.len() > MAX_CONTENT_SIZE {
            return Err(CommError::InvalidContent(format!(
                "Message content exceeds maximum size of {} bytes",
                MAX_CONTENT_SIZE
            )));
        }
        Ok(())
    }

    fn validate_sender(sender: &str) -> CommResult<()> {
        if sender.is_empty() {
            return Err(CommError::InvalidSender(
                "Sender cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    /// Compute a SHA-256 hash signature (legacy, used as fallback).
    fn compute_sha256_signature(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Compute a message signature.
    ///
    /// When an Ed25519 key pair is set, produces a real cryptographic
    /// signature (128 hex chars / 64 bytes). Otherwise falls back to a
    /// SHA-256 content hash (64 hex chars / 32 bytes).
    fn compute_signature(&self, content: &str) -> String {
        match &self.key_pair {
            Some(kp) => kp.sign(content),
            None => Self::compute_sha256_signature(content),
        }
    }

    /// Set the Ed25519 key pair used for signing outgoing messages.
    pub fn set_signing_key(&mut self, key_pair: CommKeyPair) {
        self.key_pair = Some(key_pair);
    }

    /// Get the hex-encoded Ed25519 public key, if a key pair is set.
    pub fn get_public_key(&self) -> Option<String> {
        self.key_pair.as_ref().map(|kp| kp.public_key_hex())
    }

    /// Verify that a message's signature is valid.
    ///
    /// Ed25519 signatures are 64 bytes (128 hex chars). If the stored
    /// signature is that length and a key pair is set, Ed25519 verification
    /// is attempted first. Falls back to SHA-256 hash comparison for
    /// legacy signatures (64 hex chars / 32 bytes).
    ///
    /// Returns true if valid (or no signature stored), false if mismatch.
    pub fn verify_message_signature(&mut self, message_id: u64) -> bool {
        let (content, stored_sig, sender) = match self.messages.get(&message_id) {
            Some(msg) => (
                msg.content.clone(),
                msg.signature.clone(),
                msg.sender.clone(),
            ),
            None => return true,
        };
        match stored_sig {
            Some(ref sig) => {
                // Ed25519 signatures are 128 hex chars (64 bytes).
                // SHA-256 hashes are 64 hex chars (32 bytes).
                let valid = if sig.len() == 128 {
                    // Try Ed25519 verification if we have a public key
                    if let Some(ref kp) = self.key_pair {
                        crypto::verify_signature(&kp.public_key_hex(), &content, sig)
                    } else {
                        // No key pair set — cannot verify Ed25519 sig
                        false
                    }
                } else {
                    // Legacy SHA-256 hash comparison
                    let expected = Self::compute_sha256_signature(&content);
                    *sig == expected
                };

                if !valid {
                    self.log_audit(
                        AuditEventType::SignatureWarning,
                        &sender,
                        &format!(
                            "Signature mismatch for message {}: stored={}",
                            message_id, sig
                        ),
                        Some(message_id.to_string()),
                    );
                }
                valid
            }
            None => true,
        }
    }

    /// Check consent for sending a message to participants on the channel.
    /// For affect-enriched or other rich content types, requires explicit
    /// SendMessages consent from each recipient on the channel.
    fn check_send_consent(
        &self,
        channel_id: u64,
        sender: &str,
        content: &str,
    ) -> CommResult<()> {
        let channel = match self.channels.get(&channel_id) {
            Some(ch) => ch,
            None => return Ok(()),
        };

        // Detect rich content that needs explicit consent
        let is_rich_content = content.starts_with("[affect:");

        if !is_rich_content {
            return Ok(());
        }

        // For rich content, check that each participant (other than sender) has
        // granted SendMessages consent to the sender.
        for participant in &channel.participants {
            if participant == sender {
                continue;
            }
            let has_consent = self.consent_gates.iter().any(|e| {
                e.grantor == *participant
                    && e.grantee == sender
                    && e.scope == ConsentScope::SendMessages
                    && e.status == ConsentStatus::Granted
            });
            if !has_consent {
                return Err(CommError::ConsentDenied {
                    reason: format!(
                        "Participant '{}' has not granted SendMessages consent to '{}'",
                        participant, sender
                    ),
                });
            }
        }

        Ok(())
    }

    /// Check and enforce rate limits for a sender. Returns Ok(()) if allowed,
    /// or RateLimitExceeded if the sender has exceeded the configured rate.
    fn check_rate_limit(&mut self, sender: &str) -> CommResult<()> {
        let now_epoch = Utc::now().timestamp() as u64;
        let limit_per_minute = self.rate_limit_config.messages_per_minute;

        let tracker = self
            .rate_trackers
            .entry(sender.to_string())
            .or_default();

        // Reset minute window if more than 60 seconds have elapsed
        if now_epoch - tracker.last_minute_reset >= 60 {
            tracker.message_count_minute = 0;
            tracker.last_minute_reset = now_epoch;
        }

        // Reset hour window if more than 3600 seconds have elapsed
        if now_epoch - tracker.last_hour_reset >= 3600 {
            tracker.message_count_hour = 0;
            tracker.last_hour_reset = now_epoch;
        }

        // Check minute limit
        if tracker.message_count_minute >= limit_per_minute {
            return Err(CommError::RateLimitExceeded {
                limit: format!(
                    "{} messages per minute (sender: {})",
                    limit_per_minute, sender
                ),
            });
        }

        // Increment counters
        tracker.message_count_minute += 1;
        tracker.message_count_hour += 1;

        Ok(())
    }

    /// Check that a channel allows sending. Returns Ok(()) if allowed,
    /// or an appropriate error if the channel is in a blocking state.
    fn check_channel_allows_send(&self, channel_id: u64) -> CommResult<()> {
        let channel = self
            .channels
            .get(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;

        match channel.state {
            ChannelState::Active => Ok(()),
            ChannelState::SilentCommunion | ChannelState::HiveMode => Ok(()),
            ChannelState::Paused => Err(CommError::ChannelStateViolation(
                channel_id,
                "paused".to_string(),
            )),
            ChannelState::Draining => Err(CommError::ChannelStateViolation(
                channel_id,
                "draining".to_string(),
            )),
            ChannelState::Closed => Err(CommError::ChannelStateViolation(
                channel_id,
                "closed".to_string(),
            )),
            ChannelState::Archived => Err(CommError::ChannelStateViolation(
                channel_id,
                "archived".to_string(),
            )),
            ChannelState::PendingConsent => Err(CommError::ChannelStateViolation(
                channel_id,
                "pending_consent".to_string(),
            )),
        }
    }

    /// Check that a channel allows receiving. Returns Ok(()) if allowed.
    fn check_channel_allows_receive(&self, channel_id: u64) -> CommResult<()> {
        let channel = self
            .channels
            .get(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;

        match channel.state {
            ChannelState::Active | ChannelState::Draining => Ok(()),
            ChannelState::SilentCommunion | ChannelState::HiveMode => Ok(()),
            ChannelState::Archived => Ok(()),
            ChannelState::Paused => Err(CommError::ChannelStateViolation(
                channel_id,
                "paused".to_string(),
            )),
            ChannelState::Closed => Err(CommError::ChannelStateViolation(
                channel_id,
                "closed".to_string(),
            )),
            ChannelState::PendingConsent => Err(CommError::ChannelStateViolation(
                channel_id,
                "pending_consent".to_string(),
            )),
        }
    }

    /// Check that an agent's trust level meets a channel's minimum requirement.
    ///
    /// Returns `Ok(())` if the channel has no `min_trust_level` or if the
    /// agent's trust level meets or exceeds it.  Returns `TrustError`
    /// otherwise.
    fn check_trust_for_channel(&self, agent: &str, channel_id: u64) -> CommResult<()> {
        let channel = self
            .channels
            .get(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;
        if let Some(min_trust) = channel.config.min_trust_level {
            let agent_trust = self.get_trust_level(agent);
            if agent_trust < min_trust {
                return Err(CommError::TrustError(format!(
                    "Trust level insufficient: {:?} < {:?}",
                    agent_trust, min_trust
                )));
            }
        }
        Ok(())
    }

    /// General consent check for an agent performing an action in a given scope.
    ///
    /// Returns `true` (allow) when:
    /// - No consent gate exists for the requested scope (open-by-default), OR
    /// - An explicit `Granted` entry exists for the agent + scope.
    ///
    /// Returns `false` (deny) when:
    /// - A consent gate exists for the scope but the agent's entry is not
    ///   `Granted` (i.e. it is `Denied`, `Revoked`, `Pending`, or `Expired`).
    fn check_consent_for_action(&self, agent: &str, _resource: &str, scope: ConsentScope) -> bool {
        // Collect all gates matching this scope
        let scope_gates: Vec<&ConsentGateEntry> = self
            .consent_gates
            .iter()
            .filter(|e| e.scope == scope)
            .collect();

        // Open-by-default: if there are no gates for this scope at all, allow.
        if scope_gates.is_empty() {
            return true;
        }

        // Check if the agent has an explicit Granted entry (as grantee)
        scope_gates.iter().any(|e| {
            e.grantee == agent && e.status == ConsentStatus::Granted
        })
    }

    // -----------------------------------------------------------------------
    // Channel state management
    // -----------------------------------------------------------------------

    /// Pause a channel. Blocks new sends and receives.
    pub fn pause_channel(&mut self, channel_id: u64) -> CommResult<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;
        channel.state = ChannelState::Paused;
        Ok(())
    }

    /// Resume a paused channel back to Active state.
    pub fn resume_channel(&mut self, channel_id: u64) -> CommResult<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;
        channel.state = ChannelState::Active;
        Ok(())
    }

    /// Set a channel to Draining state. Allows receive but blocks send.
    pub fn drain_channel(&mut self, channel_id: u64) -> CommResult<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;
        channel.state = ChannelState::Draining;
        Ok(())
    }

    /// Close a channel. Blocks all operations.
    pub fn close_channel(&mut self, channel_id: u64) -> CommResult<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;
        let name = channel.name.clone();
        channel.state = ChannelState::Closed;

        // --- Audit logging ---
        self.log_audit(
            AuditEventType::ChannelClosed,
            "system",
            &format!("Closed channel '{}' (id={})", name, channel_id),
            Some(channel_id.to_string()),
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Message engine
    // -----------------------------------------------------------------------

    /// Send a message to a channel.
    ///
    /// Enforces rate limiting, consent gates, and channel state before
    /// delivering. If the channel is Paused, Draining, or Closed, the
    /// message is automatically dead-lettered and an error is returned.
    pub fn send_message(
        &mut self,
        channel_id: u64,
        sender: &str,
        content: &str,
        msg_type: MessageType,
    ) -> CommResult<Message> {
        Self::validate_sender(sender)?;
        Self::validate_content(content)?;

        // --- Rate limiting (before any other processing) ---
        self.check_rate_limit(sender)?;

        // --- Consent enforcement ---
        self.check_send_consent(channel_id, sender, content)?;

        // Check channel existence
        if !self.channels.contains_key(&channel_id) {
            // Dead-letter the message for channel not found
            let id = self.next_message_id;
            self.next_message_id += 1;
            let msg = Message {
                id,
                channel_id,
                sender: sender.to_string(),
                recipient: None,
                content: content.to_string(),
                message_type: msg_type,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
                signature: Some(self.compute_signature(content)),
                acknowledged_by: Vec::new(),
                status: MessageStatus::DeadLettered,
                priority: MessagePriority::default(),
                reply_to: None,
                correlation_id: None,
                thread_id: None,
                comm_timestamp: CommTimestamp::default(),
                rich_content_json: None,
                comm_id: None,
            };
            self.dead_letters.push(DeadLetter {
                original_message: msg,
                reason: DeadLetterReason::ChannelNotFound,
                dead_lettered_at: Utc::now(),
                retry_count: 0,
            });
            return Err(CommError::ChannelNotFound(channel_id));
        }

        // --- Trust enforcement ---
        self.check_trust_for_channel(sender, channel_id)?;

        // Check channel state — dead-letter on violation
        if let Err(e) = self.check_channel_allows_send(channel_id) {
            let id = self.next_message_id;
            self.next_message_id += 1;
            let channel_state = self.channels.get(&channel_id).unwrap().state;
            let reason = match channel_state {
                ChannelState::Closed => DeadLetterReason::ChannelClosed,
                _ => DeadLetterReason::ValidationFailed(format!(
                    "Channel is {}",
                    channel_state
                )),
            };
            let msg = Message {
                id,
                channel_id,
                sender: sender.to_string(),
                recipient: None,
                content: content.to_string(),
                message_type: msg_type,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
                signature: Some(self.compute_signature(content)),
                acknowledged_by: Vec::new(),
                status: MessageStatus::DeadLettered,
                priority: MessagePriority::default(),
                reply_to: None,
                correlation_id: None,
                thread_id: None,
                comm_timestamp: CommTimestamp::default(),
                rich_content_json: None,
                comm_id: None,
            };
            self.dead_letters.push(DeadLetter {
                original_message: msg,
                reason,
                dead_lettered_at: Utc::now(),
                retry_count: 0,
            });
            return Err(e);
        }

        let id = self.next_message_id;
        self.next_message_id += 1;

        // Increment Lamport counter for causal ordering
        self.lamport_counter += 1;
        let mut ts = CommTimestamp::now(sender);
        ts.lamport = self.lamport_counter;
        ts.vector_clock.insert(sender.to_string(), self.lamport_counter);

        let message = Message {
            id,
            channel_id,
            sender: sender.to_string(),
            recipient: None,
            content: content.to_string(),
            message_type: msg_type,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            signature: Some(self.compute_signature(content)),
            acknowledged_by: Vec::new(),
            status: MessageStatus::Sent,
            priority: MessagePriority::default(),
            reply_to: None,
            correlation_id: None,
            thread_id: None,
            comm_timestamp: ts,
            rich_content_json: None,
            comm_id: None,
        };

        self.messages.insert(id, message.clone());

        // --- Audit logging ---
        self.log_audit(
            AuditEventType::MessageSent,
            sender,
            &format!("Sent {} message to channel {}", msg_type, channel_id),
            Some(id.to_string()),
        );

        // Bridge point: memory_bridge.log_conversation() for temporal chaining

        Ok(message)
    }

    /// Send a message with a specific priority.
    pub fn send_message_with_priority(
        &mut self,
        channel_id: u64,
        sender: &str,
        content: &str,
        msg_type: MessageType,
        priority: MessagePriority,
    ) -> CommResult<Message> {
        let mut msg = self.send_message(channel_id, sender, content, msg_type)?;
        // Update priority on the stored message
        if let Some(stored) = self.messages.get_mut(&msg.id) {
            stored.priority = priority;
            msg.priority = priority;
        }
        Ok(msg)
    }

    /// Receive messages from a channel, optionally filtered by recipient and time.
    ///
    /// Verifies message signatures on retrieval and logs a warning audit
    /// event if any signature does not match. Mismatched messages are still
    /// returned (reads are never blocked).
    pub fn receive_messages(
        &mut self,
        channel_id: u64,
        recipient: Option<&str>,
        since: Option<DateTime<Utc>>,
    ) -> CommResult<Vec<Message>> {
        if !self.channels.contains_key(&channel_id) {
            return Err(CommError::ChannelNotFound(channel_id));
        }

        // Draining channels allow receive; only Paused and Closed block it
        self.check_channel_allows_receive(channel_id)?;

        let mut msgs: Vec<Message> = self
            .messages
            .values()
            .filter(|m| {
                if m.channel_id != channel_id {
                    return false;
                }
                if let Some(ref recip) = recipient {
                    if let Some(ref msg_recip) = m.recipient {
                        if msg_recip != recip {
                            return false;
                        }
                    }
                }
                if let Some(ref s) = since {
                    if m.timestamp < *s {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        msgs.sort_by_key(|m| m.timestamp);

        // --- Signature verification on retrieval ---
        let msg_ids: Vec<u64> = msgs.iter().map(|m| m.id).collect();
        for msg_id in msg_ids {
            self.verify_message_signature(msg_id);
        }

        // --- Vector clock merge on receive ---
        // Merge each message's vector clock into the store's lamport counter
        // so that subsequent sends reflect causal awareness of received messages.
        for m in &msgs {
            if m.comm_timestamp.lamport > self.lamport_counter {
                self.lamport_counter = m.comm_timestamp.lamport;
            }
        }

        Ok(msgs)
    }

    /// Acknowledge receipt of a message.
    pub fn acknowledge_message(&mut self, message_id: u64, recipient: &str) -> CommResult<()> {
        Self::validate_sender(recipient)?;

        let message = self
            .messages
            .get_mut(&message_id)
            .ok_or(CommError::MessageNotFound(message_id))?;

        if !message.acknowledged_by.contains(&recipient.to_string()) {
            message.acknowledged_by.push(recipient.to_string());
        }
        message.status = MessageStatus::Acknowledged;
        Ok(())
    }

    /// Broadcast a message to all participants in a broadcast channel.
    pub fn broadcast(
        &mut self,
        channel_id: u64,
        sender: &str,
        content: &str,
    ) -> CommResult<Vec<Message>> {
        Self::validate_sender(sender)?;
        Self::validate_content(content)?;

        // --- Trust enforcement ---
        self.check_trust_for_channel(sender, channel_id)?;

        let channel = self
            .channels
            .get(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?
            .clone();

        let mut delivered = Vec::new();

        for participant in &channel.participants {
            if participant == sender {
                continue;
            }

            let id = self.next_message_id;
            self.next_message_id += 1;

            // Increment Lamport counter for each broadcast copy
            self.lamport_counter += 1;
            let mut ts = CommTimestamp::now(sender);
            ts.lamport = self.lamport_counter;
            ts.vector_clock.insert(sender.to_string(), self.lamport_counter);

            let message = Message {
                id,
                channel_id,
                sender: sender.to_string(),
                recipient: Some(participant.clone()),
                content: content.to_string(),
                message_type: MessageType::Broadcast,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
                signature: Some(self.compute_signature(content)),
                acknowledged_by: Vec::new(),
                status: MessageStatus::Sent,
                priority: MessagePriority::default(),
                reply_to: None,
                correlation_id: None,
                thread_id: None,
                comm_timestamp: ts,
                rich_content_json: None,
                comm_id: None,
            };

            self.messages.insert(id, message.clone());
            delivered.push(message);
        }

        Ok(delivered)
    }

    // -----------------------------------------------------------------------
    // Message threading
    // -----------------------------------------------------------------------

    /// Send a reply linked to a parent message.
    pub fn send_reply(
        &mut self,
        channel_id: u64,
        message_id: u64,
        sender: &str,
        content: &str,
        msg_type: MessageType,
    ) -> CommResult<Message> {
        Self::validate_sender(sender)?;
        Self::validate_content(content)?;

        // Verify parent message exists
        let parent = self
            .messages
            .get(&message_id)
            .ok_or(CommError::MessageNotFound(message_id))?
            .clone();

        // Verify channel exists and allows sending
        self.check_channel_allows_send(channel_id)?;

        // Inherit thread_id from parent, or use parent's id as thread_id
        let thread_id = parent
            .thread_id
            .clone()
            .unwrap_or_else(|| format!("thread-{}", parent.id));

        let id = self.next_message_id;
        self.next_message_id += 1;

        // Increment Lamport counter for causal ordering
        self.lamport_counter += 1;
        let mut ts = CommTimestamp::now(sender);
        ts.lamport = self.lamport_counter;
        ts.vector_clock.insert(sender.to_string(), self.lamport_counter);

        let message = Message {
            id,
            channel_id,
            sender: sender.to_string(),
            recipient: None,
            content: content.to_string(),
            message_type: msg_type,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            signature: Some(self.compute_signature(content)),
            acknowledged_by: Vec::new(),
            status: MessageStatus::Sent,
            priority: MessagePriority::default(),
            reply_to: Some(message_id),
            correlation_id: None,
            thread_id: Some(thread_id),
            comm_timestamp: ts,
            rich_content_json: None,
            comm_id: None,
        };

        self.messages.insert(id, message.clone());

        // Also set the parent's thread_id if it wasn't set yet
        if let Some(parent_msg) = self.messages.get_mut(&message_id) {
            if parent_msg.thread_id.is_none() {
                parent_msg.thread_id = Some(format!("thread-{}", parent_msg.id));
            }
        }

        Ok(message)
    }

    /// Get all messages in a thread, ordered by timestamp.
    pub fn get_thread(&self, thread_id: &str) -> Vec<Message> {
        let mut msgs: Vec<Message> = self
            .messages
            .values()
            .filter(|m| m.thread_id.as_deref() == Some(thread_id))
            .cloned()
            .collect();
        msgs.sort_by_key(|m| m.timestamp);
        msgs
    }

    /// Get all direct replies to a specific message.
    pub fn get_replies(&self, message_id: u64) -> Vec<Message> {
        let mut replies: Vec<Message> = self
            .messages
            .values()
            .filter(|m| m.reply_to == Some(message_id))
            .cloned()
            .collect();
        replies.sort_by_key(|m| m.timestamp);
        replies
    }

    // -----------------------------------------------------------------------
    // Channel management
    // -----------------------------------------------------------------------

    /// Create a new communication channel.
    pub fn create_channel(
        &mut self,
        name: &str,
        channel_type: ChannelType,
        config: Option<ChannelConfig>,
    ) -> CommResult<Channel> {
        Self::validate_channel_name(name)?;

        let id = self.next_channel_id;
        self.next_channel_id += 1;

        let channel = Channel {
            id,
            name: name.to_string(),
            channel_type,
            created_at: Utc::now(),
            participants: Vec::new(),
            config: config.unwrap_or_default(),
            state: ChannelState::Active,
            comm_id: None,
        };

        self.channels.insert(id, channel.clone());

        // --- Audit logging ---
        self.log_audit(
            AuditEventType::ChannelCreated,
            "system",
            &format!("Created channel '{}' (type={}, id={})", name, channel_type, id),
            Some(id.to_string()),
        );

        Ok(channel)
    }

    /// Join a channel as a participant.
    pub fn join_channel(&mut self, channel_id: u64, participant: &str) -> CommResult<()> {
        Self::validate_sender(participant)?;

        // Bridge point: identity_bridge.resolve_identity() for participant verification

        // --- Trust enforcement ---
        self.check_trust_for_channel(participant, channel_id)?;

        // --- Consent enforcement ---
        if !self.check_consent_for_action(participant, &self.channels.get(&channel_id)
            .map(|c| c.name.clone()).unwrap_or_default(), ConsentScope::JoinChannels)
        {
            return Err(CommError::ConsentDenied {
                reason: "Consent not granted for joining channels".to_string(),
            });
        }

        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;

        if channel.participants.contains(&participant.to_string()) {
            return Err(CommError::AlreadyInChannel(
                participant.to_string(),
                channel_id,
            ));
        }

        if channel.config.max_participants > 0
            && channel.participants.len() >= channel.config.max_participants as usize
        {
            return Err(CommError::ChannelFull(channel_id));
        }

        channel.participants.push(participant.to_string());
        Ok(())
    }

    /// Leave a channel.
    pub fn leave_channel(&mut self, channel_id: u64, participant: &str) -> CommResult<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;

        let pos = channel
            .participants
            .iter()
            .position(|p| p == participant)
            .ok_or_else(|| CommError::NotInChannel(participant.to_string(), channel_id))?;

        channel.participants.remove(pos);
        Ok(())
    }

    /// List all channels.
    pub fn list_channels(&self) -> Vec<Channel> {
        let mut channels: Vec<Channel> = self.channels.values().cloned().collect();
        channels.sort_by_key(|c| c.id);
        channels
    }

    /// Get a specific channel by id.
    pub fn get_channel(&self, channel_id: u64) -> Option<Channel> {
        self.channels.get(&channel_id).cloned()
    }

    /// Update channel configuration.
    pub fn set_channel_config(
        &mut self,
        channel_id: u64,
        config: ChannelConfig,
    ) -> CommResult<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;
        channel.config = config;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Pub/Sub
    // -----------------------------------------------------------------------

    /// Subscribe to a topic.
    pub fn subscribe(&mut self, topic: &str, subscriber: &str) -> CommResult<Subscription> {
        Self::validate_sender(subscriber)?;
        Self::validate_channel_name(topic)?;

        let id = self.next_subscription_id;
        self.next_subscription_id += 1;

        let subscription = Subscription {
            id,
            topic: topic.to_string(),
            subscriber: subscriber.to_string(),
            created_at: Utc::now(),
        };

        self.subscriptions.insert(id, subscription.clone());
        Ok(subscription)
    }

    /// Remove a subscription.
    pub fn unsubscribe(&mut self, subscription_id: u64) -> CommResult<()> {
        if self.subscriptions.remove(&subscription_id).is_none() {
            return Err(CommError::SubscriptionNotFound(subscription_id));
        }
        Ok(())
    }

    /// Publish a message to all subscribers of a topic.
    pub fn publish(
        &mut self,
        topic: &str,
        sender: &str,
        content: &str,
    ) -> CommResult<Vec<Message>> {
        Self::validate_sender(sender)?;
        Self::validate_content(content)?;

        // Find or create the topic channel
        let channel_id = self
            .channels
            .values()
            .find(|c| c.name == topic && c.channel_type == ChannelType::PubSub)
            .map(|c| c.id);

        let channel_id = match channel_id {
            Some(id) => id,
            None => {
                let ch = self.create_channel(topic, ChannelType::PubSub, None)?;
                ch.id
            }
        };

        // --- Trust enforcement ---
        self.check_trust_for_channel(sender, channel_id)?;

        // Get all subscribers for this topic
        let subscribers: Vec<String> = self
            .subscriptions
            .values()
            .filter(|s| s.topic == topic && s.subscriber != sender)
            .map(|s| s.subscriber.clone())
            .collect();

        let mut delivered = Vec::new();

        for subscriber in subscribers {
            let id = self.next_message_id;
            self.next_message_id += 1;

            // Increment Lamport counter for each pub/sub delivery
            self.lamport_counter += 1;
            let mut ts = CommTimestamp::now(sender);
            ts.lamport = self.lamport_counter;
            ts.vector_clock.insert(sender.to_string(), self.lamport_counter);

            let message = Message {
                id,
                channel_id,
                sender: sender.to_string(),
                recipient: Some(subscriber),
                content: content.to_string(),
                message_type: MessageType::Notification,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
                signature: Some(self.compute_signature(content)),
                acknowledged_by: Vec::new(),
                status: MessageStatus::Sent,
                priority: MessagePriority::default(),
                reply_to: None,
                correlation_id: None,
                thread_id: None,
                comm_timestamp: ts,
                rich_content_json: None,
                comm_id: None,
            };

            self.messages.insert(id, message.clone());
            delivered.push(message);
        }

        Ok(delivered)
    }

    // -----------------------------------------------------------------------
    // Dead letter queue
    // -----------------------------------------------------------------------

    /// Return the number of dead letters in the queue.
    pub fn dead_letter_count(&self) -> usize {
        self.dead_letters.len()
    }

    /// List all dead letters, sorted by dead-lettered time (oldest first).
    pub fn list_dead_letters(&self) -> Vec<DeadLetter> {
        let mut dl = self.dead_letters.clone();
        dl.sort_by_key(|d| d.dead_lettered_at);
        dl
    }

    /// Attempt to replay (re-send) a dead letter by index.
    ///
    /// If the channel is now available and active, the message is re-sent
    /// and removed from the dead letter queue. Otherwise, the retry count
    /// is incremented and the dead letter remains.
    pub fn replay_dead_letter(&mut self, index: usize) -> CommResult<Message> {
        if index >= self.dead_letters.len() {
            return Err(CommError::DeadLetterNotFound(index));
        }

        let dl = self.dead_letters[index].clone();
        let orig = &dl.original_message;

        // Try to re-send
        match self.send_message(
            orig.channel_id,
            &orig.sender,
            &orig.content,
            orig.message_type,
        ) {
            Ok(msg) => {
                // Remove from dead letter queue on success
                self.dead_letters.remove(index);
                Ok(msg)
            }
            Err(e) => {
                // Increment retry count on the existing dead letter (the send_message
                // already created a new dead letter entry, so remove that duplicate
                // and just update the original)
                let new_len = self.dead_letters.len();
                // The failed send_message may have added a new dead letter at the end
                if new_len > dl.retry_count as usize + self.dead_letters.len() {
                    // Remove the duplicate that send_message just added
                    self.dead_letters.pop();
                }
                // The original dead letter is still at `index` (or shifted if something
                // was removed before it). Increment its retry count.
                if index < self.dead_letters.len() {
                    self.dead_letters[index].retry_count += 1;
                }
                Err(e)
            }
        }
    }

    /// Clear all dead letters from the queue.
    pub fn clear_dead_letters(&mut self) {
        self.dead_letters.clear();
    }

    // -----------------------------------------------------------------------
    // TTL enforcement
    // -----------------------------------------------------------------------

    /// Expire messages that have exceeded their channel's TTL.
    ///
    /// Scans all messages. If the channel has `ttl_seconds > 0` and the
    /// message is older than the TTL, the message is moved to the dead
    /// letter queue with reason `Expired`.
    ///
    /// Returns the count of expired messages.
    pub fn expire_messages(&mut self) -> usize {
        let now = Utc::now();
        let mut expired_ids: Vec<u64> = Vec::new();

        for msg in self.messages.values() {
            if let Some(channel) = self.channels.get(&msg.channel_id) {
                if channel.config.ttl_seconds > 0 {
                    let age = now
                        .signed_duration_since(msg.timestamp)
                        .num_seconds();
                    if age > channel.config.ttl_seconds as i64 {
                        expired_ids.push(msg.id);
                    }
                }
            }
        }

        let count = expired_ids.len();

        for id in expired_ids {
            if let Some(mut msg) = self.messages.remove(&id) {
                msg.status = MessageStatus::Expired;
                self.dead_letters.push(DeadLetter {
                    original_message: msg,
                    reason: DeadLetterReason::Expired,
                    dead_lettered_at: now,
                    retry_count: 0,
                });
            }
        }

        count
    }

    // -----------------------------------------------------------------------
    // Compact
    // -----------------------------------------------------------------------

    /// Compact the store by removing messages from closed channels and
    /// enforcing retention policies.
    ///
    /// Returns the count of removed messages.
    pub fn compact(&mut self) -> usize {
        let mut removed = 0usize;

        // 1. Remove messages from closed channels
        let closed_channel_ids: Vec<u64> = self
            .channels
            .values()
            .filter(|c| c.state == ChannelState::Closed)
            .map(|c| c.id)
            .collect();

        let ids_to_remove: Vec<u64> = self
            .messages
            .values()
            .filter(|m| closed_channel_ids.contains(&m.channel_id))
            .map(|m| m.id)
            .collect();

        for id in ids_to_remove {
            self.messages.remove(&id);
            removed += 1;
        }

        // 2. Enforce RetentionPolicy::MessageCount per channel
        for channel in self.channels.values() {
            if let RetentionPolicy::MessageCount(max_count) = channel.config.retention_policy {
                let mut channel_msgs: Vec<(u64, DateTime<Utc>)> = self
                    .messages
                    .values()
                    .filter(|m| m.channel_id == channel.id)
                    .map(|m| (m.id, m.timestamp))
                    .collect();

                if channel_msgs.len() > max_count as usize {
                    // Sort by timestamp ascending (oldest first)
                    channel_msgs.sort_by_key(|&(_, ts)| ts);
                    let to_remove = channel_msgs.len() - max_count as usize;
                    for (id, _) in channel_msgs.into_iter().take(to_remove) {
                        self.messages.remove(&id);
                        removed += 1;
                    }
                }
            }
        }

        removed
    }

    // -----------------------------------------------------------------------
    // Query engine
    // -----------------------------------------------------------------------

    /// Query message history with filters.
    pub fn query_history(&self, channel_id: u64, filter: &MessageFilter) -> Vec<Message> {
        let mut results: Vec<Message> = self
            .messages
            .values()
            .filter(|m| {
                if m.channel_id != channel_id {
                    return false;
                }
                if let Some(ref since) = filter.since {
                    if m.timestamp < *since {
                        return false;
                    }
                }
                if let Some(ref before) = filter.before {
                    if m.timestamp > *before {
                        return false;
                    }
                }
                if let Some(ref sender) = filter.sender {
                    if m.sender != *sender {
                        return false;
                    }
                }
                if let Some(ref msg_type) = filter.message_type {
                    if m.message_type != *msg_type {
                        return false;
                    }
                }
                if let Some(priority_val) = filter.priority {
                    let msg_priority = m.priority as u32;
                    if msg_priority != priority_val {
                        return false;
                    }
                }
                if let Some(filter_thread) = filter.thread_id {
                    match &m.thread_id {
                        Some(tid) => {
                            if let Ok(parsed) = tid.parse::<u64>() {
                                if parsed != filter_thread {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                if let Some(ref substr) = filter.content_contains {
                    if !m.content.to_lowercase().contains(&substr.to_lowercase()) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        results.sort_by_key(|m| m.timestamp);

        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        results
    }

    /// Full-text search across all messages.
    pub fn search_messages(&self, query_text: &str, max_results: usize) -> Vec<Message> {
        let query_lower = query_text.to_lowercase();
        let mut results: Vec<Message> = self
            .messages
            .values()
            .filter(|m| m.content.to_lowercase().contains(&query_lower))
            .cloned()
            .collect();

        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results.truncate(max_results);
        results
    }

    /// Get a specific message by id.
    pub fn get_message(&self, message_id: u64) -> Option<Message> {
        self.messages.get(&message_id).cloned()
    }

    // -----------------------------------------------------------------------
    // Persistence (.acomm file format)
    // -----------------------------------------------------------------------

    /// Save the store to a `.acomm` file (bincode + gzip + binary header).
    ///
    /// Acquires an exclusive [`CommFileLock`] for the duration of the write so
    /// that concurrent readers/writers on the same path do not corrupt data.
    ///
    /// The on-disk format is: `[ACOM header (20 bytes)] [gzip(bincode(store))]`.
    /// The ACOM header includes a CRC32 checksum of the compressed payload so
    /// that corruption can be detected on load.
    pub fn save(&self, path: &Path) -> CommResult<()> {
        // Recover stale locks (older than 60 s) before attempting to acquire.
        CommFileLock::recover_stale(path, 60)?;

        let _lock = CommFileLock::acquire(path)?;

        let store_bytes =
            bincode::serialize(self).map_err(|e| CommError::Serialization(e.to_string()))?;

        // Gzip-compress the bincode payload into an in-memory buffer.
        let mut gz_buf = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut gz_buf, Compression::default());
            encoder.write_all(&store_bytes)?;
            encoder.finish()?;
        }

        // Wrap the compressed data with the binary format header (magic + Blake3).
        let output = format::write_with_header(&gz_buf);

        let mut file = std::fs::File::create(path)?;
        file.write_all(&output)?;

        // Lock released via Drop of `_lock`.
        Ok(())
    }

    /// Load a store from a `.acomm` file.
    ///
    /// Acquires a shared [`CommFileLock`] for the duration of the read so that
    /// concurrent writers are held off while the data is being read.
    ///
    /// Supports both the new binary format (ACOM header + CRC32) and the
    /// legacy format (raw gzip with embedded ACOMM001 header) for backward
    /// compatibility.
    pub fn load(path: &Path) -> CommResult<Self> {
        // Recover stale locks (older than 60 s) before attempting to acquire.
        CommFileLock::recover_stale(path, 60)?;

        let _lock = CommFileLock::acquire_shared(path)?;

        let mut raw = Vec::new();
        {
            let mut file = std::fs::File::open(path)?;
            file.read_to_end(&mut raw)?;
        }

        if format::is_new_format(&raw) {
            // ---- New binary format (v2/v3) ----
            let gz_data = format::read_with_header(&raw)
                .map_err(|e| CommError::InvalidFile(e))?;

            let mut decoder = GzDecoder::new(&gz_data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;

            let store: CommStore = bincode::deserialize(&decompressed)
                .map_err(|e| CommError::InvalidFile(format!("Bad store data: {e}")))?;
            Ok(store)
        } else {
            // ---- Legacy format (v1): raw gzip with ACOMM001 header ----
            let mut decoder = GzDecoder::new(&raw[..]);
            let mut data = Vec::new();
            decoder.read_to_end(&mut data)?;

            // Deserialize legacy header first
            let header: AcommHeader = bincode::deserialize(&data)
                .map_err(|e| CommError::InvalidFile(format!("Bad header: {e}")))?;

            if header.magic != *ACOMM_MAGIC {
                return Err(CommError::InvalidFile(
                    "Invalid magic bytes — not an .acomm file".to_string(),
                ));
            }

            if header.version != ACOMM_VERSION {
                return Err(CommError::InvalidFile(format!(
                    "Unsupported version: {} (expected {})",
                    header.version, ACOMM_VERSION
                )));
            }

            // Skip the header bytes to get to the store payload
            let header_size = bincode::serialized_size(&header)
                .map_err(|e| CommError::Serialization(e.to_string()))? as usize;

            let store: CommStore = bincode::deserialize(&data[header_size..])
                .map_err(|e| CommError::InvalidFile(format!("Bad store data: {e}")))?;

            // Lock released via Drop of `_lock`.
            Ok(store)
        }
    }

    /// Get summary statistics for the store.
    pub fn stats(&self) -> CommStoreStats {
        // Count messages by type
        let mut messages_by_type: HashMap<String, usize> = HashMap::new();
        let mut messages_by_priority: HashMap<String, usize> = HashMap::new();
        let mut oldest_message: Option<DateTime<Utc>> = None;
        let mut newest_message: Option<DateTime<Utc>> = None;

        for msg in self.messages.values() {
            *messages_by_type
                .entry(msg.message_type.to_string())
                .or_insert(0) += 1;
            *messages_by_priority
                .entry(msg.priority.to_string())
                .or_insert(0) += 1;

            match oldest_message {
                None => oldest_message = Some(msg.timestamp),
                Some(ref ts) if msg.timestamp < *ts => oldest_message = Some(msg.timestamp),
                _ => {}
            }
            match newest_message {
                None => newest_message = Some(msg.timestamp),
                Some(ref ts) if msg.timestamp > *ts => newest_message = Some(msg.timestamp),
                _ => {}
            }
        }

        // Count channels by state
        let mut channels_by_state: HashMap<String, usize> = HashMap::new();
        for ch in self.channels.values() {
            *channels_by_state
                .entry(ch.state.to_string())
                .or_insert(0) += 1;
        }

        CommStoreStats {
            channel_count: self.channels.len(),
            message_count: self.messages.len(),
            subscription_count: self.subscriptions.len(),
            total_participants: self
                .channels
                .values()
                .map(|c| c.participants.len())
                .sum(),
            dead_letter_count: self.dead_letters.len(),
            messages_by_type,
            messages_by_priority,
            channels_by_state,
            oldest_message,
            newest_message,
            consent_gate_count: self.consent_gates.len(),
            trust_override_count: self.trust_levels.len(),
            temporal_queue_count: self.temporal_queue.iter().filter(|m| !m.delivered).count(),
            hive_count: self.hive_minds.len(),
            comm_log_count: self.comm_log.len(),
            federation_enabled: self.federation_config.enabled,
            federated_zone_count: self.federation_config.zones.len(),
            audit_log_count: self.audit_log.len(),
        }
    }

    // -----------------------------------------------------------------------
    // Consent management
    // -----------------------------------------------------------------------

    /// Grant consent from grantor to grantee for a specific scope.
    pub fn grant_consent(
        &mut self,
        grantor: &str,
        grantee: &str,
        scope: ConsentScope,
        reason: Option<String>,
        expires_at: Option<String>,
    ) -> CommResult<&ConsentGateEntry> {
        if grantor.is_empty() || grantee.is_empty() {
            return Err(CommError::ConsentError(
                "Grantor and grantee must be non-empty".to_string(),
            ));
        }
        let scope_str = scope.to_string();
        let now = Utc::now().to_rfc3339();
        // Check if an existing entry exists for this triple
        if let Some(entry) = self.consent_gates.iter_mut().find(|e| {
            e.grantor == grantor && e.grantee == grantee && e.scope == scope
        }) {
            entry.status = ConsentStatus::Granted;
            entry.updated_at = now;
            entry.reason = reason;
            entry.expires_at = expires_at;
            // Audit log
            self.audit_log.push(AuditEntry {
                event_type: AuditEventType::ConsentGranted,
                timestamp: Utc::now().to_rfc3339(),
                agent_id: grantor.to_string(),
                description: format!("Granted {} consent to '{}'", scope_str, grantee),
                related_id: Some(format!("{}->{}", grantor, grantee)),
            });
            let idx = self.consent_gates.iter().position(|e| {
                e.grantor == grantor && e.grantee == grantee && e.scope == scope
            }).unwrap();
            return Ok(&self.consent_gates[idx]);
        }
        // Create new entry
        let entry = ConsentGateEntry {
            grantor: grantor.to_string(),
            grantee: grantee.to_string(),
            scope,
            status: ConsentStatus::Granted,
            created_at: now.clone(),
            updated_at: now,
            expires_at,
            reason,
        };
        self.consent_gates.push(entry);
        // Audit log
        self.audit_log.push(AuditEntry {
            event_type: AuditEventType::ConsentGranted,
            timestamp: Utc::now().to_rfc3339(),
            agent_id: grantor.to_string(),
            description: format!("Granted {} consent to '{}'", scope_str, grantee),
            related_id: Some(format!("{}->{}", grantor, grantee)),
        });
        Ok(self.consent_gates.last().unwrap())
    }

    /// Revoke consent.
    pub fn revoke_consent(
        &mut self,
        grantor: &str,
        grantee: &str,
        scope: &ConsentScope,
    ) -> CommResult<()> {
        if let Some(entry) = self.consent_gates.iter_mut().find(|e| {
            e.grantor == grantor && e.grantee == grantee && e.scope == *scope
        }) {
            entry.status = ConsentStatus::Revoked;
            entry.updated_at = Utc::now().to_rfc3339();
            // Audit log
            self.log_audit(
                AuditEventType::ConsentRevoked,
                grantor,
                &format!("Revoked {} consent from '{}'", scope, grantee),
                Some(format!("{}->{}", grantor, grantee)),
            );
            Ok(())
        } else {
            Err(CommError::ConsentError(format!(
                "No consent entry found for {grantor} -> {grantee} ({scope})"
            )))
        }
    }

    /// Check if consent is granted.
    pub fn check_consent(
        &self,
        grantor: &str,
        grantee: &str,
        scope: &ConsentScope,
    ) -> bool {
        self.consent_gates.iter().any(|e| {
            e.grantor == grantor
                && e.grantee == grantee
                && e.scope == *scope
                && e.status == ConsentStatus::Granted
        })
    }

    /// List all consent gates, optionally filtered by agent.
    pub fn list_consent_gates(&self, agent: Option<&str>) -> Vec<&ConsentGateEntry> {
        self.consent_gates
            .iter()
            .filter(|e| {
                agent.map_or(true, |a| e.grantor == a || e.grantee == a)
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Trust management
    // -----------------------------------------------------------------------

    /// Set trust level for an agent.
    pub fn set_trust_level(
        &mut self,
        agent_id: &str,
        level: CommTrustLevel,
    ) -> CommResult<()> {
        if agent_id.is_empty() {
            return Err(CommError::TrustError(
                "Agent ID must be non-empty".to_string(),
            ));
        }
        self.trust_levels.insert(agent_id.to_string(), level);

        // --- Audit logging ---
        self.log_audit(
            AuditEventType::TrustUpdated,
            agent_id,
            &format!("Trust level set to {} for '{}'", level, agent_id),
            Some(agent_id.to_string()),
        );

        Ok(())
    }

    /// Get trust level for an agent (default: Standard).
    pub fn get_trust_level(&self, agent_id: &str) -> CommTrustLevel {
        self.trust_levels
            .get(agent_id)
            .copied()
            .unwrap_or(CommTrustLevel::Standard)
    }

    /// List all trust level overrides.
    pub fn list_trust_levels(&self) -> &HashMap<String, CommTrustLevel> {
        &self.trust_levels
    }

    // -----------------------------------------------------------------------
    // Temporal scheduling
    // -----------------------------------------------------------------------

    /// Schedule a message for future delivery.
    pub fn schedule_message(
        &mut self,
        channel_id: u64,
        sender: &str,
        content: &str,
        target: TemporalTarget,
        affect: Option<AffectState>,
    ) -> CommResult<&TemporalMessage> {
        // Validate channel exists
        if !self.channels.contains_key(&channel_id) {
            return Err(CommError::ChannelNotFound(channel_id));
        }
        Self::validate_sender(sender)?;
        Self::validate_content(content)?;

        // --- Consent enforcement ---
        if !self.check_consent_for_action(sender, "temporal", ConsentScope::ScheduleMessages) {
            return Err(CommError::ConsentDenied {
                reason: "Consent not granted for scheduling messages".to_string(),
            });
        }

        let id = self.next_temporal_id;
        self.next_temporal_id += 1;

        let msg = TemporalMessage {
            id,
            channel_id,
            sender: sender.to_string(),
            content: content.to_string(),
            target,
            scheduled_at: Utc::now().to_rfc3339(),
            delivered: false,
            affect,
        };
        self.temporal_queue.push(msg);

        // Bridge point: time_bridge.schedule_at() for precise timing

        // --- Audit logging ---
        self.audit_log.push(AuditEntry {
            event_type: AuditEventType::ScheduledMessage,
            timestamp: Utc::now().to_rfc3339(),
            agent_id: sender.to_string(),
            description: format!(
                "Scheduled message to channel {} (temporal_id={})",
                channel_id, id
            ),
            related_id: Some(id.to_string()),
        });

        Ok(self.temporal_queue.last().unwrap())
    }

    /// List all scheduled (undelivered) temporal messages.
    pub fn list_scheduled(&self) -> Vec<&TemporalMessage> {
        self.temporal_queue
            .iter()
            .filter(|m| !m.delivered)
            .collect()
    }

    /// Cancel a scheduled message.
    pub fn cancel_scheduled(&mut self, temporal_id: u64) -> CommResult<()> {
        if let Some(msg) = self.temporal_queue.iter_mut().find(|m| m.id == temporal_id) {
            if msg.delivered {
                return Err(CommError::TemporalError(
                    "Cannot cancel already-delivered message".to_string(),
                ));
            }
            self.temporal_queue.retain(|m| m.id != temporal_id);
            Ok(())
        } else {
            Err(CommError::TemporalError(format!(
                "Scheduled message {temporal_id} not found"
            )))
        }
    }

    /// Deliver all pending temporal messages that are due (Immediate targets).
    /// Returns the number of messages delivered.
    pub fn deliver_pending_temporal(&mut self) -> usize {
        let mut delivered = 0;
        let mut to_deliver = Vec::new();

        for msg in &self.temporal_queue {
            if msg.delivered {
                continue;
            }
            match &msg.target {
                TemporalTarget::Immediate => {
                    to_deliver.push((msg.id, msg.channel_id, msg.sender.clone(), msg.content.clone()));
                }
                _ => {} // Other targets need time/condition checking
            }
        }

        for (temporal_id, channel_id, sender, content) in to_deliver {
            if self.send_message(channel_id, &sender, &content, MessageType::Text).is_ok() {
                if let Some(msg) = self.temporal_queue.iter_mut().find(|m| m.id == temporal_id) {
                    msg.delivered = true;
                }
                delivered += 1;
            }
        }
        delivered
    }

    // -----------------------------------------------------------------------
    // Affect messaging
    // -----------------------------------------------------------------------

    /// Send a message with affect/emotional context.
    pub fn send_affect_message(
        &mut self,
        channel_id: u64,
        sender: &str,
        content: &str,
        affect: AffectState,
    ) -> CommResult<Message> {
        // Validate
        Self::validate_sender(sender)?;
        Self::validate_content(content)?;
        self.check_channel_allows_send(channel_id)?;

        // Embed affect as JSON prefix in content for storage
        let affect_json = serde_json::to_string(&affect)
            .map_err(|e| CommError::Serialization(e.to_string()))?;
        let enriched = format!("[affect:{}]{}", affect_json, content);

        self.send_message(channel_id, sender, &enriched, MessageType::Text)
    }

    // -----------------------------------------------------------------------
    // Federation management
    // -----------------------------------------------------------------------

    /// Configure federation settings.
    pub fn configure_federation(
        &mut self,
        enabled: bool,
        local_zone: &str,
        default_policy: FederationPolicy,
    ) -> CommResult<()> {
        if local_zone.is_empty() {
            return Err(CommError::FederationError(
                "Local zone must be non-empty".to_string(),
            ));
        }

        // --- Consent enforcement ---
        // If any Federate consent gates exist, the configuring agent (system)
        // must have an explicit grant.
        if !self.check_consent_for_action("system", local_zone, ConsentScope::Federate) {
            return Err(CommError::ConsentDenied {
                reason: "Consent not granted for federation".to_string(),
            });
        }

        self.federation_config.enabled = enabled;
        self.federation_config.local_zone = local_zone.to_string();
        self.federation_config.default_policy = default_policy;

        // --- Audit logging ---
        self.log_audit(
            AuditEventType::FederationConfigured,
            "system",
            &format!(
                "Federation configured: enabled={}, zone='{}', policy={}",
                enabled, local_zone, default_policy
            ),
            Some(local_zone.to_string()),
        );

        Ok(())
    }

    /// Get current federation configuration.
    pub fn get_federation_config(&self) -> &FederationConfig {
        &self.federation_config
    }

    /// Add a federated zone.
    pub fn add_federated_zone(&mut self, zone: FederatedZone) -> CommResult<()> {
        if zone.zone_id.is_empty() {
            return Err(CommError::FederationError(
                "Zone ID must be non-empty".to_string(),
            ));
        }
        // Check for duplicates
        if self.federation_config.zones.iter().any(|z| z.zone_id == zone.zone_id) {
            return Err(CommError::FederationError(format!(
                "Zone '{}' already exists", zone.zone_id
            )));
        }
        self.federation_config.zones.push(zone);
        Ok(())
    }

    /// Remove a federated zone.
    pub fn remove_federated_zone(&mut self, zone_id: &str) -> CommResult<()> {
        let before = self.federation_config.zones.len();
        self.federation_config.zones.retain(|z| z.zone_id != zone_id);
        if self.federation_config.zones.len() == before {
            return Err(CommError::FederationError(format!(
                "Zone '{zone_id}' not found"
            )));
        }
        Ok(())
    }

    /// List all federated zones.
    pub fn list_federated_zones(&self) -> &[FederatedZone] {
        &self.federation_config.zones
    }

    // -----------------------------------------------------------------------
    // Hive mind management
    // -----------------------------------------------------------------------

    /// Form a new hive mind.
    pub fn form_hive(
        &mut self,
        name: &str,
        coordinator: &str,
        decision_mode: CollectiveDecisionMode,
    ) -> CommResult<&HiveMind> {
        if name.is_empty() {
            return Err(CommError::HiveError(
                "Hive name must be non-empty".to_string(),
            ));
        }
        if coordinator.is_empty() {
            return Err(CommError::HiveError(
                "Coordinator must be non-empty".to_string(),
            ));
        }

        // --- Consent enforcement ---
        if !self.check_consent_for_action(coordinator, name, ConsentScope::HiveParticipation) {
            return Err(CommError::ConsentDenied {
                reason: "Consent not granted for hive participation".to_string(),
            });
        }

        let id = self.next_hive_id;
        self.next_hive_id += 1;
        let now = Utc::now().to_rfc3339();
        let hive = HiveMind {
            id,
            name: name.to_string(),
            constituents: vec![HiveConstituent {
                agent_id: coordinator.to_string(),
                role: HiveRole::Coordinator,
                joined_at: now.clone(),
            }],
            decision_mode,
            formed_at: now,
            metadata: HashMap::new(),
            coherence_level: 1.0,
            separation_policy: "graceful".to_string(),
        };
        self.hive_minds.insert(id, hive);

        // Bridge point: contract_bridge.validate_channel_contract() for SLA enforcement

        // --- Audit logging ---
        self.audit_log.push(AuditEntry {
            event_type: AuditEventType::HiveFormed,
            timestamp: Utc::now().to_rfc3339(),
            agent_id: coordinator.to_string(),
            description: format!("Formed hive '{}' (id={})", name, id),
            related_id: Some(id.to_string()),
        });

        Ok(self.hive_minds.get(&id).unwrap())
    }

    /// Dissolve a hive mind.
    pub fn dissolve_hive(&mut self, hive_id: u64) -> CommResult<()> {
        if self.hive_minds.remove(&hive_id).is_none() {
            return Err(CommError::HiveError(format!(
                "Hive {hive_id} not found"
            )));
        }

        // --- Audit logging ---
        self.log_audit(
            AuditEventType::HiveDissolved,
            "system",
            &format!("Dissolved hive (id={})", hive_id),
            Some(hive_id.to_string()),
        );

        Ok(())
    }

    /// Join a hive mind.
    pub fn join_hive(
        &mut self,
        hive_id: u64,
        agent_id: &str,
        role: HiveRole,
    ) -> CommResult<()> {
        // --- Consent enforcement (checked before mutable borrow) ---
        {
            let hive_name = self
                .hive_minds
                .get(&hive_id)
                .map(|h| h.name.clone())
                .unwrap_or_default();
            if !self.check_consent_for_action(agent_id, &hive_name, ConsentScope::HiveParticipation) {
                return Err(CommError::ConsentDenied {
                    reason: "Consent not granted for hive participation".to_string(),
                });
            }
        }

        let hive = self
            .hive_minds
            .get_mut(&hive_id)
            .ok_or_else(|| CommError::HiveError(format!("Hive {hive_id} not found")))?;

        if hive.constituents.iter().any(|c| c.agent_id == agent_id) {
            return Err(CommError::HiveError(format!(
                "Agent '{agent_id}' is already in hive {hive_id}"
            )));
        }

        hive.constituents.push(HiveConstituent {
            agent_id: agent_id.to_string(),
            role,
            joined_at: Utc::now().to_rfc3339(),
        });
        Ok(())
    }

    /// Leave a hive mind.
    pub fn leave_hive(&mut self, hive_id: u64, agent_id: &str) -> CommResult<()> {
        let hive = self
            .hive_minds
            .get_mut(&hive_id)
            .ok_or_else(|| CommError::HiveError(format!("Hive {hive_id} not found")))?;

        let before = hive.constituents.len();
        hive.constituents.retain(|c| c.agent_id != agent_id);
        if hive.constituents.len() == before {
            return Err(CommError::HiveError(format!(
                "Agent '{agent_id}' is not in hive {hive_id}"
            )));
        }
        Ok(())
    }

    /// List all hive minds.
    pub fn list_hives(&self) -> Vec<&HiveMind> {
        self.hive_minds.values().collect()
    }

    /// Get a specific hive mind.
    pub fn get_hive(&self, hive_id: u64) -> Option<&HiveMind> {
        self.hive_minds.get(&hive_id)
    }

    // -----------------------------------------------------------------------
    // Communication log (mirrors memory's conversation_log)
    // -----------------------------------------------------------------------

    /// Log a communication context entry.
    pub fn log_communication(
        &mut self,
        content: &str,
        role: &str,
        topic: Option<String>,
        linked_message_id: Option<u64>,
        affect: Option<AffectState>,
    ) -> &CommunicationLogEntry {
        let idx = self.next_log_index;
        self.next_log_index += 1;
        let entry = CommunicationLogEntry {
            index: idx,
            content: content.to_string(),
            role: role.to_string(),
            topic,
            timestamp: Utc::now().to_rfc3339(),
            linked_message_id,
            affect,
        };
        self.comm_log.push(entry);
        self.comm_log.last().unwrap()
    }

    /// Get communication log entries.
    pub fn get_comm_log(&self, limit: Option<usize>) -> &[CommunicationLogEntry] {
        match limit {
            Some(n) if n < self.comm_log.len() => &self.comm_log[self.comm_log.len() - n..],
            _ => &self.comm_log,
        }
    }

    // -----------------------------------------------------------------------
    // Audit log
    // -----------------------------------------------------------------------

    /// Log an audit event.
    pub fn log_audit(
        &mut self,
        event_type: AuditEventType,
        agent_id: &str,
        description: &str,
        related_id: Option<String>,
    ) {
        let entry = AuditEntry {
            event_type,
            timestamp: Utc::now().to_rfc3339(),
            agent_id: agent_id.to_string(),
            description: description.to_string(),
            related_id,
        };
        self.audit_log.push(entry);
    }

    /// Get recent audit log entries.
    pub fn get_audit_log(&self, limit: Option<usize>) -> Vec<&AuditEntry> {
        match limit {
            Some(n) if n < self.audit_log.len() => {
                self.audit_log[self.audit_log.len() - n..].iter().collect()
            }
            _ => self.audit_log.iter().collect(),
        }
    }

    // -----------------------------------------------------------------------
    // Semantic operations
    // -----------------------------------------------------------------------

    /// Send a semantic message (structured meaning payload).
    pub fn send_semantic(
        &mut self,
        channel_id: u64,
        sender: &str,
        topic: &str,
        focus_nodes: Vec<String>,
        depth: u64,
    ) -> CommResult<SemanticOperation> {
        // Verify channel exists
        if !self.channels.contains_key(&channel_id) {
            return Err(CommError::ChannelNotFound(channel_id));
        }
        let id = self.next_semantic_id;
        self.next_semantic_id += 1;
        let op = SemanticOperation {
            id,
            topic: topic.to_string(),
            focus_nodes,
            depth,
            timestamp: Utc::now().timestamp() as u64,
            operation: "send".to_string(),
            channel_id: Some(channel_id),
            sender: Some(sender.to_string()),
        };
        self.semantic_operations.push(op.clone());
        Ok(op)
    }

    /// Extract semantics from a message.
    pub fn extract_semantic(&self, message_id: u64) -> CommResult<SemanticOperation> {
        let msg = self
            .messages
            .get(&message_id)
            .ok_or(CommError::MessageNotFound(message_id))?;
        Ok(SemanticOperation {
            id: 0,
            topic: String::new(),
            focus_nodes: vec![],
            depth: 1,
            timestamp: Utc::now().timestamp() as u64,
            operation: "extract".to_string(),
            channel_id: Some(msg.channel_id),
            sender: Some(msg.sender.clone()),
        })
    }

    /// Graft (merge) semantic layers.
    pub fn graft_semantic(
        &mut self,
        source_id: u64,
        target_id: u64,
        strategy: &str,
    ) -> CommResult<SemanticOperation> {
        let _ = (source_id, target_id, strategy);
        let id = self.next_semantic_id;
        self.next_semantic_id += 1;
        let op = SemanticOperation {
            id,
            topic: String::new(),
            focus_nodes: vec![],
            depth: 1,
            timestamp: Utc::now().timestamp() as u64,
            operation: format!("graft:{}->{}:{}", source_id, target_id, strategy),
            channel_id: None,
            sender: None,
        };
        self.semantic_operations.push(op.clone());
        Ok(op)
    }

    /// List semantic conflicts.
    pub fn list_semantic_conflicts(
        &self,
        channel_id: Option<u64>,
        severity: Option<&str>,
    ) -> Vec<&SemanticConflict> {
        self.semantic_conflicts
            .iter()
            .filter(|c| {
                channel_id.map_or(true, |cid| c.channel_id == Some(cid))
                    && severity.map_or(true, |s| c.severity == s)
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Affect queries
    // -----------------------------------------------------------------------

    /// Get the current affect state for an agent.
    pub fn get_affect_state(&self, agent_id: &str) -> Option<&AffectState> {
        self.affect_states.get(agent_id)
    }

    /// Set the affect resistance threshold.
    pub fn set_affect_resistance(&mut self, resistance: f64) -> f64 {
        let clamped = resistance.clamp(0.0, 1.0);
        self.affect_resistance = clamped;
        clamped
    }


    // -----------------------------------------------------------------------
    // Affect contagion pipeline
    // -----------------------------------------------------------------------

    /// Process affect contagion across all participants in a channel.
    ///
    /// For each message with affect metadata (valence, arousal, dominance),
    /// apply a simple contagion model: each receiver's state is nudged toward
    /// the sender's state, weighted by `(1 - affect_resistance)`.
    pub fn process_affect_contagion(
        &mut self,
        channel_id: u64,
    ) -> Vec<(String, f64, f64, f64)> {
        let channel = match self.channels.get(&channel_id) {
            Some(ch) => ch.clone(),
            None => return Vec::new(),
        };
        let participants = channel.participants.clone();
        let resistance = self.affect_resistance;

        // Collect messages with affect metadata
        let mut affect_messages: Vec<(String, f64, f64, f64)> = Vec::new();
        for msg in self.messages.values() {
            if msg.channel_id != channel_id {
                continue;
            }
            let valence = msg
                .metadata
                .get("valence")
                .and_then(|v| v.parse::<f64>().ok());
            let arousal = msg
                .metadata
                .get("arousal")
                .and_then(|v| v.parse::<f64>().ok());
            let dominance = msg
                .metadata
                .get("dominance")
                .and_then(|v| v.parse::<f64>().ok());

            if let (Some(v), Some(a), Some(d)) = (valence, arousal, dominance) {
                affect_messages.push((msg.sender.clone(), v, a, d));
            }
        }

        let mut results: Vec<(String, f64, f64, f64)> = Vec::new();

        for (sender, v, a, d) in &affect_messages {
            for participant in &participants {
                if participant == sender {
                    continue;
                }
                let weight = 1.0 - resistance;
                let current = self
                    .affect_states
                    .get(participant)
                    .cloned()
                    .unwrap_or_default();
                let new_valence =
                    (current.valence + (v - current.valence) * weight).clamp(-1.0, 1.0);
                let new_arousal =
                    (current.arousal + (a - current.arousal) * weight).clamp(0.0, 1.0);
                let new_dominance =
                    (current.dominance + (d - current.dominance) * weight).clamp(0.0, 1.0);

                let state = self
                    .affect_states
                    .entry(participant.clone())
                    .or_insert_with(AffectState::default);
                state.valence = new_valence;
                state.arousal = new_arousal;
                state.dominance = new_dominance;

                results.push((
                    participant.clone(),
                    new_valence,
                    new_arousal,
                    new_dominance,
                ));
            }
        }

        results
    }

    /// Retrieve the full affect history for an agent.
    ///
    /// Builds a history from the current affect state and any messages
    /// sent by or to the agent that carried affect metadata.
    pub fn get_affect_history(&self, agent: &str) -> types::AffectHistory {
        use crate::types::{AffectHistory, AffectHistoryEntry};

        let mut entries: Vec<AffectHistoryEntry> = Vec::new();

        // Scan messages for affect metadata involving this agent
        for msg in self.messages.values() {
            let involves_agent = msg.sender == agent
                || msg
                    .recipient
                    .as_deref()
                    .map_or(false, |r| r == agent);

            if !involves_agent {
                continue;
            }

            let valence = msg
                .metadata
                .get("valence")
                .and_then(|v| v.parse::<f64>().ok());
            let arousal = msg
                .metadata
                .get("arousal")
                .and_then(|v| v.parse::<f64>().ok());
            let dominance = msg
                .metadata
                .get("dominance")
                .and_then(|v| v.parse::<f64>().ok());

            if valence.is_some() || arousal.is_some() || dominance.is_some() {
                entries.push(AffectHistoryEntry {
                    timestamp: msg.timestamp.timestamp() as u64,
                    emotion: String::new(),
                    intensity: 0.0,
                    valence: valence.unwrap_or(0.0),
                    arousal: arousal.unwrap_or(0.0),
                    dominance: dominance.unwrap_or(0.5),
                    source: if msg.sender == agent {
                        "direct".to_string()
                    } else {
                        "contagion".to_string()
                    },
                });
            }
        }

        // Add current state if it exists
        if let Some(state) = self.affect_states.get(agent) {
            entries.push(AffectHistoryEntry {
                timestamp: chrono::Utc::now().timestamp() as u64,
                emotion: String::new(),
                intensity: 0.0,
                valence: state.valence,
                arousal: state.arousal,
                dominance: state.dominance,
                source: "current".to_string(),
            });
        }

        entries.sort_by_key(|e| e.timestamp);

        AffectHistory {
            agent: agent.to_string(),
            states: entries,
        }
    }

    /// Apply temporal decay to all agent affect states.
    ///
    /// Each dimension is multiplied by `(1.0 - decay_rate)`, then clamped
    /// to valid ranges: valence [-1.0, 1.0], arousal [0.0, 1.0],
    /// dominance [0.0, 1.0].
    pub fn apply_affect_decay(&mut self, decay_rate: f64) {
        let factor = 1.0 - decay_rate.clamp(0.0, 1.0);
        for state in self.affect_states.values_mut() {
            state.valence = (state.valence * factor).clamp(-1.0, 1.0);
            state.arousal = (state.arousal * factor).clamp(0.0, 1.0);
            state.dominance = (state.dominance * factor).clamp(0.0, 1.0);
        }
    }

    // -----------------------------------------------------------------------
    // Message forwarding with echo tracking
    // -----------------------------------------------------------------------

    /// Forward a message to another channel with echo tracking metadata.
    ///
    /// Creates a new message in `target_channel` with content prefixed
    /// "[Forwarded] " and metadata tracking the forwarding chain.
    pub fn forward_message(
        &mut self,
        original_id: u64,
        target_channel: u64,
        forwarder: &str,
    ) -> Result<u64, String> {
        let original = self
            .messages
            .get(&original_id)
            .cloned()
            .ok_or_else(|| format!("Message {} not found", original_id))?;

        if !self.channels.contains_key(&target_channel) {
            return Err(format!("Target channel {} not found", target_channel));
        }

        // Determine echo depth and original root
        let parent_depth: u32 = original
            .metadata
            .get("echo_depth")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let root_id = original
            .metadata
            .get("original_message_id")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(original_id);

        let content = format!("[Forwarded] {}", original.content);

        let id = self.next_message_id;
        self.next_message_id += 1;

        self.lamport_counter += 1;
        let mut ts = types::CommTimestamp::now(forwarder);
        ts.lamport = self.lamport_counter;

        let mut metadata = HashMap::new();
        metadata.insert("forwarded_from".to_string(), original_id.to_string());
        metadata.insert("echo_depth".to_string(), (parent_depth + 1).to_string());
        metadata.insert("original_message_id".to_string(), root_id.to_string());
        metadata.insert("forwarder".to_string(), forwarder.to_string());

        let message = Message {
            id,
            channel_id: target_channel,
            sender: forwarder.to_string(),
            recipient: None,
            content,
            message_type: MessageType::Text,
            timestamp: Utc::now(),
            metadata,
            signature: Some(self.compute_signature(&original.content)),
            acknowledged_by: Vec::new(),
            status: MessageStatus::Sent,
            priority: MessagePriority::default(),
            reply_to: None,
            correlation_id: None,
            thread_id: None,
            comm_timestamp: ts,
            rich_content_json: None,
            comm_id: None,
        };

        self.messages.insert(id, message);

        self.log_audit(
            AuditEventType::MessageSent,
            forwarder,
            &format!(
                "Forwarded message {} to channel {} (depth {})",
                original_id,
                target_channel,
                parent_depth + 1
            ),
            Some(id.to_string()),
        );

        Ok(id)
    }

    /// Trace the full forwarding (echo) chain of a message.
    ///
    /// Follows "forwarded_from" metadata backwards to the root, then
    /// searches forward for all messages forwarded from any message in the
    /// chain.
    pub fn query_echo_chain(&self, message_id: u64) -> Vec<types::EchoChainEntry> {
        use crate::types::EchoChainEntry;

        let mut chain: Vec<EchoChainEntry> = Vec::new();

        // Walk backwards to the root
        let mut current_id = message_id;
        let mut visited: std::collections::HashSet<u64> = std::collections::HashSet::new();
        loop {
            if !visited.insert(current_id) {
                break; // cycle protection
            }
            let msg = match self.messages.get(&current_id) {
                Some(m) => m,
                None => break,
            };
            let depth: u32 = msg
                .metadata
                .get("echo_depth")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            let forwarder = msg
                .metadata
                .get("forwarder")
                .cloned()
                .unwrap_or_else(|| msg.sender.clone());

            chain.push(EchoChainEntry {
                message_id: current_id,
                channel_id: msg.channel_id,
                sender: msg.sender.clone(),
                forwarder,
                depth,
                timestamp: msg.timestamp.timestamp() as u64,
            });

            match msg
                .metadata
                .get("forwarded_from")
                .and_then(|v| v.parse::<u64>().ok())
            {
                Some(parent) => current_id = parent,
                None => break,
            }
        }

        chain.reverse(); // root first

        // Walk forward: find all messages forwarded from any message in the chain
        let chain_ids: std::collections::HashSet<u64> =
            chain.iter().map(|e| e.message_id).collect();
        for msg in self.messages.values() {
            if chain_ids.contains(&msg.id) {
                continue; // already in chain
            }
            if let Some(parent_str) = msg.metadata.get("forwarded_from") {
                if let Ok(parent_id) = parent_str.parse::<u64>() {
                    if chain_ids.contains(&parent_id) {
                        let depth: u32 = msg
                            .metadata
                            .get("echo_depth")
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(0);
                        let forwarder = msg
                            .metadata
                            .get("forwarder")
                            .cloned()
                            .unwrap_or_else(|| msg.sender.clone());
                        chain.push(EchoChainEntry {
                            message_id: msg.id,
                            channel_id: msg.channel_id,
                            sender: msg.sender.clone(),
                            forwarder,
                            depth,
                            timestamp: msg.timestamp.timestamp() as u64,
                        });
                    }
                }
            }
        }

        chain.sort_by_key(|e| (e.depth, e.timestamp));
        chain
    }

    /// Get the forwarding depth of a message in its echo chain.
    ///
    /// Returns the "echo_depth" metadata value, or 0 if the message is an
    /// original (not forwarded).
    pub fn get_echo_depth(&self, message_id: u64) -> u32 {
        self.messages
            .get(&message_id)
            .and_then(|msg| {
                msg.metadata
                    .get("echo_depth")
                    .and_then(|v| v.parse().ok())
            })
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Conversation summarization
    // -----------------------------------------------------------------------

    /// Generate detailed conversation statistics for a channel.
    pub fn summarize_conversation(
        &self,
        channel_id: u64,
    ) -> Result<types::ConversationSummaryDetailed, String> {
        use crate::types::ConversationSummaryDetailed;

        let channel = self
            .channels
            .get(&channel_id)
            .ok_or_else(|| format!("Channel {} not found", channel_id))?;

        let msgs: Vec<&Message> = self
            .messages
            .values()
            .filter(|m| m.channel_id == channel_id)
            .collect();

        let message_count = msgs.len();
        let participants = channel.participants.clone();
        let participant_count = participants.len();

        let first_message_time = msgs
            .iter()
            .map(|m| m.timestamp.timestamp() as u64)
            .min();
        let last_message_time = msgs
            .iter()
            .map(|m| m.timestamp.timestamp() as u64)
            .max();

        let duration_secs = match (first_message_time, last_message_time) {
            (Some(first), Some(last)) if last > first => last - first,
            _ => 0,
        };

        let messages_per_minute = if duration_secs > 0 {
            (message_count as f64) / (duration_secs as f64 / 60.0)
        } else if message_count > 0 {
            message_count as f64
        } else {
            0.0
        };

        // Count messages per sender
        let mut sender_counts: HashMap<String, usize> = HashMap::new();
        for msg in &msgs {
            *sender_counts
                .entry(msg.sender.clone())
                .or_insert(0) += 1;
        }
        let mut top_senders: Vec<(String, usize)> = sender_counts.into_iter().collect();
        top_senders.sort_by(|a, b| b.1.cmp(&a.1));

        let most_active_participant = top_senders.first().map(|(name, _)| name.clone());
        let most_active_count = top_senders.first().map(|(_, c)| *c).unwrap_or(0);

        // Thread count
        let thread_ids: std::collections::HashSet<&str> = msgs
            .iter()
            .filter_map(|m| m.thread_id.as_deref())
            .collect();
        let thread_count = thread_ids.len();

        // Reply count
        let reply_count = msgs.iter().filter(|m| m.reply_to.is_some()).count();

        // Average message length
        let avg_message_length = if message_count > 0 {
            msgs.iter().map(|m| m.content.len()).sum::<usize>() as f64 / message_count as f64
        } else {
            0.0
        };

        // Check for affect data
        let has_affect_data = msgs.iter().any(|m| m.metadata.contains_key("valence"));

        Ok(ConversationSummaryDetailed {
            channel_id,
            channel_name: channel.name.clone(),
            participant_count,
            message_count,
            participants,
            first_message_time,
            last_message_time,
            duration_secs,
            messages_per_minute,
            top_senders,
            most_active_participant,
            most_active_count,
            avg_message_length,
            thread_count,
            reply_count,
            has_affect_data,
        })
    }

    // -----------------------------------------------------------------------
    // Hive extensions
    // -----------------------------------------------------------------------

    /// Broadcast a question to all hive members and return aggregated response.
    pub fn hive_think(
        &self,
        hive_id: u64,
        question: &str,
        timeout_ms: u64,
    ) -> CommResult<serde_json::Value> {
        let hive = self
            .hive_minds
            .get(&hive_id)
            .ok_or_else(|| CommError::HiveError(format!("Hive {hive_id} not found")))?;
        Ok(serde_json::json!({
            "hive_id": hive_id,
            "hive_name": hive.name,
            "question": question,
            "timeout_ms": timeout_ms,
            "members": hive.constituents.len(),
            "status": "thought_broadcast",
        }))
    }

    /// Initiate a deep mind-meld session with a partner agent.
    pub fn initiate_meld(
        &mut self,
        partner_id: &str,
        depth: &str,
        duration_ms: u64,
    ) -> MeldSession {
        let id = format!("meld-{}", Utc::now().timestamp_millis());
        let session = MeldSession {
            id: id.clone(),
            partner_id: partner_id.to_string(),
            depth: depth.to_string(),
            start_time: Utc::now().timestamp() as u64,
            duration_ms,
            active: true,
        };
        self.meld_sessions.push(session.clone());
        session
    }

    // -----------------------------------------------------------------------
    // Consent flow (pending requests)
    // -----------------------------------------------------------------------

    /// List pending consent requests.
    pub fn list_pending_consent(
        &self,
        agent_id: Option<&str>,
        consent_type: Option<&str>,
    ) -> Vec<&ConsentRequest> {
        self.pending_consent_requests
            .iter()
            .filter(|r| {
                !r.responded
                    && agent_id.map_or(true, |a| r.to == a || r.from == a)
                    && consent_type.map_or(true, |ct| r.consent_type == ct)
            })
            .collect()
    }

    /// Respond to a pending consent request.
    pub fn respond_consent(
        &mut self,
        request_id: &str,
        response: &str,
    ) -> CommResult<()> {
        let req = self
            .pending_consent_requests
            .iter_mut()
            .find(|r| r.id == request_id)
            .ok_or_else(|| {
                CommError::ConsentError(format!("Consent request '{request_id}' not found"))
            })?;
        if req.responded {
            return Err(CommError::ConsentError(format!(
                "Consent request '{request_id}' already responded"
            )));
        }
        req.responded = true;
        req.response = Some(response.to_string());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Query helpers
    // -----------------------------------------------------------------------

    /// Query relationships between agents.
    pub fn query_relationships(
        &self,
        agent_id: &str,
        relationship_type: Option<&str>,
        depth: u64,
    ) -> serde_json::Value {
        let _ = depth;
        let mut relationships = Vec::new();
        // Check trust levels
        if let Some(level) = self.trust_levels.get(agent_id) {
            relationships.push(serde_json::json!({
                "type": "trust",
                "agent": agent_id,
                "level": level.to_string(),
            }));
        }
        // Check consent gates
        for gate in &self.consent_gates {
            if gate.grantor == agent_id || gate.grantee == agent_id {
                if relationship_type.map_or(true, |rt| rt == "consent") {
                    relationships.push(serde_json::json!({
                        "type": "consent",
                        "from": gate.grantor,
                        "to": gate.grantee,
                        "scope": gate.scope.to_string(),
                        "status": gate.status,
                    }));
                }
            }
        }
        // Check hive membership
        for hive in self.hive_minds.values() {
            if hive.constituents.iter().any(|c| c.agent_id == agent_id) {
                if relationship_type.map_or(true, |rt| rt == "hive") {
                    relationships.push(serde_json::json!({
                        "type": "hive_membership",
                        "hive_id": hive.id,
                        "hive_name": hive.name,
                    }));
                }
            }
        }
        serde_json::json!({
            "agent_id": agent_id,
            "relationships": relationships,
        })
    }

    /// Query conversation echoes (messages that reference or reply to a message).
    pub fn query_echoes(
        &self,
        message_id: u64,
        depth: u64,
    ) -> CommResult<serde_json::Value> {
        let msg = self
            .messages
            .get(&message_id)
            .ok_or(CommError::MessageNotFound(message_id))?;
        let _ = depth;
        // Find messages that mention the same topic or are in the same channel
        let echoes: Vec<serde_json::Value> = self
            .messages
            .values()
            .filter(|m| m.channel_id == msg.channel_id && m.id != message_id)
            .take(50)
            .map(|m| {
                serde_json::json!({
                    "message_id": m.id,
                    "sender": m.sender,
                    "channel_id": m.channel_id,
                    "timestamp": m.timestamp.to_rfc3339(),
                })
            })
            .collect();
        Ok(serde_json::json!({
            "source_message_id": message_id,
            "echo_count": echoes.len(),
            "echoes": echoes,
        }))
    }

    /// Query conversation summaries.
    pub fn query_conversations(
        &self,
        channel_id: Option<u64>,
        participant: Option<&str>,
        limit: u64,
    ) -> Vec<ConversationSummary> {
        let mut summaries: Vec<ConversationSummary> = self
            .channels
            .values()
            .filter(|ch| {
                channel_id.map_or(true, |cid| ch.id == cid)
                    && participant.map_or(true, |p| ch.participants.contains(&p.to_string()))
            })
            .map(|ch| {
                let msg_count = self
                    .messages
                    .values()
                    .filter(|m| m.channel_id == ch.id)
                    .count() as u64;
                let last_activity = self
                    .messages
                    .values()
                    .filter(|m| m.channel_id == ch.id)
                    .map(|m| m.timestamp.timestamp() as u64)
                    .max()
                    .unwrap_or(0);
                ConversationSummary {
                    channel_id: ch.id,
                    participants: ch.participants.clone(),
                    message_count: msg_count,
                    last_activity,
                }
            })
            .collect();
        summaries.truncate(limit as usize);
        summaries
    }

    // -----------------------------------------------------------------------
    // Federation extensions
    // -----------------------------------------------------------------------

    /// Get federation status.
    pub fn get_federation_status(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.federation_config.enabled,
            "local_zone": self.federation_config.local_zone,
            "zone_count": self.federation_config.zones.len(),
            "zones": self.federation_config.zones.iter().map(|z| &z.zone_id).collect::<Vec<_>>(),
            "default_policy": format!("{}", self.federation_config.default_policy),
        })
    }

    /// Set federation policy for a zone.
    pub fn set_federation_policy(
        &mut self,
        zone_id: &str,
        allow_semantic: bool,
        allow_affect: bool,
        allow_hive: bool,
        max_message_size: u64,
    ) -> ZonePolicyConfig {
        let config = ZonePolicyConfig {
            zone_id: zone_id.to_string(),
            allow_semantic,
            allow_affect,
            allow_hive,
            max_message_size,
        };
        self.zone_policies.insert(zone_id.to_string(), config.clone());
        config
    }

    // -----------------------------------------------------------------------
    // Grounding (mirrors memory's memory_ground)
    // -----------------------------------------------------------------------

    /// Ground a claim against the communication store.
    pub fn ground_claim(&self, claim: &str) -> GroundingResult {
        let claim_lower = claim.to_lowercase();
        let mut evidence = Vec::new();
        let mut score = 0.0f64;

        // Check channels
        for ch in self.channels.values() {
            if claim_lower.contains(&ch.name.to_lowercase()) {
                evidence.push(GroundingEvidence {
                    evidence_type: "channel".to_string(),
                    source: String::new(),
                    timestamp: 0,
                    content: format!("Channel '{}' (id={}, state={})", ch.name, ch.id, ch.state),
                    relevance: 0.9,
                });
                score = score.max(0.8);
            }
            for p in &ch.participants {
                if claim_lower.contains(&p.to_lowercase()) {
                    evidence.push(GroundingEvidence {
                        evidence_type: "participant".to_string(),
                        source: String::new(),
                        timestamp: 0,
                        content: format!("'{}' is a participant in channel '{}'", p, ch.name),
                        relevance: 0.8,
                    });
                    score = score.max(0.7);
                }
            }
        }

        // Check messages
        for msg in self.messages.values() {
            if claim_lower.contains(&msg.sender.to_lowercase())
                || claim_lower.contains(&msg.content.to_lowercase().chars().take(50).collect::<String>())
            {
                evidence.push(GroundingEvidence {
                    evidence_type: "message".to_string(),
                    source: String::new(),
                    timestamp: 0,
                    content: format!(
                        "Message from '{}' in channel {} at {}",
                        msg.sender, msg.channel_id, msg.timestamp
                    ),
                    relevance: 0.7,
                });
                score = score.max(0.6);
            }
        }

        // Check consent gates
        for gate in &self.consent_gates {
            if claim_lower.contains(&gate.grantor.to_lowercase())
                || claim_lower.contains(&gate.grantee.to_lowercase())
            {
                evidence.push(GroundingEvidence {
                    evidence_type: "consent".to_string(),
                    source: String::new(),
                    timestamp: 0,
                    content: format!(
                        "Consent: {} -> {} ({}, status={})",
                        gate.grantor, gate.grantee, gate.scope, gate.status
                    ),
                    relevance: 0.8,
                });
                score = score.max(0.7);
            }
        }

        // Check trust levels
        for (agent, level) in &self.trust_levels {
            if claim_lower.contains(&agent.to_lowercase()) {
                evidence.push(GroundingEvidence {
                    evidence_type: "trust".to_string(),
                    source: String::new(),
                    timestamp: 0,
                    content: format!("Trust level for '{}': {}", agent, level),
                    relevance: 0.8,
                });
                score = score.max(0.7);
            }
        }

        // Check hive minds
        for hive in self.hive_minds.values() {
            if claim_lower.contains(&hive.name.to_lowercase()) {
                evidence.push(GroundingEvidence {
                    evidence_type: "hive".to_string(),
                    source: String::new(),
                    timestamp: 0,
                    content: format!(
                        "Hive '{}' (id={}, members={})",
                        hive.name, hive.id, hive.constituents.len()
                    ),
                    relevance: 0.8,
                });
                score = score.max(0.7);
            }
        }

        let status = if score >= 0.7 {
            GroundingStatus::Verified
        } else if score >= 0.3 {
            GroundingStatus::Partial
        } else {
            GroundingStatus::Ungrounded
        };

        GroundingResult {
            claim: claim.to_string(),
            status,
            evidence,
            confidence: score,
        }
    }

    // -----------------------------------------------------------------------
    // Key management
    // -----------------------------------------------------------------------

    /// Generate a new key entry with metadata.
    ///
    /// Creates a key entry with a pseudo-random fingerprint. This is a stub
    /// that manages key metadata; real cryptographic key material would be
    /// generated by a dedicated crypto layer.
    pub fn generate_key(
        &mut self,
        algorithm: &str,
        channel_id: Option<u64>,
    ) -> CommResult<KeyEntry> {
        let id = self.next_key_id;
        self.next_key_id += 1;

        // Generate a pseudo-random fingerprint from id + timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let fingerprint = format!("{:016x}", now.wrapping_mul(6364136223846793005).wrapping_add(id));

        let entry = KeyEntry {
            id,
            algorithm: algorithm.to_string(),
            created_at: now,
            status: "active".to_string(),
            channel_id,
            fingerprint,
        };

        self.key_store.push(entry.clone());
        Ok(entry)
    }

    /// List all key entries.
    pub fn list_keys(&self) -> Vec<&KeyEntry> {
        self.key_store.iter().collect()
    }

    /// Get a specific key by ID.
    pub fn get_key(&self, key_id: u64) -> CommResult<&KeyEntry> {
        self.key_store
            .iter()
            .find(|k| k.id == key_id)
            .ok_or(CommError::KeyNotFound(key_id))
    }

    /// Rotate a key: mark the old key as "rotated" and create a new key
    /// with the same algorithm and channel binding.
    pub fn rotate_key(&mut self, key_id: u64) -> CommResult<KeyEntry> {
        // Find and mark old key as rotated
        let (algorithm, channel_id) = {
            let old_key = self
                .key_store
                .iter_mut()
                .find(|k| k.id == key_id)
                .ok_or(CommError::KeyNotFound(key_id))?;

            if old_key.status == "revoked" {
                return Err(CommError::KeyNotFound(key_id));
            }

            let alg = old_key.algorithm.clone();
            let ch = old_key.channel_id;
            old_key.status = "rotated".to_string();
            (alg, ch)
        };

        // Generate a new key with the same settings
        self.generate_key(&algorithm, channel_id)
    }

    /// Revoke a key by ID.
    pub fn revoke_key(&mut self, key_id: u64) -> CommResult<()> {
        let key = self
            .key_store
            .iter_mut()
            .find(|k| k.id == key_id)
            .ok_or(CommError::KeyNotFound(key_id))?;

        key.status = "revoked".to_string();
        Ok(())
    }

    /// Export a key's fingerprint (stub for real key export).
    pub fn export_key(&self, key_id: u64) -> CommResult<String> {
        let key = self.get_key(key_id)?;
        Ok(key.fingerprint.clone())
    }

    // -----------------------------------------------------------------------
    // Grounding: evidence search & fuzzy suggest
    // -----------------------------------------------------------------------

    /// Search messages, channels, and agents for evidence matching a query.
    ///
    /// Returns detailed evidence entries with timestamps and relevance scores.
    pub fn ground_evidence(&self, query: &str) -> Vec<GroundingEvidence> {
        let query_lower = query.to_lowercase();
        let mut evidence = Vec::new();

        // Search messages
        for msg in self.messages.values() {
            let content_lower = msg.content.to_lowercase();
            let sender_lower = msg.sender.to_lowercase();
            if content_lower.contains(&query_lower) || sender_lower.contains(&query_lower) {
                let relevance = if content_lower.contains(&query_lower) && sender_lower.contains(&query_lower) {
                    1.0
                } else if content_lower.contains(&query_lower) {
                    0.8
                } else {
                    0.6
                };
                evidence.push(GroundingEvidence {
                    evidence_type: format!("message(id={}, ts={})", msg.id, msg.timestamp.timestamp()),
                    source: "messages".to_string(),
                    timestamp: msg.timestamp.timestamp() as u64,
                    content: format!(
                        "[{}] {}: {}",
                        msg.timestamp.to_rfc3339(),
                        msg.sender,
                        msg.content.chars().take(200).collect::<String>()
                    ),
                    relevance,
                });
            }
        }

        // Search channels
        for ch in self.channels.values() {
            if ch.name.to_lowercase().contains(&query_lower) {
                evidence.push(GroundingEvidence {
                    evidence_type: format!("channel(id={}, created={})", ch.id, ch.created_at.timestamp()),
                    source: "channels".to_string(),
                    timestamp: ch.created_at.timestamp() as u64,
                    content: format!(
                        "Channel '{}' (type={}, state={}, participants={})",
                        ch.name, ch.channel_type, ch.state, ch.participants.len()
                    ),
                    relevance: 0.9,
                });
            }
            // Search participants
            for p in &ch.participants {
                if p.to_lowercase().contains(&query_lower) {
                    evidence.push(GroundingEvidence {
                        evidence_type: format!("agent(channel={})", ch.name),
                        source: "agents".to_string(),
                        timestamp: 0,
                        content: format!("Agent '{}' in channel '{}'", p, ch.name),
                        relevance: 0.7,
                    });
                }
            }
        }

        // Search hive minds
        for hive in self.hive_minds.values() {
            if hive.name.to_lowercase().contains(&query_lower) {
                evidence.push(GroundingEvidence {
                    evidence_type: format!("hive(id={})", hive.id),
                    source: "hives".to_string(),
                    timestamp: 0,
                    content: format!(
                        "Hive '{}' with {} constituents",
                        hive.name,
                        hive.constituents.len()
                    ),
                    relevance: 0.8,
                });
            }
        }

        // Sort by relevance descending
        evidence.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        evidence
    }

    /// Return fuzzy/contains suggestions based on agent names, channel names,
    /// or message content matching the query.
    pub fn ground_suggest(&self, query: &str, limit: usize) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let mut suggestions = Vec::new();

        // Suggest channel names
        for ch in self.channels.values() {
            if ch.name.to_lowercase().contains(&query_lower) {
                suggestions.push(format!("channel:{}", ch.name));
            }
        }

        // Suggest agent names from participants
        let mut seen_agents = std::collections::HashSet::new();
        for ch in self.channels.values() {
            for p in &ch.participants {
                if p.to_lowercase().contains(&query_lower) && seen_agents.insert(p.clone()) {
                    suggestions.push(format!("agent:{}", p));
                }
            }
        }

        // Suggest from trust levels
        for agent in self.trust_levels.keys() {
            if agent.to_lowercase().contains(&query_lower) && seen_agents.insert(agent.clone()) {
                suggestions.push(format!("agent:{}", agent));
            }
        }

        // Suggest hive mind names
        for hive in self.hive_minds.values() {
            if hive.name.to_lowercase().contains(&query_lower) {
                suggestions.push(format!("hive:{}", hive.name));
            }
        }

        // Suggest from message content (unique snippets)
        let mut content_seen = std::collections::HashSet::new();
        for msg in self.messages.values() {
            if msg.content.to_lowercase().contains(&query_lower) {
                let snippet: String = msg.content.chars().take(80).collect();
                if content_seen.insert(snippet.clone()) {
                    suggestions.push(format!("message:{}", snippet));
                }
            }
        }

        suggestions.truncate(limit);
        suggestions
    }

    // -----------------------------------------------------------------------
    // CommId and MessageContent helpers
    // -----------------------------------------------------------------------

    /// Assign CommIds to all messages and channels that don't already have one.
    ///
    /// Deterministically derives the UUID from the legacy u64 id so that
    /// repeated calls are idempotent.
    pub fn assign_comm_ids(&mut self) {
        let msg_ids: Vec<u64> = self.messages.keys().copied().collect();
        for id in msg_ids {
            if let Some(msg) = self.messages.get_mut(&id) {
                if msg.comm_id.is_none() {
                    msg.comm_id = Some(CommId::from_u64(msg.id));
                }
            }
        }
        let chan_ids: Vec<u64> = self.channels.keys().copied().collect();
        for id in chan_ids {
            if let Some(channel) = self.channels.get_mut(&id) {
                if channel.comm_id.is_none() {
                    channel.comm_id = Some(CommId::from_u64(channel.id));
                }
            }
        }
    }

    /// Look up a message by its CommId.
    pub fn get_message_by_comm_id(&self, comm_id: &CommId) -> Option<&Message> {
        self.messages.values().find(|m| m.comm_id.as_ref() == Some(comm_id))
    }

    /// Look up a channel by its CommId.
    pub fn get_channel_by_comm_id(&self, comm_id: &CommId) -> Option<&Channel> {
        self.channels.values().find(|c| c.comm_id.as_ref() == Some(comm_id))
    }

    /// Send a message with rich content.
    ///
    /// Sends a regular message and attaches a `MessageContent` (serialized
    /// as JSON) to the `rich_content_json` field.
    pub fn send_rich_message(
        &mut self,
        channel_id: u64,
        sender: &str,
        content: MessageContent,
        msg_type: MessageType,
    ) -> CommResult<Message> {
        let text = content.as_text().to_string();
        let rich_json = serde_json::to_string(&content)
            .map_err(|e| CommError::InvalidContent(format!("Failed to serialize rich content: {}", e)))?;
        let mut msg = self.send_message(channel_id, sender, &text, msg_type)?;
        // Update the stored message with rich content
        if let Some(stored) = self.messages.get_mut(&msg.id) {
            stored.rich_content_json = Some(rich_json.clone());
            msg.rich_content_json = Some(rich_json);
        }
        Ok(msg)
    }

    /// Get the rich content of a message (if any).
    pub fn get_rich_content(&self, message_id: u64) -> CommResult<Option<MessageContent>> {
        let msg = self.messages.get(&message_id)
            .ok_or(CommError::MessageNotFound(message_id))?;
        match &msg.rich_content_json {
            Some(json_str) => {
                let content: MessageContent = serde_json::from_str(json_str)
                    .map_err(|e| CommError::InvalidContent(format!("Failed to parse rich content: {}", e)))?;
                Ok(Some(content))
            }
            None => Ok(None),
        }
    }
}

/// Summary statistics for a CommStore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommStoreStats {
    /// Number of channels.
    pub channel_count: usize,
    /// Number of messages.
    pub message_count: usize,
    /// Number of active subscriptions.
    pub subscription_count: usize,
    /// Total number of participants across all channels.
    pub total_participants: usize,
    /// Number of messages in the dead letter queue.
    #[serde(default)]
    pub dead_letter_count: usize,
    /// Count of messages grouped by message type.
    #[serde(default)]
    pub messages_by_type: HashMap<String, usize>,
    /// Count of messages grouped by priority.
    #[serde(default)]
    pub messages_by_priority: HashMap<String, usize>,
    /// Count of channels grouped by state.
    #[serde(default)]
    pub channels_by_state: HashMap<String, usize>,
    /// Timestamp of the oldest message in the store.
    #[serde(default)]
    pub oldest_message: Option<DateTime<Utc>>,
    /// Timestamp of the newest message in the store.
    #[serde(default)]
    pub newest_message: Option<DateTime<Utc>>,
    /// Number of consent gates.
    #[serde(default)]
    pub consent_gate_count: usize,
    /// Number of trust level overrides.
    #[serde(default)]
    pub trust_override_count: usize,
    /// Number of scheduled temporal messages.
    #[serde(default)]
    pub temporal_queue_count: usize,
    /// Number of hive minds.
    #[serde(default)]
    pub hive_count: usize,
    /// Number of communication log entries.
    #[serde(default)]
    pub comm_log_count: usize,
    /// Whether federation is enabled.
    #[serde(default)]
    pub federation_enabled: bool,
    /// Number of federated zones.
    #[serde(default)]
    pub federated_zone_count: usize,
    /// Number of audit log entries.
    #[serde(default)]
    pub audit_log_count: usize,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use filetime::FileTime;

    fn new_store_with_channel() -> (CommStore, u64) {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("test-channel", ChannelType::Group, None)
            .unwrap();
        (store, ch.id)
    }

    // -- Channel tests --

    #[test]
    fn test_create_channel() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("my-channel", ChannelType::Group, None)
            .unwrap();
        assert_eq!(ch.name, "my-channel");
        assert_eq!(ch.channel_type, ChannelType::Group);
        assert!(ch.participants.is_empty());
    }

    #[test]
    fn test_create_channel_invalid_name_empty() {
        let mut store = CommStore::new();
        let result = store.create_channel("", ChannelType::Group, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_channel_invalid_name_special_chars() {
        let mut store = CommStore::new();
        let result = store.create_channel("bad channel!", ChannelType::Group, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_channel_long_name() {
        let mut store = CommStore::new();
        let long_name = "a".repeat(129);
        let result = store.create_channel(&long_name, ChannelType::Group, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_channels() {
        let mut store = CommStore::new();
        store
            .create_channel("alpha", ChannelType::Direct, None)
            .unwrap();
        store
            .create_channel("beta", ChannelType::Group, None)
            .unwrap();
        let channels = store.list_channels();
        assert_eq!(channels.len(), 2);
        assert_eq!(channels[0].name, "alpha");
        assert_eq!(channels[1].name, "beta");
    }

    #[test]
    fn test_get_channel() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("find-me", ChannelType::Broadcast, None)
            .unwrap();
        assert!(store.get_channel(ch.id).is_some());
        assert!(store.get_channel(999).is_none());
    }

    #[test]
    fn test_join_channel() {
        let (mut store, cid) = new_store_with_channel();
        store.join_channel(cid, "alice").unwrap();
        let ch = store.get_channel(cid).unwrap();
        assert_eq!(ch.participants, vec!["alice"]);
    }

    #[test]
    fn test_join_channel_duplicate() {
        let (mut store, cid) = new_store_with_channel();
        store.join_channel(cid, "alice").unwrap();
        let result = store.join_channel(cid, "alice");
        assert!(result.is_err());
    }

    #[test]
    fn test_join_channel_full() {
        let mut store = CommStore::new();
        let config = ChannelConfig {
            max_participants: 1,
            ..Default::default()
        };
        let ch = store
            .create_channel("tiny", ChannelType::Group, Some(config))
            .unwrap();
        store.join_channel(ch.id, "alice").unwrap();
        let result = store.join_channel(ch.id, "bob");
        assert!(result.is_err());
    }

    #[test]
    fn test_leave_channel() {
        let (mut store, cid) = new_store_with_channel();
        store.join_channel(cid, "alice").unwrap();
        store.leave_channel(cid, "alice").unwrap();
        let ch = store.get_channel(cid).unwrap();
        assert!(ch.participants.is_empty());
    }

    #[test]
    fn test_leave_channel_not_member() {
        let (mut store, cid) = new_store_with_channel();
        let result = store.leave_channel(cid, "ghost");
        assert!(result.is_err());
    }

    // -- Message tests --

    #[test]
    fn test_send_message() {
        let (mut store, cid) = new_store_with_channel();
        let msg = store
            .send_message(cid, "alice", "hello world", MessageType::Text)
            .unwrap();
        assert_eq!(msg.sender, "alice");
        assert_eq!(msg.content, "hello world");
        assert_eq!(msg.message_type, MessageType::Text);
        assert!(msg.signature.is_some());
    }

    #[test]
    fn test_send_message_empty_content() {
        let (mut store, cid) = new_store_with_channel();
        let result = store.send_message(cid, "alice", "", MessageType::Text);
        assert!(result.is_err());
    }

    #[test]
    fn test_send_message_empty_sender() {
        let (mut store, cid) = new_store_with_channel();
        let result = store.send_message(cid, "", "hi", MessageType::Text);
        assert!(result.is_err());
    }

    #[test]
    fn test_send_message_nonexistent_channel() {
        let mut store = CommStore::new();
        let result = store.send_message(999, "alice", "hi", MessageType::Text);
        assert!(result.is_err());
    }

    #[test]
    fn test_receive_messages() {
        let (mut store, cid) = new_store_with_channel();
        store
            .send_message(cid, "alice", "msg1", MessageType::Text)
            .unwrap();
        store
            .send_message(cid, "bob", "msg2", MessageType::Text)
            .unwrap();
        let msgs = store.receive_messages(cid, None, None).unwrap();
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn test_receive_messages_with_since() {
        let (mut store, cid) = new_store_with_channel();
        store
            .send_message(cid, "alice", "old msg", MessageType::Text)
            .unwrap();
        let cutoff = Utc::now();
        store
            .send_message(cid, "alice", "new msg", MessageType::Text)
            .unwrap();
        let msgs = store.receive_messages(cid, None, Some(cutoff)).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "new msg");
    }

    #[test]
    fn test_acknowledge_message() {
        let (mut store, cid) = new_store_with_channel();
        let msg = store
            .send_message(cid, "alice", "ack me", MessageType::Text)
            .unwrap();
        store.acknowledge_message(msg.id, "bob").unwrap();
        let updated = store.get_message(msg.id).unwrap();
        assert!(updated.acknowledged_by.contains(&"bob".to_string()));
    }

    #[test]
    fn test_acknowledge_nonexistent() {
        let mut store = CommStore::new();
        let result = store.acknowledge_message(999, "bob");
        assert!(result.is_err());
    }

    #[test]
    fn test_broadcast() {
        let (mut store, cid) = new_store_with_channel();
        store.join_channel(cid, "alice").unwrap();
        store.join_channel(cid, "bob").unwrap();
        store.join_channel(cid, "carol").unwrap();
        let msgs = store.broadcast(cid, "alice", "hello everyone").unwrap();
        // alice broadcasts to bob and carol (not self)
        assert_eq!(msgs.len(), 2);
    }

    // -- Pub/Sub tests --

    #[test]
    fn test_subscribe() {
        let mut store = CommStore::new();
        let sub = store.subscribe("weather", "sensor-1").unwrap();
        assert_eq!(sub.topic, "weather");
        assert_eq!(sub.subscriber, "sensor-1");
    }

    #[test]
    fn test_unsubscribe() {
        let mut store = CommStore::new();
        let sub = store.subscribe("weather", "sensor-1").unwrap();
        store.unsubscribe(sub.id).unwrap();
        assert!(store.unsubscribe(sub.id).is_err());
    }

    #[test]
    fn test_publish() {
        let mut store = CommStore::new();
        store.subscribe("alerts", "agent-a").unwrap();
        store.subscribe("alerts", "agent-b").unwrap();
        let msgs = store.publish("alerts", "monitor", "CPU high").unwrap();
        assert_eq!(msgs.len(), 2);
    }

    // -- Query tests --

    #[test]
    fn test_search_messages() {
        let (mut store, cid) = new_store_with_channel();
        store
            .send_message(cid, "alice", "hello world", MessageType::Text)
            .unwrap();
        store
            .send_message(cid, "bob", "goodbye world", MessageType::Text)
            .unwrap();
        store
            .send_message(cid, "carol", "hello there", MessageType::Text)
            .unwrap();
        let results = store.search_messages("hello", 10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_history_with_filter() {
        let (mut store, cid) = new_store_with_channel();
        store
            .send_message(cid, "alice", "text msg", MessageType::Text)
            .unwrap();
        store
            .send_message(cid, "bob", "command msg", MessageType::Command)
            .unwrap();
        let filter = MessageFilter {
            message_type: Some(MessageType::Command),
            ..Default::default()
        };
        let results = store.query_history(cid, &filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message_type, MessageType::Command);
    }

    #[test]
    fn test_get_message() {
        let (mut store, cid) = new_store_with_channel();
        let msg = store
            .send_message(cid, "alice", "find me", MessageType::Text)
            .unwrap();
        assert!(store.get_message(msg.id).is_some());
        assert!(store.get_message(999).is_none());
    }

    // -- Persistence tests --

    #[test]
    fn test_save_and_load() {
        let (mut store, cid) = new_store_with_channel();
        store.join_channel(cid, "alice").unwrap();
        store
            .send_message(cid, "alice", "persisted", MessageType::Text)
            .unwrap();
        store.subscribe("topic-a", "alice").unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.acomm");
        store.save(&path).unwrap();

        let loaded = CommStore::load(&path).unwrap();
        assert_eq!(loaded.channels.len(), 1);
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(loaded.subscriptions.len(), 1);
    }

    #[test]
    fn test_load_invalid_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.acomm");
        std::fs::write(&path, b"not a valid file").unwrap();
        let result = CommStore::load(&path);
        assert!(result.is_err());
    }

    // -- Stats --

    #[test]
    fn test_stats() {
        let (mut store, cid) = new_store_with_channel();
        store.join_channel(cid, "alice").unwrap();
        store
            .send_message(cid, "alice", "hi", MessageType::Text)
            .unwrap();
        let stats = store.stats();
        assert_eq!(stats.channel_count, 1);
        assert_eq!(stats.message_count, 1);
        assert_eq!(stats.total_participants, 1);
    }

    // -- Set channel config --

    #[test]
    fn test_set_channel_config() {
        let (mut store, cid) = new_store_with_channel();
        let config = ChannelConfig {
            max_participants: 10,
            ttl_seconds: 3600,
            persistence: false,
            encryption_required: true,
            ..Default::default()
        };
        store.set_channel_config(cid, config).unwrap();
        let ch = store.get_channel(cid).unwrap();
        assert_eq!(ch.config.max_participants, 10);
        assert!(ch.config.encryption_required);
    }

    // -- Message type parsing --

    #[test]
    fn test_message_type_roundtrip() {
        let types = vec![
            MessageType::Text,
            MessageType::Command,
            MessageType::Query,
            MessageType::Response,
            MessageType::Broadcast,
            MessageType::Notification,
            MessageType::Acknowledgment,
            MessageType::Error,
        ];
        for mt in types {
            let s = mt.to_string();
            let parsed: MessageType = s.parse().unwrap();
            assert_eq!(parsed, mt);
        }
    }

    // -- Channel type parsing --

    #[test]
    fn test_channel_type_roundtrip() {
        let types = vec![
            ChannelType::Direct,
            ChannelType::Group,
            ChannelType::Broadcast,
            ChannelType::PubSub,
        ];
        for ct in types {
            let s = ct.to_string();
            let parsed: ChannelType = s.parse().unwrap();
            assert_eq!(parsed, ct);
        }
    }

    // ===================================================================
    // NEW TESTS — Features 1-10
    // ===================================================================

    // -- test_message_priority_ordering --

    #[test]
    fn test_message_priority_ordering() {
        // MessagePriority derives Ord, so we can sort by priority
        let mut priorities = vec![
            MessagePriority::Critical,
            MessagePriority::Low,
            MessagePriority::Urgent,
            MessagePriority::Normal,
            MessagePriority::High,
        ];
        priorities.sort();
        assert_eq!(
            priorities,
            vec![
                MessagePriority::Low,
                MessagePriority::Normal,
                MessagePriority::High,
                MessagePriority::Urgent,
                MessagePriority::Critical,
            ]
        );
    }

    // -- test_channel_state_pause_blocks_send --

    #[test]
    fn test_channel_state_pause_blocks_send() {
        let (mut store, cid) = new_store_with_channel();
        store.pause_channel(cid).unwrap();
        let result = store.send_message(cid, "alice", "blocked", MessageType::Text);
        assert!(result.is_err());
        // Should have dead-lettered it
        assert_eq!(store.dead_letter_count(), 1);
    }

    // -- test_channel_state_drain_allows_receive --

    #[test]
    fn test_channel_state_drain_allows_receive() {
        let (mut store, cid) = new_store_with_channel();
        // Send a message while active
        store
            .send_message(cid, "alice", "before drain", MessageType::Text)
            .unwrap();
        // Now drain the channel
        store.drain_channel(cid).unwrap();
        // Receive should still work
        let msgs = store.receive_messages(cid, None, None).unwrap();
        assert_eq!(msgs.len(), 1);
        // But sending should fail
        let result = store.send_message(cid, "bob", "blocked", MessageType::Text);
        assert!(result.is_err());
    }

    // -- test_channel_state_close_blocks_all --

    #[test]
    fn test_channel_state_close_blocks_all() {
        let (mut store, cid) = new_store_with_channel();
        store
            .send_message(cid, "alice", "before close", MessageType::Text)
            .unwrap();
        store.close_channel(cid).unwrap();
        // Send should fail
        let send_result = store.send_message(cid, "bob", "nope", MessageType::Text);
        assert!(send_result.is_err());
        // Receive should also fail
        let recv_result = store.receive_messages(cid, None, None);
        assert!(recv_result.is_err());
    }

    // -- test_channel_resume_after_pause --

    #[test]
    fn test_channel_resume_after_pause() {
        let (mut store, cid) = new_store_with_channel();
        store.pause_channel(cid).unwrap();
        let ch = store.get_channel(cid).unwrap();
        assert_eq!(ch.state, ChannelState::Paused);

        store.resume_channel(cid).unwrap();
        let ch = store.get_channel(cid).unwrap();
        assert_eq!(ch.state, ChannelState::Active);

        // Should be able to send again
        let msg = store
            .send_message(cid, "alice", "resumed", MessageType::Text)
            .unwrap();
        assert_eq!(msg.content, "resumed");
    }

    // -- test_send_reply --

    #[test]
    fn test_send_reply() {
        let (mut store, cid) = new_store_with_channel();
        let parent = store
            .send_message(cid, "alice", "original question", MessageType::Query)
            .unwrap();
        let reply = store
            .send_reply(cid, parent.id, "bob", "answer here", MessageType::Response)
            .unwrap();
        assert_eq!(reply.reply_to, Some(parent.id));
        assert!(reply.thread_id.is_some());
        // Parent should also have a thread_id now
        let updated_parent = store.get_message(parent.id).unwrap();
        assert!(updated_parent.thread_id.is_some());
        assert_eq!(updated_parent.thread_id, reply.thread_id);
    }

    // -- test_get_thread --

    #[test]
    fn test_get_thread() {
        let (mut store, cid) = new_store_with_channel();
        let parent = store
            .send_message(cid, "alice", "start thread", MessageType::Text)
            .unwrap();
        let r1 = store
            .send_reply(cid, parent.id, "bob", "reply 1", MessageType::Response)
            .unwrap();
        let thread_id = r1.thread_id.clone().unwrap();
        store
            .send_reply(cid, parent.id, "carol", "reply 2", MessageType::Response)
            .unwrap();

        let thread = store.get_thread(&thread_id);
        // Parent + 2 replies = 3 messages in the thread
        assert_eq!(thread.len(), 3);
        // Ordered by timestamp
        assert!(thread[0].timestamp <= thread[1].timestamp);
        assert!(thread[1].timestamp <= thread[2].timestamp);
    }

    // -- test_get_replies --

    #[test]
    fn test_get_replies() {
        let (mut store, cid) = new_store_with_channel();
        let parent = store
            .send_message(cid, "alice", "parent msg", MessageType::Text)
            .unwrap();
        store
            .send_reply(cid, parent.id, "bob", "reply A", MessageType::Response)
            .unwrap();
        store
            .send_reply(cid, parent.id, "carol", "reply B", MessageType::Response)
            .unwrap();
        // Also send a non-reply message
        store
            .send_message(cid, "dave", "unrelated", MessageType::Text)
            .unwrap();

        let replies = store.get_replies(parent.id);
        assert_eq!(replies.len(), 2);
        assert!(replies.iter().all(|r| r.reply_to == Some(parent.id)));
    }

    // -- test_dead_letter_on_closed_channel --

    #[test]
    fn test_dead_letter_on_closed_channel() {
        let (mut store, cid) = new_store_with_channel();
        store.close_channel(cid).unwrap();

        let result = store.send_message(cid, "alice", "dropped", MessageType::Text);
        assert!(result.is_err());
        assert_eq!(store.dead_letter_count(), 1);

        let dls = store.list_dead_letters();
        assert_eq!(dls.len(), 1);
        assert_eq!(dls[0].original_message.content, "dropped");
        assert_eq!(dls[0].reason, DeadLetterReason::ChannelClosed);
    }

    // -- test_dead_letter_replay --

    #[test]
    fn test_dead_letter_replay() {
        let (mut store, cid) = new_store_with_channel();
        store.close_channel(cid).unwrap();

        // This will fail and dead-letter
        let _ = store.send_message(cid, "alice", "retry me", MessageType::Text);
        assert_eq!(store.dead_letter_count(), 1);

        // Reopen the channel
        store.resume_channel(cid).unwrap();

        // Replay the dead letter
        let msg = store.replay_dead_letter(0).unwrap();
        assert_eq!(msg.content, "retry me");
        assert_eq!(msg.status, MessageStatus::Sent);
        // Dead letter should be removed after successful replay
        assert_eq!(store.dead_letter_count(), 0);
    }

    // -- test_expire_messages --

    #[test]
    fn test_expire_messages() {
        let mut store = CommStore::new();
        let config = ChannelConfig {
            ttl_seconds: 1, // 1 second TTL
            ..Default::default()
        };
        let ch = store
            .create_channel("ephemeral", ChannelType::Group, Some(config))
            .unwrap();

        // Insert a message with an old timestamp by directly manipulating
        let id = 100;
        let old_time = Utc::now() - chrono::Duration::seconds(10);
        let msg = Message {
            id,
            channel_id: ch.id,
            sender: "alice".to_string(),
            recipient: None,
            content: "old message".to_string(),
            message_type: MessageType::Text,
            timestamp: old_time,
            metadata: HashMap::new(),
            signature: None,
            acknowledged_by: Vec::new(),
            status: MessageStatus::Sent,
            priority: MessagePriority::Normal,
            reply_to: None,
            correlation_id: None,
            thread_id: None,
            comm_timestamp: CommTimestamp::default(),
            rich_content_json: None,
            comm_id: None,
        };
        store.messages.insert(id, msg);

        // Also add a fresh message
        store
            .send_message(ch.id, "bob", "fresh message", MessageType::Text)
            .unwrap();

        let expired_count = store.expire_messages();
        assert_eq!(expired_count, 1);
        // The old message should be gone from messages
        assert!(store.get_message(100).is_none());
        // But should be in dead letters
        assert_eq!(store.dead_letter_count(), 1);
        let dls = store.list_dead_letters();
        assert_eq!(dls[0].reason, DeadLetterReason::Expired);
        // The fresh message should still be there
        assert_eq!(store.messages.len(), 1);
    }

    // -- test_compact_removes_closed_channel_messages --

    #[test]
    fn test_compact_removes_closed_channel_messages() {
        let (mut store, cid) = new_store_with_channel();
        store
            .send_message(cid, "alice", "msg1", MessageType::Text)
            .unwrap();
        store
            .send_message(cid, "alice", "msg2", MessageType::Text)
            .unwrap();

        // Create another active channel with a message
        let ch2 = store
            .create_channel("active-ch", ChannelType::Group, None)
            .unwrap();
        store
            .send_message(ch2.id, "bob", "active msg", MessageType::Text)
            .unwrap();

        assert_eq!(store.messages.len(), 3);

        // Close the first channel
        store.close_channel(cid).unwrap();

        let removed = store.compact();
        assert_eq!(removed, 2); // 2 messages from closed channel
        assert_eq!(store.messages.len(), 1); // only the active channel message remains
    }

    // -- test_delivery_mode_default --

    #[test]
    fn test_delivery_mode_default() {
        let config = ChannelConfig::default();
        assert_eq!(config.delivery_mode, DeliveryMode::AtLeastOnce);
        assert_eq!(config.retention_policy, RetentionPolicy::Forever);
    }

    // -- test_enhanced_stats --

    #[test]
    fn test_enhanced_stats() {
        let (mut store, cid) = new_store_with_channel();
        store
            .send_message(cid, "alice", "text msg", MessageType::Text)
            .unwrap();
        store
            .send_message(cid, "bob", "command msg", MessageType::Command)
            .unwrap();
        store
            .send_message_with_priority(
                cid,
                "carol",
                "urgent msg",
                MessageType::Text,
                MessagePriority::Urgent,
            )
            .unwrap();

        // Close a channel to get dead letters
        let ch2 = store
            .create_channel("closable", ChannelType::Group, None)
            .unwrap();
        store.close_channel(ch2.id).unwrap();
        let _ = store.send_message(ch2.id, "dave", "dropped", MessageType::Text);

        let stats = store.stats();
        assert_eq!(stats.channel_count, 2);
        assert_eq!(stats.message_count, 3);
        assert_eq!(stats.dead_letter_count, 1);

        // messages_by_type: 2 text, 1 command
        assert_eq!(stats.messages_by_type.get("text"), Some(&2));
        assert_eq!(stats.messages_by_type.get("command"), Some(&1));

        // messages_by_priority: 2 normal, 1 urgent
        assert_eq!(stats.messages_by_priority.get("normal"), Some(&2));
        assert_eq!(stats.messages_by_priority.get("urgent"), Some(&1));

        // channels_by_state: 1 active, 1 closed
        assert_eq!(stats.channels_by_state.get("active"), Some(&1));
        assert_eq!(stats.channels_by_state.get("closed"), Some(&1));

        // oldest/newest should be set
        assert!(stats.oldest_message.is_some());
        assert!(stats.newest_message.is_some());
    }

    // -- test_message_status_transitions --

    #[test]
    fn test_message_status_transitions() {
        let (mut store, cid) = new_store_with_channel();

        // Message starts as Sent (set during send_message)
        let msg = store
            .send_message(cid, "alice", "track me", MessageType::Text)
            .unwrap();
        assert_eq!(msg.status, MessageStatus::Sent);

        // After acknowledgment, status becomes Acknowledged
        store.acknowledge_message(msg.id, "bob").unwrap();
        let updated = store.get_message(msg.id).unwrap();
        assert_eq!(updated.status, MessageStatus::Acknowledged);

        // Default status is Created
        assert_eq!(MessageStatus::default(), MessageStatus::Created);
    }

    // -- test_send_message_with_priority --

    #[test]
    fn test_send_message_with_priority() {
        let (mut store, cid) = new_store_with_channel();
        let msg = store
            .send_message_with_priority(
                cid,
                "alice",
                "critical alert",
                MessageType::Notification,
                MessagePriority::Critical,
            )
            .unwrap();
        assert_eq!(msg.priority, MessagePriority::Critical);

        // Verify stored message also has the priority
        let stored = store.get_message(msg.id).unwrap();
        assert_eq!(stored.priority, MessagePriority::Critical);
    }

    // -- test_dead_letter_clear --

    #[test]
    fn test_dead_letter_clear() {
        let (mut store, cid) = new_store_with_channel();
        store.close_channel(cid).unwrap();
        let _ = store.send_message(cid, "alice", "dl1", MessageType::Text);
        let _ = store.send_message(cid, "alice", "dl2", MessageType::Text);
        assert_eq!(store.dead_letter_count(), 2);

        store.clear_dead_letters();
        assert_eq!(store.dead_letter_count(), 0);
    }

    // -- test_compact_enforces_retention_policy --

    #[test]
    fn test_compact_enforces_retention_policy() {
        let mut store = CommStore::new();
        let config = ChannelConfig {
            retention_policy: RetentionPolicy::MessageCount(2),
            ..Default::default()
        };
        let ch = store
            .create_channel("limited", ChannelType::Group, Some(config))
            .unwrap();

        // Send 5 messages
        for i in 0..5 {
            store
                .send_message(ch.id, "alice", &format!("msg-{i}"), MessageType::Text)
                .unwrap();
        }
        assert_eq!(store.messages.len(), 5);

        let removed = store.compact();
        assert_eq!(removed, 3); // 5 - 2 = 3 removed
        assert_eq!(store.messages.len(), 2);
    }

    // -- test_save_and_load_with_new_fields --

    #[test]
    fn test_save_and_load_with_new_fields() {
        let (mut store, cid) = new_store_with_channel();
        store.join_channel(cid, "alice").unwrap();

        // Use new features
        let msg = store
            .send_message_with_priority(
                cid,
                "alice",
                "priority msg",
                MessageType::Text,
                MessagePriority::High,
            )
            .unwrap();
        store
            .send_reply(cid, msg.id, "bob", "reply", MessageType::Response)
            .unwrap();

        // Close another channel to create dead letters
        let ch2 = store
            .create_channel("closing", ChannelType::Group, None)
            .unwrap();
        store.close_channel(ch2.id).unwrap();
        let _ = store.send_message(ch2.id, "carol", "dead", MessageType::Text);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_new.acomm");
        store.save(&path).unwrap();

        let loaded = CommStore::load(&path).unwrap();
        assert_eq!(loaded.channels.len(), 2);
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(loaded.dead_letters.len(), 1);

        // Verify new fields survived round-trip
        let loaded_msg = loaded.get_message(msg.id).unwrap();
        assert_eq!(loaded_msg.priority, MessagePriority::High);

        let ch2_loaded = loaded.get_channel(ch2.id).unwrap();
        assert_eq!(ch2_loaded.state, ChannelState::Closed);
    }

    // -- test_channel_state_display --

    #[test]
    fn test_channel_state_display() {
        assert_eq!(ChannelState::Active.to_string(), "active");
        assert_eq!(ChannelState::Paused.to_string(), "paused");
        assert_eq!(ChannelState::Draining.to_string(), "draining");
        assert_eq!(ChannelState::Closed.to_string(), "closed");
    }

    // -- test_dead_letter_on_nonexistent_channel --

    #[test]
    fn test_dead_letter_on_nonexistent_channel() {
        let mut store = CommStore::new();
        let result = store.send_message(999, "alice", "nowhere", MessageType::Text);
        assert!(result.is_err());
        assert_eq!(store.dead_letter_count(), 1);
        let dls = store.list_dead_letters();
        assert_eq!(dls[0].reason, DeadLetterReason::ChannelNotFound);
    }

    // --- Consent tests ---

    #[test]
    fn consent_grant_and_check() {
        let mut store = CommStore::new();
        store
            .grant_consent("alice", "bob", ConsentScope::ReadMessages, None, None)
            .unwrap();
        assert!(store.check_consent("alice", "bob", &ConsentScope::ReadMessages));
        assert!(!store.check_consent("bob", "alice", &ConsentScope::ReadMessages));
    }

    #[test]
    fn consent_revoke() {
        let mut store = CommStore::new();
        store
            .grant_consent("alice", "bob", ConsentScope::SendMessages, None, None)
            .unwrap();
        assert!(store.check_consent("alice", "bob", &ConsentScope::SendMessages));
        store
            .revoke_consent("alice", "bob", &ConsentScope::SendMessages)
            .unwrap();
        assert!(!store.check_consent("alice", "bob", &ConsentScope::SendMessages));
    }

    #[test]
    fn consent_list_filtered() {
        let mut store = CommStore::new();
        store.grant_consent("alice", "bob", ConsentScope::ReadMessages, None, None).unwrap();
        store.grant_consent("charlie", "bob", ConsentScope::SendMessages, None, None).unwrap();
        assert_eq!(store.list_consent_gates(Some("bob")).len(), 2);
        assert_eq!(store.list_consent_gates(Some("alice")).len(), 1);
        assert_eq!(store.list_consent_gates(None).len(), 2);
    }

    // --- Trust tests ---

    #[test]
    fn trust_set_and_get() {
        let mut store = CommStore::new();
        assert_eq!(store.get_trust_level("agent-1"), CommTrustLevel::Standard);
        store.set_trust_level("agent-1", CommTrustLevel::High).unwrap();
        assert_eq!(store.get_trust_level("agent-1"), CommTrustLevel::High);
    }

    #[test]
    fn trust_list() {
        let mut store = CommStore::new();
        store.set_trust_level("a", CommTrustLevel::Full).unwrap();
        store.set_trust_level("b", CommTrustLevel::Minimal).unwrap();
        assert_eq!(store.list_trust_levels().len(), 2);
    }

    // --- Temporal tests ---

    #[test]
    fn temporal_schedule_and_list() {
        let (mut store, ch_id) = new_store_with_channel();
        store.join_channel(ch_id, "alice").unwrap();
        store
            .schedule_message(ch_id, "alice", "hello future", TemporalTarget::Immediate, None)
            .unwrap();
        assert_eq!(store.list_scheduled().len(), 1);
    }

    #[test]
    fn temporal_cancel() {
        let (mut store, ch_id) = new_store_with_channel();
        store.join_channel(ch_id, "alice").unwrap();
        let msg = store
            .schedule_message(ch_id, "alice", "later", TemporalTarget::FutureRelative { delay_seconds: 3600 }, None)
            .unwrap();
        let tid = msg.id;
        assert_eq!(store.list_scheduled().len(), 1);
        store.cancel_scheduled(tid).unwrap();
        assert_eq!(store.list_scheduled().len(), 0);
    }

    #[test]
    fn temporal_deliver_immediate() {
        let (mut store, ch_id) = new_store_with_channel();
        store.join_channel(ch_id, "alice").unwrap();
        store
            .schedule_message(ch_id, "alice", "now!", TemporalTarget::Immediate, None)
            .unwrap();
        let delivered = store.deliver_pending_temporal();
        assert_eq!(delivered, 1);
        assert_eq!(store.list_scheduled().len(), 0);
    }

    // --- Federation tests ---

    #[test]
    fn federation_configure() {
        let mut store = CommStore::new();
        store
            .configure_federation(true, "zone-a", FederationPolicy::Allow)
            .unwrap();
        let config = store.get_federation_config();
        assert!(config.enabled);
        assert_eq!(config.local_zone, "zone-a");
    }

    #[test]
    fn federation_add_remove_zone() {
        let mut store = CommStore::new();
        store.add_federated_zone(FederatedZone {
            zone_id: "zone-b".to_string(),
            name: "Zone B".to_string(),
            endpoint: "https://b.example.com".to_string(),
            policy: FederationPolicy::Allow,
            trust_level: CommTrustLevel::High,
        }).unwrap();
        assert_eq!(store.list_federated_zones().len(), 1);
        store.remove_federated_zone("zone-b").unwrap();
        assert_eq!(store.list_federated_zones().len(), 0);
    }

    #[test]
    fn federation_duplicate_zone_error() {
        let mut store = CommStore::new();
        let zone = FederatedZone {
            zone_id: "z1".to_string(),
            name: "Z1".to_string(),
            endpoint: String::new(),
            policy: FederationPolicy::Deny,
            trust_level: CommTrustLevel::Basic,
        };
        store.add_federated_zone(zone.clone()).unwrap();
        assert!(store.add_federated_zone(zone).is_err());
    }

    // --- Hive mind tests ---

    #[test]
    fn hive_form_and_list() {
        let mut store = CommStore::new();
        store.form_hive("test-hive", "alice", CollectiveDecisionMode::Majority).unwrap();
        assert_eq!(store.list_hives().len(), 1);
    }

    #[test]
    fn hive_join_and_leave() {
        let mut store = CommStore::new();
        let hive = store.form_hive("h1", "alice", CollectiveDecisionMode::Consensus).unwrap();
        let hid = hive.id;
        store.join_hive(hid, "bob", HiveRole::Member).unwrap();
        assert_eq!(store.get_hive(hid).unwrap().constituents.len(), 2);
        store.leave_hive(hid, "bob").unwrap();
        assert_eq!(store.get_hive(hid).unwrap().constituents.len(), 1);
    }

    #[test]
    fn hive_dissolve() {
        let mut store = CommStore::new();
        let hive = store.form_hive("h2", "alice", CollectiveDecisionMode::CoordinatorDecides).unwrap();
        let hid = hive.id;
        store.dissolve_hive(hid).unwrap();
        assert!(store.get_hive(hid).is_none());
    }

    #[test]
    fn hive_join_duplicate_error() {
        let mut store = CommStore::new();
        let hive = store.form_hive("h3", "alice", CollectiveDecisionMode::Unanimous).unwrap();
        let hid = hive.id;
        assert!(store.join_hive(hid, "alice", HiveRole::Member).is_err());
    }

    // --- Communication log tests ---

    #[test]
    fn comm_log_entries() {
        let mut store = CommStore::new();
        store.log_communication("hello", "user", Some("greeting".to_string()), None, None);
        store.log_communication("hi back", "agent", Some("greeting".to_string()), None, None);
        assert_eq!(store.get_comm_log(None).len(), 2);
        assert_eq!(store.get_comm_log(Some(1)).len(), 1);
    }

    // --- Grounding tests ---

    #[test]
    fn grounding_verified_channel() {
        let (store, _ch_id) = new_store_with_channel();
        let result = store.ground_claim("test-channel exists");
        assert_eq!(result.status, GroundingStatus::Verified);
        assert!(!result.evidence.is_empty());
    }

    #[test]
    fn grounding_ungrounded() {
        let store = CommStore::new();
        let result = store.ground_claim("nonexistent-thing");
        assert_eq!(result.status, GroundingStatus::Ungrounded);
        assert!(result.evidence.is_empty());
    }

    #[test]
    fn grounding_trust_evidence() {
        let mut store = CommStore::new();
        store.set_trust_level("agent-x", CommTrustLevel::High).unwrap();
        let result = store.ground_claim("agent-x has trust");
        assert_ne!(result.status, GroundingStatus::Ungrounded);
    }

    // --- Stats tests ---

    #[test]
    fn stats_include_new_fields() {
        let mut store = CommStore::new();
        store.grant_consent("a", "b", ConsentScope::ReadMessages, None, None).unwrap();
        store.set_trust_level("x", CommTrustLevel::Full).unwrap();
        store.form_hive("h", "coord", CollectiveDecisionMode::Majority).unwrap();
        let stats = store.stats();
        assert_eq!(stats.consent_gate_count, 1);
        assert_eq!(stats.trust_override_count, 1);
        assert_eq!(stats.hive_count, 1);
    }

    // --- Affect messaging test ---

    #[test]
    fn affect_message_send() {
        let (mut store, ch_id) = new_store_with_channel();
        store.join_channel(ch_id, "alice").unwrap();
        let msg = store
            .send_affect_message(
                ch_id,
                "alice",
                "I'm excited!",
                AffectState {
                    valence: 0.8,
                    arousal: 0.9,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(msg.content.contains("[affect:"));
        assert!(msg.content.contains("I'm excited!"));
    }

    // ===================================================================
    // NEW TESTS — Consent, Trust, Temporal, Federation, Hive, Grounding,
    //             Comm Log, Stats, Save/Load round-trips (30 tests)
    // ===================================================================

    // --- Consent tests ---

    #[test]
    fn consent_grant_with_reason_and_expiry() {
        let mut store = CommStore::new();
        let entry = store
            .grant_consent(
                "alice",
                "bob",
                ConsentScope::ScheduleMessages,
                Some("project collaboration".to_string()),
                Some("2030-12-31T23:59:59Z".to_string()),
            )
            .unwrap();
        assert_eq!(entry.grantor, "alice");
        assert_eq!(entry.grantee, "bob");
        assert_eq!(entry.scope, ConsentScope::ScheduleMessages);
        assert_eq!(entry.status, ConsentStatus::Granted);
        assert_eq!(entry.reason.as_deref(), Some("project collaboration"));
        assert_eq!(entry.expires_at.as_deref(), Some("2030-12-31T23:59:59Z"));
    }

    #[test]
    fn consent_update_existing() {
        let mut store = CommStore::new();
        // First grant with no reason
        store
            .grant_consent("alice", "bob", ConsentScope::ReadMessages, None, None)
            .unwrap();
        assert!(store.check_consent("alice", "bob", &ConsentScope::ReadMessages));

        // Grant again with reason and expiry — should update, not duplicate
        store
            .grant_consent(
                "alice",
                "bob",
                ConsentScope::ReadMessages,
                Some("updated reason".to_string()),
                Some("2031-01-01T00:00:00Z".to_string()),
            )
            .unwrap();

        // Should still be only one entry for this (grantor, grantee, scope) triple
        let gates = store.list_consent_gates(Some("alice"));
        let matching: Vec<_> = gates
            .iter()
            .filter(|e| {
                e.grantor == "alice"
                    && e.grantee == "bob"
                    && e.scope == ConsentScope::ReadMessages
            })
            .collect();
        assert_eq!(matching.len(), 1, "Should update existing, not create duplicate");
        assert_eq!(matching[0].reason.as_deref(), Some("updated reason"));
    }

    #[test]
    fn consent_revoke_nonexistent_error() {
        let mut store = CommStore::new();
        let result = store.revoke_consent("alice", "bob", &ConsentScope::Federate);
        assert!(result.is_err(), "Revoking non-existent consent should fail");
    }

    #[test]
    fn consent_empty_grantor_error() {
        let mut store = CommStore::new();
        let result = store.grant_consent("", "bob", ConsentScope::ReadMessages, None, None);
        assert!(result.is_err(), "Empty grantor should return error");
    }

    #[test]
    fn consent_list_empty_returns_empty() {
        let store = CommStore::new();
        let gates = store.list_consent_gates(None);
        assert!(gates.is_empty(), "Fresh store should have no consent gates");
        let gates_filtered = store.list_consent_gates(Some("nobody"));
        assert!(gates_filtered.is_empty());
    }

    // --- Trust tests ---

    #[test]
    fn trust_empty_agent_error() {
        let mut store = CommStore::new();
        let result = store.set_trust_level("", CommTrustLevel::High);
        assert!(result.is_err(), "Empty agent_id should return error");
    }

    #[test]
    fn trust_override_existing() {
        let mut store = CommStore::new();
        store.set_trust_level("agent-1", CommTrustLevel::Basic).unwrap();
        assert_eq!(store.get_trust_level("agent-1"), CommTrustLevel::Basic);
        store.set_trust_level("agent-1", CommTrustLevel::Absolute).unwrap();
        assert_eq!(store.get_trust_level("agent-1"), CommTrustLevel::Absolute);
        // Should still be only one entry
        assert_eq!(store.list_trust_levels().len(), 1);
    }

    #[test]
    fn trust_default_is_standard() {
        let store = CommStore::new();
        assert_eq!(
            store.get_trust_level("unknown-agent"),
            CommTrustLevel::Standard,
            "Fresh agent should default to Standard trust"
        );
    }

    // --- Temporal tests ---

    #[test]
    fn temporal_schedule_to_nonexistent_channel() {
        let mut store = CommStore::new();
        let result = store.schedule_message(
            999,
            "alice",
            "hello",
            TemporalTarget::Immediate,
            None,
        );
        assert!(result.is_err(), "Scheduling to non-existent channel should fail");
    }

    #[test]
    fn temporal_cancel_delivered_error() {
        let (mut store, ch_id) = new_store_with_channel();
        let msg = store
            .schedule_message(ch_id, "alice", "now", TemporalTarget::Immediate, None)
            .unwrap();
        let tid = msg.id;

        // Deliver it
        let delivered = store.deliver_pending_temporal();
        assert_eq!(delivered, 1);

        // Try to cancel the already-delivered message
        let result = store.cancel_scheduled(tid);
        assert!(result.is_err(), "Cannot cancel already-delivered message");
    }

    #[test]
    fn temporal_cancel_nonexistent_error() {
        let mut store = CommStore::new();
        let result = store.cancel_scheduled(9999);
        assert!(result.is_err(), "Cancelling non-existent temporal ID should fail");
    }

    #[test]
    fn temporal_deliver_future_relative_not_delivered() {
        let (mut store, ch_id) = new_store_with_channel();
        store
            .schedule_message(
                ch_id,
                "alice",
                "later msg",
                TemporalTarget::FutureRelative { delay_seconds: 3600 },
                None,
            )
            .unwrap();

        // deliver_pending_temporal only delivers Immediate targets
        let delivered = store.deliver_pending_temporal();
        assert_eq!(delivered, 0, "FutureRelative messages should not be delivered by deliver_pending_temporal");
        assert_eq!(store.list_scheduled().len(), 1, "Message should still be in queue");
    }

    #[test]
    fn temporal_multiple_immediate() {
        let (mut store, ch_id) = new_store_with_channel();
        for i in 0..5 {
            store
                .schedule_message(
                    ch_id,
                    "alice",
                    &format!("immediate-{i}"),
                    TemporalTarget::Immediate,
                    None,
                )
                .unwrap();
        }
        assert_eq!(store.list_scheduled().len(), 5);

        let delivered = store.deliver_pending_temporal();
        assert_eq!(delivered, 5, "All 5 Immediate messages should be delivered");
        assert_eq!(store.list_scheduled().len(), 0, "No undelivered messages should remain");
        // The delivered messages should be in the message store
        assert_eq!(store.messages.len(), 5);
    }

    // --- Federation tests ---

    #[test]
    fn federation_empty_zone_error() {
        let mut store = CommStore::new();
        let result = store.configure_federation(true, "", FederationPolicy::Allow);
        assert!(result.is_err(), "Empty local_zone should return error");
    }

    #[test]
    fn federation_remove_nonexistent_zone_error() {
        let mut store = CommStore::new();
        let result = store.remove_federated_zone("does-not-exist");
        assert!(result.is_err(), "Removing non-existent zone should return error");
    }

    #[test]
    fn federation_zone_with_trust_level() {
        let mut store = CommStore::new();
        store
            .add_federated_zone(FederatedZone {
                zone_id: "trusted-zone".to_string(),
                name: "Trusted Zone".to_string(),
                endpoint: "https://trusted.example.com".to_string(),
                policy: FederationPolicy::Allow,
                trust_level: CommTrustLevel::Full,
            })
            .unwrap();
        let zones = store.list_federated_zones();
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].trust_level, CommTrustLevel::Full);
        assert_eq!(zones[0].zone_id, "trusted-zone");
        assert_eq!(zones[0].policy, FederationPolicy::Allow);
    }

    // --- Hive tests ---

    #[test]
    fn hive_form_empty_name_error() {
        let mut store = CommStore::new();
        let result = store.form_hive("", "alice", CollectiveDecisionMode::Majority);
        assert!(result.is_err(), "Empty hive name should return error");
    }

    #[test]
    fn hive_dissolve_nonexistent_error() {
        let mut store = CommStore::new();
        let result = store.dissolve_hive(9999);
        assert!(result.is_err(), "Dissolving non-existent hive should return error");
    }

    #[test]
    fn hive_leave_nonexistent_member_error() {
        let mut store = CommStore::new();
        let hive = store
            .form_hive("test-hive", "alice", CollectiveDecisionMode::Majority)
            .unwrap();
        let hid = hive.id;
        let result = store.leave_hive(hid, "ghost");
        assert!(result.is_err(), "Leaving hive when not a member should fail");
    }

    #[test]
    fn hive_multiple_members() {
        let mut store = CommStore::new();
        let hive = store
            .form_hive("big-hive", "coordinator", CollectiveDecisionMode::Consensus)
            .unwrap();
        let hid = hive.id;
        // Coordinator is already the first member
        assert_eq!(store.get_hive(hid).unwrap().constituents.len(), 1);

        store.join_hive(hid, "agent-a", HiveRole::Member).unwrap();
        store.join_hive(hid, "agent-b", HiveRole::Member).unwrap();
        store.join_hive(hid, "agent-c", HiveRole::Observer).unwrap();

        let hive = store.get_hive(hid).unwrap();
        assert_eq!(hive.constituents.len(), 4, "Should have coordinator + 3 members");
        // Verify roles
        assert_eq!(hive.constituents[0].role, HiveRole::Coordinator);
        assert_eq!(hive.constituents[1].role, HiveRole::Member);
        assert_eq!(hive.constituents[3].role, HiveRole::Observer);
    }

    // --- Grounding tests ---

    #[test]
    fn grounding_message_content() {
        let (mut store, ch_id) = new_store_with_channel();
        store
            .send_message(ch_id, "alice", "the deployment succeeded", MessageType::Text)
            .unwrap();
        // Ground a claim that references the sender
        let result = store.ground_claim("alice sent a message");
        assert_ne!(result.status, GroundingStatus::Ungrounded);
        assert!(
            result.evidence.iter().any(|e| e.evidence_type == "message"),
            "Should have message evidence"
        );
    }

    #[test]
    fn grounding_hive_name() {
        let mut store = CommStore::new();
        store
            .form_hive("project-alpha", "alice", CollectiveDecisionMode::Majority)
            .unwrap();
        let result = store.ground_claim("project-alpha hive exists");
        assert_eq!(result.status, GroundingStatus::Verified);
        assert!(
            result.evidence.iter().any(|e| e.evidence_type == "hive"),
            "Should have hive evidence"
        );
    }

    #[test]
    fn grounding_consent_evidence() {
        let mut store = CommStore::new();
        store
            .grant_consent("alice", "bob", ConsentScope::ReadMessages, None, None)
            .unwrap();
        let result = store.ground_claim("alice has granted consent");
        assert_ne!(result.status, GroundingStatus::Ungrounded);
        assert!(
            result.evidence.iter().any(|e| e.evidence_type == "consent"),
            "Should have consent evidence"
        );
    }

    // --- Communication log tests ---

    #[test]
    fn comm_log_with_affect() {
        let mut store = CommStore::new();
        let affect = AffectState {
            valence: 0.7,
            arousal: 0.5,
            dominance: 0.6,
            emotions: vec![Emotion::Joy, Emotion::Excitement],
            urgency: UrgencyLevel::High,
            meta_confidence: 0.9,
        };
        let entry = store.log_communication(
            "Great progress today!",
            "agent",
            Some("status-update".to_string()),
            None,
            Some(affect),
        );
        assert_eq!(entry.content, "Great progress today!");
        assert_eq!(entry.role, "agent");
        assert_eq!(entry.topic.as_deref(), Some("status-update"));
        assert!(entry.affect.is_some());
        let stored_affect = entry.affect.as_ref().unwrap();
        assert_eq!(stored_affect.valence, 0.7);
        assert_eq!(stored_affect.emotions.len(), 2);
    }

    #[test]
    fn comm_log_limit() {
        let mut store = CommStore::new();
        for i in 0..10 {
            store.log_communication(
                &format!("entry-{i}"),
                "user",
                None,
                None,
                None,
            );
        }
        assert_eq!(store.get_comm_log(None).len(), 10, "All 10 entries should be present");
        let last_3 = store.get_comm_log(Some(3));
        assert_eq!(last_3.len(), 3, "Should return last 3 entries");
        assert_eq!(last_3[0].content, "entry-7");
        assert_eq!(last_3[1].content, "entry-8");
        assert_eq!(last_3[2].content, "entry-9");
    }

    // --- Stats tests ---

    #[test]
    fn stats_comprehensive() {
        let mut store = CommStore::new();

        // Create channels
        let ch1 = store.create_channel("chan-1", ChannelType::Group, None).unwrap();
        let ch2 = store.create_channel("chan-2", ChannelType::Direct, None).unwrap();
        store.join_channel(ch1.id, "alice").unwrap();
        store.join_channel(ch1.id, "bob").unwrap();
        store.join_channel(ch2.id, "carol").unwrap();

        // Send messages
        store.send_message(ch1.id, "alice", "msg1", MessageType::Text).unwrap();
        store.send_message(ch1.id, "bob", "msg2", MessageType::Command).unwrap();
        store.send_message(ch2.id, "carol", "msg3", MessageType::Query).unwrap();

        // Add consent
        store.grant_consent("alice", "bob", ConsentScope::ReadMessages, None, None).unwrap();
        store.grant_consent("bob", "carol", ConsentScope::SendMessages, None, None).unwrap();

        // Add trust overrides
        store.set_trust_level("alice", CommTrustLevel::High).unwrap();

        // Form a hive
        store.form_hive("stats-hive", "alice", CollectiveDecisionMode::Majority).unwrap();

        // Schedule a temporal message
        store.schedule_message(ch1.id, "alice", "future", TemporalTarget::FutureRelative { delay_seconds: 100 }, None).unwrap();

        // Configure federation
        store.configure_federation(true, "local-zone", FederationPolicy::Allow).unwrap();
        store.add_federated_zone(FederatedZone {
            zone_id: "remote".to_string(),
            name: "Remote Zone".to_string(),
            endpoint: String::new(),
            policy: FederationPolicy::Selective,
            trust_level: CommTrustLevel::Basic,
        }).unwrap();

        // Add comm log entries
        store.log_communication("log1", "user", None, None, None);
        store.log_communication("log2", "agent", None, None, None);

        let stats = store.stats();
        assert_eq!(stats.channel_count, 2);
        assert_eq!(stats.message_count, 3);
        assert_eq!(stats.total_participants, 3); // alice + bob in ch1, carol in ch2
        assert_eq!(stats.consent_gate_count, 2);
        assert_eq!(stats.trust_override_count, 1);
        assert_eq!(stats.hive_count, 1);
        assert_eq!(stats.temporal_queue_count, 1);
        assert!(stats.federation_enabled);
        assert_eq!(stats.federated_zone_count, 1);
        assert_eq!(stats.comm_log_count, 2);
        assert_eq!(stats.dead_letter_count, 0);
        // Check messages_by_type
        assert_eq!(stats.messages_by_type.get("text"), Some(&1));
        assert_eq!(stats.messages_by_type.get("command"), Some(&1));
        assert_eq!(stats.messages_by_type.get("query"), Some(&1));
        // Check channels_by_state
        assert_eq!(stats.channels_by_state.get("active"), Some(&2));
        // Oldest and newest should be set
        assert!(stats.oldest_message.is_some());
        assert!(stats.newest_message.is_some());
    }

    // --- Save/Load round-trip tests ---

    #[test]
    fn save_load_consent_roundtrip() {
        let mut store = CommStore::new();
        store
            .grant_consent(
                "alice",
                "bob",
                ConsentScope::Federate,
                Some("federation partnership".to_string()),
                Some("2030-06-15T00:00:00Z".to_string()),
            )
            .unwrap();
        store
            .grant_consent("carol", "dave", ConsentScope::HiveParticipation, None, None)
            .unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("consent_roundtrip.acomm");
        store.save(&path).unwrap();

        let loaded = CommStore::load(&path).unwrap();
        assert_eq!(loaded.consent_gates.len(), 2);
        assert!(loaded.check_consent("alice", "bob", &ConsentScope::Federate));
        assert!(loaded.check_consent("carol", "dave", &ConsentScope::HiveParticipation));
        // Verify reason survived
        let alice_gate = loaded
            .consent_gates
            .iter()
            .find(|e| e.grantor == "alice" && e.grantee == "bob")
            .unwrap();
        assert_eq!(alice_gate.reason.as_deref(), Some("federation partnership"));
        assert_eq!(alice_gate.expires_at.as_deref(), Some("2030-06-15T00:00:00Z"));
    }

    #[test]
    fn save_load_trust_roundtrip() {
        let mut store = CommStore::new();
        store.set_trust_level("agent-a", CommTrustLevel::Absolute).unwrap();
        store.set_trust_level("agent-b", CommTrustLevel::None).unwrap();
        store.set_trust_level("agent-c", CommTrustLevel::Minimal).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trust_roundtrip.acomm");
        store.save(&path).unwrap();

        let loaded = CommStore::load(&path).unwrap();
        assert_eq!(loaded.list_trust_levels().len(), 3);
        assert_eq!(loaded.get_trust_level("agent-a"), CommTrustLevel::Absolute);
        assert_eq!(loaded.get_trust_level("agent-b"), CommTrustLevel::None);
        assert_eq!(loaded.get_trust_level("agent-c"), CommTrustLevel::Minimal);
        // Unknown agent should still return default
        assert_eq!(loaded.get_trust_level("unknown"), CommTrustLevel::Standard);
    }

    #[test]
    fn save_load_hive_roundtrip() {
        let mut store = CommStore::new();
        let hive = store
            .form_hive("persistent-hive", "coordinator", CollectiveDecisionMode::Unanimous)
            .unwrap();
        let hid = hive.id;
        store.join_hive(hid, "member-a", HiveRole::Member).unwrap();
        store.join_hive(hid, "observer-b", HiveRole::Observer).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hive_roundtrip.acomm");
        store.save(&path).unwrap();

        let loaded = CommStore::load(&path).unwrap();
        assert_eq!(loaded.hive_minds.len(), 1);
        let loaded_hive = loaded.get_hive(hid).unwrap();
        assert_eq!(loaded_hive.name, "persistent-hive");
        assert_eq!(loaded_hive.constituents.len(), 3);
        assert_eq!(loaded_hive.decision_mode, CollectiveDecisionMode::Unanimous);
        assert_eq!(loaded_hive.constituents[0].role, HiveRole::Coordinator);
        assert_eq!(loaded_hive.constituents[0].agent_id, "coordinator");
        assert_eq!(loaded_hive.constituents[1].agent_id, "member-a");
        assert_eq!(loaded_hive.constituents[2].agent_id, "observer-b");
    }

    #[test]
    fn save_load_temporal_roundtrip() {
        let mut store = CommStore::new();
        let ch = store.create_channel("temporal-ch", ChannelType::Group, None).unwrap();
        store
            .schedule_message(
                ch.id,
                "alice",
                "deliver later",
                TemporalTarget::FutureAbsolute {
                    deliver_at: "2030-01-01T00:00:00Z".to_string(),
                },
                Some(AffectState {
                    valence: 0.5,
                    arousal: 0.3,
                    ..Default::default()
                }),
            )
            .unwrap();
        store
            .schedule_message(ch.id, "bob", "deliver now", TemporalTarget::Immediate, None)
            .unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("temporal_roundtrip.acomm");
        store.save(&path).unwrap();

        let loaded = CommStore::load(&path).unwrap();
        assert_eq!(loaded.temporal_queue.len(), 2);
        // Both should be undelivered after load
        assert_eq!(loaded.list_scheduled().len(), 2);
        // First message should have affect
        let first = &loaded.temporal_queue[0];
        assert_eq!(first.sender, "alice");
        assert_eq!(first.content, "deliver later");
        assert!(first.affect.is_some());
        assert!(!first.delivered);
        // Second message
        let second = &loaded.temporal_queue[1];
        assert_eq!(second.sender, "bob");
        assert!(second.affect.is_none());
    }

    // --- New ChannelState variant tests ---

    #[test]
    fn channel_state_new_variants_display() {
        assert_eq!(ChannelState::Archived.to_string(), "archived");
        assert_eq!(ChannelState::SilentCommunion.to_string(), "silent_communion");
        assert_eq!(ChannelState::HiveMode.to_string(), "hive_mode");
        assert_eq!(ChannelState::PendingConsent.to_string(), "pending_consent");
    }

    #[test]
    fn channel_state_archived_blocks_send_allows_receive() {
        let mut store = CommStore::new();
        let ch = store.create_channel("arch-ch", ChannelType::Group, None).unwrap();
        let ch_id = ch.id;
        store.join_channel(ch_id, "alice").unwrap();
        // Set state to Archived manually
        store.channels.get_mut(&ch_id).unwrap().state = ChannelState::Archived;
        // Send should fail
        assert!(store.send_message(ch_id, "alice", "hello", MessageType::Text).is_err());
    }

    #[test]
    fn channel_state_pending_consent_blocks_both() {
        let mut store = CommStore::new();
        let ch = store.create_channel("pc-ch", ChannelType::Group, None).unwrap();
        let ch_id = ch.id;
        store.join_channel(ch_id, "alice").unwrap();
        store.channels.get_mut(&ch_id).unwrap().state = ChannelState::PendingConsent;
        assert!(store.send_message(ch_id, "alice", "hello", MessageType::Text).is_err());
    }

    // --- CommTimestamp tests ---

    #[test]
    fn comm_timestamp_increment_and_merge() {
        let mut ts = CommTimestamp::now("a");
        ts.increment("a");
        assert_eq!(ts.lamport, 1);
        assert_eq!(ts.vector_clock["a"], 1);

        let mut ts2 = CommTimestamp::now("b");
        ts2.increment("b");
        ts2.increment("b");

        ts.merge(&ts2, "a");
        assert_eq!(ts.lamport, 3); // max(1,2)+1
        assert_eq!(ts.vector_clock["a"], 2); // was 1, merge increments
        assert_eq!(ts.vector_clock["b"], 2);
    }

    #[test]
    fn comm_timestamp_happens_before_basic() {
        let mut a = CommTimestamp::now("x");
        a.increment("x");
        let mut b = a.clone();
        b.increment("x");
        assert!(a.happens_before(&b));
        assert!(!b.happens_before(&a));
        // Equal clocks: not happens-before
        let c = a.clone();
        assert!(!a.happens_before(&c));
    }

    // --- Audit log tests ---

    #[test]
    fn audit_log_and_retrieve() {
        let mut store = CommStore::new();
        store.log_audit(AuditEventType::ChannelCreated, "agent-1", "Created general", Some("1".to_string()));
        store.log_audit(AuditEventType::MessageSent, "agent-2", "Sent hello", None);
        store.log_audit(AuditEventType::ConsentGranted, "agent-1", "Granted read to agent-2", Some("consent-1".to_string()));

        let all = store.get_audit_log(None);
        assert_eq!(all.len(), 3);

        let last2 = store.get_audit_log(Some(2));
        assert_eq!(last2.len(), 2);
        assert_eq!(last2[0].description, "Sent hello");
        assert_eq!(last2[1].description, "Granted read to agent-2");
    }

    #[test]
    fn audit_log_in_stats() {
        let mut store = CommStore::new();
        store.log_audit(AuditEventType::AuthFailure, "attacker", "Bad credentials", None);
        store.log_audit(AuditEventType::RateLimitExceeded, "spammer", "Too many messages", None);
        let stats = store.stats();
        assert_eq!(stats.audit_log_count, 2);
    }

    // --- RateLimitConfig tests ---

    #[test]
    fn rate_limit_config_defaults_in_store() {
        let store = CommStore::new();
        assert_eq!(store.rate_limit_config.messages_per_minute, 60);
        assert_eq!(store.rate_limit_config.semantic_per_minute, 10);
        assert_eq!(store.rate_limit_config.affect_per_minute, 30);
        assert_eq!(store.rate_limit_config.hive_per_hour, 5);
        assert_eq!(store.rate_limit_config.federation_per_minute, 20);
    }

    // -----------------------------------------------------------------------
    // Consent enforcement tests
    // -----------------------------------------------------------------------

    #[test]
    fn consent_enforcement_blocks_semantic_without_consent() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("consent-test", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch.id, "sender-agent").unwrap();
        store.join_channel(ch.id, "receiver-agent").unwrap();

        // Attempt to send affect-enriched content without consent
        let result = store.send_message(
            ch.id,
            "sender-agent",
            "[affect: joy intensity=0.8] Great news!",
            MessageType::Text,
        );

        assert!(result.is_err(), "Affect-enriched message should be blocked without consent");
        match result.unwrap_err() {
            CommError::ConsentDenied { reason } => {
                assert!(
                    reason.contains("receiver-agent"),
                    "Error should mention the participant who hasn't granted consent"
                );
                assert!(
                    reason.contains("SendMessages"),
                    "Error should mention the required consent scope"
                );
            }
            other => panic!("Expected ConsentDenied, got: {:?}", other),
        }
    }

    #[test]
    fn consent_enforcement_allows_with_grant() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("consent-allow-test", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch.id, "sender-agent").unwrap();
        store.join_channel(ch.id, "receiver-agent").unwrap();

        // Grant SendMessages consent from receiver to sender
        store
            .grant_consent(
                "receiver-agent",
                "sender-agent",
                ConsentScope::SendMessages,
                Some("Allow affect messages".to_string()),
                None,
            )
            .unwrap();

        // Now sending affect-enriched content should succeed
        let result = store.send_message(
            ch.id,
            "sender-agent",
            "[affect: joy intensity=0.8] Great news!",
            MessageType::Text,
        );

        assert!(
            result.is_ok(),
            "Affect-enriched message should be allowed after consent grant"
        );

        // Plain text should also work (no consent needed)
        let result2 = store.send_message(
            ch.id,
            "sender-agent",
            "Plain text message",
            MessageType::Text,
        );
        assert!(result2.is_ok(), "Plain text should always be allowed");
    }

    // -----------------------------------------------------------------------
    // Rate limiting tests
    // -----------------------------------------------------------------------

    #[test]
    fn rate_limiting_blocks_after_threshold() {
        let mut store = CommStore::new();
        // Set a very low rate limit for testing
        store.rate_limit_config.messages_per_minute = 3;

        let ch = store
            .create_channel("rate-test", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch.id, "fast-sender").unwrap();

        // Send messages up to the limit
        for i in 0..3 {
            let result = store.send_message(
                ch.id,
                "fast-sender",
                &format!("Message {}", i),
                MessageType::Text,
            );
            assert!(result.is_ok(), "Message {} should succeed (within limit)", i);
        }

        // The 4th message should be rate-limited
        let result = store.send_message(
            ch.id,
            "fast-sender",
            "One too many",
            MessageType::Text,
        );
        assert!(result.is_err(), "4th message should be rate-limited");
        match result.unwrap_err() {
            CommError::RateLimitExceeded { limit } => {
                assert!(
                    limit.contains("fast-sender"),
                    "Error should mention the sender"
                );
                assert!(
                    limit.contains("3"),
                    "Error should mention the limit threshold"
                );
            }
            other => panic!("Expected RateLimitExceeded, got: {:?}", other),
        }

        // A different sender should NOT be rate-limited
        store.join_channel(ch.id, "other-sender").unwrap();
        let result = store.send_message(
            ch.id,
            "other-sender",
            "I can still send",
            MessageType::Text,
        );
        assert!(
            result.is_ok(),
            "Different sender should have independent rate limit"
        );
    }

    #[test]
    fn rate_limiting_resets_after_window() {
        let mut store = CommStore::new();
        store.rate_limit_config.messages_per_minute = 2;

        let ch = store
            .create_channel("rate-reset-test", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch.id, "resetting-sender").unwrap();

        // Send up to the limit
        store
            .send_message(ch.id, "resetting-sender", "msg1", MessageType::Text)
            .unwrap();
        store
            .send_message(ch.id, "resetting-sender", "msg2", MessageType::Text)
            .unwrap();

        // Verify rate limit is hit
        let blocked = store.send_message(
            ch.id,
            "resetting-sender",
            "blocked",
            MessageType::Text,
        );
        assert!(blocked.is_err(), "Should be rate-limited");

        // Simulate window reset by manipulating the tracker's last_minute_reset
        // to be more than 60 seconds ago.
        if let Some(tracker) = store.rate_trackers.get_mut("resetting-sender") {
            tracker.last_minute_reset = tracker.last_minute_reset.saturating_sub(61);
        }

        // Now sending should succeed (window reset)
        let result = store.send_message(
            ch.id,
            "resetting-sender",
            "after reset",
            MessageType::Text,
        );
        assert!(
            result.is_ok(),
            "Message should succeed after rate limit window resets"
        );
    }

    // -----------------------------------------------------------------------
    // Audit log tests
    // -----------------------------------------------------------------------

    #[test]
    fn audit_log_records_operations() {
        let mut store = CommStore::new();
        assert!(store.audit_log.is_empty(), "Audit log should start empty");

        // 1) create_channel should generate ChannelCreated audit
        let ch = store
            .create_channel("audit-test", ChannelType::Group, None)
            .unwrap();
        let channel_created_count = store
            .audit_log
            .iter()
            .filter(|e| e.event_type == AuditEventType::ChannelCreated)
            .count();
        assert_eq!(channel_created_count, 1, "Should have one ChannelCreated audit entry");

        // 2) send_message should generate MessageSent audit
        store.join_channel(ch.id, "audit-agent").unwrap();
        store
            .send_message(ch.id, "audit-agent", "hello", MessageType::Text)
            .unwrap();
        let msg_sent_count = store
            .audit_log
            .iter()
            .filter(|e| e.event_type == AuditEventType::MessageSent)
            .count();
        assert_eq!(msg_sent_count, 1, "Should have one MessageSent audit entry");

        // 3) grant_consent should generate ConsentGranted audit
        store
            .grant_consent(
                "grantor-a",
                "grantee-b",
                ConsentScope::ReadMessages,
                None,
                None,
            )
            .unwrap();
        let consent_granted_count = store
            .audit_log
            .iter()
            .filter(|e| e.event_type == AuditEventType::ConsentGranted)
            .count();
        assert_eq!(consent_granted_count, 1, "Should have one ConsentGranted audit entry");

        // 4) revoke_consent should generate ConsentRevoked audit
        let _ = store.revoke_consent("grantor-a", "grantee-b", &ConsentScope::ReadMessages);
        let consent_revoked_count = store
            .audit_log
            .iter()
            .filter(|e| e.event_type == AuditEventType::ConsentRevoked)
            .count();
        assert_eq!(consent_revoked_count, 1, "Should have one ConsentRevoked audit entry");

        // 5) close_channel should generate ChannelClosed audit
        store.close_channel(ch.id).unwrap();
        let channel_closed_count = store
            .audit_log
            .iter()
            .filter(|e| e.event_type == AuditEventType::ChannelClosed)
            .count();
        assert_eq!(channel_closed_count, 1, "Should have one ChannelClosed audit entry");

        // Verify total audit entries accumulated
        assert!(
            store.audit_log.len() >= 5,
            "Audit log should have at least 5 entries, got {}",
            store.audit_log.len()
        );
    }

    // -----------------------------------------------------------------------
    // Signature verification tests
    // -----------------------------------------------------------------------

    #[test]
    fn signature_verification_detects_tampering() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("sig-test", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch.id, "agent-sig").unwrap();

        let msg = store
            .send_message(ch.id, "agent-sig", "Original content", MessageType::Text)
            .unwrap();
        let msg_id = msg.id;

        // Verify the untampered message passes signature check
        assert!(
            store.verify_message_signature(msg_id),
            "Untampered message should pass signature verification"
        );

        // Tamper with the message content directly
        if let Some(message) = store.messages.get_mut(&msg_id) {
            message.content = "Tampered content".to_string();
        }

        // Now signature verification should fail
        let audit_len_before = store.audit_log.len();
        let result = store.verify_message_signature(msg_id);
        assert!(
            !result,
            "Tampered message should fail signature verification"
        );

        // Verify a SignatureWarning audit entry was logged
        let sig_warnings = store
            .audit_log
            .iter()
            .skip(audit_len_before)
            .filter(|e| e.event_type == AuditEventType::SignatureWarning)
            .count();
        assert_eq!(
            sig_warnings, 1,
            "Should have logged a SignatureWarning audit entry for the tampered message"
        );
    }

    // -- File locking tests --

    #[test]
    fn file_locking_exclusive_blocks_second_try() {
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("test.acomm");

        // Acquire an exclusive lock.
        let lock1 = CommFileLock::acquire(&data_path).unwrap();

        // A non-blocking try_acquire on the same path should fail.
        let result = CommFileLock::try_acquire(&data_path);
        assert!(
            result.is_err(),
            "try_acquire should fail while exclusive lock is held"
        );

        // Clean up.
        lock1.release().unwrap();
    }

    #[test]
    fn file_locking_release_allows_reacquire() {
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("test.acomm");

        // Acquire, then release.
        let lock1 = CommFileLock::acquire(&data_path).unwrap();
        lock1.release().unwrap();

        // Now acquiring again should succeed.
        let lock2 = CommFileLock::acquire(&data_path).unwrap();
        lock2.release().unwrap();
    }

    #[test]
    fn file_locking_stale_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("test.acomm");
        let lock_path = data_path.with_extension("acomm.lock");

        // Create a fake stale lock file.
        std::fs::File::create(&lock_path).unwrap();

        // Backdate the lock file's mtime to 120 seconds in the past.
        let old_time = FileTime::from_unix_time(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
                - 120,
            0,
        );
        filetime::set_file_mtime(&lock_path, old_time).unwrap();

        // A max_age of 60 s means anything older than 60 s is stale.
        let recovered = CommFileLock::recover_stale(&data_path, 60).unwrap();
        assert!(recovered, "Should have recovered stale lock file");
        assert!(
            !lock_path.exists(),
            "Stale lock file should have been removed"
        );
    }

    #[test]
    fn file_locking_stale_recovery_not_stale() {
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("test.acomm");
        let lock_path = data_path.with_extension("acomm.lock");

        // Create a fresh lock file.
        std::fs::File::create(&lock_path).unwrap();

        // With a large max_age, the lock should not be considered stale.
        let recovered = CommFileLock::recover_stale(&data_path, 3600).unwrap();
        assert!(!recovered, "Fresh lock should not be recovered");
        assert!(lock_path.exists(), "Fresh lock file should still exist");
    }

    #[test]
    fn file_locking_no_lock_file_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("test.acomm");

        // No lock file exists — recovery should return false.
        let recovered = CommFileLock::recover_stale(&data_path, 0).unwrap();
        assert!(!recovered, "No lock file means nothing to recover");
    }

    #[test]
    fn file_locking_save_load_with_locks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("locked.acomm");

        // Build a store with some data.
        let mut store = CommStore::new();
        store
            .create_channel("lock-test", ChannelType::Group, None)
            .unwrap();
        store
            .send_message(1, "agent-a", "hello from lock test", MessageType::Text)
            .unwrap();

        // Save (internally acquires exclusive lock).
        store.save(&path).unwrap();

        // Load (internally acquires shared lock).
        let loaded = CommStore::load(&path).unwrap();
        assert_eq!(loaded.channels.len(), 1);
        assert_eq!(loaded.messages.len(), 1);

        let msg = loaded.messages.values().next().unwrap();
        assert_eq!(msg.content, "hello from lock test");
    }

    #[test]
    fn file_locking_shared_allows_multiple_readers() {
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("test.acomm");

        // Multiple shared locks should coexist.
        let lock1 = CommFileLock::acquire_shared(&data_path).unwrap();
        let lock2 = CommFileLock::acquire_shared(&data_path).unwrap();

        // Both held simultaneously — no panic, no error.
        lock1.release().unwrap();
        lock2.release().unwrap();
    }

    #[test]
    fn file_locking_drop_releases_lock() {
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("test.acomm");

        // Acquire and then drop without explicit release.
        {
            let _lock = CommFileLock::acquire(&data_path).unwrap();
            // _lock dropped here.
        }

        // Should be able to re-acquire after drop.
        let lock2 = CommFileLock::try_acquire(&data_path).unwrap();
        lock2.release().unwrap();
    }


    // -- Affect Contagion / Echo Chain / Summarization tests --

    #[test]
    fn affect_contagion_basic() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("affect-test", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch.id, "alice").unwrap();
        store.join_channel(ch.id, "bob").unwrap();

        // Send a message with affect metadata
        let msg = store
            .send_message(ch.id, "alice", "I am so happy!", MessageType::Text)
            .unwrap();
        // Manually set affect metadata on the message
        store
            .messages
            .get_mut(&msg.id)
            .unwrap()
            .metadata
            .insert("valence".to_string(), "0.9".to_string());
        store
            .messages
            .get_mut(&msg.id)
            .unwrap()
            .metadata
            .insert("arousal".to_string(), "0.7".to_string());
        store
            .messages
            .get_mut(&msg.id)
            .unwrap()
            .metadata
            .insert("dominance".to_string(), "0.6".to_string());

        store.set_affect_resistance(0.0);
        let results = store.process_affect_contagion(ch.id);

        // Bob should be affected
        assert!(!results.is_empty());
        let bob_result = results.iter().find(|(name, _, _, _)| name == "bob");
        assert!(bob_result.is_some());
        let (_, valence, arousal, _) = bob_result.unwrap();
        assert!(*valence > 0.0, "Bob's valence should be positive");
        assert!(*arousal > 0.0, "Bob's arousal should be positive");
    }

    #[test]
    fn affect_contagion_empty_channel() {
        let mut store = CommStore::new();
        // Non-existent channel
        let results = store.process_affect_contagion(999);
        assert!(results.is_empty());
    }

    #[test]
    fn affect_history_empty() {
        let store = CommStore::new();
        let history = store.get_affect_history("nonexistent-agent");
        assert_eq!(history.agent, "nonexistent-agent");
        assert!(history.states.is_empty());
    }

    #[test]
    fn affect_history_with_state() {
        let mut store = CommStore::new();
        store.affect_states.insert(
            "agent-x".to_string(),
            AffectState {
                valence: 0.5,
                arousal: 0.3,
                dominance: 0.7,
                ..AffectState::default()
            },
        );
        let history = store.get_affect_history("agent-x");
        assert_eq!(history.agent, "agent-x");
        assert!(!history.states.is_empty());
        let last = history.states.last().unwrap();
        assert_eq!(last.source, "current");
        assert!((last.valence - 0.5).abs() < 0.01);
    }

    #[test]
    fn affect_decay_reduces_state() {
        let mut store = CommStore::new();
        store.affect_states.insert(
            "agent-y".to_string(),
            AffectState {
                valence: 0.8,
                arousal: 0.6,
                dominance: 0.5,
                ..AffectState::default()
            },
        );
        store.apply_affect_decay(0.5);
        let state = store.affect_states.get("agent-y").unwrap();
        assert!((state.valence - 0.4).abs() < 0.01, "valence should halve");
        assert!((state.arousal - 0.3).abs() < 0.01, "arousal should halve");
    }

    #[test]
    fn forward_message_basic() {
        let mut store = CommStore::new();
        let ch1 = store
            .create_channel("source-chan", ChannelType::Group, None)
            .unwrap();
        let ch2 = store
            .create_channel("target-chan", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch1.id, "alice").unwrap();
        store.join_channel(ch2.id, "bob").unwrap();

        let msg = store
            .send_message(ch1.id, "alice", "Hello world", MessageType::Text)
            .unwrap();

        let fwd_id = store
            .forward_message(msg.id, ch2.id, "bob")
            .unwrap();

        let fwd_msg = store.messages.get(&fwd_id).unwrap();
        assert!(fwd_msg.content.starts_with("[Forwarded]"));
        assert_eq!(fwd_msg.channel_id, ch2.id);
        assert_eq!(
            fwd_msg.metadata.get("forwarded_from").unwrap(),
            &msg.id.to_string()
        );
        assert_eq!(fwd_msg.metadata.get("echo_depth").unwrap(), "1");
    }

    #[test]
    fn forward_message_not_found() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("target", ChannelType::Group, None)
            .unwrap();
        let result = store.forward_message(999, ch.id, "bob");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn forward_message_target_not_found() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("source", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch.id, "alice").unwrap();
        let msg = store
            .send_message(ch.id, "alice", "test", MessageType::Text)
            .unwrap();
        let result = store.forward_message(msg.id, 999, "bob");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn echo_chain_and_depth() {
        let mut store = CommStore::new();
        let ch1 = store
            .create_channel("ch1", ChannelType::Group, None)
            .unwrap();
        let ch2 = store
            .create_channel("ch2", ChannelType::Group, None)
            .unwrap();
        let ch3 = store
            .create_channel("ch3", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch1.id, "alice").unwrap();
        store.join_channel(ch2.id, "bob").unwrap();
        store.join_channel(ch3.id, "charlie").unwrap();

        let orig = store
            .send_message(ch1.id, "alice", "Original", MessageType::Text)
            .unwrap();
        assert_eq!(store.get_echo_depth(orig.id), 0);

        let fwd1 = store.forward_message(orig.id, ch2.id, "bob").unwrap();
        assert_eq!(store.get_echo_depth(fwd1), 1);

        let fwd2 = store.forward_message(fwd1, ch3.id, "charlie").unwrap();
        assert_eq!(store.get_echo_depth(fwd2), 2);

        let chain = store.query_echo_chain(fwd2);
        assert!(chain.len() >= 3);
        assert_eq!(chain[0].message_id, orig.id);
        assert_eq!(chain[0].depth, 0);
    }

    #[test]
    fn summarize_conversation_basic() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("summary-test", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch.id, "alice").unwrap();
        store.join_channel(ch.id, "bob").unwrap();

        store
            .send_message(ch.id, "alice", "Hello!", MessageType::Text)
            .unwrap();
        store
            .send_message(ch.id, "bob", "Hi there!", MessageType::Text)
            .unwrap();
        store
            .send_message(ch.id, "alice", "How are you?", MessageType::Text)
            .unwrap();

        let summary = store.summarize_conversation(ch.id).unwrap();
        assert_eq!(summary.channel_id, ch.id);
        assert_eq!(summary.channel_name, "summary-test");
        assert_eq!(summary.message_count, 3);
        assert_eq!(summary.participant_count, 2);
        assert!(summary.avg_message_length > 0.0);
        assert!(!summary.has_affect_data);
    }

    #[test]
    fn summarize_conversation_not_found() {
        let store = CommStore::new();
        let result = store.summarize_conversation(999);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // CommId and Rich Content integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_assign_comm_ids_fills_missing() {
        let mut store = CommStore::new();
        let ch_id = store.create_channel("test", ChannelType::Group, None).unwrap().id;
        store.send_message(ch_id, "alice", "hello", MessageType::Text).unwrap();
        store.send_message(ch_id, "bob", "world", MessageType::Text).unwrap();

        // Before assign, all comm_ids should be None
        for msg in store.messages.values() {
            assert!(msg.comm_id.is_none());
        }
        for chan in store.channels.values() {
            assert!(chan.comm_id.is_none());
        }

        store.assign_comm_ids();

        // After assign, all should be Some
        for msg in store.messages.values() {
            assert!(msg.comm_id.is_some());
        }
        for chan in store.channels.values() {
            assert!(chan.comm_id.is_some());
        }
    }

    #[test]
    fn test_assign_comm_ids_idempotent() {
        let mut store = CommStore::new();
        let ch_id = store.create_channel("test", ChannelType::Group, None).unwrap().id;
        store.send_message(ch_id, "alice", "hello", MessageType::Text).unwrap();

        store.assign_comm_ids();
        let first_id = store.messages.values().next().unwrap().comm_id;

        store.assign_comm_ids();
        let second_id = store.messages.values().next().unwrap().comm_id;

        assert_eq!(first_id, second_id, "assign_comm_ids should be idempotent");
    }

    #[test]
    fn test_get_message_by_comm_id() {
        let mut store = CommStore::new();
        let ch_id = store.create_channel("test", ChannelType::Group, None).unwrap().id;
        let msg = store.send_message(ch_id, "alice", "hello", MessageType::Text).unwrap();

        store.assign_comm_ids();

        let comm_id = store.messages.get(&msg.id).unwrap().comm_id.unwrap();
        let found = store.get_message_by_comm_id(&comm_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, msg.id);
    }

    #[test]
    fn test_get_channel_by_comm_id() {
        let mut store = CommStore::new();
        let ch = store.create_channel("test", ChannelType::Group, None).unwrap().clone();

        store.assign_comm_ids();

        let comm_id = store.channels.get(&ch.id).unwrap().comm_id.unwrap();
        let found = store.get_channel_by_comm_id(&comm_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, ch.id);
    }

    #[test]
    fn test_send_rich_message() {
        let mut store = CommStore::new();
        let ch_id = store.create_channel("test", ChannelType::Group, None).unwrap().id;
        store.join_channel(ch_id, "alice").unwrap();

        let content = MessageContent::Semantic(SemanticContent {
            text: "The weather is nice".into(),
            fragments: vec!["weather".into(), "nice".into()],
            context: Some("small talk".into()),
            perspective: None,
        });

        let msg = store.send_rich_message(ch_id, "alice", content, MessageType::Text).unwrap();
        assert!(msg.rich_content_json.is_some());
        assert_eq!(msg.content, "The weather is nice");

        // Parse it back
        let rich = store.get_rich_content(msg.id).unwrap();
        assert!(rich.is_some());
        let rc = rich.unwrap();
        assert!(rc.is_rich());
        assert_eq!(rc.as_text(), "The weather is nice");
    }

    #[test]
    fn test_get_rich_content_none_for_plain_message() {
        let mut store = CommStore::new();
        let ch_id = store.create_channel("test", ChannelType::Group, None).unwrap().id;
        let msg = store.send_message(ch_id, "alice", "plain", MessageType::Text).unwrap();

        let rich = store.get_rich_content(msg.id).unwrap();
        assert!(rich.is_none());
    }

    #[test]
    fn test_get_rich_content_message_not_found() {
        let store = CommStore::new();
        let result = store.get_rich_content(9999);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_backward_compat_without_new_fields() {
        // Simulate deserializing an old message without the new fields
        let json = r#"{
            "id": 1,
            "channel_id": 1,
            "sender": "alice",
            "recipient": null,
            "content": "hello",
            "message_type": "Text",
            "timestamp": "2026-01-01T00:00:00Z",
            "metadata": {},
            "signature": null,
            "acknowledged_by": [],
            "status": "Sent",
            "priority": "Normal",
            "reply_to": null,
            "correlation_id": null,
            "thread_id": null,
            "comm_timestamp": {"wall_clock": "2026-01-01T00:00:00Z", "lamport": 0, "agent_id": "alice", "vector_clock": {}}
        }"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert!(msg.rich_content_json.is_none());
        assert!(msg.comm_id.is_none());
        assert_eq!(msg.content, "hello");
    }

    // =====================================================================
    // Trust enforcement tests
    // =====================================================================

    #[test]
    fn test_trust_enforcement_send_message_blocked() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("secure", ChannelType::Group, None)
            .unwrap();
        let ch_id = ch.id;

        // Set channel to require High trust
        let mut config = store.get_channel(ch_id).unwrap().config.clone();
        config.min_trust_level = Some(CommTrustLevel::High);
        store.set_channel_config(ch_id, config).unwrap();

        // Set alice's trust level to Basic (below High)
        store.set_trust_level("alice", CommTrustLevel::Basic).unwrap();
        store.join_channel(ch_id, "alice").unwrap_err(); // also blocked at join

        // Set to High so she can join, then lower and try to send
        store.set_trust_level("alice", CommTrustLevel::High).unwrap();
        store.join_channel(ch_id, "alice").unwrap();

        // Lower trust and try to send
        store.set_trust_level("alice", CommTrustLevel::Basic).unwrap();
        let result = store.send_message(ch_id, "alice", "hello", MessageType::Text);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Trust level insufficient"),
            "Expected trust error, got: {}",
            err
        );
    }

    #[test]
    fn test_trust_enforcement_send_message_allowed() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("secure", ChannelType::Group, None)
            .unwrap();
        let ch_id = ch.id;

        // Set channel to require Basic trust
        let mut config = store.get_channel(ch_id).unwrap().config.clone();
        config.min_trust_level = Some(CommTrustLevel::Basic);
        store.set_channel_config(ch_id, config).unwrap();

        // Set alice's trust to Standard (above Basic)
        store.set_trust_level("alice", CommTrustLevel::Standard).unwrap();
        store.join_channel(ch_id, "alice").unwrap();
        let msg = store.send_message(ch_id, "alice", "hello", MessageType::Text);
        assert!(msg.is_ok());
    }

    #[test]
    fn test_trust_enforcement_no_min_trust_allows_all() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("open", ChannelType::Group, None)
            .unwrap();
        let ch_id = ch.id;
        // No min_trust_level set (None) — anyone can send
        store.set_trust_level("alice", CommTrustLevel::None).unwrap();
        store.join_channel(ch_id, "alice").unwrap();
        let msg = store.send_message(ch_id, "alice", "hello", MessageType::Text);
        assert!(msg.is_ok());
    }

    #[test]
    fn test_trust_enforcement_broadcast_blocked() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("secure-bc", ChannelType::Broadcast, None)
            .unwrap();
        let ch_id = ch.id;

        // Require Full trust
        let mut config = store.get_channel(ch_id).unwrap().config.clone();
        config.min_trust_level = Some(CommTrustLevel::Full);
        store.set_channel_config(ch_id, config).unwrap();

        // Alice has Standard trust — too low
        store.set_trust_level("alice", CommTrustLevel::Standard).unwrap();
        let result = store.broadcast(ch_id, "alice", "hello everyone");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Trust level insufficient"));
    }

    #[test]
    fn test_trust_enforcement_join_channel_blocked() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("exclusive", ChannelType::Group, None)
            .unwrap();
        let ch_id = ch.id;

        // Require High trust to join
        let mut config = store.get_channel(ch_id).unwrap().config.clone();
        config.min_trust_level = Some(CommTrustLevel::High);
        store.set_channel_config(ch_id, config).unwrap();

        store.set_trust_level("bob", CommTrustLevel::Basic).unwrap();
        let result = store.join_channel(ch_id, "bob");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Trust level insufficient"));
    }

    #[test]
    fn test_trust_enforcement_publish_blocked() {
        let mut store = CommStore::new();
        // Pre-create a pubsub channel with trust requirement
        let ch = store
            .create_channel("news-feed", ChannelType::PubSub, None)
            .unwrap();
        let ch_id = ch.id;

        let mut config = store.get_channel(ch_id).unwrap().config.clone();
        config.min_trust_level = Some(CommTrustLevel::High);
        store.set_channel_config(ch_id, config).unwrap();

        store.set_trust_level("low-trust-pub", CommTrustLevel::Minimal).unwrap();

        // Subscribe someone
        store.subscribe("news-feed", "subscriber1").unwrap();

        let result = store.publish("news-feed", "low-trust-pub", "breaking news");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Trust level insufficient"));
    }

    // =====================================================================
    // Consent enforcement expansion tests
    // =====================================================================

    #[test]
    fn test_consent_join_channel_open_by_default() {
        // When no JoinChannels consent gates exist, joining should succeed
        let (mut store, ch_id) = new_store_with_channel();
        let result = store.join_channel(ch_id, "alice");
        assert!(result.is_ok());
    }

    #[test]
    fn test_consent_join_channel_blocked() {
        let (mut store, ch_id) = new_store_with_channel();
        // Create a JoinChannels consent gate that grants to bob but not alice
        store.grant_consent(
            "system", "bob", ConsentScope::JoinChannels,
            Some("approved".to_string()), None,
        ).unwrap();

        // Now JoinChannels scope has at least one gate, so alice (no grant) is blocked
        let result = store.join_channel(ch_id, "alice");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Consent"));

        // bob should be allowed
        let result = store.join_channel(ch_id, "bob");
        assert!(result.is_ok());
    }

    #[test]
    fn test_consent_schedule_message_open_by_default() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("temporal-ch", ChannelType::Temporal, None)
            .unwrap();
        // No ScheduleMessages consent gates — should work
        let result = store.schedule_message(
            ch.id, "alice", "future msg", TemporalTarget::Immediate, None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_consent_schedule_message_blocked() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("temporal-ch", ChannelType::Temporal, None)
            .unwrap();
        // Create a ScheduleMessages gate for bob (alice has no grant)
        store.grant_consent(
            "system", "bob", ConsentScope::ScheduleMessages,
            Some("allowed".to_string()), None,
        ).unwrap();

        let result = store.schedule_message(
            ch.id, "alice", "blocked msg", TemporalTarget::Immediate, None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Consent"));
    }

    #[test]
    fn test_consent_form_hive_open_by_default() {
        let mut store = CommStore::new();
        let result = store.form_hive(
            "test-hive", "coordinator",
            CollectiveDecisionMode::Consensus,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_consent_form_hive_blocked() {
        let mut store = CommStore::new();
        // Create a HiveParticipation gate for someone else
        store.grant_consent(
            "system", "other-agent", ConsentScope::HiveParticipation,
            Some("approved".to_string()), None,
        ).unwrap();

        // coordinator has no grant, so should be blocked
        let result = store.form_hive(
            "test-hive", "coordinator",
            CollectiveDecisionMode::Consensus,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Consent"));
    }

    #[test]
    fn test_consent_join_hive_blocked() {
        let mut store = CommStore::new();
        // Grant coordinator consent so they can form
        store.grant_consent(
            "system", "coordinator", ConsentScope::HiveParticipation,
            Some("approved".to_string()), None,
        ).unwrap();
        let hive = store.form_hive(
            "test-hive", "coordinator",
            CollectiveDecisionMode::Consensus,
        ).unwrap();
        let hive_id = hive.id;

        // joiner has no HiveParticipation grant
        let result = store.join_hive(hive_id, "joiner", HiveRole::Member);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Consent"));

        // Grant joiner consent
        store.grant_consent(
            "system", "joiner", ConsentScope::HiveParticipation,
            Some("approved".to_string()), None,
        ).unwrap();
        let result = store.join_hive(hive_id, "joiner", HiveRole::Member);
        assert!(result.is_ok());
    }

    #[test]
    fn test_consent_configure_federation_open_by_default() {
        let mut store = CommStore::new();
        let result = store.configure_federation(
            true, "zone-a", FederationPolicy::Allow,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_consent_configure_federation_blocked() {
        let mut store = CommStore::new();
        // Create a Federate consent gate for another agent
        store.grant_consent(
            "admin", "other-system", ConsentScope::Federate,
            Some("allowed".to_string()), None,
        ).unwrap();

        // "system" has no Federate grant
        let result = store.configure_federation(
            true, "zone-a", FederationPolicy::Allow,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Consent"));
    }

    // =====================================================================
    // Vector clock tests
    // =====================================================================

    #[test]
    fn test_vector_clock_increment() {
        let mut ts = CommTimestamp::now("agent-a");
        assert_eq!(ts.lamport, 0);
        assert_eq!(*ts.vector_clock.get("agent-a").unwrap(), 0);

        ts.increment("agent-a");
        assert_eq!(ts.lamport, 1);
        assert_eq!(*ts.vector_clock.get("agent-a").unwrap(), 1);

        ts.increment("agent-a");
        assert_eq!(ts.lamport, 2);
        assert_eq!(*ts.vector_clock.get("agent-a").unwrap(), 2);
    }

    #[test]
    fn test_vector_clock_merge() {
        let mut ts_a = CommTimestamp::now("agent-a");
        ts_a.increment("agent-a"); // lamport=1, vc={a:1}
        ts_a.increment("agent-a"); // lamport=2, vc={a:2}

        let mut ts_b = CommTimestamp::now("agent-b");
        ts_b.increment("agent-b"); // lamport=1, vc={b:1}

        // Merge b into a
        ts_a.merge(&ts_b, "agent-a");
        // lamport should be max(2,1)+1 = 3
        assert_eq!(ts_a.lamport, 3);
        // vector_clock should have a:3 (incremented on merge), b:1
        assert_eq!(*ts_a.vector_clock.get("agent-a").unwrap(), 3);
        assert_eq!(*ts_a.vector_clock.get("agent-b").unwrap(), 1);
    }

    #[test]
    fn test_vector_clock_happens_before() {
        let mut ts_a = CommTimestamp::now("agent-a");
        ts_a.increment("agent-a"); // vc={a:1}

        let mut ts_b = ts_a.clone();
        ts_b.increment("agent-a"); // vc={a:2}

        // ts_a should happen-before ts_b
        assert!(ts_a.happens_before(&ts_b));
        // ts_b should NOT happen-before ts_a
        assert!(!ts_b.happens_before(&ts_a));
    }

    #[test]
    fn test_vector_clock_concurrent() {
        let mut ts_a = CommTimestamp::now("agent-a");
        ts_a.increment("agent-a"); // vc={a:1}

        let mut ts_b = CommTimestamp::now("agent-b");
        ts_b.increment("agent-b"); // vc={b:1}

        // Neither happens before the other (concurrent)
        assert!(!ts_a.happens_before(&ts_b));
        assert!(!ts_b.happens_before(&ts_a));
    }

    #[test]
    fn test_vector_clock_populated_in_send_message() {
        let (mut store, ch_id) = new_store_with_channel();
        store.join_channel(ch_id, "alice").unwrap();

        let msg = store
            .send_message(ch_id, "alice", "hello", MessageType::Text)
            .unwrap();

        // The vector clock should have alice's entry set to the lamport counter
        assert!(msg.comm_timestamp.lamport > 0);
        assert_eq!(
            *msg.comm_timestamp.vector_clock.get("alice").unwrap(),
            msg.comm_timestamp.lamport,
        );
    }

    #[test]
    fn test_vector_clock_populated_in_broadcast() {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("bc-chan", ChannelType::Broadcast, None)
            .unwrap();
        let ch_id = ch.id;
        store.join_channel(ch_id, "alice").unwrap();
        store.join_channel(ch_id, "bob").unwrap();

        let msgs = store.broadcast(ch_id, "alice", "broadcast msg").unwrap();
        assert!(!msgs.is_empty());
        for m in &msgs {
            assert!(m.comm_timestamp.lamport > 0);
            assert_eq!(
                *m.comm_timestamp.vector_clock.get("alice").unwrap(),
                m.comm_timestamp.lamport,
            );
        }
    }

    #[test]
    fn test_receive_messages_merges_lamport() {
        let (mut store, ch_id) = new_store_with_channel();
        store.join_channel(ch_id, "alice").unwrap();
        store.join_channel(ch_id, "bob").unwrap();

        // Send two messages
        store.send_message(ch_id, "alice", "msg1", MessageType::Text).unwrap();
        store.send_message(ch_id, "alice", "msg2", MessageType::Text).unwrap();
        let lamport_after_send = store.lamport_counter;

        // Receive them — should merge lamport (though they're from the same store,
        // the mechanism works)
        let msgs = store.receive_messages(ch_id, None, None).unwrap();
        assert_eq!(msgs.len(), 2);
        // lamport_counter should be at least as high as after sending
        assert!(store.lamport_counter >= lamport_after_send);
    }

    #[test]
    fn test_channel_config_backward_compat_no_min_trust() {
        // Ensure ChannelConfig without min_trust_level deserializes correctly
        let json = r#"{
            "max_participants": 10,
            "ttl_seconds": 0,
            "persistence": true,
            "encryption_required": false
        }"#;
        let config: ChannelConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.min_trust_level, None);
    }

    // -----------------------------------------------------------------------
    // Agent registry tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_agent_registry_basic() {
        let mut store = CommStore::new();
        let agent = CommunicatingAgent {
            agent_id: "agent-1".to_string(),
            availability: Availability::Available,
            ..Default::default()
        };
        store.register_agent(agent).unwrap();
        assert!(store.get_agent("agent-1").is_some());
        assert_eq!(store.list_agents().len(), 1);
    }

    #[test]
    fn test_agent_availability_update() {
        let mut store = CommStore::new();
        let agent = CommunicatingAgent {
            agent_id: "agent-1".to_string(),
            availability: Availability::Available,
            ..Default::default()
        };
        store.register_agent(agent).unwrap();
        store
            .update_agent_availability("agent-1", Availability::Busy)
            .unwrap();
        assert_eq!(
            store.get_agent("agent-1").unwrap().availability,
            Availability::Busy
        );
    }

    #[test]
    fn test_agent_unregister() {
        let mut store = CommStore::new();
        let agent = CommunicatingAgent {
            agent_id: "agent-1".to_string(),
            ..Default::default()
        };
        store.register_agent(agent).unwrap();
        store.unregister_agent("agent-1").unwrap();
        assert!(store.get_agent("agent-1").is_none());
        assert!(store.unregister_agent("agent-1").is_err());
    }

    #[test]
    fn test_bridge_config_set() {
        let mut store = CommStore::new();
        let config = BridgeConfig {
            identity_enabled: true,
            memory_enabled: true,
            ..Default::default()
        };
        store.set_bridge_config(config);
        assert!(store.bridge_config.identity_enabled);
        assert!(store.bridge_config.memory_enabled);
        assert!(!store.bridge_config.time_enabled);
    }

    #[test]
    fn test_agent_update_nonexistent() {
        let mut store = CommStore::new();
        let result = store.update_agent_availability("ghost", Availability::Busy);
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_registry_audit_log() {
        let mut store = CommStore::new();
        let agent = CommunicatingAgent {
            agent_id: "agent-audit".to_string(),
            ..Default::default()
        };
        store.register_agent(agent).unwrap();
        store.unregister_agent("agent-audit").unwrap();

        // Should have at least 2 audit entries (register + unregister)
        let register_entries: Vec<_> = store
            .audit_log
            .iter()
            .filter(|e| e.event_type == AuditEventType::AgentRegistered)
            .collect();
        let unregister_entries: Vec<_> = store
            .audit_log
            .iter()
            .filter(|e| e.event_type == AuditEventType::AgentUnregistered)
            .collect();
        assert_eq!(register_entries.len(), 1);
        assert_eq!(unregister_entries.len(), 1);
    }

    #[test]
    fn test_agents_serde_backward_compat() {
        // A CommStore serialized without agents should deserialize fine
        let store = CommStore::new();
        let json = serde_json::to_string(&store).unwrap();
        let deserialized: CommStore = serde_json::from_str(&json).unwrap();
        assert!(deserialized.agents.is_empty());
    }
}
