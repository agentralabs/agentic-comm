//! MCP response types for tools, resources, and prompts.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Content types that can be returned by tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    /// Text content.
    #[serde(rename = "text")]
    Text {
        /// The text content.
        text: String,
    },
    /// Image content (base64-encoded).
    #[serde(rename = "image")]
    Image {
        /// Base64-encoded image data.
        data: String,
        /// MIME type (e.g. "image/png").
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// Embedded resource content.
    #[serde(rename = "resource")]
    Resource {
        /// The embedded resource.
        resource: ResourceContent,
    },
}

/// Result from a tools/call invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    /// Content returned by the tool.
    pub content: Vec<ToolContent>,
    /// Whether the tool call errored.
    #[serde(default, rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl ToolCallResult {
    /// Create a successful text result.
    pub fn text(text: String) -> Self {
        Self {
            content: vec![ToolContent::Text { text }],
            is_error: None,
        }
    }

    /// Create a JSON result.
    pub fn json(value: &impl Serialize) -> Self {
        let text = serde_json::to_string_pretty(value).unwrap_or_else(|e| e.to_string());
        Self::text(text)
    }

    /// Create an error result.
    pub fn error(message: String) -> Self {
        Self {
            content: vec![ToolContent::Text { text: message }],
            is_error: Some(true),
        }
    }
}

/// Tool definition for tools/list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name (unique).
    pub name: String,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for the input parameters.
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Result from tools/list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolListResult {
    /// Available tools.
    pub tools: Vec<ToolDefinition>,
    /// Cursor for next page.
    #[serde(
        default,
        rename = "nextCursor",
        skip_serializing_if = "Option::is_none"
    )]
    pub next_cursor: Option<String>,
}

/// Resource content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    /// Resource URI.
    pub uri: String,
    /// MIME type.
    #[serde(default, rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Binary content (base64).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}
