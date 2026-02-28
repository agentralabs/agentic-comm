//! Session manager — owns the CommStore and tracks operations.

use std::path::PathBuf;
use std::time::Instant;

use agentic_comm::CommStore;
use chrono::Utc;
use serde::Serialize;

use crate::config::loader::resolve_comm_path;
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

/// Manages the communication store and session state.
pub struct SessionManager {
    /// The underlying communication store.
    pub store: CommStore,
    /// Path the store was loaded from / will be saved to.
    pub store_path: PathBuf,
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
}

impl SessionManager {
    /// Create a new session manager, loading from the default or specified path.
    pub fn new(path: Option<PathBuf>) -> Self {
        let store_path = path.unwrap_or_else(|| resolve_comm_path());
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
            operation_log: Vec::new(),
            context_log: Vec::new(),
            session_start_time: Instant::now(),
            last_message_id: None,
            session_active: false,
        }
    }

    /// Mark the session as started (called on `initialized` notification).
    pub fn mark_session_started(&mut self) {
        self.session_active = true;
        self.session_start_time = Instant::now();
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

    /// Whether the session is active.
    pub fn is_active(&self) -> bool {
        self.session_active
    }
}
