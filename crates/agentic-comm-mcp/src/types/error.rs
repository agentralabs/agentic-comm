//! Error types and JSON-RPC error codes for the MCP server.

use super::message::{JsonRpcError, JsonRpcErrorObject, RequestId, JSONRPC_VERSION};

/// Standard JSON-RPC 2.0 error codes.
pub mod error_codes {
    /// Invalid JSON was received.
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist / is not available.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s).
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error.
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// MCP-specific error codes (per MCP spec).
pub mod mcp_error_codes {
    /// The request was cancelled by the client.
    pub const REQUEST_CANCELLED: i32 = -32800;
    /// Content too large to process.
    pub const CONTENT_TOO_LARGE: i32 = -32801;
    /// Resource not found.
    pub const RESOURCE_NOT_FOUND: i32 = -32802;
    /// Tool not found.
    pub const TOOL_NOT_FOUND: i32 = -32803;

    /// AgenticComm specific: Channel not found.
    pub const CHANNEL_NOT_FOUND: i32 = -32850;
    /// AgenticComm specific: Message not found.
    pub const MESSAGE_NOT_FOUND: i32 = -32851;
    /// AgenticComm specific: Subscription not found.
    pub const SUBSCRIPTION_NOT_FOUND: i32 = -32852;

    /// Server: Unauthorized (missing or invalid bearer token).
    pub const UNAUTHORIZED: i32 = -32900;
}

/// All errors that can occur in the MCP server.
#[derive(thiserror::Error, Debug)]
pub enum McpError {
    /// Invalid JSON received.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Request object is malformed.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Method does not exist.
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Parameters are invalid.
    #[error("Invalid params: {0}")]
    InvalidParams(String),

    /// Internal server error.
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Request was cancelled by the client.
    #[error("Request cancelled")]
    RequestCancelled,

    /// Content exceeds size limits.
    #[error("Content too large: {size} bytes exceeds {max} bytes")]
    ContentTooLarge {
        /// Actual size.
        size: usize,
        /// Maximum allowed size.
        max: usize,
    },

    /// MCP resource not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// MCP tool not found.
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// AgenticComm channel not found.
    #[error("Channel not found: {0}")]
    ChannelNotFound(u64),

    /// AgenticComm message not found.
    #[error("Message not found: {0}")]
    MessageNotFound(u64),

    /// AgenticComm subscription not found.
    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(u64),

    /// Transport-level error.
    #[error("Transport error: {0}")]
    Transport(String),

    /// I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error from the AgenticComm core library.
    #[error("AgenticComm error: {0}")]
    AgenticComm(String),

    /// Unauthorized — missing or invalid bearer token.
    #[error("Unauthorized")]
    Unauthorized,
}

impl McpError {
    /// Returns true if this is a protocol-level error (should be a JSON-RPC error).
    /// Tool execution errors (channel not found, message not found, etc.) return false
    /// and should use `ToolCallResult::error()` with `isError: true` instead.
    pub fn is_protocol_error(&self) -> bool {
        matches!(
            self,
            McpError::ParseError(_)
                | McpError::InvalidRequest(_)
                | McpError::MethodNotFound(_)
                | McpError::ToolNotFound(_)
                | McpError::RequestCancelled
                | McpError::ContentTooLarge { .. }
                | McpError::ResourceNotFound(_)
                | McpError::Unauthorized
        )
    }

    /// Return the JSON-RPC error code for this error type.
    pub fn error_code(&self) -> i32 {
        use error_codes::*;
        use mcp_error_codes::*;
        match self {
            McpError::ParseError(_) => PARSE_ERROR,
            McpError::InvalidRequest(_) => INVALID_REQUEST,
            McpError::MethodNotFound(_) => METHOD_NOT_FOUND,
            McpError::InvalidParams(_) => INVALID_PARAMS,
            McpError::InternalError(_) => INTERNAL_ERROR,
            McpError::RequestCancelled => REQUEST_CANCELLED,
            McpError::ContentTooLarge { .. } => CONTENT_TOO_LARGE,
            McpError::ResourceNotFound(_) => RESOURCE_NOT_FOUND,
            McpError::ToolNotFound(_) => TOOL_NOT_FOUND,
            McpError::ChannelNotFound(_) => CHANNEL_NOT_FOUND,
            McpError::MessageNotFound(_) => MESSAGE_NOT_FOUND,
            McpError::SubscriptionNotFound(_) => SUBSCRIPTION_NOT_FOUND,
            McpError::Transport(_) => INTERNAL_ERROR,
            McpError::Io(_) => INTERNAL_ERROR,
            McpError::Json(_) => PARSE_ERROR,
            McpError::AgenticComm(_) => INTERNAL_ERROR,
            McpError::Unauthorized => UNAUTHORIZED,
        }
    }

    /// Convert this error into a JSON-RPC error response.
    pub fn to_json_rpc_error(&self, id: RequestId) -> JsonRpcError {
        JsonRpcError {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            error: JsonRpcErrorObject {
                code: self.error_code(),
                message: self.to_string(),
                data: None,
            },
        }
    }
}

impl From<agentic_comm::CommError> for McpError {
    fn from(e: agentic_comm::CommError) -> Self {
        McpError::AgenticComm(e.to_string())
    }
}

/// Convenience result type for MCP operations.
pub type McpResult<T> = Result<T, McpError>;
