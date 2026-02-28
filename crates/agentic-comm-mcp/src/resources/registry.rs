//! Resource registration and dispatch for MCP resources.

use std::sync::Arc;
use tokio::sync::Mutex;

use serde_json;

use crate::session::manager::SessionManager;
use crate::types::{
    McpError, McpResult, ReadResourceResult, ResourceContent, ResourceDefinition,
    ResourceTemplateDefinition, ResourceTemplateListResult,
};

/// Registry of all available MCP resources.
pub struct ResourceRegistry;

impl ResourceRegistry {
    /// List all concrete (non-templated) resources.
    pub fn list_resources() -> Vec<ResourceDefinition> {
        vec![
            ResourceDefinition {
                uri: "comm://store/stats".to_string(),
                name: "Store Statistics".to_string(),
                description: Some(
                    "Communication store statistics (channel count, message count, etc.)"
                        .to_string(),
                ),
                mime_type: Some("application/json".to_string()),
            },
            ResourceDefinition {
                uri: "comm://channels".to_string(),
                name: "Communication Channels".to_string(),
                description: Some("List all communication channels".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            ResourceDefinition {
                uri: "comm://consent".to_string(),
                name: "Consent Gates".to_string(),
                description: Some("List consent gates".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            ResourceDefinition {
                uri: "comm://trust".to_string(),
                name: "Trust Level Overrides".to_string(),
                description: Some("List trust level overrides".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            ResourceDefinition {
                uri: "comm://hives".to_string(),
                name: "Active Hive Minds".to_string(),
                description: Some("List active hive minds".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            ResourceDefinition {
                uri: "comm://federation".to_string(),
                name: "Federation Configuration".to_string(),
                description: Some("Get federation configuration".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            ResourceDefinition {
                uri: "comm://relationships".to_string(),
                name: "Agent Relationships".to_string(),
                description: Some("List all agent relationships".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            ResourceDefinition {
                uri: "comm://affect".to_string(),
                name: "Affect State".to_string(),
                description: Some("Get current affect state for all agents".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            ResourceDefinition {
                uri: "comm://dead-letters".to_string(),
                name: "Dead Letter Queue".to_string(),
                description: Some("Dead letter queue contents".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ]
    }

    /// List all resource URI templates.
    pub fn list_templates() -> Vec<ResourceTemplateDefinition> {
        vec![
            ResourceTemplateDefinition {
                uri_template: "comm://channels/{channel_id}/messages".to_string(),
                name: "Channel Messages".to_string(),
                description: Some("List messages in a channel".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            ResourceTemplateDefinition {
                uri_template: "comm://channels/{channel_id}".to_string(),
                name: "Channel Details".to_string(),
                description: Some("Get details for a specific channel".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            ResourceTemplateDefinition {
                uri_template: "comm://messages/{message_id}".to_string(),
                name: "Message Details".to_string(),
                description: Some("Get details for a specific message".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ]
    }

    /// List templates wrapped in the standard result type.
    pub fn list_templates_result() -> ResourceTemplateListResult {
        ResourceTemplateListResult {
            resource_templates: Self::list_templates(),
            next_cursor: None,
        }
    }

    /// Read a resource by URI, dispatching to the appropriate handler.
    pub async fn read(
        uri: &str,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        // Match static resources first
        if uri == "comm://store/stats" {
            return Self::read_store_stats(session).await;
        }
        if uri == "comm://channels" {
            return Self::read_channels(session).await;
        }
        if uri == "comm://consent" {
            return Self::read_consent(session).await;
        }
        if uri == "comm://trust" {
            return Self::read_trust(session).await;
        }
        if uri == "comm://hives" {
            return Self::read_hives(session).await;
        }
        if uri == "comm://federation" {
            return Self::read_federation(session).await;
        }
        if uri == "comm://relationships" {
            return Self::read_relationships(session).await;
        }
        if uri == "comm://affect" {
            return Self::read_affect(session).await;
        }
        if uri == "comm://dead-letters" {
            return Self::read_dead_letters(session).await;
        }

        // Match templated resources: comm://channels/{id}/messages
        if let Some(rest) = uri.strip_prefix("comm://channels/") {
            if let Some(id_str) = rest.strip_suffix("/messages") {
                let channel_id: u64 = id_str.parse().map_err(|_| {
                    McpError::InvalidParams(format!("Invalid channel ID: {id_str}"))
                })?;
                return Self::read_channel_messages(channel_id, session).await;
            }
            // Match comm://channels/{id} (single channel details)
            let channel_id: u64 = rest.parse().map_err(|_| {
                McpError::InvalidParams(format!("Invalid channel ID: {rest}"))
            })?;
            return Self::read_channel_detail(channel_id, session).await;
        }

        // Match templated resources: comm://messages/{id}
        if let Some(id_str) = uri.strip_prefix("comm://messages/") {
            let message_id: u64 = id_str.parse().map_err(|_| {
                McpError::InvalidParams(format!("Invalid message ID: {id_str}"))
            })?;
            return Self::read_message_detail(message_id, session).await;
        }

        Err(McpError::ResourceNotFound(uri.to_string()))
    }

    async fn read_channels(
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let session = session.lock().await;
        let channels = session.store.list_channels();
        let text = serde_json::to_string_pretty(&channels)
            .map_err(|e| McpError::InternalError(e.to_string()))?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri: "comm://channels".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }

    async fn read_channel_messages(
        channel_id: u64,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let mut session = session.lock().await;
        let messages = session
            .store
            .receive_messages(channel_id, None, None)
            .map_err(McpError::from)?;
        let text = serde_json::to_string_pretty(&messages)
            .map_err(|e| McpError::InternalError(e.to_string()))?;
        let uri = format!("comm://channels/{channel_id}/messages");

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri,
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }

    async fn read_consent(
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let session = session.lock().await;
        let consent = session.store.list_consent_gates(None);
        let text = serde_json::to_string_pretty(&consent)
            .map_err(|e| McpError::InternalError(e.to_string()))?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri: "comm://consent".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }

    async fn read_trust(
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let session = session.lock().await;
        let trust = session.store.list_trust_levels();
        let text = serde_json::to_string_pretty(&trust)
            .map_err(|e| McpError::InternalError(e.to_string()))?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri: "comm://trust".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }

    async fn read_hives(
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let session = session.lock().await;
        let hives = session.store.list_hives();
        let text = serde_json::to_string_pretty(&hives)
            .map_err(|e| McpError::InternalError(e.to_string()))?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri: "comm://hives".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }

    async fn read_federation(
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let session = session.lock().await;
        let federation = session.store.get_federation_config();
        let text = serde_json::to_string_pretty(&federation)
            .map_err(|e| McpError::InternalError(e.to_string()))?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri: "comm://federation".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }

    async fn read_channel_detail(
        channel_id: u64,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let session = session.lock().await;
        let channel = session.store.get_channel(channel_id).ok_or_else(|| {
            McpError::ResourceNotFound(format!("comm://channels/{channel_id}"))
        })?;
        let text = serde_json::to_string_pretty(&channel)
            .map_err(|e| McpError::InternalError(e.to_string()))?;
        let uri = format!("comm://channels/{channel_id}");

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri,
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }

    async fn read_message_detail(
        message_id: u64,
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let session = session.lock().await;
        let message = session.store.messages.get(&message_id).ok_or_else(|| {
            McpError::ResourceNotFound(format!("comm://messages/{message_id}"))
        })?;
        let text = serde_json::to_string_pretty(&message)
            .map_err(|e| McpError::InternalError(e.to_string()))?;
        let uri = format!("comm://messages/{message_id}");

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri,
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }

    async fn read_relationships(
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let session = session.lock().await;
        // Collect relationships for all known agents
        let mut all_agents: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for agent_id in session.store.trust_levels.keys() {
            all_agents.insert(agent_id.clone());
        }
        for gate in &session.store.consent_gates {
            all_agents.insert(gate.grantor.clone());
            all_agents.insert(gate.grantee.clone());
        }
        for hive in session.store.hive_minds.values() {
            for c in &hive.constituents {
                all_agents.insert(c.agent_id.clone());
            }
        }
        let mut results = Vec::new();
        for agent_id in &all_agents {
            let rel = session.store.query_relationships(agent_id, None, 1);
            results.push(rel);
        }
        let text = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::InternalError(e.to_string()))?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri: "comm://relationships".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }

    async fn read_affect(
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let session = session.lock().await;
        let affect = &session.store.affect_states;
        let text = serde_json::to_string_pretty(&affect)
            .map_err(|e| McpError::InternalError(e.to_string()))?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri: "comm://affect".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }

    async fn read_store_stats(
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let session = session.lock().await;
        let stats = session.store.stats();
        let text = serde_json::to_string_pretty(&stats)
            .map_err(|e| McpError::InternalError(e.to_string()))?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri: "comm://store/stats".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }

    async fn read_dead_letters(
        session: &Arc<Mutex<SessionManager>>,
    ) -> McpResult<ReadResourceResult> {
        let session = session.lock().await;
        let dead_letters = session.store.list_dead_letters();
        let text = serde_json::to_string_pretty(&dead_letters)
            .map_err(|e| McpError::InternalError(e.to_string()))?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContent {
                uri: "comm://dead-letters".to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            }],
        })
    }
}
