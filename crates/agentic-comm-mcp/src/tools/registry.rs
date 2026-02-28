//! Tool registration and dispatch for the MCP server.

use serde_json::{json, Value};

use crate::session::manager::SessionManager;
use crate::tools::communication_log::{CommunicationLogEntry, CommunicationLogInput};
use crate::tools::validation;
use crate::types::response::{ToolCallResult, ToolDefinition};
use crate::types::McpError;

use agentic_comm::{ChannelConfig, ChannelType, MessageFilter, MessageType, MessagePriority, CommTrustLevel, ConsentScope, AffectState, UrgencyLevel, TemporalTarget, CollectiveDecisionMode, FederationPolicy, FederatedZone, HiveRole, CommKeyPair, EncryptionKey, EncryptedPayload, KeyEntry, CommWorkspace, WorkspaceRole};

/// Tool registry — lists all available tools and dispatches calls.
pub struct ToolRegistry;

impl ToolRegistry {
    /// Return definitions for all 95 tools.
    pub fn list_tools() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "comm_send_message".to_string(),
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
                name: "comm_receive_messages".to_string(),
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
                name: "comm_create_channel".to_string(),
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
                name: "comm_list_channels".to_string(),
                description: Some("List all available communication channels".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "comm_join_channel".to_string(),
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
                name: "comm_leave_channel".to_string(),
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
                name: "comm_get_channel_info".to_string(),
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
                name: "comm_subscribe".to_string(),
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
                name: "comm_unsubscribe".to_string(),
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
                name: "comm_publish".to_string(),
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
                name: "comm_broadcast".to_string(),
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
                name: "comm_query_history".to_string(),
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
                name: "comm_search_messages".to_string(),
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
                name: "comm_get_message".to_string(),
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
                name: "comm_acknowledge_message".to_string(),
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
                name: "comm_set_channel_config".to_string(),
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
                name: "comm_communication_log".to_string(),
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
                name: "comm_manage_consent".to_string(),
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
                name: "comm_check_consent".to_string(),
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
                name: "comm_set_trust_level".to_string(),
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
                name: "comm_get_trust_level".to_string(),
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
                name: "comm_schedule_message".to_string(),
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
                name: "comm_list_scheduled".to_string(),
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
                name: "comm_form_hive".to_string(),
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
                name: "comm_get_stats".to_string(),
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
                name: "comm_send_affect".to_string(),
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
            // ---------------------------------------------------------------
            // Evidence search tool
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_evidence".to_string(),
                description: Some(
                    "Search the communication store for evidence matching a query".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query to find matching messages, channels, and agents"
                        }
                    },
                    "required": ["query"]
                }),
            },
            // ---------------------------------------------------------------
            // Suggestion tool
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_suggest".to_string(),
                description: Some(
                    "Get fuzzy suggestions for agent names, channel names, or message content".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query for fuzzy matching"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of suggestions to return (default: 10)",
                            "default": 10
                        }
                    },
                    "required": ["query"]
                }),
            },
            // ---------------------------------------------------------------
            // Consent listing tool
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_list_consent_gates".to_string(),
                description: Some(
                    "List all consent gates, optionally filtered by agent".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent": {
                            "type": "string",
                            "description": "Optional agent ID to filter consent gates"
                        }
                    }
                }),
            },
            // ---------------------------------------------------------------
            // Trust listing tool
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_list_trust_levels".to_string(),
                description: Some(
                    "List all trust level overrides".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // ---------------------------------------------------------------
            // Temporal management tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_cancel_scheduled".to_string(),
                description: Some(
                    "Cancel a scheduled temporal message".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "temporal_id": {
                            "type": "integer",
                            "description": "Temporal message ID to cancel"
                        }
                    },
                    "required": ["temporal_id"]
                }),
            },
            ToolDefinition {
                name: "comm_deliver_pending".to_string(),
                description: Some(
                    "Deliver all pending temporal messages that are due".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // ---------------------------------------------------------------
            // Federation tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_configure_federation".to_string(),
                description: Some(
                    "Configure federation settings".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "enabled": {
                            "type": "boolean",
                            "description": "Whether federation is enabled"
                        },
                        "local_zone": {
                            "type": "string",
                            "description": "Local zone identifier"
                        },
                        "policy": {
                            "type": "string",
                            "description": "Default federation policy: allow, deny, selective",
                            "enum": ["allow", "deny", "selective"]
                        }
                    },
                    "required": ["enabled", "local_zone", "policy"]
                }),
            },
            ToolDefinition {
                name: "comm_add_federated_zone".to_string(),
                description: Some(
                    "Add a federated zone".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "zone_id": {
                            "type": "string",
                            "description": "Zone identifier"
                        },
                        "name": {
                            "type": "string",
                            "description": "Human-readable zone name"
                        },
                        "endpoint": {
                            "type": "string",
                            "description": "Endpoint URL or address"
                        },
                        "policy": {
                            "type": "string",
                            "description": "Zone policy: allow, deny, selective",
                            "enum": ["allow", "deny", "selective"]
                        },
                        "trust_level": {
                            "type": "string",
                            "description": "Trust level for this zone: none, minimal, basic, standard, high, full, absolute",
                            "enum": ["none", "minimal", "basic", "standard", "high", "full", "absolute"]
                        }
                    },
                    "required": ["zone_id"]
                }),
            },
            ToolDefinition {
                name: "comm_remove_federated_zone".to_string(),
                description: Some(
                    "Remove a federated zone".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "zone_id": {
                            "type": "string",
                            "description": "Zone identifier to remove"
                        }
                    },
                    "required": ["zone_id"]
                }),
            },
            ToolDefinition {
                name: "comm_list_federated_zones".to_string(),
                description: Some(
                    "List all federated zones".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // ---------------------------------------------------------------
            // Hive mind management tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_dissolve_hive".to_string(),
                description: Some(
                    "Dissolve a hive mind".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "hive_id": {
                            "type": "integer",
                            "description": "Hive mind ID to dissolve"
                        }
                    },
                    "required": ["hive_id"]
                }),
            },
            ToolDefinition {
                name: "comm_join_hive".to_string(),
                description: Some(
                    "Join a hive mind".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "hive_id": {
                            "type": "integer",
                            "description": "Hive mind ID to join"
                        },
                        "agent_id": {
                            "type": "string",
                            "description": "Agent identity joining the hive"
                        },
                        "role": {
                            "type": "string",
                            "description": "Role in the hive: coordinator, member, observer. Default: member",
                            "default": "member",
                            "enum": ["coordinator", "member", "observer"]
                        }
                    },
                    "required": ["hive_id", "agent_id"]
                }),
            },
            ToolDefinition {
                name: "comm_leave_hive".to_string(),
                description: Some(
                    "Leave a hive mind".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "hive_id": {
                            "type": "integer",
                            "description": "Hive mind ID to leave"
                        },
                        "agent_id": {
                            "type": "string",
                            "description": "Agent identity leaving the hive"
                        }
                    },
                    "required": ["hive_id", "agent_id"]
                }),
            },
            ToolDefinition {
                name: "comm_list_hives".to_string(),
                description: Some(
                    "List all hive minds".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "comm_get_hive".to_string(),
                description: Some(
                    "Get hive mind details".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "hive_id": {
                            "type": "integer",
                            "description": "Hive mind ID to retrieve"
                        }
                    },
                    "required": ["hive_id"]
                }),
            },
            // ---------------------------------------------------------------
            // Communication log tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_log_communication".to_string(),
                description: Some(
                    "Log a communication entry".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Communication content to log"
                        },
                        "role": {
                            "type": "string",
                            "description": "Role of the communicator (e.g., user, agent, system)"
                        },
                        "topic": {
                            "type": "string",
                            "description": "Optional topic or category"
                        },
                        "linked_message_id": {
                            "type": "integer",
                            "description": "Optional linked message ID"
                        },
                        "affect": {
                            "type": "object",
                            "description": "Optional affect state with valence, arousal, urgency",
                            "properties": {
                                "valence": {
                                    "type": "number",
                                    "description": "Emotional valence (-1.0 to 1.0)"
                                },
                                "arousal": {
                                    "type": "number",
                                    "description": "Emotional arousal (0.0 to 1.0)"
                                },
                                "urgency": {
                                    "type": "string",
                                    "description": "Urgency level: background, low, normal, high, urgent, critical"
                                }
                            }
                        }
                    },
                    "required": ["content", "role"]
                }),
            },
            ToolDefinition {
                name: "comm_get_comm_log".to_string(),
                description: Some(
                    "Get communication log entries".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of entries to return (default: all, range: 1-10000)"
                        }
                    }
                }),
            },
            // ---------------------------------------------------------------
            // Audit log tool
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_get_audit_log".to_string(),
                description: Some(
                    "Get audit log entries".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of entries to return (default: all, range: 1-10000)"
                        }
                    }
                }),
            },
            // ---------------------------------------------------------------
            // Semantic tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_send_semantic".to_string(),
                description: Some(
                    "Send a structured semantic message to a channel".to_string(),
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
                            "description": "Sender agent identity"
                        },
                        "topic": {
                            "type": "string",
                            "description": "Semantic topic (1-128 chars, alphanumeric + hyphen + underscore + dot)"
                        },
                        "focus_nodes": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Semantic focus nodes (default: [])"
                        },
                        "depth": {
                            "type": "integer",
                            "description": "Semantic depth level (default: 1)"
                        }
                    },
                    "required": ["channel_id", "sender", "topic"]
                }),
            },
            ToolDefinition {
                name: "comm_extract_semantic".to_string(),
                description: Some(
                    "Extract semantic structure from an existing message".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message_id": {
                            "type": "integer",
                            "description": "ID of the message to extract semantics from"
                        }
                    },
                    "required": ["message_id"]
                }),
            },
            ToolDefinition {
                name: "comm_graft_semantic".to_string(),
                description: Some(
                    "Graft (merge) two semantic layers together".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "source_id": {
                            "type": "integer",
                            "description": "Source semantic operation ID"
                        },
                        "target_id": {
                            "type": "integer",
                            "description": "Target semantic operation ID"
                        },
                        "strategy": {
                            "type": "string",
                            "description": "Merge strategy: union, intersect, override (default: union)"
                        }
                    },
                    "required": ["source_id", "target_id"]
                }),
            },
            ToolDefinition {
                name: "comm_list_semantic_conflicts".to_string(),
                description: Some(
                    "List semantic conflicts, optionally filtered by channel or severity".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Filter by channel ID"
                        },
                        "severity": {
                            "type": "string",
                            "description": "Filter by severity: low, medium, high, critical"
                        }
                    }
                }),
            },
            // ---------------------------------------------------------------
            // Affect tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_get_affect_state".to_string(),
                description: Some(
                    "Get the current affect state for an agent".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent_id": {
                            "type": "string",
                            "description": "Agent identity to query"
                        }
                    },
                    "required": ["agent_id"]
                }),
            },
            ToolDefinition {
                name: "comm_set_affect_resistance".to_string(),
                description: Some(
                    "Set the affect resistance threshold (0.0-1.0)".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "resistance": {
                            "type": "number",
                            "description": "Resistance threshold between 0.0 and 1.0 (default: 0.5)"
                        }
                    },
                    "required": ["resistance"]
                }),
            },
            // ---------------------------------------------------------------
            // Hive extension tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_hive_think".to_string(),
                description: Some(
                    "Broadcast a question to all hive members and return aggregated response".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "hive_id": {
                            "type": "integer",
                            "description": "Hive mind ID"
                        },
                        "question": {
                            "type": "string",
                            "description": "Question to broadcast to hive members"
                        },
                        "timeout_ms": {
                            "type": "integer",
                            "description": "Timeout in milliseconds (default: 5000)"
                        }
                    },
                    "required": ["hive_id", "question"]
                }),
            },
            ToolDefinition {
                name: "comm_initiate_meld".to_string(),
                description: Some(
                    "Initiate a deep mind-meld session with a partner agent".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "partner_id": {
                            "type": "string",
                            "description": "Partner agent identity to meld with"
                        },
                        "depth": {
                            "type": "string",
                            "description": "Meld depth: shallow, medium, deep (default: shallow)"
                        },
                        "duration_ms": {
                            "type": "integer",
                            "description": "Meld duration in milliseconds (default: 10000)"
                        }
                    },
                    "required": ["partner_id"]
                }),
            },
            // ---------------------------------------------------------------
            // Consent flow tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_list_pending_consent".to_string(),
                description: Some(
                    "List pending consent requests, optionally filtered by agent or type".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent_id": {
                            "type": "string",
                            "description": "Filter by agent identity"
                        },
                        "consent_type": {
                            "type": "string",
                            "description": "Filter by consent type"
                        }
                    }
                }),
            },
            ToolDefinition {
                name: "comm_respond_consent".to_string(),
                description: Some(
                    "Respond to a pending consent request".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "request_id": {
                            "type": "string",
                            "description": "Consent request ID to respond to"
                        },
                        "response": {
                            "type": "string",
                            "description": "Response: approve, deny, defer"
                        }
                    },
                    "required": ["request_id", "response"]
                }),
            },
            // ---------------------------------------------------------------
            // Query tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_query_relationships".to_string(),
                description: Some(
                    "Query relationships between agents (trust, consent, hive membership)".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent_id": {
                            "type": "string",
                            "description": "Agent identity to query relationships for"
                        },
                        "relationship_type": {
                            "type": "string",
                            "description": "Filter by type: trust, consent, hive (default: all)"
                        },
                        "depth": {
                            "type": "integer",
                            "description": "Traversal depth (default: 1)"
                        }
                    },
                    "required": ["agent_id"]
                }),
            },
            ToolDefinition {
                name: "comm_query_echoes".to_string(),
                description: Some(
                    "Query conversation echoes (messages that reference or relate to a message)".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message_id": {
                            "type": "integer",
                            "description": "Source message ID to find echoes for"
                        },
                        "depth": {
                            "type": "integer",
                            "description": "Echo depth (default: 1)"
                        }
                    },
                    "required": ["message_id"]
                }),
            },
            ToolDefinition {
                name: "comm_query_conversations".to_string(),
                description: Some(
                    "Query conversation summaries, optionally filtered by channel or participant".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Filter by channel ID"
                        },
                        "participant": {
                            "type": "string",
                            "description": "Filter by participant agent identity"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of summaries to return (default: 50, range: 1-10000)"
                        }
                    }
                }),
            },
            // ---------------------------------------------------------------
            // Federation extension tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_get_federation_status".to_string(),
                description: Some(
                    "Get the current federation status and zone information".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "comm_set_federation_policy".to_string(),
                description: Some(
                    "Set federation policy for a specific zone".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "zone_id": {
                            "type": "string",
                            "description": "Zone ID to configure"
                        },
                        "allow_semantic": {
                            "type": "boolean",
                            "description": "Allow semantic operations in this zone (default: true)"
                        },
                        "allow_affect": {
                            "type": "boolean",
                            "description": "Allow affect propagation in this zone (default: true)"
                        },
                        "allow_hive": {
                            "type": "boolean",
                            "description": "Allow hive operations in this zone (default: true)"
                        },
                        "max_message_size": {
                            "type": "integer",
                            "description": "Maximum message size in bytes (default: 1048576)"
                        }
                    },
                    "required": ["zone_id"]
                }),
            },
            // ---------------------------------------------------------------
            // Crypto / encryption tools
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_generate_keypair".to_string(),
                description: Some(
                    "Generate an Ed25519 key pair for cryptographic signing".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "comm_encrypt_message".to_string(),
                description: Some(
                    "Encrypt content with ChaCha20-Poly1305".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Plaintext content to encrypt"
                        }
                    },
                    "required": ["content"]
                }),
            },
            ToolDefinition {
                name: "comm_decrypt_message".to_string(),
                description: Some(
                    "Decrypt content encrypted with ChaCha20-Poly1305".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "ciphertext": {
                            "type": "string",
                            "description": "Hex-encoded ciphertext"
                        },
                        "nonce": {
                            "type": "string",
                            "description": "Hex-encoded 12-byte nonce"
                        },
                        "epoch": {
                            "type": "integer",
                            "description": "Key epoch used for encryption (default: 1)"
                        }
                    },
                    "required": ["ciphertext", "nonce"]
                }),
            },
            ToolDefinition {
                name: "comm_verify_signature".to_string(),
                description: Some(
                    "Verify an Ed25519 signature against a public key".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "public_key": {
                            "type": "string",
                            "description": "Hex-encoded Ed25519 public key (64 hex chars)"
                        },
                        "content": {
                            "type": "string",
                            "description": "Original content that was signed"
                        },
                        "signature": {
                            "type": "string",
                            "description": "Hex-encoded Ed25519 signature (128 hex chars)"
                        }
                    },
                    "required": ["public_key", "content", "signature"]
                }),
            },
            ToolDefinition {
                name: "comm_get_public_key".to_string(),
                description: Some(
                    "Get the current Ed25519 public key".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // ---------------------------------------------------------------
            // Messaging: replies, threads, priority
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_send_reply".to_string(),
                description: Some("Send a reply linked to a parent message".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Target channel ID"
                        },
                        "reply_to_id": {
                            "type": "integer",
                            "description": "ID of the message being replied to"
                        },
                        "sender": {
                            "type": "string",
                            "description": "Sender identity"
                        },
                        "content": {
                            "type": "string",
                            "description": "Reply message body (1 byte to 1 MB)"
                        },
                        "message_type": {
                            "type": "string",
                            "description": "Message type: text, command, query, response, broadcast, notification, acknowledgment, error. Default: text",
                            "default": "text"
                        }
                    },
                    "required": ["channel_id", "reply_to_id", "sender", "content"]
                }),
            },
            ToolDefinition {
                name: "comm_get_thread".to_string(),
                description: Some("Retrieve all messages in a thread by thread ID".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "thread_id": {
                            "type": "string",
                            "description": "Thread identifier"
                        }
                    },
                    "required": ["thread_id"]
                }),
            },
            ToolDefinition {
                name: "comm_get_replies".to_string(),
                description: Some("Retrieve all direct replies to a specific message".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message_id": {
                            "type": "integer",
                            "description": "ID of the parent message"
                        }
                    },
                    "required": ["message_id"]
                }),
            },
            ToolDefinition {
                name: "comm_send_with_priority".to_string(),
                description: Some("Send a message with a specific priority level".to_string()),
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
                        "priority": {
                            "type": "string",
                            "description": "Priority level: low, normal, high, urgent, critical. Default: normal",
                            "default": "normal"
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
            // ---------------------------------------------------------------
            // Channel state management
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_pause_channel".to_string(),
                description: Some("Pause a channel, blocking new sends and receives".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Channel ID to pause"
                        }
                    },
                    "required": ["channel_id"]
                }),
            },
            ToolDefinition {
                name: "comm_resume_channel".to_string(),
                description: Some("Resume a paused channel back to active state".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Channel ID to resume"
                        }
                    },
                    "required": ["channel_id"]
                }),
            },
            ToolDefinition {
                name: "comm_drain_channel".to_string(),
                description: Some("Set a channel to draining state, allowing receive but blocking send".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Channel ID to drain"
                        }
                    },
                    "required": ["channel_id"]
                }),
            },
            ToolDefinition {
                name: "comm_close_channel".to_string(),
                description: Some("Close a channel, blocking all operations".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "channel_id": {
                            "type": "integer",
                            "description": "Channel ID to close"
                        }
                    },
                    "required": ["channel_id"]
                }),
            },
            // ---------------------------------------------------------------
            // Dead letter queue
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_list_dead_letters".to_string(),
                description: Some("List all messages in the dead letter queue".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "comm_replay_dead_letter".to_string(),
                description: Some("Retry delivery of a dead letter by its index".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "index": {
                            "type": "integer",
                            "description": "Zero-based index of the dead letter to replay"
                        }
                    },
                    "required": ["index"]
                }),
            },
            ToolDefinition {
                name: "comm_clear_dead_letters".to_string(),
                description: Some("Clear all messages from the dead letter queue".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // ---------------------------------------------------------------
            // Maintenance
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_expire_messages".to_string(),
                description: Some("Expire messages that have exceeded their channel TTL".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "comm_compact".to_string(),
                description: Some("Compact the store by removing messages from closed channels".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            // ---------------------------------------------------------------
            // Key management
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_generate_key".to_string(),
                description: Some("Generate a new encryption key with metadata".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "algorithm": {
                            "type": "string",
                            "description": "Encryption algorithm name (e.g. aes-256-gcm, x25519)"
                        },
                        "label": {
                            "type": "string",
                            "description": "Optional channel ID to bind the key to",
                            "default": null
                        }
                    },
                    "required": ["algorithm"]
                }),
            },
            ToolDefinition {
                name: "comm_list_keys".to_string(),
                description: Some("List all key entries in the key store".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "comm_rotate_key".to_string(),
                description: Some("Rotate a key by marking it rotated and generating a replacement".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "key_id": {
                            "type": "integer",
                            "description": "ID of the key to rotate"
                        }
                    },
                    "required": ["key_id"]
                }),
            },
            ToolDefinition {
                name: "comm_revoke_key".to_string(),
                description: Some("Revoke a key by its ID".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "key_id": {
                            "type": "integer",
                            "description": "ID of the key to revoke"
                        }
                    },
                    "required": ["key_id"]
                }),
            },
            ToolDefinition {
                name: "comm_export_key".to_string(),
                description: Some("Export a key fingerprint by its ID".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "key_id": {
                            "type": "integer",
                            "description": "ID of the key to export"
                        },
                        "include_private": {
                            "type": "boolean",
                            "description": "Whether to include private key material (stub, always false)",
                            "default": false
                        }
                    },
                    "required": ["key_id"]
                }),
            },
            ToolDefinition {
                name: "comm_get_key".to_string(),
                description: Some("Retrieve a specific key entry by its ID".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "key_id": {
                            "type": "integer",
                            "description": "ID of the key to retrieve"
                        }
                    },
                    "required": ["key_id"]
                }),
            },
            // ---------------------------------------------------------------
            // Federation zone policy
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_set_zone_policy".to_string(),
                description: Some("Set federation policy configuration for a specific zone".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "zone": {
                            "type": "string",
                            "description": "Zone identifier"
                        },
                        "allow_semantic": {
                            "type": "boolean",
                            "description": "Allow semantic operations through this zone",
                            "default": true
                        },
                        "allow_affect": {
                            "type": "boolean",
                            "description": "Allow affect propagation through this zone",
                            "default": true
                        },
                        "allow_hive": {
                            "type": "boolean",
                            "description": "Allow hive operations through this zone",
                            "default": true
                        },
                        "max_message_size": {
                            "type": "integer",
                            "description": "Maximum message size in bytes (0 = unlimited)",
                            "default": 1048576
                        }
                    },
                    "required": ["zone"]
                }),
            },
            // Workspace tools (multi-store comparison)
            ToolDefinition {
                name: "comm_workspace_create".to_string(),
                description: Some("Create a new workspace for multi-store comparison".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Human-readable workspace name"
                        }
                    },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "comm_workspace_add".to_string(),
                description: Some("Add an .acomm store context to an existing workspace".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "workspace_id": {
                            "type": "string",
                            "description": "Workspace identifier returned by comm_workspace_create"
                        },
                        "path": {
                            "type": "string",
                            "description": "Path to the .acomm file to add"
                        },
                        "label": {
                            "type": "string",
                            "description": "Optional human-readable label for this context"
                        },
                        "role": {
                            "type": "string",
                            "description": "Context role: primary, secondary, reference, archive. Default: secondary",
                            "default": "secondary"
                        }
                    },
                    "required": ["workspace_id", "path"]
                }),
            },
            ToolDefinition {
                name: "comm_workspace_list".to_string(),
                description: Some("List all contexts loaded in a workspace".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "workspace_id": {
                            "type": "string",
                            "description": "Workspace identifier"
                        }
                    },
                    "required": ["workspace_id"]
                }),
            },
            ToolDefinition {
                name: "comm_workspace_query".to_string(),
                description: Some("Search across all workspace contexts for matching messages, channels, and agents".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "workspace_id": {
                            "type": "string",
                            "description": "Workspace identifier"
                        },
                        "query": {
                            "type": "string",
                            "description": "Search text to match against messages, channels, and agents"
                        },
                        "max_per_context": {
                            "type": "integer",
                            "description": "Maximum matches per context (default: 20)",
                            "default": 20
                        }
                    },
                    "required": ["workspace_id", "query"]
                }),
            },
            ToolDefinition {
                name: "comm_workspace_compare".to_string(),
                description: Some("Compare presence of an item across all workspace contexts".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "workspace_id": {
                            "type": "string",
                            "description": "Workspace identifier"
                        },
                        "item": {
                            "type": "string",
                            "description": "Item to compare (agent name, channel name, or content keyword)"
                        },
                        "max_per_context": {
                            "type": "integer",
                            "description": "Maximum matches per context (default: 50)",
                            "default": 50
                        }
                    },
                    "required": ["workspace_id", "item"]
                }),
            },
            ToolDefinition {
                name: "comm_workspace_xref".to_string(),
                description: Some("Cross-reference an item across all workspace contexts".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "workspace_id": {
                            "type": "string",
                            "description": "Workspace identifier"
                        },
                        "item": {
                            "type": "string",
                            "description": "Item to cross-reference across all contexts"
                        }
                    },
                    "required": ["workspace_id", "item"]
                }),
            },
            // ---------------------------------------------------------------
            // Session management tools (Phase 0: 20-Year Clock)
            // ---------------------------------------------------------------
            ToolDefinition {
                name: "comm_session_start".to_string(),
                description: Some(
                    "Start a new communication session with optional metadata".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "metadata": {
                            "type": "object",
                            "description": "Optional session metadata (project, purpose, etc.)"
                        }
                    }
                }),
            },
            ToolDefinition {
                name: "comm_session_end".to_string(),
                description: Some(
                    "End the current session and return summary statistics".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "summary": {
                            "type": "string",
                            "description": "Optional session summary note"
                        }
                    }
                }),
            },
            ToolDefinition {
                name: "comm_session_resume".to_string(),
                description: Some(
                    "Resume context from current or previous session".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of recent items to return (default: 15)",
                            "default": 15
                        }
                    }
                }),
            },
            ToolDefinition {
                name: "comm_conversation_log".to_string(),
                description: Some(
                    "Log a user prompt and agent response into the session temporal chain".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "user_message": {
                            "type": "string",
                            "description": "The user's message or prompt"
                        },
                        "agent_response": {
                            "type": "string",
                            "description": "The agent's response"
                        },
                        "topic": {
                            "type": "string",
                            "description": "Category or topic of the conversation"
                        }
                    }
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
            "comm_send_message" => validation::validate_send_message(params),
            "comm_receive_messages" => validation::validate_receive_messages(params),
            "comm_create_channel" => validation::validate_create_channel(params),
            "comm_list_channels" => Ok(()), // No required params
            "comm_join_channel" => validation::validate_join_channel(params),
            "comm_leave_channel" => validation::validate_leave_channel(params),
            "comm_get_channel_info" => validation::validate_get_channel_info(params),
            "comm_subscribe" => validation::validate_subscribe(params),
            "comm_unsubscribe" => validation::validate_unsubscribe(params),
            "comm_publish" => validation::validate_publish(params),
            "comm_broadcast" => validation::validate_broadcast(params),
            "comm_query_history" => validation::validate_query_history(params),
            "comm_search_messages" => validation::validate_search_messages(params),
            "comm_get_message" => validation::validate_get_message(params),
            "comm_acknowledge_message" => validation::validate_acknowledge_message(params),
            "comm_set_channel_config" => validation::validate_set_channel_config(params),
            "comm_communication_log" => validation::validate_communication_log(params),
            "comm_manage_consent" => validation::validate_manage_consent(params),
            "comm_check_consent" => validation::validate_check_consent(params),
            "comm_set_trust_level" => validation::validate_set_trust_level(params),
            "comm_get_trust_level" => validation::validate_get_trust_level(params),
            "comm_schedule_message" => validation::validate_schedule_message(params),
            "comm_list_scheduled" => validation::validate_list_scheduled(params),
            "comm_form_hive" => validation::validate_form_hive(params),
            "comm_get_stats" => validation::validate_get_stats(params),
            "comm_send_affect" => validation::validate_send_affect(params),
            "comm_ground" => (|| {
                let claim = validation::require_string(params, "claim")?;
                validation::validate_query(claim)?;
                Ok(())
            })(),
            "comm_evidence" => (|| {
                let query = validation::require_string(params, "query")?;
                validation::validate_query(query)?;
                Ok(())
            })(),
            "comm_suggest" => (|| {
                let query = validation::require_string(params, "query")?;
                validation::validate_query(query)?;
                Ok(())
            })(),
            "comm_list_consent_gates" => validation::validate_list_consent_gates(params),
            "comm_list_trust_levels" => Ok(()), // No required params
            "comm_cancel_scheduled" => validation::validate_cancel_scheduled(params),
            "comm_deliver_pending" => Ok(()), // No required params
            "comm_configure_federation" => validation::validate_configure_federation(params),
            "comm_add_federated_zone" => validation::validate_add_federated_zone(params),
            "comm_remove_federated_zone" => validation::validate_remove_federated_zone(params),
            "comm_list_federated_zones" => Ok(()), // No required params
            "comm_dissolve_hive" => validation::validate_dissolve_hive(params),
            "comm_join_hive" => validation::validate_join_hive(params),
            "comm_leave_hive" => validation::validate_leave_hive(params),
            "comm_list_hives" => Ok(()), // No required params
            "comm_get_hive" => validation::validate_get_hive(params),
            "comm_log_communication" => validation::validate_log_communication(params),
            "comm_get_comm_log" => validation::validate_get_comm_log(params),
            "comm_get_audit_log" => validation::validate_get_audit_log(params),
            // Semantic tools
            "comm_send_semantic" => validation::validate_send_semantic(params),
            "comm_extract_semantic" => validation::validate_extract_semantic(params),
            "comm_graft_semantic" => validation::validate_graft_semantic(params),
            "comm_list_semantic_conflicts" => validation::validate_list_semantic_conflicts(params),
            // Affect tools
            "comm_get_affect_state" => validation::validate_get_affect_state(params),
            "comm_set_affect_resistance" => validation::validate_set_affect_resistance(params),
            // Hive extension tools
            "comm_hive_think" => validation::validate_hive_think(params),
            "comm_initiate_meld" => validation::validate_initiate_meld(params),
            // Consent flow tools
            "comm_list_pending_consent" => validation::validate_list_pending_consent(params),
            "comm_respond_consent" => validation::validate_respond_consent(params),
            // Query tools
            "comm_query_relationships" => validation::validate_query_relationships(params),
            "comm_query_echoes" => validation::validate_query_echoes(params),
            "comm_query_conversations" => validation::validate_query_conversations(params),
            // Federation extension tools
            "comm_get_federation_status" => validation::validate_get_federation_status(params),
            "comm_set_federation_policy" => validation::validate_set_federation_policy(params),
            // Crypto / encryption tools
            "comm_generate_keypair" => Ok(()), // No required params
            "comm_encrypt_message" => validation::validate_encrypt_message(params),
            "comm_decrypt_message" => validation::validate_decrypt_message(params),
            "comm_verify_signature" => validation::validate_verify_signature(params),
            "comm_get_public_key" => Ok(()), // No required params
            // Messaging: replies, threads, priority
            "comm_send_reply" => validation::validate_send_reply(params),
            "comm_get_thread" => validation::validate_get_thread(params),
            "comm_get_replies" => validation::validate_get_replies(params),
            "comm_send_with_priority" => validation::validate_send_with_priority(params),
            // Channel state management
            "comm_pause_channel" => validation::validate_channel_state_change(params),
            "comm_resume_channel" => validation::validate_channel_state_change(params),
            "comm_drain_channel" => validation::validate_channel_state_change(params),
            "comm_close_channel" => validation::validate_channel_state_change(params),
            // Dead letter queue
            "comm_list_dead_letters" => Ok(()), // No required params
            "comm_replay_dead_letter" => validation::validate_replay_dead_letter(params),
            "comm_clear_dead_letters" => Ok(()), // No required params
            // Maintenance
            "comm_expire_messages" => Ok(()), // No required params
            "comm_compact" => Ok(()), // No required params
            // Key management
            "comm_generate_key" => validation::validate_generate_key(params),
            "comm_list_keys" => Ok(()), // No required params
            "comm_rotate_key" => validation::validate_key_id(params),
            "comm_revoke_key" => validation::validate_key_id(params),
            "comm_export_key" => validation::validate_key_id(params),
            "comm_get_key" => validation::validate_key_id(params),
            // Federation zone policy
            "comm_set_zone_policy" => validation::validate_set_zone_policy(params),
            // Workspace tools
            "comm_workspace_create" => validation::validate_workspace_create(params),
            "comm_workspace_add" => validation::validate_workspace_add(params),
            "comm_workspace_list" => validation::validate_workspace_id(params),
            "comm_workspace_query" => validation::validate_workspace_query(params),
            "comm_workspace_compare" => validation::validate_workspace_compare(params),
            "comm_workspace_xref" => validation::validate_workspace_xref(params),
            // Session management tools (Phase 0)
            "comm_session_start" => Ok(()), // No required params
            "comm_session_end" => Ok(()), // No required params
            "comm_session_resume" => validation::validate_session_resume(params),
            "comm_conversation_log" => Ok(()), // All params optional
            _ => return Err(McpError::ToolNotFound(tool_name.to_string())),
        };

        // If validation failed, return the error result directly
        if let Err(error_result) = validation_result {
            return Ok(error_result);
        }

        // Dispatch to actual handler
        match tool_name {
            "comm_send_message" => Self::handle_send_message(params, session),
            "comm_receive_messages" => Self::handle_receive_messages(params, session),
            "comm_create_channel" => Self::handle_create_channel(params, session),
            "comm_list_channels" => Self::handle_list_channels(session),
            "comm_join_channel" => Self::handle_join_channel(params, session),
            "comm_leave_channel" => Self::handle_leave_channel(params, session),
            "comm_get_channel_info" => Self::handle_get_channel_info(params, session),
            "comm_subscribe" => Self::handle_subscribe(params, session),
            "comm_unsubscribe" => Self::handle_unsubscribe(params, session),
            "comm_publish" => Self::handle_publish(params, session),
            "comm_broadcast" => Self::handle_broadcast(params, session),
            "comm_query_history" => Self::handle_query_history(params, session),
            "comm_search_messages" => Self::handle_search_messages(params, session),
            "comm_get_message" => Self::handle_get_message(params, session),
            "comm_acknowledge_message" => Self::handle_acknowledge_message(params, session),
            "comm_set_channel_config" => Self::handle_set_channel_config(params, session),
            "comm_communication_log" => Self::handle_communication_log(params, session),
            "comm_manage_consent" => Self::handle_manage_consent(params, session),
            "comm_check_consent" => Self::handle_check_consent(params, session),
            "comm_set_trust_level" => Self::handle_set_trust_level(params, session),
            "comm_get_trust_level" => Self::handle_get_trust_level(params, session),
            "comm_schedule_message" => Self::handle_schedule_message(params, session),
            "comm_list_scheduled" => Self::handle_list_scheduled(session),
            "comm_form_hive" => Self::handle_form_hive(params, session),
            "comm_get_stats" => Self::handle_get_stats(session),
            "comm_send_affect" => Self::handle_send_affect(params, session),
            "comm_ground" => Self::handle_comm_ground(params, session),
            "comm_evidence" => Self::handle_comm_evidence(params, session),
            "comm_suggest" => Self::handle_comm_suggest(params, session),
            "comm_list_consent_gates" => Self::handle_list_consent_gates(params, session),
            "comm_list_trust_levels" => Self::handle_list_trust_levels(session),
            "comm_cancel_scheduled" => Self::handle_cancel_scheduled(params, session),
            "comm_deliver_pending" => Self::handle_deliver_pending(session),
            "comm_configure_federation" => Self::handle_configure_federation(params, session),
            "comm_add_federated_zone" => Self::handle_add_federated_zone(params, session),
            "comm_remove_federated_zone" => Self::handle_remove_federated_zone(params, session),
            "comm_list_federated_zones" => Self::handle_list_federated_zones(session),
            "comm_dissolve_hive" => Self::handle_dissolve_hive(params, session),
            "comm_join_hive" => Self::handle_join_hive(params, session),
            "comm_leave_hive" => Self::handle_leave_hive(params, session),
            "comm_list_hives" => Self::handle_list_hives(session),
            "comm_get_hive" => Self::handle_get_hive(params, session),
            "comm_log_communication" => Self::handle_log_communication(params, session),
            "comm_get_comm_log" => Self::handle_get_comm_log(params, session),
            "comm_get_audit_log" => Self::handle_get_audit_log(params, session),
            // Semantic tools
            "comm_send_semantic" => Self::handle_send_semantic(params, session),
            "comm_extract_semantic" => Self::handle_extract_semantic(params, session),
            "comm_graft_semantic" => Self::handle_graft_semantic(params, session),
            "comm_list_semantic_conflicts" => Self::handle_list_semantic_conflicts(params, session),
            // Affect tools
            "comm_get_affect_state" => Self::handle_get_affect_state(params, session),
            "comm_set_affect_resistance" => Self::handle_set_affect_resistance(params, session),
            // Hive extension tools
            "comm_hive_think" => Self::handle_hive_think(params, session),
            "comm_initiate_meld" => Self::handle_initiate_meld(params, session),
            // Consent flow tools
            "comm_list_pending_consent" => Self::handle_list_pending_consent(params, session),
            "comm_respond_consent" => Self::handle_respond_consent(params, session),
            // Query tools
            "comm_query_relationships" => Self::handle_query_relationships(params, session),
            "comm_query_echoes" => Self::handle_query_echoes(params, session),
            "comm_query_conversations" => Self::handle_query_conversations(params, session),
            // Federation extension tools
            "comm_get_federation_status" => Self::handle_get_federation_status(session),
            "comm_set_federation_policy" => Self::handle_set_federation_policy(params, session),
            // Crypto / encryption tools
            "comm_generate_keypair" => Self::handle_generate_keypair(session),
            "comm_encrypt_message" => Self::handle_encrypt_message(params, session),
            "comm_decrypt_message" => Self::handle_decrypt_message(params, session),
            "comm_verify_signature" => Self::handle_verify_signature(params, session),
            "comm_get_public_key" => Self::handle_get_public_key(session),
            // Messaging: replies, threads, priority
            "comm_send_reply" => Self::handle_send_reply(params, session),
            "comm_get_thread" => Self::handle_get_thread(params, session),
            "comm_get_replies" => Self::handle_get_replies(params, session),
            "comm_send_with_priority" => Self::handle_send_with_priority(params, session),
            // Channel state management
            "comm_pause_channel" => Self::handle_pause_channel(params, session),
            "comm_resume_channel" => Self::handle_resume_channel(params, session),
            "comm_drain_channel" => Self::handle_drain_channel(params, session),
            "comm_close_channel" => Self::handle_close_channel(params, session),
            // Dead letter queue
            "comm_list_dead_letters" => Self::handle_list_dead_letters(session),
            "comm_replay_dead_letter" => Self::handle_replay_dead_letter(params, session),
            "comm_clear_dead_letters" => Self::handle_clear_dead_letters(session),
            // Maintenance
            "comm_expire_messages" => Self::handle_expire_messages(session),
            "comm_compact" => Self::handle_compact(session),
            // Key management
            "comm_generate_key" => Self::handle_generate_key(params, session),
            "comm_list_keys" => Self::handle_list_keys(session),
            "comm_rotate_key" => Self::handle_rotate_key(params, session),
            "comm_revoke_key" => Self::handle_revoke_key(params, session),
            "comm_export_key" => Self::handle_export_key(params, session),
            "comm_get_key" => Self::handle_get_key(params, session),
            // Federation zone policy
            "comm_set_zone_policy" => Self::handle_set_zone_policy(params, session),
            // Workspace tools
            "comm_workspace_create" => Self::handle_workspace_create(params, session),
            "comm_workspace_add" => Self::handle_workspace_add(params, session),
            "comm_workspace_list" => Self::handle_workspace_list(params, session),
            "comm_workspace_query" => Self::handle_workspace_query(params, session),
            "comm_workspace_compare" => Self::handle_workspace_compare(params, session),
            "comm_workspace_xref" => Self::handle_workspace_xref(params, session),
            // Session management tools (Phase 0)
            "comm_session_start" => Self::handle_session_start(params, session),
            "comm_session_end" => Self::handle_session_end(params, session),
            "comm_session_resume" => Self::handle_session_resume(params, session),
            "comm_conversation_log" => Self::handle_conversation_log(params, session),
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
                session.record_operation("comm_send_message", Some(msg.id));
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
                session.record_operation("comm_receive_messages", None);
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
                session.record_operation("comm_create_channel", Some(ch.id));
                Ok(ToolCallResult::json(&ch))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_list_channels(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channels = session.store.list_channels();
        session.record_operation("comm_list_channels", None);
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
                session.record_operation("comm_join_channel", Some(channel_id));
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
                session.record_operation("comm_leave_channel", Some(channel_id));
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

        session.record_operation("comm_get_channel_info", Some(channel_id));
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
                session.record_operation("comm_subscribe", Some(sub.id));
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
                session.record_operation("comm_unsubscribe", Some(subscription_id));
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
                session.record_operation("comm_publish", None);
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
                session.record_operation("comm_broadcast", Some(channel_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "comm_broadcast",
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
            ..Default::default()
        };

        let results = session.store.query_history(channel_id, &filter);
        session.record_operation("comm_query_history", Some(channel_id));
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
        session.record_operation("comm_search_messages", None);
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

        session.record_operation("comm_get_message", Some(message_id));
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
                session.record_operation("comm_acknowledge_message", Some(message_id));
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
                session.record_operation("comm_set_channel_config", Some(channel_id));
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
        session.record_operation("comm_communication_log", None);

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
                        session.record_operation("comm_manage_consent", None);
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
                        session.record_operation("comm_manage_consent", None);
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
        session.record_operation("comm_check_consent", None);
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
                session.record_operation("comm_set_trust_level", None);
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
        session.record_operation("comm_get_trust_level", None);
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
                session.record_operation("comm_schedule_message", Some(tid));
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
        session.record_operation("comm_list_scheduled", None);
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
                session.record_operation("comm_form_hive", Some(hid));
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
        session.record_operation("comm_get_stats", None);
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
                session.record_operation("comm_send_affect", Some(msg.id));
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

    // -----------------------------------------------------------------------
    // Evidence search handler
    // -----------------------------------------------------------------------

    fn handle_comm_evidence(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("query is required".to_string()))?;

        let evidence = session.store.ground_evidence(query);
        session.record_operation("comm_evidence", None);
        Ok(ToolCallResult::json(&evidence))
    }

    // -----------------------------------------------------------------------
    // Suggestion handler
    // -----------------------------------------------------------------------

    fn handle_comm_suggest(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("query is required".to_string()))?;
        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let suggestions = session.store.ground_suggest(query, limit);
        session.record_operation("comm_suggest", None);
        Ok(ToolCallResult::json(&suggestions))
    }

    // -----------------------------------------------------------------------
    // Consent listing handler
    // -----------------------------------------------------------------------

    fn handle_list_consent_gates(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let agent = params.get("agent").and_then(|v| v.as_str());
        let gates: Vec<_> = session.store.list_consent_gates(agent)
            .into_iter()
            .cloned()
            .collect();
        session.record_operation("comm_list_consent_gates", None);
        Ok(ToolCallResult::json(&gates))
    }

    // -----------------------------------------------------------------------
    // Trust listing handler
    // -----------------------------------------------------------------------

    fn handle_list_trust_levels(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let levels = session.store.list_trust_levels().clone();
        session.record_operation("comm_list_trust_levels", None);
        Ok(ToolCallResult::json(&levels))
    }

    // -----------------------------------------------------------------------
    // Temporal management handlers
    // -----------------------------------------------------------------------

    fn handle_cancel_scheduled(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let temporal_id = params
            .get("temporal_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("temporal_id is required".to_string()))?;

        match session.store.cancel_scheduled(temporal_id) {
            Ok(()) => {
                session.record_operation("comm_cancel_scheduled", Some(temporal_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "cancelled",
                    "temporal_id": temporal_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_deliver_pending(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let count = session.store.deliver_pending_temporal();
        session.record_operation("comm_deliver_pending", None);
        Ok(ToolCallResult::json(&json!({
            "status": "delivered",
            "count": count,
        })))
    }

    // -----------------------------------------------------------------------
    // Federation handlers
    // -----------------------------------------------------------------------

    fn handle_configure_federation(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let enabled = params
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| McpError::InvalidParams("enabled is required".to_string()))?;
        let local_zone = params
            .get("local_zone")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("local_zone is required".to_string()))?;
        let policy_str = params
            .get("policy")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("policy is required".to_string()))?;
        let policy: FederationPolicy = policy_str
            .parse()
            .map_err(|e: String| McpError::InvalidParams(e))?;

        match session.store.configure_federation(enabled, local_zone, policy) {
            Ok(()) => {
                let config = session.store.get_federation_config().clone();
                session.record_operation("comm_configure_federation", None);
                Ok(ToolCallResult::json(&config))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_add_federated_zone(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let zone_id = params
            .get("zone_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("zone_id is required".to_string()))?;
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let endpoint = params
            .get("endpoint")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let policy: FederationPolicy = params
            .get("policy")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();
        let trust_level: CommTrustLevel = params
            .get("trust_level")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();

        let zone = FederatedZone {
            zone_id: zone_id.to_string(),
            name,
            endpoint,
            policy,
            trust_level,
        };

        match session.store.add_federated_zone(zone) {
            Ok(()) => {
                session.record_operation("comm_add_federated_zone", None);
                Ok(ToolCallResult::json(&json!({
                    "status": "added",
                    "zone_id": zone_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_remove_federated_zone(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let zone_id = params
            .get("zone_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("zone_id is required".to_string()))?;

        match session.store.remove_federated_zone(zone_id) {
            Ok(()) => {
                session.record_operation("comm_remove_federated_zone", None);
                Ok(ToolCallResult::json(&json!({
                    "status": "removed",
                    "zone_id": zone_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_list_federated_zones(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let zones: Vec<_> = session.store.list_federated_zones().to_vec();
        session.record_operation("comm_list_federated_zones", None);
        Ok(ToolCallResult::json(&zones))
    }

    // -----------------------------------------------------------------------
    // Hive mind management handlers
    // -----------------------------------------------------------------------

    fn handle_dissolve_hive(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let hive_id = params
            .get("hive_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("hive_id is required".to_string()))?;

        match session.store.dissolve_hive(hive_id) {
            Ok(()) => {
                session.record_operation("comm_dissolve_hive", Some(hive_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "dissolved",
                    "hive_id": hive_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_join_hive(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let hive_id = params
            .get("hive_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("hive_id is required".to_string()))?;
        let agent_id = params
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("agent_id is required".to_string()))?;
        let role_str = params
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("member");
        let role = match role_str {
            "coordinator" => HiveRole::Coordinator,
            "member" => HiveRole::Member,
            "observer" => HiveRole::Observer,
            other => return Ok(ToolCallResult::error(format!(
                "Unknown hive role: {other}. Must be: coordinator, member, observer"
            ))),
        };

        match session.store.join_hive(hive_id, agent_id, role) {
            Ok(()) => {
                // Clone hive info after mutation for response
                let hive_info = session.store.get_hive(hive_id).cloned();
                session.record_operation("comm_join_hive", Some(hive_id));
                match hive_info {
                    Some(hive) => Ok(ToolCallResult::json(&hive)),
                    None => Ok(ToolCallResult::json(&json!({
                        "status": "joined",
                        "hive_id": hive_id,
                        "agent_id": agent_id,
                        "role": role_str,
                    }))),
                }
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_leave_hive(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let hive_id = params
            .get("hive_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("hive_id is required".to_string()))?;
        let agent_id = params
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("agent_id is required".to_string()))?;

        match session.store.leave_hive(hive_id, agent_id) {
            Ok(()) => {
                session.record_operation("comm_leave_hive", Some(hive_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "left",
                    "hive_id": hive_id,
                    "agent_id": agent_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_list_hives(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let hives: Vec<_> = session.store.list_hives()
            .into_iter()
            .cloned()
            .collect();
        session.record_operation("comm_list_hives", None);
        Ok(ToolCallResult::json(&hives))
    }

    fn handle_get_hive(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let hive_id = params
            .get("hive_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("hive_id is required".to_string()))?;

        session.record_operation("comm_get_hive", Some(hive_id));
        match session.store.get_hive(hive_id) {
            Some(hive) => Ok(ToolCallResult::json(&hive)),
            None => Ok(ToolCallResult::error(format!(
                "Hive {hive_id} not found"
            ))),
        }
    }

    // -----------------------------------------------------------------------
    // Communication log handlers
    // -----------------------------------------------------------------------

    fn handle_log_communication(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("content is required".to_string()))?;
        let role = params
            .get("role")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("role is required".to_string()))?;
        let topic = params
            .get("topic")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let linked_message_id = params
            .get("linked_message_id")
            .and_then(|v| v.as_u64());

        let affect = if let Some(affect_obj) = params.get("affect") {
            let valence = affect_obj.get("valence").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let arousal = affect_obj.get("arousal").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let urgency_str = affect_obj
                .get("urgency")
                .and_then(|v| v.as_str())
                .unwrap_or("normal");
            let urgency: UrgencyLevel = urgency_str
                .parse()
                .unwrap_or(UrgencyLevel::Normal);
            Some(AffectState {
                valence,
                arousal,
                urgency,
                ..Default::default()
            })
        } else {
            None
        };

        let entry = session.store.log_communication(
            content,
            role,
            topic,
            linked_message_id,
            affect,
        ).clone();
        session.record_operation("comm_log_communication", None);
        Ok(ToolCallResult::json(&entry))
    }

    fn handle_get_comm_log(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);

        let entries = session.store.get_comm_log(limit).to_vec();
        session.record_operation("comm_get_comm_log", None);
        Ok(ToolCallResult::json(&entries))
    }

    // -----------------------------------------------------------------------
    // Audit log handler
    // -----------------------------------------------------------------------

    fn handle_get_audit_log(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let _limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);

        // NOTE: get_audit_log may not exist yet on CommStore. Return empty array for now.
        session.record_operation("comm_get_audit_log", None);
        Ok(ToolCallResult::json(&json!([])))
    }

    // -----------------------------------------------------------------------
    // Semantic tool handlers
    // -----------------------------------------------------------------------

    fn handle_send_semantic(
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
        let topic = params
            .get("topic")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("topic is required".to_string()))?;
        let focus_nodes: Vec<String> = params
            .get("focus_nodes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let depth = params
            .get("depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        match session
            .store
            .send_semantic(channel_id, sender, topic, focus_nodes, depth)
        {
            Ok(op) => {
                session.record_operation("comm_send_semantic", None);
                Ok(ToolCallResult::json(&op))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_extract_semantic(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let message_id = params
            .get("message_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("message_id is required".to_string()))?;

        match session.store.extract_semantic(message_id) {
            Ok(op) => {
                session.record_operation("comm_extract_semantic", None);
                Ok(ToolCallResult::json(&op))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_graft_semantic(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let source_id = params
            .get("source_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("source_id is required".to_string()))?;
        let target_id = params
            .get("target_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("target_id is required".to_string()))?;
        let strategy = params
            .get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("union");

        match session
            .store
            .graft_semantic(source_id, target_id, strategy)
        {
            Ok(op) => {
                session.record_operation("comm_graft_semantic", None);
                Ok(ToolCallResult::json(&op))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_list_semantic_conflicts(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64());
        let severity = params
            .get("severity")
            .and_then(|v| v.as_str());

        let conflicts = session
            .store
            .list_semantic_conflicts(channel_id, severity);
        let result = ToolCallResult::json(&conflicts);
        session.record_operation("comm_list_semantic_conflicts", None);
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Affect tool handlers
    // -----------------------------------------------------------------------

    fn handle_get_affect_state(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let agent_id = params
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("agent_id is required".to_string()))?;

        let state = session.store.get_affect_state(agent_id).cloned();
        session.record_operation("comm_get_affect_state", None);
        match state {
            Some(ref s) => Ok(ToolCallResult::json(s)),
            None => Ok(ToolCallResult::json(&json!({
                "agent_id": agent_id,
                "state": null,
                "message": "No affect state recorded for this agent"
            }))),
        }
    }

    fn handle_set_affect_resistance(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let resistance = params
            .get("resistance")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::InvalidParams("resistance is required".to_string()))?;

        let actual = session.store.set_affect_resistance(resistance);
        session.record_operation("comm_set_affect_resistance", None);
        Ok(ToolCallResult::json(&json!({
            "resistance": actual,
            "message": format!("Affect resistance set to {}", actual),
        })))
    }

    // -----------------------------------------------------------------------
    // Hive extension handlers
    // -----------------------------------------------------------------------

    fn handle_hive_think(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let hive_id = params
            .get("hive_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("hive_id is required".to_string()))?;
        let question = params
            .get("question")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("question is required".to_string()))?;
        let timeout_ms = params
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(5000);

        match session.store.hive_think(hive_id, question, timeout_ms) {
            Ok(result) => {
                session.record_operation("comm_hive_think", None);
                Ok(ToolCallResult::json(&result))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_initiate_meld(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let partner_id = params
            .get("partner_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("partner_id is required".to_string()))?;
        let depth = params
            .get("depth")
            .and_then(|v| v.as_str())
            .unwrap_or("shallow");
        let duration_ms = params
            .get("duration_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(10000);

        let meld = session
            .store
            .initiate_meld(partner_id, depth, duration_ms);
        session.record_operation("comm_initiate_meld", None);
        Ok(ToolCallResult::json(&meld))
    }

    // -----------------------------------------------------------------------
    // Consent flow handlers
    // -----------------------------------------------------------------------

    fn handle_list_pending_consent(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let agent_id = params
            .get("agent_id")
            .and_then(|v| v.as_str());
        let consent_type = params
            .get("consent_type")
            .and_then(|v| v.as_str());

        let requests = session
            .store
            .list_pending_consent(agent_id, consent_type);
        let result = ToolCallResult::json(&requests);
        session.record_operation("comm_list_pending_consent", None);
        Ok(result)
    }

    fn handle_respond_consent(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let request_id = params
            .get("request_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("request_id is required".to_string()))?;
        let response = params
            .get("response")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("response is required".to_string()))?;

        match session.store.respond_consent(request_id, response) {
            Ok(()) => {
                session.record_operation("comm_respond_consent", None);
                Ok(ToolCallResult::json(&json!({
                    "request_id": request_id,
                    "response": response,
                    "status": "responded",
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    // -----------------------------------------------------------------------
    // Query tool handlers
    // -----------------------------------------------------------------------

    fn handle_query_relationships(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let agent_id = params
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("agent_id is required".to_string()))?;
        let relationship_type = params
            .get("relationship_type")
            .and_then(|v| v.as_str());
        let depth = params
            .get("depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        let result = session
            .store
            .query_relationships(agent_id, relationship_type, depth);
        session.record_operation("comm_query_relationships", None);
        Ok(ToolCallResult::json(&result))
    }

    fn handle_query_echoes(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let message_id = params
            .get("message_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("message_id is required".to_string()))?;
        let depth = params
            .get("depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        match session.store.query_echoes(message_id, depth) {
            Ok(result) => {
                session.record_operation("comm_query_echoes", None);
                Ok(ToolCallResult::json(&result))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_query_conversations(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64());
        let participant = params
            .get("participant")
            .and_then(|v| v.as_str());
        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(50);

        let summaries = session
            .store
            .query_conversations(channel_id, participant, limit);
        session.record_operation("comm_query_conversations", None);
        Ok(ToolCallResult::json(&summaries))
    }

    // -----------------------------------------------------------------------
    // Federation extension handlers
    // -----------------------------------------------------------------------

    fn handle_get_federation_status(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let status = session.store.get_federation_status();
        session.record_operation("comm_get_federation_status", None);
        Ok(ToolCallResult::json(&status))
    }

    fn handle_set_federation_policy(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let zone_id = params
            .get("zone_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("zone_id is required".to_string()))?;
        let allow_semantic = params
            .get("allow_semantic")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let allow_affect = params
            .get("allow_affect")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let allow_hive = params
            .get("allow_hive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let max_message_size = params
            .get("max_message_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1_048_576);

        let config = session.store.set_federation_policy(
            zone_id,
            allow_semantic,
            allow_affect,
            allow_hive,
            max_message_size,
        );
        session.record_operation("comm_set_federation_policy", None);
        Ok(ToolCallResult::json(&config))
    }

    // -----------------------------------------------------------------------
    // Crypto / encryption handlers
    // -----------------------------------------------------------------------

    fn handle_generate_keypair(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let kp = CommKeyPair::generate();
        let public_key = kp.public_key_hex();
        let fingerprint = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(hex::decode(&public_key).unwrap_or_default());
            hex::encode(&hasher.finalize()[..8])
        };
        session.store.set_signing_key(kp);
        session.record_operation("comm_generate_keypair", None);
        Ok(ToolCallResult::json(&json!({
            "public_key": public_key,
            "fingerprint": fingerprint,
            "algorithm": "Ed25519",
        })))
    }

    fn handle_encrypt_message(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("content is required".to_string()))?;

        // Generate a temporary encryption key (ChaCha20-Poly1305 is symmetric,
        // independent of the Ed25519 signing key pair).
        let enc_key = EncryptionKey::generate();

        match agentic_comm::encryption::encrypt(&enc_key, content) {
            Ok(payload) => {
                session.record_operation("comm_encrypt_message", None);
                Ok(ToolCallResult::json(&json!({
                    "ciphertext": payload.ciphertext,
                    "nonce": payload.nonce,
                    "epoch": payload.epoch,
                    "algorithm": payload.algorithm,
                    "key_fingerprint": enc_key.fingerprint(),
                })))
            }
            Err(e) => Ok(ToolCallResult::error(format!("Encryption failed: {e}"))),
        }
    }

    fn handle_decrypt_message(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let ciphertext = params
            .get("ciphertext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("ciphertext is required".to_string()))?;
        let nonce = params
            .get("nonce")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("nonce is required".to_string()))?;
        let epoch = params
            .get("epoch")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        let payload = EncryptedPayload {
            ciphertext: ciphertext.to_string(),
            nonce: nonce.to_string(),
            epoch,
            algorithm: "ChaCha20-Poly1305".to_string(),
        };

        // Decryption requires the same key used for encryption. In a full
        // implementation the key would be looked up by epoch/fingerprint; here
        // we attempt with a freshly generated key which will fail unless the
        // caller provides the right key externally. This placeholder returns an
        // error explaining the key requirement.
        let enc_key = EncryptionKey::generate();
        match agentic_comm::encryption::decrypt(&enc_key, &payload) {
            Ok(plaintext) => {
                session.record_operation("comm_decrypt_message", None);
                Ok(ToolCallResult::json(&json!({
                    "plaintext": plaintext,
                })))
            }
            Err(e) => {
                session.record_operation("comm_decrypt_message", None);
                Ok(ToolCallResult::error(format!("Decryption failed: {e}")))
            }
        }
    }

    fn handle_verify_signature(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let public_key = params
            .get("public_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("public_key is required".to_string()))?;
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("content is required".to_string()))?;
        let signature = params
            .get("signature")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("signature is required".to_string()))?;

        let valid = agentic_comm::crypto::verify_signature(public_key, content, signature);
        session.record_operation("comm_verify_signature", None);
        Ok(ToolCallResult::json(&json!({
            "valid": valid,
        })))
    }

    fn handle_get_public_key(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        session.record_operation("comm_get_public_key", None);
        match session.store.get_public_key() {
            Some(pk) => Ok(ToolCallResult::json(&json!({
                "public_key": pk,
                "algorithm": "Ed25519",
            }))),
            None => Ok(ToolCallResult::json(&json!({
                "public_key": null,
                "message": "No key pair is currently set. Use comm_generate_keypair to create one.",
            }))),
        }
    }

    // -----------------------------------------------------------------------
    // Reply, thread, and priority handlers
    // -----------------------------------------------------------------------

    fn handle_send_reply(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;
        let reply_to_id = params
            .get("reply_to_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("reply_to_id is required".to_string()))?;
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

        match session.store.send_reply(channel_id, reply_to_id, sender, content, msg_type) {
            Ok(msg) => {
                session.record_operation("comm_send_reply", Some(msg.id));
                Ok(ToolCallResult::json(&msg))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_get_thread(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let thread_id = params
            .get("thread_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("thread_id is required".to_string()))?;

        let messages = session.store.get_thread(thread_id);
        session.record_operation("comm_get_thread", None);
        Ok(ToolCallResult::json(&messages))
    }

    fn handle_get_replies(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let message_id = params
            .get("message_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("message_id is required".to_string()))?;

        let replies = session.store.get_replies(message_id);
        session.record_operation("comm_get_replies", Some(message_id));
        Ok(ToolCallResult::json(&replies))
    }

    fn handle_send_with_priority(
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
        let priority_str = params
            .get("priority")
            .and_then(|v| v.as_str())
            .unwrap_or("normal");
        let priority = match priority_str {
            "low" => MessagePriority::Low,
            "normal" => MessagePriority::Normal,
            "high" => MessagePriority::High,
            "urgent" => MessagePriority::Urgent,
            "critical" => MessagePriority::Critical,
            other => return Ok(ToolCallResult::error(format!(
                "Unknown priority: {other}. Must be: low, normal, high, urgent, critical"
            ))),
        };

        match session.store.send_message_with_priority(channel_id, sender, content, msg_type, priority) {
            Ok(msg) => {
                session.record_operation("comm_send_with_priority", Some(msg.id));
                Ok(ToolCallResult::json(&msg))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    // -----------------------------------------------------------------------
    // Channel state management handlers
    // -----------------------------------------------------------------------

    fn handle_pause_channel(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;

        match session.store.pause_channel(channel_id) {
            Ok(()) => {
                session.record_operation("comm_pause_channel", Some(channel_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "paused",
                    "channel_id": channel_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_resume_channel(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;

        match session.store.resume_channel(channel_id) {
            Ok(()) => {
                session.record_operation("comm_resume_channel", Some(channel_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "resumed",
                    "channel_id": channel_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_drain_channel(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;

        match session.store.drain_channel(channel_id) {
            Ok(()) => {
                session.record_operation("comm_drain_channel", Some(channel_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "draining",
                    "channel_id": channel_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_close_channel(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;

        match session.store.close_channel(channel_id) {
            Ok(()) => {
                session.record_operation("comm_close_channel", Some(channel_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "closed",
                    "channel_id": channel_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    // -----------------------------------------------------------------------
    // Dead letter queue handlers
    // -----------------------------------------------------------------------

    fn handle_list_dead_letters(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let dead_letters = session.store.list_dead_letters();
        session.record_operation("comm_list_dead_letters", None);
        Ok(ToolCallResult::json(&dead_letters))
    }

    fn handle_replay_dead_letter(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let index = params
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("index is required".to_string()))? as usize;

        match session.store.replay_dead_letter(index) {
            Ok(msg) => {
                session.record_operation("comm_replay_dead_letter", Some(msg.id));
                Ok(ToolCallResult::json(&msg))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_clear_dead_letters(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        session.store.clear_dead_letters();
        session.record_operation("comm_clear_dead_letters", None);
        Ok(ToolCallResult::json(&json!({
            "status": "cleared",
        })))
    }

    // -----------------------------------------------------------------------
    // Maintenance handlers
    // -----------------------------------------------------------------------

    fn handle_expire_messages(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let count = session.store.expire_messages();
        session.record_operation("comm_expire_messages", None);
        Ok(ToolCallResult::json(&json!({
            "status": "expired",
            "count": count,
        })))
    }

    fn handle_compact(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let count = session.store.compact();
        session.record_operation("comm_compact", None);
        Ok(ToolCallResult::json(&json!({
            "status": "compacted",
            "removed_count": count,
        })))
    }

    // -----------------------------------------------------------------------
    // Key management handlers
    // -----------------------------------------------------------------------

    fn handle_generate_key(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let algorithm = params
            .get("algorithm")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("algorithm is required".to_string()))?;
        let channel_id = params
            .get("label")
            .and_then(|v| v.as_u64());

        match session.store.generate_key(algorithm, channel_id) {
            Ok(entry) => {
                session.record_operation("comm_generate_key", Some(entry.id));
                Ok(ToolCallResult::json(&entry))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_list_keys(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let keys: Vec<&KeyEntry> = session.store.list_keys();
        let keys_owned: Vec<KeyEntry> = keys.into_iter().cloned().collect();
        session.record_operation("comm_list_keys", None);
        Ok(ToolCallResult::json(&keys_owned))
    }

    fn handle_rotate_key(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let key_id = params
            .get("key_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("key_id is required".to_string()))?;

        match session.store.rotate_key(key_id) {
            Ok(new_entry) => {
                session.record_operation("comm_rotate_key", Some(new_entry.id));
                Ok(ToolCallResult::json(&new_entry))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_revoke_key(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let key_id = params
            .get("key_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("key_id is required".to_string()))?;

        match session.store.revoke_key(key_id) {
            Ok(()) => {
                session.record_operation("comm_revoke_key", Some(key_id));
                Ok(ToolCallResult::json(&json!({
                    "status": "revoked",
                    "key_id": key_id,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_export_key(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let key_id = params
            .get("key_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("key_id is required".to_string()))?;
        let _include_private = params
            .get("include_private")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        match session.store.export_key(key_id) {
            Ok(fingerprint) => {
                session.record_operation("comm_export_key", Some(key_id));
                Ok(ToolCallResult::json(&json!({
                    "key_id": key_id,
                    "fingerprint": fingerprint,
                    "include_private": false,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_get_key(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let key_id = params
            .get("key_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("key_id is required".to_string()))?;

        session.record_operation("comm_get_key", Some(key_id));
        match session.store.get_key(key_id) {
            Ok(entry) => Ok(ToolCallResult::json(&entry)),
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    // -----------------------------------------------------------------------
    // Federation zone policy handler
    // -----------------------------------------------------------------------

    fn handle_set_zone_policy(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let zone = params
            .get("zone")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("zone is required".to_string()))?;
        let allow_semantic = params
            .get("allow_semantic")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let allow_affect = params
            .get("allow_affect")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let allow_hive = params
            .get("allow_hive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let max_message_size = params
            .get("max_message_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1_048_576);

        let config = session.store.set_federation_policy(
            zone,
            allow_semantic,
            allow_affect,
            allow_hive,
            max_message_size,
        );
        session.record_operation("comm_set_zone_policy", None);
        Ok(ToolCallResult::json(&config))
    }

    // -----------------------------------------------------------------------
    // Workspace handlers (multi-store comparison)
    // -----------------------------------------------------------------------

    fn handle_workspace_create(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("name is required".to_string()))?;

        let ws = CommWorkspace::new(name);
        let id = ws.id.clone();
        let ws_name = ws.name.clone();
        session.workspaces.insert(id.clone(), ws);
        session.record_operation("comm_workspace_create", None);

        Ok(ToolCallResult::json(&json!({
            "workspace_id": id,
            "name": ws_name,
            "status": "created"
        })))
    }

    fn handle_workspace_add(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let workspace_id = params
            .get("workspace_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("workspace_id is required".to_string()))?;
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("path is required".to_string()))?;
        let label = params.get("label").and_then(|v| v.as_str());
        let role_str = params
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("secondary");

        let role: WorkspaceRole = role_str.parse().map_err(|e: String| {
            McpError::InvalidParams(e)
        })?;

        let ws = session.workspaces.get_mut(workspace_id).ok_or_else(|| {
            McpError::InvalidParams(format!("Workspace not found: {workspace_id}"))
        })?;

        ws.add_context(path, label, role).map_err(|e| {
            McpError::AgenticComm(e)
        })?;

        let ctx_count = ws.contexts.len();
        session.record_operation("comm_workspace_add", None);

        Ok(ToolCallResult::json(&json!({
            "status": "added",
            "workspace_id": workspace_id,
            "path": path,
            "context_count": ctx_count
        })))
    }

    fn handle_workspace_list(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let workspace_id = params
            .get("workspace_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("workspace_id is required".to_string()))?;

        let ws = session.workspaces.get(workspace_id).ok_or_else(|| {
            McpError::InvalidParams(format!("Workspace not found: {workspace_id}"))
        })?;

        let result = ToolCallResult::json(&ws.list_contexts());
        session.record_operation("comm_workspace_list", None);
        Ok(result)
    }

    fn handle_workspace_query(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let workspace_id = params
            .get("workspace_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("workspace_id is required".to_string()))?;
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("query is required".to_string()))?;
        let max_per_context = params
            .get("max_per_context")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;

        let ws = session.workspaces.get(workspace_id).ok_or_else(|| {
            McpError::InvalidParams(format!("Workspace not found: {workspace_id}"))
        })?;

        let results = ws.query(query, max_per_context);
        let result = ToolCallResult::json(&results);
        session.record_operation("comm_workspace_query", None);
        Ok(result)
    }

    fn handle_workspace_compare(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let workspace_id = params
            .get("workspace_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("workspace_id is required".to_string()))?;
        let item = params
            .get("item")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("item is required".to_string()))?;
        let max_per_context = params
            .get("max_per_context")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;

        let ws = session.workspaces.get(workspace_id).ok_or_else(|| {
            McpError::InvalidParams(format!("Workspace not found: {workspace_id}"))
        })?;

        let comparison = ws.compare(item, max_per_context);
        let result = ToolCallResult::json(&comparison);
        session.record_operation("comm_workspace_compare", None);
        Ok(result)
    }

    fn handle_workspace_xref(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let workspace_id = params
            .get("workspace_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("workspace_id is required".to_string()))?;
        let item = params
            .get("item")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("item is required".to_string()))?;

        let ws = session.workspaces.get(workspace_id).ok_or_else(|| {
            McpError::InvalidParams(format!("Workspace not found: {workspace_id}"))
        })?;

        let xrefs = ws.xref(item);
        let result: Vec<_> = xrefs
            .into_iter()
            .map(|(label, found, count)| {
                json!({
                    "context": label,
                    "found": found,
                    "count": count
                })
            })
            .collect();

        let tool_result = ToolCallResult::json(&result);
        session.record_operation("comm_workspace_xref", None);
        Ok(tool_result)
    }

    // ── Session management handlers (Phase 0: 20-Year Clock) ─────────

    fn handle_session_start(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let metadata = params.get("metadata").cloned();
        session.start_session_manual(metadata);
        session.record_operation("comm_session_start", None);

        Ok(ToolCallResult::json(&json!({
            "status": "session_started",
            "project_identity": &session.project_identity[..12],
            "store_path": session.store_path.display().to_string(),
        })))
    }

    fn handle_session_end(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let summary_text = params
            .get("summary")
            .and_then(|v| v.as_str());
        let summary = session.end_session_manual(summary_text);
        // Don't record_operation here — session is already ended

        Ok(ToolCallResult::json(&json!({
            "status": "session_ended",
            "duration_secs": summary.duration_secs,
            "tool_calls": summary.tool_calls,
            "conversation_entries": summary.conversation_entries,
            "messages_sent": summary.messages_sent,
            "channels_created": summary.channels_created,
        })))
    }

    fn handle_session_resume(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(15) as usize;
        let data = session.resume_session_data(limit);
        session.record_operation("comm_session_resume", None);

        Ok(ToolCallResult::json(&data))
    }

    fn handle_conversation_log(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let user_message = params.get("user_message").and_then(|v| v.as_str());
        let agent_response = params.get("agent_response").and_then(|v| v.as_str());
        let topic = params.get("topic").and_then(|v| v.as_str());

        let entry = session.log_conversation(user_message, agent_response, topic);
        session.record_operation("comm_conversation_log", None);

        Ok(ToolCallResult::json(&json!({
            "status": "logged",
            "temporal_id": entry.temporal_id,
            "prev_temporal_id": entry.prev_temporal_id,
            "timestamp": entry.timestamp,
        })))
    }
}
