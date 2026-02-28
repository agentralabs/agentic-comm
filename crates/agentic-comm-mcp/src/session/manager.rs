//! Session manager — owns the CommStore and tracks operations.

use std::path::PathBuf;
use std::time::Instant;

use agentic_comm::CommStore;
use chrono::Utc;
use serde::Serialize;

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
        }
    }

    /// Mark the session as started (called on `initialized` notification).
    pub fn mark_session_started(&mut self) {
        self.session_active = true;
        self.session_start_time = Instant::now();
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
}
