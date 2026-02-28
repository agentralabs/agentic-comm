//! Prompt registration and dispatch for MCP prompt templates.

use serde_json::Value;

use crate::types::{McpError, McpResult, PromptArgument, PromptDefinition, PromptGetResult};

use super::{affect_calibrate, compose_message, hive_decision, semantic_extract};

/// Registry of all available MCP prompts.
pub struct PromptRegistry;

impl PromptRegistry {
    /// List all available prompt definitions.
    pub fn list_prompts() -> Vec<PromptDefinition> {
        vec![
            PromptDefinition {
                name: "compose_message".to_string(),
                description: Some(
                    "Help compose a message with appropriate content and affect".to_string(),
                ),
                arguments: Some(vec![
                    PromptArgument {
                        name: "recipient".to_string(),
                        description: Some("Target recipient agent ID".to_string()),
                        required: true,
                    },
                    PromptArgument {
                        name: "context".to_string(),
                        description: Some("Communication context or purpose".to_string()),
                        required: false,
                    },
                    PromptArgument {
                        name: "mood".to_string(),
                        description: Some(
                            "Desired mood/affect (e.g. friendly, urgent, formal)".to_string(),
                        ),
                        required: false,
                    },
                ]),
            },
            PromptDefinition {
                name: "semantic_extract".to_string(),
                description: Some(
                    "Help extract the right semantic fragment to share".to_string(),
                ),
                arguments: Some(vec![
                    PromptArgument {
                        name: "topic".to_string(),
                        description: Some("Topic to extract a fragment about".to_string()),
                        required: true,
                    },
                    PromptArgument {
                        name: "audience".to_string(),
                        description: Some("Target audience for the fragment".to_string()),
                        required: false,
                    },
                ]),
            },
            PromptDefinition {
                name: "hive_decision".to_string(),
                description: Some(
                    "Help make a collective decision in a hive mind".to_string(),
                ),
                arguments: Some(vec![
                    PromptArgument {
                        name: "problem".to_string(),
                        description: Some("Problem to decide on collectively".to_string()),
                        required: true,
                    },
                    PromptArgument {
                        name: "hive_id".to_string(),
                        description: Some("Hive mind ID to use".to_string()),
                        required: false,
                    },
                ]),
            },
            PromptDefinition {
                name: "affect_calibrate".to_string(),
                description: Some(
                    "Help calibrate affect for communication".to_string(),
                ),
                arguments: Some(vec![
                    PromptArgument {
                        name: "situation".to_string(),
                        description: Some("Situation requiring affect calibration".to_string()),
                        required: true,
                    },
                    PromptArgument {
                        name: "relationship".to_string(),
                        description: Some(
                            "Relationship context between communicating agents".to_string(),
                        ),
                        required: false,
                    },
                ]),
            },
        ]
    }

    /// Expand a prompt with the given arguments, dispatching to the appropriate handler.
    pub fn get(name: &str, arguments: Option<Value>) -> McpResult<PromptGetResult> {
        let args = arguments.unwrap_or(Value::Object(serde_json::Map::new()));

        match name {
            "compose_message" => compose_message::expand(args),
            "semantic_extract" => semantic_extract::expand(args),
            "hive_decision" => hive_decision::expand(args),
            "affect_calibrate" => affect_calibrate::expand(args),
            _ => Err(McpError::PromptNotFound(name.to_string())),
        }
    }
}
