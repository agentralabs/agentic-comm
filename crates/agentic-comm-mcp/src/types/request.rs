//! MCP request parameter types for tools, resources, and prompts.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Parameters for tools/call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    /// Tool name.
    pub name: String,
    /// Tool arguments.
    #[serde(default)]
    pub arguments: Value,
}

/// Parameters for resources/read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReadParams {
    /// Resource URI.
    pub uri: String,
}

/// Parameters for prompts/get.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGetParams {
    /// Prompt name.
    pub name: String,
    /// Prompt arguments.
    #[serde(default)]
    pub arguments: Option<Value>,
}
