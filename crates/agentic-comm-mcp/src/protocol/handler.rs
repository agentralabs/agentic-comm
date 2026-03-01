//! Main request dispatcher — receives JSON-RPC messages, routes to handlers.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::prompts::PromptRegistry;
use crate::resources::ResourceRegistry;
use crate::session::manager::SessionManager;
use crate::tools::registry::ToolRegistry;
use crate::types::*;

/// Parameters expected by the MCP `initialize` method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Client protocol version.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client capabilities.
    #[serde(default)]
    pub capabilities: Value,
    /// Client info.
    #[serde(rename = "clientInfo", default)]
    pub client_info: Value,
}

/// Result returned from `initialize`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Server protocol version.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: Value,
    /// Server info.
    #[serde(rename = "serverInfo")]
    pub server_info: Value,
}

/// The main protocol handler that dispatches incoming JSON-RPC messages.
pub struct ProtocolHandler {
    session: Arc<Mutex<SessionManager>>,
    shutdown_requested: Arc<AtomicBool>,
    /// Tracks whether an auto-session was started so we can auto-end it.
    auto_session_started: AtomicBool,
}

impl ProtocolHandler {
    /// Create a new protocol handler with the given session manager.
    pub fn new(session: Arc<Mutex<SessionManager>>) -> Self {
        Self {
            session,
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            auto_session_started: AtomicBool::new(false),
        }
    }

