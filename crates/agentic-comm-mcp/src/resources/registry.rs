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
        ]
    }

    /// List all resource URI templates.
    pub fn list_templates() -> Vec<ResourceTemplateDefinition> {
        vec![ResourceTemplateDefinition {
            uri_template: "comm://channels/{channel_id}/messages".to_string(),
            name: "Channel Messages".to_string(),
            description: Some("List messages in a channel".to_string()),
            mime_type: Some("application/json".to_string()),
        }]
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

        // Match templated resources: comm://channels/{id}/messages
        if let Some(rest) = uri.strip_prefix("comm://channels/") {
            if let Some(id_str) = rest.strip_suffix("/messages") {
                let channel_id: u64 = id_str.parse().map_err(|_| {
                    McpError::InvalidParams(format!("Invalid channel ID: {id_str}"))
                })?;
                return Self::read_channel_messages(channel_id, session).await;
            }
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
        let session = session.lock().await;
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
}
