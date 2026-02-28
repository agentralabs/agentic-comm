//! 20-Year Clock context-capture tool for communication actions.
//!
//! Records WHY a communication action is happening, linking intent
//! to the temporal chain of the session.

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Input parameters for the communication_log tool.
#[derive(Debug, Clone, Deserialize)]
pub struct CommunicationLogInput {
    /// Why the communication action is happening.
    pub intent: String,
    /// What was noticed or concluded.
    pub observation: Option<String>,
    /// Link to a related message id.
    pub related_message_id: Option<u64>,
    /// Category or topic.
    pub topic: Option<String>,
}

/// A single communication log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationLogEntry {
    /// When this was logged.
    pub timestamp: String,
    /// Why the action happened.
    pub intent: String,
    /// What was observed.
    pub observation: Option<String>,
    /// Related message.
    pub related_message_id: Option<u64>,
    /// Topic/category.
    pub topic: Option<String>,
}

impl CommunicationLogEntry {
    /// Create a new log entry from the input.
    pub fn from_input(input: &CommunicationLogInput) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            intent: input.intent.clone(),
            observation: input.observation.clone(),
            related_message_id: input.related_message_id,
            topic: input.topic.clone(),
        }
    }
}
