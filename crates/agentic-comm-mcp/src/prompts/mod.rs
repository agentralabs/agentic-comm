//! MCP prompt templates for common communication operations.

pub mod affect_calibrate;
pub mod channel_health;
pub mod comm_status;
pub mod compose_message;
pub mod hive_decision;
pub mod registry;
pub mod semantic_extract;
pub mod thread_summary;

pub use registry::PromptRegistry;
