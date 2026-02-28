//! Session manager — owns the CommStore and tracks operations.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use agentic_comm::{CommStore, CommWorkspace};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::config::loader::{project_identity, project_store_path};
use crate::tools::communication_log::CommunicationLogEntry;

/// A record of a single tool operation.
#[derive(Debug, Clone, Serialize)]
pub struct OperationRecord {
    /// Which tool was called.
    pub tool_name: String,
    /// When it was called.
    pub timestamp: String,
    /// Optional related entity id.
    pub related_id: Option<u64>,
}

/// A record of a tool call with success/failure tracking (Phase 0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub summary: String,
    pub timestamp: u64,
    pub success: bool,
}

/// A single conversation log entry with temporal chaining (Phase 0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationLogEntry {
    pub timestamp: u64,
    pub user_message: Option<String>,
    pub agent_response: Option<String>,
    pub topic: Option<String>,
    pub temporal_id: u64,
    pub prev_temporal_id: Option<u64>,
}

/// Summary statistics for a completed session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub duration_secs: u64,
    pub tool_calls: usize,
    pub conversation_entries: usize,
    pub messages_sent: usize,
    pub channels_created: usize,
}

/// Data returned when resuming a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResumeData {
    pub recent_operations: Vec<ToolCallRecord>,
    pub recent_conversations: Vec<ConversationLogEntry>,
    pub session_active: bool,
    pub store_path: Option<String>,
}

/// Manages the communication store and session state.
pub struct SessionManager {
    /// The underlying communication store.
    pub store: CommStore,
    /// Path the store was loaded from / will be saved to.
    pub store_path: PathBuf,
    /// Deterministic project identity (SHA-256 of cwd).
    pub project_identity: String,
    /// Operation log for the current session.
    pub operation_log: Vec<OperationRecord>,
    /// Communication context log (20-Year Clock).
    pub context_log: Vec<CommunicationLogEntry>,
    /// When this session started.
    pub session_start_time: Instant,
    /// Last message id seen (for temporal chaining).
    pub last_message_id: Option<u64>,
    /// Whether the session has been marked as started by the client.
    session_active: bool,
    /// Ephemeral workspace storage (per-session, not persisted).
    pub workspaces: HashMap<String, CommWorkspace>,
    /// Phase 0: tool call log with success tracking.
    pub tool_call_log: Vec<ToolCallRecord>,
    /// Phase 0: conversation log entries with temporal chaining.
    pub conversation_log_entries: Vec<ConversationLogEntry>,
    /// Last temporal id in the chain.
    pub last_temporal_id: Option<u64>,
    /// Temporal chain edges (prev_id, next_id).
    pub temporal_chain: Vec<(u64, u64)>,
    /// Next temporal counter.
    next_temporal_counter: u64,
    /// Session metadata (set by comm_session_start).
    session_metadata: Option<serde_json::Value>,
    /// Session start time as unix epoch seconds (for duration calculation).
    session_start_epoch: Option<u64>,
}

impl SessionManager {
    /// Create a new session manager, loading from the default or specified path.
    ///
    /// Uses the full resolution chain:
    /// 1. CLI arg (if provided)
    /// 2. `ACOMM_STORE` env var
    /// 3. `.acomm/store.acomm` in the current directory
    /// 4. `~/.store.acomm` fallback
    pub fn new(path: Option<PathBuf>) -> Self {
        let store_path = project_store_path(path);
        let proj_id = project_identity();
        let store = if store_path.exists() {
            CommStore::load(&store_path).unwrap_or_else(|e| {
                eprintln!(
                    "Warning: could not load {}: {e}",
                    store_path.display()
                );
                CommStore::new()
            })
        } else {
            CommStore::new()
        };

        Self {
            store,
            store_path,
            project_identity: proj_id,
            operation_log: Vec::new(),
            context_log: Vec::new(),
            session_start_time: Instant::now(),
            last_message_id: None,
            session_active: false,
            workspaces: HashMap::new(),
            tool_call_log: Vec::new(),
            conversation_log_entries: Vec::new(),
            last_temporal_id: None,
            temporal_chain: Vec::new(),
            next_temporal_counter: 1,
            session_metadata: None,
            session_start_epoch: None,
        }
    }

    /// Mark the session as started (called on `initialized` notification).
    pub fn mark_session_started(&mut self) {
        self.session_active = true;
        self.session_start_time = Instant::now();
        self.session_start_epoch = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        eprintln!(
            "Session started for project {} (store: {})",
            &self.project_identity[..12],
            self.store_path.display()
        );
    }

