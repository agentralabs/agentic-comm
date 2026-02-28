//! Tool registration and dispatch for the MCP server.
//!
//! Consolidated API: 17 domain tools with an `operation` parameter.

use serde_json::{json, Value};

use crate::session::manager::SessionManager;
use crate::tools::communication_log::{CommunicationLogEntry, CommunicationLogInput};
use crate::tools::validation;
use crate::types::response::{ToolCallResult, ToolDefinition};
use crate::types::McpError;

use agentic_comm::{ChannelConfig, ChannelType, MessageFilter, MessageType, MessagePriority, CommTrustLevel, ConsentScope, AffectState, UrgencyLevel, TemporalTarget, CollectiveDecisionMode, FederationPolicy, FederatedZone, HiveRole, CommKeyPair, EncryptionKey, EncryptedPayload, KeyEntry, CommWorkspace, WorkspaceRole};

use super::invention_collaboration;
use super::invention_semantics;
use super::invention_affect;
use super::invention_federation;
use super::invention_temporal;
use super::invention_forensics;

/// Tool registry — lists all available tools and dispatches calls.
pub struct ToolRegistry;

impl ToolRegistry {
    /// Return definitions for all 17 consolidated tools.
    pub fn list_tools() -> Vec<ToolDefinition> {
        vec![
            // 1. comm_channel
            ToolDefinition {
                name: "comm_channel".to_string(),
                description: Some("Manage communication channels".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["create", "list", "join", "leave", "info", "config", "pause", "resume", "drain", "close", "expire", "compact"],
                            "description": "Operation to perform"
                        },
                        "name": { "type": "string", "description": "Channel name (for create)" },
                        "channel_type": { "type": "string", "description": "direct, group, broadcast, pubsub (for create). Default: group" },
                        "channel_id": { "type": "integer", "description": "Channel ID (for most operations)" },
                        "participant": { "type": "string", "description": "Participant identity (for join/leave)" },
                        "max_participants": { "type": "integer", "description": "Max participants (for create)" },
                        "ttl_seconds": { "type": "integer", "description": "Message TTL in seconds (for create)" },
                        "persistence": { "type": "boolean", "description": "Persist messages (for create)" },
                        "encryption_required": { "type": "boolean", "description": "Require encryption (for create/config)" },
                        "config": { "type": "object", "description": "Channel configuration (for config)" }
                    },
                    "required": ["operation"]
                }),
            },
            // 2. comm_message
            ToolDefinition {
                name: "comm_message".to_string(),
                description: Some("Send, receive, search, and manage messages".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["send", "receive", "get", "search", "acknowledge", "broadcast", "publish", "subscribe", "unsubscribe", "query_history", "forward", "reply", "get_thread", "get_replies", "send_priority", "send_rich", "get_rich_content", "query_echo_chain", "get_echo_depth", "summarize"],
                            "description": "Operation to perform"
                        },
                        "channel_id": { "type": "integer", "description": "Channel ID" },
                        "message_id": { "type": "integer", "description": "Message ID" },
                        "sender": { "type": "string", "description": "Sender identity" },
                        "recipient": { "type": "string", "description": "Recipient identity" },
                        "content": { "type": "string", "description": "Message content" },
                        "message_type": { "type": "string", "description": "text, command, event, data. Default: text" },
                        "topic": { "type": "string", "description": "PubSub topic (for publish/subscribe)" },
                        "subscriber": { "type": "string", "description": "Subscriber identity" },
                        "subscription_id": { "type": "integer", "description": "Subscription ID (for unsubscribe)" },
                        "query": { "type": "string", "description": "Search query" },
                        "max_results": { "type": "integer", "description": "Max results for search" },
                        "since": { "type": "string", "description": "ISO 8601 timestamp filter" },
                        "before": { "type": "string", "description": "ISO 8601 timestamp filter" },
                        "limit": { "type": "integer", "description": "Result limit" },
                        "parent_message_id": { "type": "integer", "description": "Parent message for reply" },
                        "priority": { "type": "string", "description": "low, normal, high, critical" },
                        "urgency": { "type": "string", "description": "Urgency level" },
                        "target_channel_id": { "type": "integer", "description": "Target channel for forward" },
                        "forwarder": { "type": "string", "description": "Forwarding identity" },
                        "content_type": { "type": "string", "description": "MIME type for rich messages" },
                        "metadata": { "type": "object", "description": "Additional metadata" }
                    },
                    "required": ["operation"]
                }),
            },
            // 3. comm_semantic
            ToolDefinition {
                name: "comm_semantic".to_string(),
                description: Some("Semantic messaging and NLP analysis".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": [
                                "send", "extract", "graft", "list_conflicts",
                                "comm_semantic_compress", "comm_semantic_decompress", "comm_semantic_translate", "comm_semantic_align",
                                "comm_echo_chamber_create", "comm_echo_chamber_detect", "comm_echo_chamber_break", "comm_echo_chamber_analyze",
                                "comm_ghost_create", "comm_ghost_detect", "comm_ghost_exorcise", "comm_ghost_history",
                                "comm_metamessage_encode", "comm_metamessage_decode", "comm_metamessage_layer", "comm_metamessage_strip"
                            ],
                            "description": "Operation to perform"
                        },
                        "channel_id": { "type": "integer" },
                        "message_id": { "type": "integer" },
                        "content": { "type": "string" },
                        "sender": { "type": "string" },
                        "intent": { "type": "string" },
                        "entities": { "type": "array" },
                        "sentiment": { "type": "string" },
                        "source_message_id": { "type": "integer" },
                        "target_message_id": { "type": "integer" }
                    },
                    "required": ["operation"]
                }),
            },
            // 4. comm_affect
            ToolDefinition {
                name: "comm_affect".to_string(),
                description: Some("Emotional state tracking and affect propagation".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": [
                                "send", "get_state", "set_resistance", "process_contagion", "get_history", "apply_decay",
                                "comm_affect_contagion_simulate", "comm_affect_contagion_immunize", "comm_affect_contagion_trace", "comm_affect_contagion_predict",
                                "comm_affect_archaeology_dig", "comm_affect_archaeology_artifacts", "comm_affect_archaeology_reconstruct",
                                "comm_affect_prophecy_predict", "comm_affect_prophecy_similar", "comm_affect_prophecy_track", "comm_affect_prophecy_warn",
                                "comm_unspeakable_encode", "comm_unspeakable_decode", "comm_unspeakable_detect", "comm_unspeakable_translate"
                            ],
                            "description": "Operation to perform"
                        },
                        "channel_id": { "type": "integer" },
                        "agent": { "type": "string" },
                        "affect_state": { "type": "string" },
                        "intensity": { "type": "number" },
                        "source_agent": { "type": "string" },
                        "decay_rate": { "type": "number" }
                    },
                    "required": ["operation"]
                }),
            },
            // 5. comm_hive
            ToolDefinition {
                name: "comm_hive".to_string(),
                description: Some("Hive-mind collective intelligence operations".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": [
                                "form", "dissolve", "join", "leave", "list", "get", "think", "meld",
                                "comm_hive_consciousness_create", "comm_hive_consciousness_dissolve", "comm_hive_consciousness_join", "comm_hive_consciousness_think",
                                "comm_collective_intelligence_vote", "comm_collective_intelligence_consensus", "comm_collective_intelligence_swarm", "comm_collective_intelligence_decide",
                                "comm_ancestor_invoke", "comm_ancestor_listen", "comm_ancestor_honor", "comm_ancestor_lineage",
                                "comm_telepathy_connect", "comm_telepathy_transmit", "comm_telepathy_receive", "comm_telepathy_sever"
                            ],
                            "description": "Operation to perform"
                        },
                        "name": { "type": "string", "description": "Hive name" },
                        "hive_id": { "type": "integer", "description": "Hive ID" },
                        "channel_id": { "type": "integer" },
                        "agent": { "type": "string" },
                        "role": { "type": "string" },
                        "thought": { "type": "string" },
                        "decision_mode": { "type": "string" },
                        "members": { "type": "array" }
                    },
                    "required": ["operation"]
                }),
            },
            // 6. comm_consent
            ToolDefinition {
                name: "comm_consent".to_string(),
                description: Some("Consent management and privacy gates".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["grant", "revoke", "check", "list_gates", "list_pending", "respond"],
                            "description": "Operation to perform"
                        },
                        "grantor": { "type": "string" },
                        "grantee": { "type": "string" },
                        "scope": { "type": "string" },
                        "channel_id": { "type": "integer" },
                        "consent_id": { "type": "integer" },
                        "response": { "type": "string" }
                    },
                    "required": ["operation"]
                }),
            },
            // 7. comm_trust
            ToolDefinition {
                name: "comm_trust".to_string(),
                description: Some("Trust level management between agents".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["set", "get", "list"],
                            "description": "Operation to perform"
                        },
                        "agent_a": { "type": "string" },
                        "agent_b": { "type": "string" },
                        "trust_level": { "type": "string" }
                    },
                    "required": ["operation"]
                }),
            },
            // 8. comm_keys
            ToolDefinition {
                name: "comm_keys".to_string(),
                description: Some("Cryptographic key and encryption operations".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["generate_keypair", "get_public_key", "encrypt", "decrypt", "verify_signature", "generate", "list", "rotate", "revoke", "export", "get"],
                            "description": "Operation to perform"
                        },
                        "key_id": { "type": "integer" },
                        "message_id": { "type": "integer" },
                        "channel_id": { "type": "integer" },
                        "content": { "type": "string" },
                        "sender": { "type": "string" },
                        "recipient": { "type": "string" },
                        "algorithm": { "type": "string" },
                        "key_size": { "type": "integer" }
                    },
                    "required": ["operation"]
                }),
            },
            // 9. comm_federation
            ToolDefinition {
                name: "comm_federation".to_string(),
                description: Some("Federation, zones, and cross-instance routing".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": [
                                "configure", "status", "set_policy", "add_zone", "remove_zone", "list_zones", "set_zone_policy",
                                "comm_federation_gateway_create", "comm_federation_gateway_connect", "comm_federation_gateway_disconnect", "comm_federation_gateway_status",
                                "comm_federation_route_message", "comm_federation_route_trace", "comm_federation_route_optimize", "comm_federation_route_policy",
                                "comm_federation_zone_create", "comm_federation_zone_list", "comm_federation_zone_merge", "comm_federation_zone_policy",
                                "comm_reality_fork", "comm_reality_merge", "comm_reality_detect", "comm_reality_bend"
                            ],
                            "description": "Operation to perform"
                        },
                        "zone_name": { "type": "string" },
                        "zone_id": { "type": "string" },
                        "policy": { "type": "object" },
                        "endpoint": { "type": "string" }
                    },
                    "required": ["operation"]
                }),
            },
            // 10. comm_temporal
            ToolDefinition {
                name: "comm_temporal".to_string(),
                description: Some("Scheduled messaging, dead letters, and temporal ops".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": [
                                "schedule", "list_scheduled", "cancel_scheduled", "deliver_pending",
                                "list_dead_letters", "replay_dead_letter", "clear_dead_letters",
                                "comm_precognition_predict", "comm_precognition_prepare", "comm_precognition_accuracy", "comm_precognition_calibrate",
                                "comm_temporal_schedule", "comm_temporal_cancel", "comm_temporal_pending", "comm_temporal_reschedule",
                                "comm_legacy_compose", "comm_legacy_seal", "comm_legacy_unseal", "comm_legacy_list",
                                "comm_dead_letter_resurrect", "comm_dead_letter_autopsy", "comm_dead_letter_phoenix", "comm_dead_letter_analyze"
                            ],
                            "description": "Operation to perform"
                        },
                        "channel_id": { "type": "integer" },
                        "sender": { "type": "string" },
                        "content": { "type": "string" },
                        "deliver_at": { "type": "string" },
                        "schedule_id": { "type": "integer" },
                        "dead_letter_id": { "type": "integer" }
                    },
                    "required": ["operation"]
                }),
            },
            // 11. comm_query
            ToolDefinition {
                name: "comm_query".to_string(),
                description: Some("Query relationships, echoes, conversations, and grounding".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["relationships", "echoes", "conversations", "ground", "evidence", "suggest"],
                            "description": "Operation to perform"
                        },
                        "claim": { "type": "string" },
                        "query": { "type": "string" },
                        "channel_id": { "type": "integer" },
                        "agent": { "type": "string" },
                        "max_results": { "type": "integer" }
                    },
                    "required": ["operation"]
                }),
            },
            // 12. comm_forensics
            ToolDefinition {
                name: "comm_forensics".to_string(),
                description: Some("Communication forensics, health, patterns, and oracles".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": [
                                "comm_forensics_investigate", "comm_forensics_evidence", "comm_forensics_timeline", "comm_forensics_report",
                                "comm_pattern_detect", "comm_pattern_recurring", "comm_pattern_anomaly", "comm_pattern_predict",
                                "comm_health_status", "comm_health_diagnose", "comm_health_prescribe", "comm_health_history",
                                "comm_oracle_query", "comm_oracle_prophecy", "comm_oracle_calibrate", "comm_oracle_verify"
                            ],
                            "description": "Operation to perform"
                        },
                        "channel_id": { "type": "integer" },
                        "message_id": { "type": "integer" },
                        "query": { "type": "string" }
                    },
                    "required": ["operation"]
                }),
            },
            // 13. comm_collaboration
            ToolDefinition {
                name: "comm_collaboration".to_string(),
                description: Some("Advanced collaboration: consciousness, collective intel, ancestors, telepathy".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": [
                                "comm_hive_consciousness_create", "comm_hive_consciousness_dissolve", "comm_hive_consciousness_join", "comm_hive_consciousness_think",
                                "comm_collective_intelligence_vote", "comm_collective_intelligence_consensus", "comm_collective_intelligence_swarm", "comm_collective_intelligence_decide",
                                "comm_ancestor_invoke", "comm_ancestor_listen", "comm_ancestor_honor", "comm_ancestor_lineage",
                                "comm_telepathy_connect", "comm_telepathy_transmit", "comm_telepathy_receive", "comm_telepathy_sever"
                            ],
                            "description": "Operation to perform"
                        },
                        "channel_id": { "type": "integer" },
                        "agent": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["operation"]
                }),
            },
            // 14. comm_workspace
            ToolDefinition {
                name: "comm_workspace".to_string(),
                description: Some("Manage communication workspaces".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["create", "add", "list", "query", "compare", "xref"],
                            "description": "Operation to perform"
                        },
                        "workspace_id": { "type": "string" },
                        "name": { "type": "string" },
                        "channel_id": { "type": "integer" },
                        "query": { "type": "string" },
                        "item": { "type": "string" },
                        "role": { "type": "string" }
                    },
                    "required": ["operation"]
                }),
            },
            // 15. comm_session
            ToolDefinition {
                name: "comm_session".to_string(),
                description: Some("Session lifecycle, logging, and conversation context".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["start", "end", "resume", "conversation_log", "communication_log", "log_communication", "get_comm_log", "get_audit_log"],
                            "description": "Operation to perform"
                        },
                        "summary": { "type": "string" },
                        "create_episode": { "type": "boolean" },
                        "limit": { "type": "integer" },
                        "user_message": { "type": "string" },
                        "agent_response": { "type": "string" },
                        "topic": { "type": "string" },
                        "intent": { "type": "string" },
                        "outcome": { "type": "string" },
                        "channel_id": { "type": "integer" },
                        "direction": { "type": "string" },
                        "participants": { "type": "array" }
                    },
                    "required": ["operation"]
                }),
            },
            // 16. comm_agent
            ToolDefinition {
                name: "comm_agent".to_string(),
                description: Some("Agent stats, CommId assignment, and identity operations".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["stats", "assign_comm_ids", "get_by_comm_id"],
                            "description": "Operation to perform"
                        },
                        "comm_id": { "type": "string", "description": "UUID for lookup" }
                    },
                    "required": ["operation"]
                }),
            },
            // 17. comm_store
            ToolDefinition {
                name: "comm_store".to_string(),
                description: Some("Store maintenance and dead-letter management".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["expire", "compact", "list_dead_letters", "clear_dead_letters"],
                            "description": "Operation to perform"
                        }
                    },
                    "required": ["operation"]
                }),
            },
        ]
    }

    // -------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------

    /// Extract the `operation` string from params.
    fn get_operation(params: &Value) -> Result<String, McpError> {
        params
            .get("operation")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| McpError::InvalidParams("operation is required".to_string()))
    }

    // -------------------------------------------------------------------
    // Dispatch
    // -------------------------------------------------------------------

    /// Dispatch a tool call to the appropriate handler.
    pub fn dispatch(
        tool_name: &str,
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        match tool_name {
            "comm_channel"       => Self::route_channel(params, session),
            "comm_message"       => Self::route_message(params, session),
            "comm_semantic"      => Self::route_semantic(params, session),
            "comm_affect"        => Self::route_affect(params, session),
            "comm_hive"          => Self::route_hive(params, session),
            "comm_consent"       => Self::route_consent(params, session),
            "comm_trust"         => Self::route_trust(params, session),
            "comm_keys"          => Self::route_keys(params, session),
            "comm_federation"    => Self::route_federation(params, session),
            "comm_temporal"      => Self::route_temporal(params, session),
            "comm_query"         => Self::route_query(params, session),
            "comm_forensics"     => Self::route_forensics(params, session),
            "comm_collaboration" => Self::route_collaboration(params, session),
            "comm_workspace"     => Self::route_workspace(params, session),
            "comm_session"       => Self::route_session(params, session),
            "comm_agent"         => Self::route_agent(params, session),
            "comm_store"         => Self::route_store(params, session),
            // Legacy aliases
            "session_start"      => Self::handle_session_start(params, session),
            "session_end"        => Self::handle_session_end(params, session),
            _ => Err(McpError::ToolNotFound(tool_name.to_string())),
        }
    }

    // -------------------------------------------------------------------
    // Route functions (one per consolidated tool)
    // -------------------------------------------------------------------

    fn route_channel(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "create"  => Self::handle_create_channel(params, session),
            "list"    => Self::handle_list_channels(session),
            "join"    => Self::handle_join_channel(params, session),
            "leave"   => Self::handle_leave_channel(params, session),
            "info"    => Self::handle_get_channel_info(params, session),
            "config"  => Self::handle_set_channel_config(params, session),
            "pause"   => Self::handle_pause_channel(params, session),
            "resume"  => Self::handle_resume_channel(params, session),
            "drain"   => Self::handle_drain_channel(params, session),
            "close"   => Self::handle_close_channel(params, session),
            "expire"  => Self::handle_expire_messages(session),
            "compact" => Self::handle_compact(session),
            _ => Ok(ToolCallResult::error(format!("comm_channel: unknown operation '{op}'"))),
        }
    }

    fn route_message(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "send"             => Self::handle_send_message(params, session),
            "receive"          => Self::handle_receive_messages(params, session),
            "get"              => Self::handle_get_message(params, session),
            "search"           => Self::handle_search_messages(params, session),
            "acknowledge"      => Self::handle_acknowledge_message(params, session),
            "broadcast"        => Self::handle_broadcast(params, session),
            "publish"          => Self::handle_publish(params, session),
            "subscribe"        => Self::handle_subscribe(params, session),
            "unsubscribe"      => Self::handle_unsubscribe(params, session),
            "query_history"    => Self::handle_query_history(params, session),
            "forward"          => Self::handle_forward_message(params, session),
            "reply"            => Self::handle_send_reply(params, session),
            "get_thread"       => Self::handle_get_thread(params, session),
            "get_replies"      => Self::handle_get_replies(params, session),
            "send_priority"    => Self::handle_send_with_priority(params, session),
            "send_rich"        => Self::handle_send_rich_message(params, session),
            "get_rich_content" => Self::handle_get_rich_content(params, session),
            "query_echo_chain" => Self::handle_query_echo_chain(params, session),
            "get_echo_depth"   => Self::handle_get_echo_depth(params, session),
            "summarize"        => Self::handle_summarize_conversation(params, session),
            _ => Ok(ToolCallResult::error(format!("comm_message: unknown operation '{op}'"))),
        }
    }

    fn route_semantic(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "send"             => Self::handle_send_semantic(params, session),
            "extract"          => Self::handle_extract_semantic(params, session),
            "graft"            => Self::handle_graft_semantic(params, session),
            "list_conflicts"   => Self::handle_list_semantic_conflicts(params, session),
            // Invention-semantics pass-through
            name if name.starts_with("comm_semantic_")
                || name.starts_with("comm_echo_chamber_")
                || name.starts_with("comm_ghost_")
                || name.starts_with("comm_metamessage_") =>
            {
                if let Some(result) = invention_semantics::try_execute(name, params.clone(), session) {
                    result.map_err(|e| McpError::InternalError(e))
                } else {
                    Ok(ToolCallResult::error(format!("comm_semantic: unknown operation '{op}'")))
                }
            }
            _ => Ok(ToolCallResult::error(format!("comm_semantic: unknown operation '{op}'"))),
        }
    }

    fn route_affect(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "send"              => Self::handle_send_affect(params, session),
            "get_state"         => Self::handle_get_affect_state(params, session),
            "set_resistance"    => Self::handle_set_affect_resistance(params, session),
            "process_contagion" => Self::handle_process_affect_contagion(params, session),
            "get_history"       => Self::handle_get_affect_history(params, session),
            "apply_decay"       => Self::handle_apply_affect_decay(params, session),
            // Invention-affect pass-through
            name if name.starts_with("comm_affect_contagion_")
                || name.starts_with("comm_affect_archaeology_")
                || name.starts_with("comm_affect_prophecy_")
                || name.starts_with("comm_unspeakable_") =>
            {
                if let Some(result) = invention_affect::try_execute(name, params.clone(), session) {
                    result.map_err(|e| McpError::InternalError(e))
                } else {
                    Ok(ToolCallResult::error(format!("comm_affect: unknown operation '{op}'")))
                }
            }
            _ => Ok(ToolCallResult::error(format!("comm_affect: unknown operation '{op}'"))),
        }
    }

    fn route_hive(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "form"     => Self::handle_form_hive(params, session),
            "dissolve" => Self::handle_dissolve_hive(params, session),
            "join"     => Self::handle_join_hive(params, session),
            "leave"    => Self::handle_leave_hive(params, session),
            "list"     => Self::handle_list_hives(session),
            "get"      => Self::handle_get_hive(params, session),
            "think"    => Self::handle_hive_think(params, session),
            "meld"     => Self::handle_initiate_meld(params, session),
            // Invention-collaboration pass-through for hive operations
            name if name.starts_with("comm_hive_consciousness_")
                || name.starts_with("comm_collective_intelligence_")
                || name.starts_with("comm_ancestor_")
                || name.starts_with("comm_telepathy_") =>
            {
                if let Some(result) = invention_collaboration::try_execute(name, params.clone(), session) {
                    result.map_err(|e| McpError::InternalError(e))
                } else {
                    Ok(ToolCallResult::error(format!("comm_hive: unknown operation '{op}'")))
                }
            }
            _ => Ok(ToolCallResult::error(format!("comm_hive: unknown operation '{op}'"))),
        }
    }

    fn route_consent(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "grant" | "revoke" => {
                // handle_manage_consent expects an "action" field
                let mut p = params.clone();
                if let Some(obj) = p.as_object_mut() {
                    obj.insert("action".to_string(), json!(op));
                }
                Self::handle_manage_consent(&p, session)
            }
            "check"        => Self::handle_check_consent(params, session),
            "list_gates"   => Self::handle_list_consent_gates(params, session),
            "list_pending" => Self::handle_list_pending_consent(params, session),
            "respond"      => Self::handle_respond_consent(params, session),
            _ => Ok(ToolCallResult::error(format!("comm_consent: unknown operation '{op}'"))),
        }
    }

    fn route_trust(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "set"  => Self::handle_set_trust_level(params, session),
            "get"  => Self::handle_get_trust_level(params, session),
            "list" => Self::handle_list_trust_levels(session),
            _ => Ok(ToolCallResult::error(format!("comm_trust: unknown operation '{op}'"))),
        }
    }

    fn route_keys(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "generate_keypair"   => Self::handle_generate_keypair(session),
            "get_public_key"     => Self::handle_get_public_key(session),
            "encrypt"            => Self::handle_encrypt_message(params, session),
            "decrypt"            => Self::handle_decrypt_message(params, session),
            "verify_signature"   => Self::handle_verify_signature(params, session),
            "generate"           => Self::handle_generate_key(params, session),
            "list"               => Self::handle_list_keys(session),
            "rotate"             => Self::handle_rotate_key(params, session),
            "revoke"             => Self::handle_revoke_key(params, session),
            "export"             => Self::handle_export_key(params, session),
            "get"                => Self::handle_get_key(params, session),
            _ => Ok(ToolCallResult::error(format!("comm_keys: unknown operation '{op}'"))),
        }
    }

    fn route_federation(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "configure"       => Self::handle_configure_federation(params, session),
            "status"          => Self::handle_get_federation_status(session),
            "set_policy"      => Self::handle_set_federation_policy(params, session),
            "add_zone"        => Self::handle_add_federated_zone(params, session),
            "remove_zone"     => Self::handle_remove_federated_zone(params, session),
            "list_zones"      => Self::handle_list_federated_zones(session),
            "set_zone_policy" => Self::handle_set_zone_policy(params, session),
            // Invention-federation pass-through
            name if name.starts_with("comm_federation_gateway_")
                || name.starts_with("comm_federation_route_")
                || name.starts_with("comm_federation_zone_")
                || name.starts_with("comm_reality_") =>
            {
                if let Some(result) = invention_federation::try_execute(name, params.clone(), session) {
                    result.map_err(|e| McpError::InternalError(e))
                } else {
                    Ok(ToolCallResult::error(format!("comm_federation: unknown operation '{op}'")))
                }
            }
            _ => Ok(ToolCallResult::error(format!("comm_federation: unknown operation '{op}'"))),
        }
    }

    fn route_temporal(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "schedule"            => Self::handle_schedule_message(params, session),
            "list_scheduled"      => Self::handle_list_scheduled(session),
            "cancel_scheduled"    => Self::handle_cancel_scheduled(params, session),
            "deliver_pending"     => Self::handle_deliver_pending(session),
            "list_dead_letters"   => Self::handle_list_dead_letters(session),
            "replay_dead_letter"  => Self::handle_replay_dead_letter(params, session),
            "clear_dead_letters"  => Self::handle_clear_dead_letters(session),
            // Invention-temporal pass-through
            name if name.starts_with("comm_precognition_")
                || name.starts_with("comm_temporal_")
                || name.starts_with("comm_legacy_")
                || name.starts_with("comm_dead_letter_") =>
            {
                if let Some(result) = invention_temporal::try_execute(name, params.clone(), session) {
                    result.map_err(|e| McpError::InternalError(e))
                } else {
                    Ok(ToolCallResult::error(format!("comm_temporal: unknown operation '{op}'")))
                }
            }
            _ => Ok(ToolCallResult::error(format!("comm_temporal: unknown operation '{op}'"))),
        }
    }

    fn route_query(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "relationships"  => Self::handle_query_relationships(params, session),
            "echoes"         => Self::handle_query_echoes(params, session),
            "conversations"  => Self::handle_query_conversations(params, session),
            "ground"         => Self::handle_comm_ground(params, session),
            "evidence"       => Self::handle_comm_evidence(params, session),
            "suggest"        => Self::handle_comm_suggest(params, session),
            _ => Ok(ToolCallResult::error(format!("comm_query: unknown operation '{op}'"))),
        }
    }

    fn route_forensics(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        // All forensics operations are invention module names
        match op.as_str() {
            name if name.starts_with("comm_forensics_")
                || name.starts_with("comm_pattern_")
                || name.starts_with("comm_health_")
                || name.starts_with("comm_oracle_") =>
            {
                if let Some(result) = invention_forensics::try_execute(name, params.clone(), session) {
                    result.map_err(|e| McpError::InternalError(e))
                } else {
                    Ok(ToolCallResult::error(format!("comm_forensics: unknown operation '{op}'")))
                }
            }
            _ => Ok(ToolCallResult::error(format!("comm_forensics: unknown operation '{op}'"))),
        }
    }

    fn route_collaboration(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        // All collaboration operations are invention module names
        match op.as_str() {
            name if name.starts_with("comm_hive_consciousness_")
                || name.starts_with("comm_collective_intelligence_")
                || name.starts_with("comm_ancestor_")
                || name.starts_with("comm_telepathy_") =>
            {
                if let Some(result) = invention_collaboration::try_execute(name, params.clone(), session) {
                    result.map_err(|e| McpError::InternalError(e))
                } else {
                    Ok(ToolCallResult::error(format!("comm_collaboration: unknown operation '{op}'")))
                }
            }
            _ => Ok(ToolCallResult::error(format!("comm_collaboration: unknown operation '{op}'"))),
        }
    }

    fn route_workspace(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "create"  => Self::handle_workspace_create(params, session),
            "add"     => Self::handle_workspace_add(params, session),
            "list"    => Self::handle_workspace_list(params, session),
            "query"   => Self::handle_workspace_query(params, session),
            "compare" => Self::handle_workspace_compare(params, session),
            "xref"    => Self::handle_workspace_xref(params, session),
            _ => Ok(ToolCallResult::error(format!("comm_workspace: unknown operation '{op}'"))),
        }
    }

    fn route_session(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "start"             => Self::handle_session_start(params, session),
            "end"               => Self::handle_session_end(params, session),
            "resume"            => Self::handle_session_resume(params, session),
            "conversation_log"  => Self::handle_conversation_log(params, session),
            "communication_log" => Self::handle_communication_log(params, session),
            "log_communication" => Self::handle_log_communication(params, session),
            "get_comm_log"      => Self::handle_get_comm_log(params, session),
            "get_audit_log"     => Self::handle_get_audit_log(params, session),
            _ => Ok(ToolCallResult::error(format!("comm_session: unknown operation '{op}'"))),
        }
    }

    fn route_agent(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "stats"           => Self::handle_get_stats(session),
            "assign_comm_ids" => Self::handle_assign_comm_ids(session),
            "get_by_comm_id"  => Self::handle_get_by_comm_id(params, session),
            _ => Ok(ToolCallResult::error(format!("comm_agent: unknown operation '{op}'"))),
        }
    }

    fn route_store(params: &Value, session: &mut SessionManager) -> Result<ToolCallResult, McpError> {
        let op = Self::get_operation(params)?;
        match op.as_str() {
            "expire"             => Self::handle_expire_messages(session),
            "compact"            => Self::handle_compact(session),
            "list_dead_letters"  => Self::handle_list_dead_letters(session),
            "clear_dead_letters" => Self::handle_clear_dead_letters(session),
            _ => Ok(ToolCallResult::error(format!("comm_store: unknown operation '{op}'"))),
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

    // -----------------------------------------------------------------------
    // Affect contagion / Echo chain / Summarization handlers
    // -----------------------------------------------------------------------

    fn handle_process_affect_contagion(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;

        let results = session.store.process_affect_contagion(channel_id);
        session.record_operation("comm_process_affect_contagion", Some(channel_id));

        let entries: Vec<serde_json::Value> = results
            .iter()
            .map(|(agent, v, a, d)| {
                json!({
                    "agent": agent,
                    "valence": v,
                    "arousal": a,
                    "dominance": d,
                })
            })
            .collect();

        Ok(ToolCallResult::json(&json!({
            "channel_id": channel_id,
            "affected_agents": entries.len(),
            "results": entries,
        })))
    }

    fn handle_get_affect_history(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let agent = params
            .get("agent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("agent is required".to_string()))?;

        let history = session.store.get_affect_history(agent);
        session.record_operation("comm_get_affect_history", None);

        Ok(ToolCallResult::json(&history))
    }

    fn handle_apply_affect_decay(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let decay_rate = params
            .get("decay_rate")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::InvalidParams("decay_rate is required".to_string()))?;

        session.store.apply_affect_decay(decay_rate);
        session.record_operation("comm_apply_affect_decay", None);

        Ok(ToolCallResult::json(&json!({
            "decay_rate": decay_rate,
            "message": format!("Applied affect decay with rate {}", decay_rate),
        })))
    }

    fn handle_forward_message(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let message_id = params
            .get("message_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("message_id is required".to_string()))?;
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;
        let forwarder = params
            .get("forwarder")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("forwarder is required".to_string()))?;

        match session.store.forward_message(message_id, channel_id, forwarder) {
            Ok(new_id) => {
                let depth = session.store.get_echo_depth(new_id);
                session.record_operation("comm_forward_message", Some(new_id));
                Ok(ToolCallResult::json(&json!({
                    "new_message_id": new_id,
                    "original_message_id": message_id,
                    "target_channel_id": channel_id,
                    "forwarder": forwarder,
                    "echo_depth": depth,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e)),
        }
    }

    fn handle_query_echo_chain(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let message_id = params
            .get("message_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("message_id is required".to_string()))?;

        let chain = session.store.query_echo_chain(message_id);
        session.record_operation("comm_query_echo_chain", Some(message_id));

        Ok(ToolCallResult::json(&json!({
            "message_id": message_id,
            "chain_length": chain.len(),
            "chain": chain,
        })))
    }

    fn handle_get_echo_depth(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let message_id = params
            .get("message_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("message_id is required".to_string()))?;

        let depth = session.store.get_echo_depth(message_id);
        session.record_operation("comm_get_echo_depth", Some(message_id));

        Ok(ToolCallResult::json(&json!({
            "message_id": message_id,
            "echo_depth": depth,
        })))
    }

    fn handle_summarize_conversation(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let channel_id = params
            .get("channel_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("channel_id is required".to_string()))?;

        match session.store.summarize_conversation(channel_id) {
            Ok(summary) => {
                session.record_operation("comm_summarize_conversation", Some(channel_id));
                Ok(ToolCallResult::json(&summary))
            }
            Err(e) => Ok(ToolCallResult::error(e)),
        }
    }

    // -----------------------------------------------------------------------
    // Rich MessageContent and CommId handlers
    // -----------------------------------------------------------------------

    fn handle_send_rich_message(
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
        let content_type = params
            .get("content_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("content_type is required".to_string()))?;
        let content_data = params
            .get("content_data")
            .ok_or_else(|| McpError::InvalidParams("content_data is required".to_string()))?;

        use agentic_comm::{
            MessageContent, SemanticContent, AffectContent, FullContent,
            TemporalContent, PrecognitiveContent, MetaContent, UnspeakableContent,
        };

        let content: MessageContent = match content_type {
            "text" => {
                let text = content_data
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                MessageContent::Text(text.to_string())
            }
            "semantic" => {
                let sc: SemanticContent = serde_json::from_value(content_data.clone())
                    .map_err(|e| McpError::InvalidParams(format!("Invalid semantic content: {}", e)))?;
                MessageContent::Semantic(sc)
            }
            "affect" => {
                let ac: AffectContent = serde_json::from_value(content_data.clone())
                    .map_err(|e| McpError::InvalidParams(format!("Invalid affect content: {}", e)))?;
                MessageContent::Affect(ac)
            }
            "full" => {
                let fc: FullContent = serde_json::from_value(content_data.clone())
                    .map_err(|e| McpError::InvalidParams(format!("Invalid full content: {}", e)))?;
                MessageContent::Full(fc)
            }
            "temporal" => {
                let tc: TemporalContent = serde_json::from_value(content_data.clone())
                    .map_err(|e| McpError::InvalidParams(format!("Invalid temporal content: {}", e)))?;
                MessageContent::Temporal(tc)
            }
            "precognitive" => {
                let pc: PrecognitiveContent = serde_json::from_value(content_data.clone())
                    .map_err(|e| McpError::InvalidParams(format!("Invalid precognitive content: {}", e)))?;
                MessageContent::Precognitive(pc)
            }
            "meta" => {
                let mc: MetaContent = serde_json::from_value(content_data.clone())
                    .map_err(|e| McpError::InvalidParams(format!("Invalid meta content: {}", e)))?;
                MessageContent::Meta(mc)
            }
            "unspeakable" => {
                let uc: UnspeakableContent = serde_json::from_value(content_data.clone())
                    .map_err(|e| McpError::InvalidParams(format!("Invalid unspeakable content: {}", e)))?;
                MessageContent::Unspeakable(uc)
            }
            other => {
                return Ok(ToolCallResult::error(format!(
                    "Unknown content_type: {}. Expected: text, semantic, affect, full, temporal, precognitive, meta, unspeakable",
                    other
                )));
            }
        };

        match session.store.send_rich_message(
            channel_id,
            sender,
            content,
            agentic_comm::MessageType::Text,
        ) {
            Ok(msg) => {
                session.record_operation("comm_send_rich_message", Some(msg.id));
                Ok(ToolCallResult::json(&json!({
                    "id": msg.id,
                    "channel_id": msg.channel_id,
                    "sender": msg.sender,
                    "content": msg.content,
                    "has_rich_content": msg.rich_content_json.is_some(),
                    "comm_id": msg.comm_id.map(|c| c.to_string()),
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_get_rich_content(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let message_id = params
            .get("message_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::InvalidParams("message_id is required".to_string()))?;

        match session.store.get_rich_content(message_id) {
            Ok(Some(content)) => {
                session.record_operation("comm_get_rich_content", Some(message_id));
                Ok(ToolCallResult::json(&json!({
                    "message_id": message_id,
                    "has_rich_content": true,
                    "content": content,
                })))
            }
            Ok(None) => {
                session.record_operation("comm_get_rich_content", Some(message_id));
                Ok(ToolCallResult::json(&json!({
                    "message_id": message_id,
                    "has_rich_content": false,
                    "content": null,
                })))
            }
            Err(e) => Ok(ToolCallResult::error(e.to_string())),
        }
    }

    fn handle_assign_comm_ids(
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        session.store.assign_comm_ids();
        session.record_operation("comm_assign_comm_ids", None);

        let msg_count = session.store.messages.values()
            .filter(|m| m.comm_id.is_some())
            .count();
        let chan_count = session.store.channels.values()
            .filter(|c| c.comm_id.is_some())
            .count();

        Ok(ToolCallResult::json(&json!({
            "assigned": true,
            "messages_with_comm_id": msg_count,
            "channels_with_comm_id": chan_count,
        })))
    }

    fn handle_get_by_comm_id(
        params: &Value,
        session: &mut SessionManager,
    ) -> Result<ToolCallResult, McpError> {
        let comm_id_str = params
            .get("comm_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("comm_id is required".to_string()))?;

        let comm_id: agentic_comm::CommId = comm_id_str
            .parse()
            .map_err(|e: uuid::Error| McpError::InvalidParams(format!("Invalid UUID: {}", e)))?;

        session.record_operation("comm_get_by_comm_id", None);

        // Try message first, then channel
        if let Some(msg) = session.store.get_message_by_comm_id(&comm_id) {
            return Ok(ToolCallResult::json(&json!({
                "found": true,
                "type": "message",
                "data": {
                    "id": msg.id,
                    "channel_id": msg.channel_id,
                    "sender": msg.sender,
                    "content": msg.content,
                    "comm_id": msg.comm_id.map(|c| c.to_string()),
                },
            })));
        }

        if let Some(chan) = session.store.get_channel_by_comm_id(&comm_id) {
            return Ok(ToolCallResult::json(&json!({
                "found": true,
                "type": "channel",
                "data": {
                    "id": chan.id,
                    "name": chan.name,
                    "channel_type": format!("{}", chan.channel_type),
                    "comm_id": chan.comm_id.map(|c| c.to_string()),
                },
            })));
        }

        Ok(ToolCallResult::json(&json!({
            "found": false,
            "comm_id": comm_id_str,
        })))
    }
}

// -------------------------------------------------------------------
// Tool extraction index
// -------------------------------------------------------------------

#[allow(dead_code)]
fn _tool_extraction_index(tool_name: &str) -> bool {
    tool_name == "comm_channel"
        || tool_name == "comm_message"
        || tool_name == "comm_semantic"
        || tool_name == "comm_affect"
        || tool_name == "comm_hive"
        || tool_name == "comm_consent"
        || tool_name == "comm_trust"
        || tool_name == "comm_keys"
        || tool_name == "comm_federation"
        || tool_name == "comm_temporal"
        || tool_name == "comm_query"
        || tool_name == "comm_forensics"
        || tool_name == "comm_collaboration"
        || tool_name == "comm_workspace"
        || tool_name == "comm_session"
        || tool_name == "comm_agent"
        || tool_name == "comm_store"
}
