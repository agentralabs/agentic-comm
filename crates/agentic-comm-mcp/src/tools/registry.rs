//! Tool registration and dispatch for the MCP server.

use serde_json::{json, Value};

use crate::session::manager::SessionManager;
use crate::tools::communication_log::{CommunicationLogEntry, CommunicationLogInput};
use crate::tools::validation;
use crate::types::response::{ToolCallResult, ToolDefinition};
use crate::types::McpError;

use agentic_comm::{ChannelConfig, ChannelType, MessageFilter, MessageType, CommTrustLevel, ConsentScope, AffectState, UrgencyLevel, TemporalTarget, CollectiveDecisionMode};

/// Tool registry — lists all available tools and dispatches calls.
pub struct ToolRegistry;

impl ToolRegistry {
    /// Return definitions for all 27 tools.
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
            // ---------------------------------------------------------------
            // Consent tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "manage_consent".to_string(),
                description: Some(
                    "Manage consent between agents (grant or revoke)".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": "Consent action: grant or revoke",
                            "enum": ["grant", "revoke"]
                        },
                        "grantor": {
                            "type": "string",
                            "description": "Agent granting or revoking consent"
                        },
                        "grantee": {
                            "type": "string",
                            "description": "Agent receiving or losing consent"
                        },
                        "scope": {
                            "type": "string",
                            "description": "Consent scope (e.g., receive-messages, join-channel, receive-semantic, receive-affect, telepathic-access, hive-formation, mind-meld, federation)"
                        }
                    },
                    "required": ["action", "grantor", "grantee", "scope"]
                }),
            },
            ToolDefinition {
                name: "check_consent".to_string(),
                description: Some(
                    "Check if consent is granted between agents for a scope".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "grantor": {
                            "type": "string",
                            "description": "Agent who may have granted consent"
                        },
                        "grantee": {
                            "type": "string",
                            "description": "Agent who may have received consent"
                        },
                        "scope": {
                            "type": "string",
                            "description": "Consent scope to check"
                        }
                    },
                    "required": ["grantor", "grantee", "scope"]
                }),
            },
            // ---------------------------------------------------------------
            // Trust tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "set_trust_level".to_string(),
                description: Some("Set trust level for an agent".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent_id": {
                            "type": "string",
                            "description": "Agent identity to set trust for"
                        },
                        "level": {
                            "type": "string",
                            "description": "Trust level: none, minimal, basic, standard, high, full, absolute",
                            "enum": ["none", "minimal", "basic", "standard", "high", "full", "absolute"]
                        }
                    },
                    "required": ["agent_id", "level"]
                }),
            },
            ToolDefinition {
                name: "get_trust_level".to_string(),
                description: Some("Get trust level for an agent".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent_id": {
                            "type": "string",
                            "description": "Agent identity to query trust for"
                        }
                    },
                    "required": ["agent_id"]
                }),
            },
            // ---------------------------------------------------------------
            // Temporal messaging tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "schedule_message".to_string(),
                description: Some(
                    "Schedule a message for future delivery".to_string(),
                ),
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
                            "description": "Message body"
                        },
                        "delay_seconds": {
                            "type": "integer",
                            "description": "Deliver after this many seconds from now"
                        },
                        "deliver_at": {
                            "type": "string",
                            "description": "ISO 8601 timestamp for delivery"
                        }
                    },
                    "required": ["channel_id", "sender", "content"]
                }),
            },
            ToolDefinition {
                name: "list_scheduled".to_string(),
                description: Some(
                    "List all scheduled temporal messages".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // ---------------------------------------------------------------
            // Hive mind tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "form_hive".to_string(),
                description: Some("Form a new hive mind group".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Hive mind name"
                        },
                        "coordinator": {
                            "type": "string",
                            "description": "Coordinator agent identity"
                        },
                        "decision_mode": {
                            "type": "string",
                            "description": "Decision mode: coordinator_decides, majority_vote, unanimous, weighted_vote. Default: coordinator_decides",
                            "default": "coordinator_decides"
                        }
                    },
                    "required": ["name", "coordinator"]
                }),
            },
            // ---------------------------------------------------------------
            // Stats tool
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "get_stats".to_string(),
                description: Some(
                    "Get comprehensive communication store statistics".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // ---------------------------------------------------------------
            // Affect tool
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "send_affect".to_string(),
                description: Some(
                    "Send a message with emotional/affect context".to_string(),
                ),
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
                            "description": "Message body"
                        },
                        "valence": {
                            "type": "number",
                            "description": "Emotional valence (-1.0 negative to 1.0 positive)"
                        },
                        "arousal": {
                            "type": "number",
                            "description": "Emotional arousal (0.0 calm to 1.0 excited)"
                        },
                        "urgency": {
                            "type": "string",
                            "description": "Urgency level: none, low, medium, high, critical",
                            "default": "none"
                        }
                    },
                    "required": ["channel_id", "sender", "content"]
                }),
            },
            // ---------------------------------------------------------------
            // Grounding tool
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_ground".to_string(),
                description: Some(
                    "Ground a claim against the communication store".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "claim": {
                            "type": "string",
                            "description": "The claim to verify against stored communications"
                        }
                    },
                    "required": ["claim"]
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
            "manage_consent" => validation::validate_manage_consent(params),
            "check_consent" => validation::validate_check_consent(params),
            "set_trust_level" => validation::validate_set_trust_level(params),
            "get_trust_level" => validation::validate_get_trust_level(params),
            "schedule_message" => validation::validate_schedule_message(params),
            "list_scheduled" => validation::validate_list_scheduled(params),
            "form_hive" => validation::validate_form_hive(params),
            "get_stats" => validation::validate_get_stats(params),
            "send_affect" => validation::validate_send_affect(params),
            "comm_ground" => (|| {
                let claim = validation::require_string(params, "claim")?;
                validation::validate_query(claim)?;
                Ok(())
            })(),
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
            "manage_consent" => Self::handle_manage_consent(params, session),
            "check_consent" => Self::handle_check_consent(params, session),
            "set_trust_level" => Self::handle_set_trust_level(params, session),
            "get_trust_level" => Self::handle_get_trust_level(params, session),
            "schedule_message" => Self::handle_schedule_message(params, session),
            "list_scheduled" => Self::handle_list_scheduled(session),
            "form_hive" => Self::handle_form_hive(params, session),
            "get_stats" => Self::handle_get_stats(session),
            "send_affect" => Self::handle_send_affect(params, session),
            "comm_ground" => Self::handle_comm_ground(params, session),
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

    // -----------------------------------------------------------------------
    // Consent handlers
    // -----------------------------------------------------------------------

    fn handle_manage_consent(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("action is required".to_string()))?;
        let grantor = params
            .get("grantor")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("grantor is required".to_string()))?;
        let grantee = params
            .get("grantee")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("grantee is required".to_string()))?;
        let scope_str = params
            .get("scope")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("scope is required".to_string()))?;
        let scope: ConsentScope = scope_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        match action {
            "grant" => {
                match session.store.grant_consent(grantor, grantee, scope, None, None) {
                    Ok(_entry) => {
                        session.record_operation("manage_consent", None);
                        Ok(ToolCallResult::json(&json!({
                            "status": "granted",
                            "action": "grant",
                            "grantor": grantor,
                            "grantee": grantee,
                            "scope": scope_str,
                        })))
                    }
                    Err(e) => Ok(ToolCallResult::error(e.to_string())),
                }
            }
            "revoke" => {
                match session.store.revoke_consent(grantor, grantee, &scope) {
                    Ok(()) => {
                        session.record_operation("manage_consent", None);
                        Ok(ToolCallResult::json(&json!({
                            "status": "revoked",
                            "action": "revoke",
                            "grantor": grantor,
                            "grantee": grantee,
                            "scope": scope_str,
                        })))
                    }
                    Err(e) => Ok(ToolCallResult::error(e.to_string())),
                }
            }
            _ => Ok(ToolCallResult::error(format!(
                "Unknown consent action: {action}. Must be 'grant' or 'revoke'"
            ))),
        }
    }

    fn handle_check_consent(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let grantor = params
            .get("grantor")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("grantor is required".to_string()))?;
        let grantee = params
            .get("grantee")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("grantee is required".to_string()))?;
        let scope_str = params
            .get("scope")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("scope is required".to_string()))?;
        let scope: ConsentScope = scope_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let granted = session.store.check_consent(grantor, grantee, &scope);
        session.record_operation("check_consent", None);
        Ok(ToolCallResult::json(&json!({
            "grantor": grantor,
            "grantee": grantee,
            "scope": scope_str,
            "granted": granted,
        })))
    }

    // -----------------------------------------------------------------------
    // Trust handlers
    // -----------------------------------------------------------------------

    fn handle_set_trust_level(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let agent_id = params
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("agent_id is required".to_string()))?;
        let level_str = params
            .get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("level is required".to_string()))?;
        let level: CommTrustLevel = level_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        match session.store.set_trust_level(agent_id, level) {
            Ok(()) => {
                session.record_operation("set_trust_level", None);
                Ok(ToolCallResult::json(&json!({
                    "status": "updated",
                    "agent_id": agent_id,
                    "level": level_str,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_get_trust_level(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let agent_id = params
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("agent_id is required".to_string()))?;

        let level = session.store.get_trust_level(agent_id);
        session.record_operation("get_trust_level", None);
        Ok(ToolCallResult::json(&json!({
            "agent_id": agent_id,
            "level": format!("{:?}", level),
        })))
    }

    // -----------------------------------------------------------------------
    // Temporal messaging handlers
    // -----------------------------------------------------------------------

    fn handle_schedule_message(
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

        let target = if let Some(delay) = params.get("delay_seconds").and_then(|v| v.as_u64()) {
            TemporalTarget::FutureRelative { delay_seconds: delay }
        } else if let Some(ts) = params.get("deliver_at").and_then(|v| v.as_str()) {
            TemporalTarget::FutureAbsolute { deliver_at: ts.to_string() }
        } else {
            TemporalTarget::Immediate
        };

        match session.store.schedule_message(channel_id, sender, content, target, None) {
            Ok(temporal_msg) => {
                let tid = temporal_msg.id;
                let result = ToolCallResult::json(&temporal_msg);
                session.record_operation("schedule_message", Some(tid));
                Ok(result)
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_list_scheduled(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let scheduled: Vec<_> = session.store.list_scheduled()
            .into_iter()
            .cloned()
            .collect();
        session.record_operation("list_scheduled", None);
        Ok(ToolCallResult::json(&scheduled))
    }

    // -----------------------------------------------------------------------
    // Hive mind handler
    // -----------------------------------------------------------------------

    fn handle_form_hive(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("name is required".to_string()))?;
        let coordinator = params
            .get("coordinator")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("coordinator is required".to_string()))?;
        let decision_mode_str = params
            .get("decision_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("coordinator_decides");
        let decision_mode = match decision_mode_str {
            "coordinator_decides" => CollectiveDecisionMode::CoordinatorDecides,
            "majority" => CollectiveDecisionMode::Majority,
            "unanimous" => CollectiveDecisionMode::Unanimous,
            "consensus" => CollectiveDecisionMode::Consensus,
            other => return Ok(ToolCallResult::error(format!(
                "Unknown decision mode: {other}. Must be: coordinator_decides, majority, unanimous, consensus"
            ))),
        };

        match session.store.form_hive(name, coordinator, decision_mode) {
            Ok(hive) => {
                let hid = hive.id;
                let result = ToolCallResult::json(&hive);
                session.record_operation("form_hive", Some(hid));
                Ok(result)
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    // -----------------------------------------------------------------------
    // Stats handler
    // -----------------------------------------------------------------------

    fn handle_get_stats(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let stats = session.store.stats();
        session.record_operation("get_stats", None);
        Ok(ToolCallResult::json(&stats))
    }

    // -----------------------------------------------------------------------
    // Affect handler
    // -----------------------------------------------------------------------

    fn handle_send_affect(
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

        let valence = params.get("valence").and_then(|v| v.as_f64());
        let arousal = params.get("arousal").and_then(|v| v.as_f64());
        let urgency_str = params
            .get("urgency")
            .and_then(|v| v.as_str())
            .unwrap_or("normal");
        let urgency: UrgencyLevel = urgency_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        let affect = AffectState {
            valence: valence.unwrap_or(0.0),
            arousal: arousal.unwrap_or(0.0),
            urgency,
            ..Default::default()
        };

        match session.store.send_affect_message(channel_id, sender, content, affect) {
            Ok(msg) => {
                session.record_operation("send_affect", Some(msg.id));
                Ok(ToolCallResult::json(&msg))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    // -----------------------------------------------------------------------
    // Grounding handler
    // -----------------------------------------------------------------------

    fn handle_comm_ground(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let claim = params
            .get("claim")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("claim is required".to_string()))?;

        let result = session.store.ground_claim(claim);
        session.record_operation("comm_ground", None);
        Ok(ToolCallResult::json(&result))
    }
}