    /// Record a tool operation.
    pub fn record_operation(&mut self, tool_name: &str, related_id: Option<u64>) {
        let record = OperationRecord {
            tool_name: tool_name.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            related_id,
        };

        // Update temporal chain
        if let Some(id) = related_id {
            self.last_message_id = Some(id);
        }

        self.operation_log.push(record);
    }

    /// Log a communication context entry (20-Year Clock).
    pub fn log_communication_context(&mut self, entry: CommunicationLogEntry) {
        self.context_log.push(entry);
    }

    /// Save the store to disk.
    pub fn save(&mut self) -> Result<(), agentic_comm::CommError> {
        // Ensure parent directory exists
        if let Some(parent) = self.store_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| agentic_comm::CommError::Io(e))?;
            }
        }
        self.store.save(&self.store_path)
    }

    /// Auto-save and stop the session. Saves the store to disk and marks
    /// the session as inactive. Returns any save error without panicking.
    pub fn auto_save_on_stop(&mut self) -> Result<(), agentic_comm::CommError> {
        if !self.session_active {
            return Ok(());
        }

        let result = self.save();
        self.session_active = false;

        let elapsed = self.session_start_time.elapsed();
        eprintln!(
            "Session ended for project {} ({:.1}s, {} ops)",
            &self.project_identity[..12],
            elapsed.as_secs_f64(),
            self.operation_log.len()
        );

        result
    }

    /// Whether the session is active.
    pub fn is_active(&self) -> bool {
        self.session_active
    }

    // ── Phase 0: Session management & temporal chaining ─────────────

    /// Record a tool call with success/failure tracking (Phase 0).
    pub fn record_tool_call(&mut self, tool_name: &str, summary: &str, success: bool) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.tool_call_log.push(ToolCallRecord {
            tool_name: tool_name.to_string(),
            summary: summary.chars().take(200).collect(),
            timestamp,
            success,
        });
    }

    /// Log a conversation entry with temporal chaining (Phase 0).
    pub fn log_conversation(
        &mut self,
        user_message: Option<&str>,
        agent_response: Option<&str>,
        topic: Option<&str>,
    ) -> ConversationLogEntry {
        let temporal_id = self.advance_temporal_chain();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = ConversationLogEntry {
            timestamp,
            user_message: user_message.map(|s| s.to_string()),
            agent_response: agent_response.map(|s| s.to_string()),
            topic: topic.map(|s| s.to_string()),
            temporal_id,
            prev_temporal_id: if temporal_id > 1 {
                Some(temporal_id - 1)
            } else {
                None
            },
        };
        self.conversation_log_entries.push(entry.clone());
        entry
    }

    /// Start a new session manually, clearing previous state.
    pub fn start_session_manual(&mut self, metadata: Option<serde_json::Value>) {
        self.tool_call_log.clear();
        self.conversation_log_entries.clear();
        self.temporal_chain.clear();
        self.last_temporal_id = None;
        self.next_temporal_counter = 1;
        self.session_metadata = metadata;
        // Also mark session started if not already
        self.mark_session_started();
    }

    /// End the current session and return summary statistics.
    pub fn end_session_manual(&mut self, _summary: Option<&str>) -> SessionSummary {
        let duration = self.session_start_epoch.map(|start| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .saturating_sub(start)
        }).unwrap_or(0);
        // auto-save
        let _ = self.auto_save_on_stop();
        SessionSummary {
            duration_secs: duration,
            tool_calls: self.tool_call_log.len(),
            conversation_entries: self.conversation_log_entries.len(),
            messages_sent: 0,
            channels_created: 0,
        }
    }

    /// Return recent session data for context resumption.
    pub fn resume_session_data(&self, limit: usize) -> SessionResumeData {
        let recent_ops: Vec<_> = self.tool_call_log.iter().rev().take(limit).cloned().collect();
        let recent_convos: Vec<_> = self
            .conversation_log_entries
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect();
        SessionResumeData {
            recent_operations: recent_ops,
            recent_conversations: recent_convos,
            session_active: self.session_start_epoch.is_some() && self.session_active,
            store_path: Some(self.store_path.display().to_string()),
        }
    }

    /// Advance the temporal chain and return the new id.
    fn advance_temporal_chain(&mut self) -> u64 {
        let new_id = self.next_temporal_counter;
        self.next_temporal_counter += 1;
        if let Some(prev) = self.last_temporal_id {
            self.temporal_chain.push((prev, new_id));
        }
        self.last_temporal_id = Some(new_id);
        new_id
    }
}
