//! Tool registration and dispatch for the MCP server.

use serde_json::{json, Value};

use crate::session::manager::SessionManager;
use crate::tools::communication_log::{CommunicationLogEntry, CommunicationLogInput};
use crate::tools::validation;
use crate::types::response::{ToolCallResult, ToolDefinition};
use crate::types::McpError;

use agentic_comm::{ChannelConfig, ChannelType, MessageFilter, MessageType};

/// Tool registry — lists all available tools and dispatches calls.
pub struct ToolRegistry;

impl ToolRegistry {
    /// Return definitions for all 17 tools.
    pub fn list_tools() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "send_message".to_string(),
                description: Some("Send a message to a channel or specific recipient".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Target channel ID"
                        },
                        "sender": {
                            "type": "string",
                            "description": "Sender identity"
                        },
                        "content": {
                            "type": "string",
                            "description": "Message body (1 byte to 1 MB)"
                        },
                        "message_type": {
                            "type": "string",
                            "description": "Message type: text, command, query, response, broadcast, notification, acknowledgment, error. Default: text",
                            "default": "text"
                        }
                    },
                    "required": ["channel_id", "sender", "content"]
                }),
            },
            ToolDefinition {
                name: "receive_messages".to_string(),
                description: Some(
                    "Retrieve pending or recent messages from a channel".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Channel ID to read from"
                        },
                        "recipient": {
                            "type": "string",
                            "description": "Optional recipient filter"
                        },
                        "since": {
                            "type": "string",
                            "description": "ISO 8601 timestamp — only return messages after this time"
                        }
                    },
                    "required": ["channel_id"]
                }),
            },
            ToolDefinition {
                name: "create_channel".to_string(),
                description: Some("Create a new communication channel".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Channel name (alphanumeric, hyphens, underscores; 1-128 chars)"
                        },
                        "channel_type": {
                            "type": "string",
                            "description": "Channel type: direct, group, broadcast, pubsub. Default: group",
                            "default": "group"
                        },
                        "max_participants": {
                            "type": "integer",
                            "description": "Maximum participants (0 = unlimited). Default: 0",
                            "default": 0
                        },
                        "ttl_seconds": {
                            "type": "integer",
                            "description": "Message TTL in seconds (0 = forever). Default: 0",
                            "default": 0
                        },
                        "persistence": {
                            "type": "boolean",
                            "description": "Whether to persist messages. Default: true",
                            "default": true
                        },
                        "encryption_required": {
                            "type": "boolean",
                            "description": "Whether encryption is required. Default: false",
                            "default": false
                        }
                    },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "list_channels".to_string(),
                description: Some("List all available communication channels".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "join_channel".to_string(),
                description: Some("Join an existing communication channel".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Channel ID to join"
                        },
                        "participant": {
                            "type": "string",
                            "description": "Name of the participant joining"
                        }
                    },
                    "required": ["channel_id", "participant"]
                }),
            },
            ToolDefinition {
                name: "leave_channel".to_string(),
                description: Some("Leave a communication channel".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Channel ID to leave"
                        },
                        "participant": {
                            "type": "string",
                            "description": "Name of the participant leaving"
                        }
                    },
                    "required": ["channel_id", "participant"]
                }),
            },
            ToolDefinition {
                name: "get_channel_info".to_string(),
                description: Some("Get detailed information about a channel".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Channel ID to inspect"
                        }
                    },
                    "required": ["channel_id"]
                }),
            },
            ToolDefinition {
                name: "subscribe".to_string(),
                description: Some(
                    "Subscribe to a pub/sub topic for message delivery".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "topic": {
                            "type": "string",
                            "description": "Topic to subscribe to (alphanumeric, hyphens, underscores, dots; 1-128 chars)"
                        },
                        "subscriber": {
                            "type": "string",
                            "description": "Subscriber identity"
                        }
                    },
                    "required": ["topic", "subscriber"]
                }),
            },
            ToolDefinition {
                name: "unsubscribe".to_string(),
                description: Some("Remove a subscription from a pub/sub topic".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "subscription_id": {
                            "type": "integer",
                            "description": "Subscription ID to remove"
                        }
                    },
                    "required": ["subscription_id"]
                }),
            },
            ToolDefinition {
                name: "publish".to_string(),
                description: Some(
                    "Publish a message to all subscribers of a topic".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "topic": {
                            "type": "string",
                            "description": "Topic to publish to"
                        },
                        "sender": {
                            "type": "string",
                            "description": "Publisher identity"
                        },
                        "content": {
                            "type": "string",
                            "description": "Message content (1 byte to 1 MB)"
                        }
                    },
                    "required": ["topic", "sender", "content"]
                }),
            },
            ToolDefinition {
                name: "broadcast".to_string(),
                description: Some(
                    "Send a message to all participants in a broadcast channel".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Broadcast channel ID"
                        },
                        "sender": {
                            "type": "string",
                            "description": "Sender identity"
                        },
                        "content": {
                            "type": "string",
                            "description": "Message content"
                        }
                    },
                    "required": ["channel_id", "sender", "content"]
                }),
            },
            ToolDefinition {
                name: "query_history".to_string(),
                description: Some("Search message history with filters".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Channel ID to query"
                        },
                        "since": {
                            "type": "string",
                            "description": "ISO 8601 timestamp — only messages after this time"
                        },
                        "before": {
                            "type": "string",
                            "description": "ISO 8601 timestamp — only messages before this time"
                        },
                        "sender": {
                            "type": "string",
                            "description": "Filter by sender"
                        },
                        "message_type": {
                            "type": "string",
                            "description": "Filter by message type"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum results (default: 100, range: 1-10000)",
                            "default": 100
                        }
                    },
                    "required": ["channel_id"]
                }),
            },
            ToolDefinition {
                name: "search_messages".to_string(),
                description: Some("Full-text search across all messages".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search text"
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum results (default: 20, range: 1-10000)",
                            "default": 20
                        }
                    },
                    "required": ["query"]
                }),
            },
            ToolDefinition {
                name: "get_message".to_string(),
                description: Some("Retrieve a specific message by ID".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message_id": {
                            "type": "integer",
                            "description": "Message ID to retrieve"
                        }
                    },
                    "required": ["message_id"]
                }),
            },
            ToolDefinition {
                name: "acknowledge_message".to_string(),
                description: Some(
                    "Mark a message as received and acknowledged".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message_id": {
                            "type": "integer",
                            "description": "Message ID to acknowledge"
                        },
                        "recipient": {
                            "type": "string",
                            "description": "Acknowledging participant"
                        }
                    },
                    "required": ["message_id", "recipient"]
                }),
            },
            ToolDefinition {
                name: "set_channel_config".to_string(),
                description: Some(
                    "Update configuration for a communication channel".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Channel ID to update"
                        },
                        "max_participants": {
                            "type": "integer",
                            "description": "Maximum participants (0 = unlimited)"
                        },
                        "ttl_seconds": {
                            "type": "integer",
                            "description": "Message TTL in seconds (0 = forever)"
                        },
                        "persistence": {
                            "type": "boolean",
                            "description": "Whether to persist messages"
                        },
                        "encryption_required": {
                            "type": "boolean",
                            "description": "Whether encryption is required"
                        }
                    },
                    "required": ["channel_id"]
                }),
            },
            ToolDefinition {
                name: "communication_log".to_string(),
                description: Some(
                    "Log intent and context behind communication actions (20-Year Clock)".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "intent": {
                            "type": "string",
                            "description": "Why the communication action is happening"
                        },
                        "observation": {
                            "type": "string",
                            "description": "What was noticed or concluded"
                        },
                        "related_message_id": {
                            "type": "integer",
                            "description": "Link to a related message ID"
                        },
                        "topic": {
                            "type": "string",
                            "description": "Category or topic (e.g., 'agent-coordination', 'debugging')"
                        }
                    },
                    "required": ["intent"]
                }),
            },
        ]
    }

    /// Dispatch a tool call to the appropriate handler.
    ///
    /// Validation runs first — if it fails, the error is returned as a
    /// `ToolCallResult` with `isError: true` (not a protocol-level error).
    pub fn dispatch(
        tool_name: &str,
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        // Run validation before dispatch. Validation errors become tool-level
        // errors with isError: true, not protocol-level JSON-RPC errors.
        let validation_result = match tool_name {
            "send_message" => validation::validate_send_message(params),
            "receive_messages" => validation::validate_receive_messages(params),
            "create_channel" => validation::validate_create_channel(params),
            "list_channels" => Ok(()), // No required params
            "join_channel" => validation::validate_join_channel(params),
            "leave_channel" => validation::validate_leave_channel(params),
            "get_channel_info" => validation::validate_get_channel_info(params),
            "subscribe" => validation::validate_subscribe(params),
            "unsubscribe" => validation::validate_unsubscribe(params),
            "publish" => validation::validate_publish(params),
            "broadcast" => validation::validate_broadcast(params),
            "query_history" => validation::validate_query_history(params),
            "search_messages" => validation::validate_search_messages(params),
            "get_message" => validation::validate_get_message(params),
            "acknowledge_message" => validation::validate_acknowledge_message(params),
            "set_channel_config" => validation::validate_set_channel_config(params),
            "communication_log" => validation::validate_communication_log(params),
            _ => return Err(McpError::ToolNotFound(tool_name.to_string())),
        };

        // If validation failed, return the error result directly
        if let Err(error_result) = validation_result {
            return Ok(error_result);
        }

        // Dispatch to actual handler
        match tool_name {
            "send_message" => Self::handle_send_message(params, session),
            "receive_messages" => Self::handle_receive_messages(params, session),
            "create_channel" => Self::handle_create_channel(params, session),
            "list_channels" => Self::handle_list_channels(session),
            "join_channel" => Self::handle_join_channel(params, session),
            "leave_channel" => Self::handle_leave_channel(params, session),
            "get_channel_info" => Self::handle_get_channel_info(params, session),
            "subscribe" => Self::handle_subscribe(params, session),
            "unsubscribe" => Self::handle_unsubscribe(params, session),
            "publish" => Self::handle_publish(params, session),
            "broadcast" => Self::handle_broadcast(params, session),
            "query_history" => Self::handle_query_history(params, session),
            "search_messages" => Self::handle_search_messages(params, session),
            "get_message" => Self::handle_get_message(params, session),
            "acknowledge_message" => Self::handle_acknowledge_message(params, session),
            "set_channel_config" => Self::handle_set_channel_config(params, session),
            "communication_log" => Self::handle_communication_log(params, session),
            _ => Err(McpError::ToolNotFound(tool_name.to_string())),
        }
    }

    // -----------------------------------------------------------------------
    // Individual tool handlers
    // -----------------------------------------------------------------------

    fn handle_send_message(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;
        let sender = params
            .get("sender")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("sender is required".to_string()))?;
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("content is required".to_string()))?;
        let msg_type_str = params
            .get("message_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");
        let msg_type: MessageType = msg_type_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        match session.store.send_message(channel_id, sender, content, msg_type) {
            Ok(msg) => {
                session.record_operation("send_message", Some(msg.id));
                Ok(ToolCallResult::json(&msg))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_receive_messages(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;
        let recipient = params.get("recipient").and_then(|v| v.as_str());
        let since = params
            .get("since")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        match session.store.receive_messages(channel_id, recipient, since) {
            Ok(msgs) => {
                session.record_operation("receive_messages", None);
                Ok(ToolCallResult::json(&msgs))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_create_channel(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("name is required".to_string()))?;
        let ch_type_str = params
            .get("channel_type")
            .and_then(|v| v.as_str())
            .unwrap_or("group");
        let ch_type: ChannelType = ch_type_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let config = ChannelConfig {
            max_participants: params
                .get("max_participants")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            ttl_seconds: params
                .get("ttl_seconds")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            persistence: params
                .get("persistence")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            encryption_required: params
                .get("encryption_required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            ..Default::default()
        };

        match session.store.create_channel(name, ch_type, Some(config)) {
            Ok(ch) => {
                session.record_operation("create_channel", Some(ch.id));
                Ok(ToolCallResult::json(&ch))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_list_channels(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channels = session.store.list_channels();
        session.record_operation("list_channels", None);
        Ok(ToolCallResult::json(&channels))
    }

    fn handle_join_channel(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;
        let participant = params
            .get("participant")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("participant is required".to_string()))?;

        match session.store.join_channel(channel_id, participant) {
            Ok(()) => {
                session.record_operation("join_channel", Some(channel_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "joined",
                    "channel_id": channel_id,
                    "participant": participant,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_leave_channel(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;
        let participant = params
            .get("participant")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("participant is required".to_string()))?;

        match session.store.leave_channel(channel_id, participant) {
            Ok(()) => {
                session.record_operation("leave_channel", Some(channel_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "left",
                    "channel_id": channel_id,
                    "participant": participant,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_get_channel_info(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;

        session.record_operation("get_channel_info", Some(channel_id));
        match session.store.get_channel(channel_id) {
            Some(ch) => Ok(ToolCallResult::json(&ch)),
            None => Ok(ToolCallResult::error(format!(
                "Channel {channel_id} not found"
            ))),
        }
    }

    fn handle_subscribe(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let topic = params
            .get("topic")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("topic is required".to_string()))?;
        let subscriber = params
            .get("subscriber")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("subscriber is required".to_string()))?;

        match session.store.subscribe(topic, subscriber) {
            Ok(sub) => {
                session.record_operation("subscribe", Some(sub.id));
                Ok(ToolCallResult::json(&sub))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_unsubscribe(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let subscription_id = params
            .get("subscription_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                McpError::InvalidParams("subscription_id is required".to_string())
            })?;

        match session.store.unsubscribe(subscription_id) {
            Ok(()) => {
                session.record_operation("unsubscribe", Some(subscription_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "unsubscribed",
                    "subscription_id": subscription_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_publish(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let topic = params
            .get("topic")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("topic is required".to_string()))?;
        let sender = params
            .get("sender")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("sender is required".to_string()))?;
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("content is required".to_string()))?;

        match session.store.publish(topic, sender, content) {
            Ok(msgs) => {
                session.record_operation("publish", None);
                Ok(ToolCallResult::json(&json!({
                    "status": "published",
                    "delivered_count": msgs.len(),
                    "topic": topic,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_broadcast(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;
        let sender = params
            .get("sender")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("sender is required".to_string()))?;
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("content is required".to_string()))?;

        match session.store.broadcast(channel_id, sender, content) {
            Ok(msgs) => {
                session.record_operation("broadcast", Some(channel_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "broadcast",
                    "delivered_count": msgs.len(),
                    "channel_id": channel_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_query_history(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;

        let since = params
            .get("since")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));
        let before = params
            .get("before")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));
        let sender = params
            .get("sender")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let msg_type = params
            .get("message_type")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<MessageType>().ok());
        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);

        let filter = MessageFilter {
            since,
            before,
            sender,
            message_type: msg_type,
            limit,
        };

        let results = session.store.query_history(channel_id, &filter);
        session.record_operation("query_history", Some(channel_id));
        Ok(ToolCallResult::json(&results))
    }

    fn handle_search_messages(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("query is required".to_string()))?;
        let max_results = params
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;

        let results = session.store.search_messages(query, max_results);
        session.record_operation("search_messages", None);
        Ok(ToolCallResult::json(&results))
    }

    fn handle_get_message(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let message_id = params
            .get("message_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("message_id is required".to_string()))?;

        session.record_operation("get_message", Some(message_id));
        match session.store.get_message(message_id) {
            Some(msg) => Ok(ToolCallResult::json(&msg)),
            None => Ok(ToolCallResult::error(format!(
                "Message {message_id} not found"
            ))),
        }
    }

    fn handle_acknowledge_message(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let message_id = params
            .get("message_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("message_id is required".to_string()))?;
        let recipient = params
            .get("recipient")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("recipient is required".to_string()))?;

        match session.store.acknowledge_message(message_id, recipient) {
            Ok(()) => {
                session.record_operation("acknowledge_message", Some(message_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "acknowledged",
                    "message_id": message_id,
                    "recipient": recipient,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_set_channel_config(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;

        // Get current config to use as defaults
        let current = session
            .store
            .get_channel(channel_id)
            .ok_or_else(|| McpError::InvalidParams(format!("Channel {channel_id} not found")))?;

        let config = ChannelConfig {
            max_participants: params
                .get("max_participants")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32)
                .unwrap_or(current.config.max_participants),
            ttl_seconds: params
                .get("ttl_seconds")
                .and_then(|v| v.as_u64())
                .unwrap_or(current.config.ttl_seconds),
            persistence: params
                .get("persistence")
                .and_then(|v| v.as_bool())
                .unwrap_or(current.config.persistence),
            encryption_required: params
                .get("encryption_required")
                .and_then(|v| v.as_bool())
                .unwrap_or(current.config.encryption_required),
            ..current.config
        };

        match session.store.set_channel_config(channel_id, config) {
            Ok(()) => {
                session.record_operation("set_channel_config", Some(channel_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "updated",
                    "channel_id": channel_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_communication_log(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let input: CommunicationLogInput = serde_json::from_value(params.clone())
            .map_err(|e| McpError::InvalidParams(e.to_string()))?;

        let entry = CommunicationLogEntry::from_input(&input);
        session.log_communication_context(entry.clone());
        session.record_operation("communication_log", None);

        Ok(ToolCallResult::json(&json!({
            "status": "logged",
            "timestamp": entry.timestamp,
            "intent": entry.intent,
        })))
    }
}
