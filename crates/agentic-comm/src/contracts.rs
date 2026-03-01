//! Contracts bridge — implements agentic-sdk v0.2.0 traits for Comm.
//!
//! This module provides `CommSister`, a contracts-compliant wrapper
//! around the core `CommStore`. It implements:
//!
//! - `Sister` — lifecycle management
//! - `SessionManagement` — append-only sequential sessions
//! - `Grounding` — word-overlap claim verification against messages
//! - `Queryable` — unified query interface
//! - `FileFormatReader/FileFormatWriter` — .acomm file I/O
//!
//! The MCP server can use `CommSister` instead of raw CommStore
//! to get compile-time contracts compliance.

use agentic_sdk::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::{CommError, CommStore};

// ═══════════════════════════════════════════════════════════════════
// ERROR BRIDGE: CommError → SisterError
// ═══════════════════════════════════════════════════════════════════

impl From<CommError> for SisterError {
    fn from(e: CommError) -> Self {
        match &e {
            CommError::ChannelNotFound(id) => {
                SisterError::not_found(format!("channel {}", id))
            }
            CommError::MessageNotFound(id) => {
                SisterError::not_found(format!("message {}", id))
            }
            CommError::SubscriptionNotFound(id) => {
                SisterError::not_found(format!("subscription {}", id))
            }
            CommError::KeyNotFound(id) => {
                SisterError::not_found(format!("key {}", id))
            }
            CommError::NotFound(what) => SisterError::not_found(what.clone()),
            CommError::InvalidChannelName(name) => SisterError::new(
                ErrorCode::InvalidInput,
                format!("Invalid channel name: {}", name),
            ),
            CommError::InvalidContent(reason) => SisterError::new(
                ErrorCode::InvalidInput,
                format!("Invalid content: {}", reason),
            ),
            CommError::InvalidSender(reason) => SisterError::new(
                ErrorCode::InvalidInput,
                format!("Invalid sender: {}", reason),
            ),
            CommError::ChannelFull(id) => SisterError::new(
                ErrorCode::ResourceExhausted,
                format!("Channel {} is full", id),
            ),
            CommError::ChannelStateViolation(id, state) => SisterError::new(
                ErrorCode::InvalidState,
                format!("Channel {} is {} — operation not allowed", id, state),
            ),
            CommError::ConsentDenied { reason } => SisterError::new(
                ErrorCode::PermissionDenied,
                format!("Consent denied: {}", reason),
            ),
            CommError::RateLimitExceeded { limit } => SisterError::new(
                ErrorCode::ResourceExhausted,
                format!("Rate limit exceeded: {}", limit),
            ),
            CommError::InvalidFile(reason) => SisterError::new(
                ErrorCode::ChecksumMismatch,
                format!("Invalid .acomm file: {}", reason),
            ),
            CommError::Io(io_err) => {
                SisterError::new(ErrorCode::StorageError, format!("I/O error: {}", io_err))
            }
            CommError::LockError(reason) => {
                SisterError::new(ErrorCode::StorageError, format!("Lock error: {}", reason))
            }
            CommError::Serialization(reason) => SisterError::new(
                ErrorCode::StorageError,
                format!("Serialization error: {}", reason),
            ),
            _ => SisterError::new(ErrorCode::Internal, e.to_string()),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// SESSION STATE
// ═══════════════════════════════════════════════════════════════════

/// Session record for tracking sessions in CommSister.
#[derive(Debug, Clone)]
struct SessionRecord {
    id: ContextId,
    session_id: u32,
    name: String,
    created_at: chrono::DateTime<chrono::Utc>,
    message_count_at_start: usize,
}

// ═══════════════════════════════════════════════════════════════════
// COMM SISTER — The contracts-compliant facade
// ═══════════════════════════════════════════════════════════════════

/// Contracts-compliant Comm sister.
///
/// Wraps `CommStore` and implements all v0.2.0 traits.
/// This is the canonical "Comm as a sister" interface.
pub struct CommSister {
    store: CommStore,
    file_path: Option<PathBuf>,
    start_time: Instant,

    // Session state
    current_session: Option<SessionRecord>,
    sessions: Vec<SessionRecord>,
    next_session_id: u32,
}

impl CommSister {
    /// Create from an existing store (for migration from SessionManager).
    pub fn from_store(store: CommStore, file_path: Option<PathBuf>) -> Self {
        Self {
            store,
            file_path,
            start_time: Instant::now(),
            current_session: None,
            sessions: vec![],
            next_session_id: 1,
        }
    }

    /// Get a reference to the underlying store.
    pub fn store(&self) -> &CommStore {
        &self.store
    }

    /// Get a mutable reference to the underlying store.
    pub fn store_mut(&mut self) -> &mut CommStore {
        &mut self.store
    }

    /// Get the current u32 session ID (for interop with existing code).
    pub fn current_session_id(&self) -> Option<u32> {
        self.current_session.as_ref().map(|s| s.session_id)
    }

    /// Total item count across channels and messages.
    fn total_items(&self) -> usize {
        self.store.channels.len() + self.store.messages.len()
    }
}

// ═══════════════════════════════════════════════════════════════════
// SISTER TRAIT
// ═══════════════════════════════════════════════════════════════════

impl Sister for CommSister {
    const SISTER_TYPE: SisterType = SisterType::Comm;
    const FILE_EXTENSION: &'static str = "acomm";

    fn init(config: SisterConfig) -> SisterResult<Self>
    where
        Self: Sized,
    {
        let file_path = config.data_path.clone();

        let store = if let Some(ref path) = file_path {
            if path.exists() {
                CommStore::load(path).map_err(SisterError::from)?
            } else if config.create_if_missing {
                CommStore::new()
            } else {
                return Err(SisterError::new(
                    ErrorCode::NotFound,
                    format!("Comm store not found: {}", path.display()),
                ));
            }
        } else {
            CommStore::new()
        };

        Ok(Self::from_store(store, file_path))
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            status: Status::Ready,
            uptime: self.start_time.elapsed(),
            resources: ResourceUsage {
                memory_bytes: self.total_items() * 512, // rough estimate
                disk_bytes: 0,
                open_handles: if self.file_path.is_some() { 1 } else { 0 },
            },
            warnings: vec![],
            last_error: None,
        }
    }

    fn version(&self) -> Version {
        Version::new(0, 1, 0) // matches agentic-comm crate version
    }

    fn shutdown(&mut self) -> SisterResult<()> {
        // End current session if active
        if self.current_session.is_some() {
            let _ = SessionManagement::end_session(self);
        }

        // Save to file if path is set
        if let Some(ref path) = self.file_path {
            self.store.save(path).map_err(SisterError::from)?;
        }

        Ok(())
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::new("comm_channel", "Manage communication channels"),
            Capability::new("comm_message", "Send, receive, and manage messages"),
            Capability::new("comm_semantic", "Semantic messaging and NLP analysis"),
            Capability::new("comm_affect", "Emotional state tracking and propagation"),
            Capability::new("comm_hive", "Hive-mind collective intelligence"),
            Capability::new("comm_consent", "Consent management and privacy gates"),
            Capability::new("comm_trust", "Trust level management between agents"),
            Capability::new("comm_keys", "Cryptographic key management"),
            Capability::new("comm_query", "Query relationships and grounding"),
            Capability::new("comm_session", "Session lifecycle and logging"),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════
// SESSION MANAGEMENT
// ═══════════════════════════════════════════════════════════════════

impl SessionManagement for CommSister {
    fn start_session(&mut self, name: &str) -> SisterResult<ContextId> {
        // End current session if active
        if self.current_session.is_some() {
            self.end_session()?;
        }

        let session_id = self.next_session_id;
        self.next_session_id += 1;
        let context_id = ContextId::new();

        let record = SessionRecord {
            id: context_id,
            session_id,
            name: name.to_string(),
            created_at: chrono::Utc::now(),
            message_count_at_start: self.store.messages.len(),
        };

        self.current_session = Some(record.clone());
        self.sessions.push(record);

        Ok(context_id)
    }

    fn end_session(&mut self) -> SisterResult<()> {
        if self.current_session.is_none() {
            return Err(SisterError::new(
                ErrorCode::InvalidState,
                "No active session to end",
            ));
        }
        self.current_session = None;
        Ok(())
    }

    fn current_session(&self) -> Option<ContextId> {
        self.current_session.as_ref().map(|s| s.id)
    }

    fn current_session_info(&self) -> SisterResult<ContextInfo> {
        let session = self
            .current_session
            .as_ref()
            .ok_or_else(|| SisterError::new(ErrorCode::InvalidState, "No active session"))?;

        let messages_in_session =
            self.store.messages.len().saturating_sub(session.message_count_at_start);

        Ok(ContextInfo {
            id: session.id,
            name: session.name.clone(),
            created_at: session.created_at,
            updated_at: chrono::Utc::now(),
            item_count: messages_in_session,
            size_bytes: messages_in_session * 512,
            metadata: Metadata::new(),
        })
    }

    fn list_sessions(&self) -> SisterResult<Vec<ContextSummary>> {
        Ok(self
            .sessions
            .iter()
            .rev() // most recent first
            .map(|s| ContextSummary {
                id: s.id,
                name: s.name.clone(),
                created_at: s.created_at,
                updated_at: s.created_at,
                item_count: 0,
                size_bytes: 0,
            })
            .collect())
    }

    fn export_session(&self, id: ContextId) -> SisterResult<ContextSnapshot> {
        let session = self
            .sessions
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| SisterError::context_not_found(id.to_string()))?;

        // Export messages from this session's time window
        let session_messages: Vec<&crate::Message> = self
            .store
            .messages
            .values()
            .filter(|m| m.timestamp >= session.created_at)
            .collect();

        let data = serde_json::to_vec(&session_messages)
            .map_err(|e| SisterError::new(ErrorCode::Internal, e.to_string()))?;
        let checksum = *blake3::hash(&data).as_bytes();

        Ok(ContextSnapshot {
            sister_type: SisterType::Comm,
            version: Version::new(0, 1, 0),
            context_info: ContextInfo {
                id,
                name: session.name.clone(),
                created_at: session.created_at,
                updated_at: chrono::Utc::now(),
                item_count: session_messages.len(),
                size_bytes: data.len(),
                metadata: Metadata::new(),
            },
            data,
            checksum,
            snapshot_at: chrono::Utc::now(),
        })
    }

    fn import_session(&mut self, snapshot: ContextSnapshot) -> SisterResult<ContextId> {
        if !snapshot.verify() {
            return Err(SisterError::new(
                ErrorCode::ChecksumMismatch,
                "Session snapshot checksum verification failed",
            ));
        }

        // Start a new session for the imported data
        let context_id = self.start_session(&snapshot.context_info.name)?;

        // Deserialize and ingest the messages
        let messages: Vec<crate::Message> = serde_json::from_slice(&snapshot.data)
            .map_err(|e| SisterError::new(ErrorCode::InvalidInput, e.to_string()))?;

        for msg in messages {
            self.store.messages.insert(msg.id, msg);
        }

        Ok(context_id)
    }
}

// ═══════════════════════════════════════════════════════════════════
// GROUNDING
// ═══════════════════════════════════════════════════════════════════

impl Grounding for CommSister {
    fn ground(&self, claim: &str) -> SisterResult<GroundingResult> {
        let matches = self.store.search_messages(claim, 10);

        if matches.is_empty() {
            let mut recent_msgs: Vec<&crate::Message> =
                self.store.messages.values().collect();
            recent_msgs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            let suggestions: Vec<String> = recent_msgs
                .iter()
                .take(3)
                .map(|m| m.content.clone())
                .collect();

            return Ok(
                GroundingResult::ungrounded(claim, "No matching messages found")
                    .with_suggestions(suggestions),
            );
        }

        let evidence: Vec<GroundingEvidence> = matches
            .iter()
            .enumerate()
            .map(|(i, msg)| {
                let score = 1.0 - (i as f64 * 0.1); // decreasing relevance
                GroundingEvidence::new(
                    "comm_message",
                    format!("msg_{}", msg.id),
                    score,
                    &msg.content,
                )
                .with_data("sender", msg.sender.clone())
                .with_data("channel_id", msg.channel_id)
                .with_data("message_type", format!("{}", msg.message_type))
                .with_data("timestamp", msg.timestamp.to_rfc3339())
            })
            .collect();

        let confidence = if matches.len() >= 3 { 0.8 } else { 0.4 };

        if confidence > 0.5 {
            Ok(GroundingResult::verified(claim, confidence)
                .with_evidence(evidence)
                .with_reason("Found matching messages via content search"))
        } else {
            Ok(GroundingResult::partial(claim, confidence)
                .with_evidence(evidence)
                .with_reason("Some evidence found but limited matches"))
        }
    }

    fn evidence(&self, query: &str, max_results: usize) -> SisterResult<Vec<EvidenceDetail>> {
        let matches = self.store.search_messages(query, max_results);

        Ok(matches
            .iter()
            .enumerate()
            .map(|(i, msg)| {
                let score = 1.0 - (i as f64 * 0.05);
                EvidenceDetail {
                    evidence_type: "comm_message".to_string(),
                    id: format!("msg_{}", msg.id),
                    score,
                    created_at: msg.timestamp,
                    source_sister: SisterType::Comm,
                    content: msg.content.clone(),
                    data: {
                        let mut meta = Metadata::new();
                        if let Ok(v) = serde_json::to_value(&msg.sender) {
                            meta.insert("sender".to_string(), v);
                        }
                        if let Ok(v) = serde_json::to_value(msg.channel_id) {
                            meta.insert("channel_id".to_string(), v);
                        }
                        if let Ok(v) = serde_json::to_value(format!("{}", msg.message_type)) {
                            meta.insert("message_type".to_string(), v);
                        }
                        meta
                    },
                }
            })
            .collect())
    }

    fn suggest(&self, query: &str, limit: usize) -> SisterResult<Vec<GroundingSuggestion>> {
        // Word-overlap fallback for near-miss suggestions
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(f64, &crate::Message)> = self
            .store
            .messages
            .values()
            .map(|msg| {
                let content_lower = msg.content.to_lowercase();
                let matched = query_words
                    .iter()
                    .filter(|w| content_lower.contains(**w))
                    .count();
                let score = if query_words.is_empty() {
                    0.0
                } else {
                    matched as f64 / query_words.len() as f64
                };
                (score, msg)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored
            .into_iter()
            .take(limit)
            .map(|(score, msg)| GroundingSuggestion {
                item_type: "comm_message".to_string(),
                id: format!("msg_{}", msg.id),
                relevance_score: score,
                description: msg.content.clone(),
                data: Metadata::new(),
            })
            .collect())
    }
}

// ═══════════════════════════════════════════════════════════════════
// QUERYABLE
// ═══════════════════════════════════════════════════════════════════

impl Queryable for CommSister {
    fn query(&self, query: Query) -> SisterResult<QueryResult> {
        let start = Instant::now();

        let results: Vec<serde_json::Value> = match query.query_type.as_str() {
            "list" => {
                let limit = query.limit.unwrap_or(50);
                let offset = query.offset.unwrap_or(0);
                let mut msgs: Vec<&crate::Message> = self.store.messages.values().collect();
                msgs.sort_by_key(|m| m.id);
                msgs.iter()
                    .skip(offset)
                    .take(limit)
                    .map(|m| {
                        serde_json::json!({
                            "id": m.id,
                            "channel_id": m.channel_id,
                            "sender": m.sender,
                            "content": m.content,
                            "message_type": format!("{}", m.message_type),
                            "timestamp": m.timestamp.to_rfc3339(),
                        })
                    })
                    .collect()
            }
            "search" => {
                let text = query.get_string("text").unwrap_or_default();
                let max = query.limit.unwrap_or(20);
                let matches = self.store.search_messages(&text, max);
                matches
                    .iter()
                    .map(|m| {
                        serde_json::json!({
                            "id": m.id,
                            "channel_id": m.channel_id,
                            "sender": m.sender,
                            "content": m.content,
                            "message_type": format!("{}", m.message_type),
                            "timestamp": m.timestamp.to_rfc3339(),
                        })
                    })
                    .collect()
            }
            "recent" => {
                let count = query.limit.unwrap_or(10);
                let mut msgs: Vec<&crate::Message> = self.store.messages.values().collect();
                msgs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                msgs.iter()
                    .take(count)
                    .map(|m| {
                        serde_json::json!({
                            "id": m.id,
                            "channel_id": m.channel_id,
                            "sender": m.sender,
                            "content": m.content,
                            "message_type": format!("{}", m.message_type),
                            "timestamp": m.timestamp.to_rfc3339(),
                        })
                    })
                    .collect()
            }
            "get" => {
                let id_str = query.get_string("id").unwrap_or_default();
                let id: u64 = id_str.parse().unwrap_or(0);
                if let Some(m) = self.store.messages.get(&id) {
                    vec![serde_json::json!({
                        "id": m.id,
                        "channel_id": m.channel_id,
                        "sender": m.sender,
                        "content": m.content,
                        "message_type": format!("{}", m.message_type),
                        "timestamp": m.timestamp.to_rfc3339(),
                    })]
                } else {
                    vec![]
                }
            }
            "channels" => {
                let limit = query.limit.unwrap_or(50);
                let mut channels: Vec<&crate::Channel> = self.store.channels.values().collect();
                channels.sort_by_key(|c| c.id);
                channels
                    .iter()
                    .take(limit)
                    .map(|c| {
                        serde_json::json!({
                            "id": c.id,
                            "name": c.name,
                            "channel_type": format!("{:?}", c.channel_type),
                            "state": format!("{:?}", c.state),
                            "participants": c.participants.len(),
                            "created_at": c.created_at.to_rfc3339(),
                        })
                    })
                    .collect()
            }
            _ => vec![],
        };

        let total = self.store.messages.len();
        let has_more = results.len() < total;

        Ok(QueryResult::new(query, results, start.elapsed()).with_pagination(total, has_more))
    }

    fn supports_query(&self, query_type: &str) -> bool {
        matches!(
            query_type,
            "list" | "search" | "recent" | "get" | "channels"
        )
    }

    fn query_types(&self) -> Vec<QueryTypeInfo> {
        vec![
            QueryTypeInfo::new("list", "List all messages with pagination")
                .optional(vec!["limit", "offset"]),
            QueryTypeInfo::new("search", "Search messages by content")
                .required(vec!["text"])
                .optional(vec!["limit"]),
            QueryTypeInfo::new("recent", "Get most recent messages").optional(vec!["limit"]),
            QueryTypeInfo::new("get", "Get a specific message by ID").required(vec!["id"]),
            QueryTypeInfo::new("channels", "List all channels").optional(vec!["limit"]),
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════
// FILE FORMAT
// ═══════════════════════════════════════════════════════════════════

impl FileFormatReader for CommSister {
    fn read_file(path: &Path) -> SisterResult<Self> {
        let store = CommStore::load(path).map_err(SisterError::from)?;
        Ok(Self::from_store(store, Some(path.to_path_buf())))
    }

    fn can_read(path: &Path) -> SisterResult<FileInfo> {
        let data = std::fs::read(path)
            .map_err(|e| SisterError::new(ErrorCode::StorageError, e.to_string()))?;
        if data.len() < crate::format::FileHeader::SIZE_V2 {
            return Err(SisterError::new(
                ErrorCode::StorageError,
                "File too small for .acomm format",
            ));
        }
        // Check magic bytes
        if &data[..4] != crate::format::MAGIC {
            return Err(SisterError::new(
                ErrorCode::ChecksumMismatch,
                "Not a valid .acomm file (bad magic bytes)",
            ));
        }
        let version = u16::from_le_bytes([data[4], data[5]]);

        let metadata = std::fs::metadata(path)
            .map_err(|e| SisterError::new(ErrorCode::StorageError, e.to_string()))?;

        Ok(FileInfo {
            sister_type: SisterType::Comm,
            version: Version::new(version as u8, 0, 0),
            created_at: chrono::Utc::now(),
            updated_at: chrono::DateTime::from(
                metadata
                    .modified()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            ),
            content_length: metadata.len(),
            needs_migration: version < crate::format::FORMAT_VERSION,
            format_id: "ACOM".to_string(),
        })
    }

    fn file_version(path: &Path) -> SisterResult<Version> {
        let data = std::fs::read(path)
            .map_err(|e| SisterError::new(ErrorCode::StorageError, e.to_string()))?;
        if data.len() < 6 {
            return Err(SisterError::new(
                ErrorCode::StorageError,
                "File too small for .acomm format",
            ));
        }
        let version = u16::from_le_bytes([data[4], data[5]]);
        Ok(Version::new(version as u8, 0, 0))
    }

    fn migrate(_data: &[u8], _from_version: Version) -> SisterResult<Vec<u8>> {
        Err(SisterError::new(
            ErrorCode::NotImplemented,
            "No migration path available (v2→v3 handled transparently on load)",
        ))
    }
}

impl FileFormatWriter for CommSister {
    fn write_file(&self, path: &Path) -> SisterResult<()> {
        self.store.save(path).map_err(SisterError::from)
    }

    fn to_bytes(&self) -> SisterResult<Vec<u8>> {
        let serialized = bincode::serialize(&self.store)
            .map_err(|e| SisterError::new(ErrorCode::StorageError, e.to_string()))?;
        Ok(crate::format::write_with_header(&serialized))
    }
}

// ═══════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_sister() -> CommSister {
        let config = SisterConfig::stateless();
        CommSister::init(config).unwrap()
    }

    fn add_test_messages(sister: &mut CommSister) {
        // Create a channel first
        sister
            .store
            .create_channel("test-channel", crate::ChannelType::Group, None)
            .unwrap();
        let channel_id = sister.store.channels.keys().next().copied().unwrap();

        // Send some messages
        sister
            .store
            .send_message(channel_id, "alice", "The deployment succeeded on staging", crate::MessageType::Text)
            .unwrap();
        sister
            .store
            .send_message(channel_id, "bob", "Rust compiler caught the bug early", crate::MessageType::Text)
            .unwrap();
        sister
            .store
            .send_message(channel_id, "carol", "Review the pull request before merging", crate::MessageType::Command)
            .unwrap();
    }

    #[test]
    fn test_sister_trait() {
        let sister = make_test_sister();
        assert_eq!(sister.sister_type(), SisterType::Comm);
        assert_eq!(sister.file_extension(), "acomm");
        assert_eq!(sister.mcp_prefix(), "comm");
        assert!(sister.is_healthy());
        assert_eq!(sister.version(), Version::new(0, 1, 0));
        assert!(!sister.capabilities().is_empty());
    }

    #[test]
    fn test_sister_info() {
        let sister = make_test_sister();
        let info = SisterInfo::from_sister(&sister);
        assert_eq!(info.sister_type, SisterType::Comm);
        assert_eq!(info.file_extension, "acomm");
        assert_eq!(info.mcp_prefix, "comm");
    }

    #[test]
    fn test_session_management() {
        let mut sister = make_test_sister();

        // No session initially
        assert!(sister.current_session().is_none());
        assert!(sister.current_session_info().is_err());

        // Start session
        let sid = sister.start_session("test_session").unwrap();
        assert!(sister.current_session().is_some());
        assert_eq!(sister.current_session().unwrap(), sid);

        // Session info
        let info = sister.current_session_info().unwrap();
        assert_eq!(info.name, "test_session");

        // List sessions
        let sessions = sister.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "test_session");

        // End session
        sister.end_session().unwrap();
        assert!(sister.current_session().is_none());

        // Can't end twice
        assert!(sister.end_session().is_err());
    }

    #[test]
    fn test_grounding_with_data() {
        let mut sister = make_test_sister();
        sister.start_session("grounding_test").unwrap();
        add_test_messages(&mut sister);

        // Ground a claim that should match (substring of message content)
        let result = sister.ground("deployment succeeded").unwrap();
        assert!(
            result.status == GroundingStatus::Verified || result.status == GroundingStatus::Partial,
            "Expected verified or partial, got {:?}",
            result.status
        );
        assert!(!result.evidence.is_empty());

        // Ground a claim that should NOT match
        let result = sister.ground("quantum teleportation physics").unwrap();
        assert_eq!(result.status, GroundingStatus::Ungrounded);
    }

    #[test]
    fn test_evidence_query() {
        let mut sister = make_test_sister();
        sister.start_session("evidence_test").unwrap();
        add_test_messages(&mut sister);

        let evidence = sister.evidence("rust", 10).unwrap();
        assert!(!evidence.is_empty(), "Expected evidence for 'rust' query");
        assert_eq!(evidence[0].source_sister, SisterType::Comm);
    }

    #[test]
    fn test_suggest_fallback() {
        let mut sister = make_test_sister();
        sister.start_session("suggest_test").unwrap();
        add_test_messages(&mut sister);

        let suggestions = sister.suggest("deployment staging", 5).unwrap();
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].relevance_score > 0.0);
    }

    #[test]
    fn test_queryable_list() {
        let mut sister = make_test_sister();
        sister.start_session("query_test").unwrap();
        add_test_messages(&mut sister);

        let result = sister.query(Query::list().limit(2)).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.has_more);
    }

    #[test]
    fn test_queryable_recent() {
        let mut sister = make_test_sister();
        sister.start_session("recent_test").unwrap();
        add_test_messages(&mut sister);

        let result = sister.recent(2).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_queryable_search() {
        let mut sister = make_test_sister();
        sister.start_session("search_test").unwrap();
        add_test_messages(&mut sister);

        let result = sister.search("rust").unwrap();
        assert!(!result.is_empty(), "Expected search results for 'rust'");
    }

    #[test]
    fn test_queryable_types() {
        let sister = make_test_sister();
        assert!(sister.supports_query("list"));
        assert!(sister.supports_query("search"));
        assert!(sister.supports_query("recent"));
        assert!(sister.supports_query("get"));
        assert!(sister.supports_query("channels"));
        assert!(!sister.supports_query("nonexistent"));

        let types = sister.query_types();
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_error_bridge() {
        let comm_err = CommError::ChannelNotFound(42);
        let sister_err: SisterError = comm_err.into();
        assert_eq!(sister_err.code, ErrorCode::NotFound);
        assert!(sister_err.message.contains("42"));

        let comm_err2 = CommError::InvalidFile("corrupt header".to_string());
        let sister_err2: SisterError = comm_err2.into();
        assert_eq!(sister_err2.code, ErrorCode::ChecksumMismatch);
    }

    #[test]
    fn test_session_export_import() {
        let mut sister = make_test_sister();
        let sid = sister.start_session("export_test").unwrap();
        add_test_messages(&mut sister);

        // Export
        let snapshot = sister.export_session(sid).unwrap();
        assert!(snapshot.verify());
        assert_eq!(snapshot.sister_type, SisterType::Comm);

        // Import into fresh sister
        let mut sister2 = make_test_sister();
        let _imported_sid = sister2.import_session(snapshot).unwrap();
        assert!(sister2.current_session().is_some());
        // Imported session should have messages
        assert!(sister2.store().messages.len() > 0);
    }

    #[test]
    fn test_config_patterns() {
        // Stateless config
        let config = SisterConfig::stateless();
        let sister = CommSister::init(config).unwrap();
        assert!(sister.is_healthy());
    }

    #[test]
    fn test_shutdown() {
        let mut sister = make_test_sister();
        sister.start_session("shutdown_test").unwrap();
        sister.shutdown().unwrap();
        // Session should be ended after shutdown
        assert!(sister.current_session().is_none());
    }
}
