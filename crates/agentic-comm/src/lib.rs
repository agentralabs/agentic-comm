//! AgenticComm — agent-to-agent and agent-to-human communication engine.
//!
//! Provides structured messaging, channels, pub/sub, message routing, and
//! communication history stored in `.acomm` files.

use std::collections::HashMap;
use std::io::{Read as IoRead, Write as IoWrite};
use std::path::Path;

use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

    /// I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid file format.
    #[error("Invalid .acomm file: {0}")]
    InvalidFile(String),
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
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelType::Direct => write!(f, "direct"),
            ChannelType::Group => write!(f, "group"),
            ChannelType::Broadcast => write!(f, "broadcast"),
            ChannelType::PubSub => write!(f, "pubsub"),
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
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            max_participants: 0,
            ttl_seconds: 0,
            persistence: true,
            encryption_required: false,
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
#[derive(Debug, Clone, Default)]
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

    fn compute_signature(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    // -----------------------------------------------------------------------
    // Message engine
    // -----------------------------------------------------------------------

    /// Send a message to a channel.
    pub fn send_message(
        &mut self,
        channel_id: u64,
        sender: &str,
        content: &str,
        msg_type: MessageType,
    ) -> CommResult<Message> {
        Self::validate_sender(sender)?;
        Self::validate_content(content)?;

        if !self.channels.contains_key(&channel_id) {
            return Err(CommError::ChannelNotFound(channel_id));
        }

        let id = self.next_message_id;
        self.next_message_id += 1;

        let message = Message {
            id,
            channel_id,
            sender: sender.to_string(),
            recipient: None,
            content: content.to_string(),
            message_type: msg_type,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            signature: Some(Self::compute_signature(content)),
            acknowledged_by: Vec::new(),
        };

        self.messages.insert(id, message.clone());
        Ok(message)
    }

    /// Receive messages from a channel, optionally filtered by recipient and time.
    pub fn receive_messages(
        &self,
        channel_id: u64,
        recipient: Option<&str>,
        since: Option<DateTime<Utc>>,
    ) -> CommResult<Vec<Message>> {
        if !self.channels.contains_key(&channel_id) {
            return Err(CommError::ChannelNotFound(channel_id));
        }

        let mut msgs: Vec<Message> = self
            .messages
            .values()
            .filter(|m| {
                if m.channel_id != channel_id {
                    return false;
                }
                if let Some(ref recip) = recipient {
                    // Include messages that are addressed to this recipient or to everyone
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

            let message = Message {
                id,
                channel_id,
                sender: sender.to_string(),
                recipient: Some(participant.clone()),
                content: content.to_string(),
                message_type: MessageType::Broadcast,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
                signature: Some(Self::compute_signature(content)),
                acknowledged_by: Vec::new(),
            };

            self.messages.insert(id, message.clone());
            delivered.push(message);
        }

        Ok(delivered)
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
        };

        self.channels.insert(id, channel.clone());
        Ok(channel)
    }

    /// Join a channel as a participant.
    pub fn join_channel(&mut self, channel_id: u64, participant: &str) -> CommResult<()> {
        Self::validate_sender(participant)?;

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

            let message = Message {
                id,
                channel_id,
                sender: sender.to_string(),
                recipient: Some(subscriber),
                content: content.to_string(),
                message_type: MessageType::Notification,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
                signature: Some(Self::compute_signature(content)),
                acknowledged_by: Vec::new(),
            };

            self.messages.insert(id, message.clone());
            delivered.push(message);
        }

        Ok(delivered)
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

    /// Save the store to a .acomm file (bincode + gzip).
    pub fn save(&self, path: &Path) -> CommResult<()> {
        let header = AcommHeader {
            magic: *ACOMM_MAGIC,
            version: ACOMM_VERSION,
            channel_count: self.channels.len() as u32,
            message_count: self.messages.len() as u64,
        };

        let header_bytes =
            bincode::serialize(&header).map_err(|e| CommError::Serialization(e.to_string()))?;

        let store_bytes =
            bincode::serialize(self).map_err(|e| CommError::Serialization(e.to_string()))?;

        let file = std::fs::File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(&header_bytes)?;
        encoder.write_all(&store_bytes)?;
        encoder.finish()?;

        Ok(())
    }

    /// Load a store from a .acomm file.
    pub fn load(path: &Path) -> CommResult<Self> {
        let file = std::fs::File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let mut data = Vec::new();
        decoder.read_to_end(&mut data)?;

        // Deserialize header first
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

        Ok(store)
    }

    /// Get summary statistics for the store.
    pub fn stats(&self) -> CommStoreStats {
        CommStoreStats {
            channel_count: self.channels.len(),
            message_count: self.messages.len(),
            subscription_count: self.subscriptions.len(),
            total_participants: self
                .channels
                .values()
                .map(|c| c.participants.len())
                .sum(),
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
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
}