    /// Returns true once a shutdown request has been handled.
    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::Relaxed)
    }

    /// Handle an incoming JSON-RPC message and optionally return a response.
    pub async fn handle_message(&self, msg: JsonRpcMessage) -> Option<Value> {
        match msg {
            JsonRpcMessage::Request(req) => Some(self.handle_request(req).await),
            JsonRpcMessage::Notification(notif) => {
                self.handle_notification(notif).await;
                None
            }
            _ => {
                // Responses and errors from the client are unexpected
                None
            }
        }
    }

    /// End the session gracefully and clean up resources.
    ///
    /// Called on EOF or read error from the stdio transport. Uses
    /// `end_session_manual` for a proper session summary, matching the
    /// behaviour of sister projects (agentic-memory, agentic-vision).
    pub async fn end_session_and_cleanup(&self) {
        if !self.auto_session_started.swap(false, Ordering::Relaxed) {
            return;
        }

        let mut session = self.session.lock().await;
        let summary = session.end_session_manual(Some("Auto-ended on transport close"));
        eprintln!(
            "Session ended: {:.0}s, {} tool calls, {} conversation entries",
            summary.duration_secs,
            summary.tool_calls,
            summary.conversation_entries,
        );
    }

    /// Legacy cleanup alias — delegates to `end_session_and_cleanup`.
    pub async fn cleanup(&self) {
        self.end_session_and_cleanup().await;
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> Value {
        if request.jsonrpc != "2.0" {
            let err = McpError::InvalidRequest("jsonrpc must be \"2.0\"".to_string());
            return serde_json::to_value(err.to_json_rpc_error(request.id))
                .unwrap_or_default();
        }

        let id = request.id.clone();
        let result = self.dispatch_request(&request).await;

        match result {
            Ok(value) => {
                serde_json::to_value(JsonRpcResponse::new(id, value)).unwrap_or_default()
            }
            Err(e) => serde_json::to_value(e.to_json_rpc_error(id)).unwrap_or_default(),
        }
    }

    async fn dispatch_request(&self, request: &JsonRpcRequest) -> McpResult<Value> {
        match request.method.as_str() {
            // Lifecycle
            "initialize" => self.handle_initialize(request.params.clone()).await,
            "shutdown" => self.handle_shutdown().await,

            // Tools
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(request.params.clone()).await,

            // Resources
            "resources/list" => self.handle_resources_list().await,
            "resources/templates/list" => self.handle_resource_templates_list().await,
            "resources/read" => self.handle_resources_read(request.params.clone()).await,
            "resources/subscribe" => Ok(Value::Object(serde_json::Map::new())),
            "resources/unsubscribe" => Ok(Value::Object(serde_json::Map::new())),

            // Prompts
            "prompts/list" => self.handle_prompts_list().await,
            "prompts/get" => self.handle_prompts_get(request.params.clone()).await,

            // Ping
            "ping" => Ok(Value::Object(serde_json::Map::new())),

            _ => Err(McpError::MethodNotFound(request.method.clone())),
        }
    }

    async fn handle_notification(&self, notification: JsonRpcNotification) {
        match notification.method.as_str() {
            "notifications/initialized" | "initialized" => {
                // Auto-start session when client confirms connection.
                self.auto_session_started.store(true, Ordering::Relaxed);
                let mut session = self.session.lock().await;
                session.start_session_manual(None);
                eprintln!("Auto-session started on MCP initialized notification");
            }
            "notifications/cancelled" | "$/cancelRequest" => {
                // Cancellation — no-op for now.
            }
            _ => {
                // Unknown notification — ignore.
            }
        }
    }

    async fn handle_initialize(&self, params: Option<Value>) -> McpResult<Value> {
        let _init_params: InitializeParams = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::InvalidParams(e.to_string()))?
            .ok_or_else(|| {
                McpError::InvalidParams("Initialize params required".to_string())
            })?;

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: serde_json::json!({
                "tools": {},
                "resources": { "subscribe": true, "listChanged": true },
                "prompts": { "listChanged": true },
            }),
            server_info: serde_json::json!({
                "name": "agentic-comm-mcp",
                "version": env!("CARGO_PKG_VERSION"),
            }),
        };

        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    async fn handle_shutdown(&self) -> McpResult<Value> {
        let mut session = self.session.lock().await;

        // Auto-end session on shutdown.
        if self.auto_session_started.swap(false, Ordering::Relaxed) {
            let summary = session.end_session_manual(Some("Auto-ended on shutdown"));
            eprintln!(
                "Session ended on shutdown: {:.0}s, {} tool calls, {} conversation entries",
                summary.duration_secs,
                summary.tool_calls,
                summary.conversation_entries,
            );
        }

        self.shutdown_requested.store(true, Ordering::Relaxed);
        Ok(Value::Object(serde_json::Map::new()))
    }

    async fn handle_tools_list(&self) -> McpResult<Value> {
        let result = ToolListResult {
            tools: ToolRegistry::list_tools(),
            next_cursor: None,
        };
        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    /// Resolve expected auth token from environment.
    /// Canonical env var is AGENTIC_TOKEN; AGENTIC_COMM_TOKEN is retained
    /// for backward compatibility with older deployments.
    fn expected_auth_token() -> Option<String> {
        std::env::var("AGENTIC_TOKEN")
            .ok()
            .or_else(|| std::env::var("AGENTIC_COMM_TOKEN").ok())
    }

    /// Check if the MCP connection is authorized via AGENTIC_TOKEN (or legacy
    /// AGENTIC_COMM_TOKEN). If set, all tool calls must include a matching token
    /// in the _meta.token field. If not set, auth is disabled (open access).
    fn check_auth(params: &Option<Value>) -> McpResult<()> {
        if let Some(expected_token) = Self::expected_auth_token() {
            let provided_token = params
                .as_ref()
                .and_then(|p| p.get("_meta"))
                .and_then(|m| m.get("token"))
                .and_then(|t| t.as_str())
                .unwrap_or("");
            if provided_token != expected_token {
                return Err(McpError::Unauthorized);
            }
        }
        // If env var not set, auth is disabled
        Ok(())
    }

    async fn handle_tools_call(&self, params: Option<Value>) -> McpResult<Value> {
        // Verify auth token before dispatching any tool call.
        Self::check_auth(&params)?;

        let call_params: ToolCallParams = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::InvalidParams(e.to_string()))?
            .ok_or_else(|| {
                McpError::InvalidParams("Tool call params required".to_string())
            })?;

        let mut session = self.session.lock().await;

        // Two-tier error handling: ToolNotFound → protocol error; everything else → tool result
        let dispatch_result = ToolRegistry::dispatch(
            &call_params.name,
            &call_params.arguments,
            &mut session,
        );

        // Auto-log tool call (Phase 0: 20-Year Clock)
        {
            let summary: String = call_params
                .arguments
                .to_string()
                .chars()
                .take(200)
                .collect();
            let success = match &dispatch_result {
                Ok(ref r) => !r.is_error.unwrap_or(false),
                Err(McpError::ToolNotFound(_)) => false,
                Err(_) => false,
            };
            session.record_tool_call(&call_params.name, &summary, success);
        }

        match dispatch_result {
            Ok(result) => serde_json::to_value(result)
                .map_err(|e| McpError::InternalError(e.to_string())),
            Err(McpError::ToolNotFound(name)) => Err(McpError::ToolNotFound(name)),
            Err(e) => {
                // Non-protocol errors become tool execution errors
                let result = ToolCallResult::error(e.to_string());
                serde_json::to_value(result)
                    .map_err(|e| McpError::InternalError(e.to_string()))
            }
        }
    }

    // ── Resources ────────────────────────────────────────────────────

    async fn handle_resources_list(&self) -> McpResult<Value> {
        let result = ResourceListResult {
            resources: ResourceRegistry::list_resources(),
            next_cursor: None,
        };
        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    async fn handle_resource_templates_list(&self) -> McpResult<Value> {
        let result = ResourceRegistry::list_templates_result();
        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    async fn handle_resources_read(&self, params: Option<Value>) -> McpResult<Value> {
        let read_params: ResourceReadParams = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::InvalidParams(e.to_string()))?
            .ok_or_else(|| {
                McpError::InvalidParams("Resource read params required".to_string())
            })?;

        let result = ResourceRegistry::read(&read_params.uri, &self.session).await?;
        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    // ── Prompts ──────────────────────────────────────────────────────

    async fn handle_prompts_list(&self) -> McpResult<Value> {
        let result = PromptListResult {
            prompts: PromptRegistry::list_prompts(),
            next_cursor: None,
        };
        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }

    async fn handle_prompts_get(&self, params: Option<Value>) -> McpResult<Value> {
        let get_params: PromptGetParams = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::InvalidParams(e.to_string()))?
            .ok_or_else(|| {
                McpError::InvalidParams("Prompt get params required".to_string())
            })?;

        let result = PromptRegistry::get(&get_params.name, get_params.arguments)?;
        serde_json::to_value(result).map_err(|e| McpError::InternalError(e.to_string()))
    }
}
