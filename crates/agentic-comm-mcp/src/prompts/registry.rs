//! Prompt registration and dispatch for MCP prompt templates.

use serde_json::Value;

use crate::types::{McpError, McpResult, PromptArgument, PromptDefinition, PromptGetResult};

use super::{
    affect_calibrate, channel_health, comm_status, compose_message, hive_decision,
    semantic_extract, thread_summary,
};

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
            PromptDefinition {
                name: "comm-status".to_string(),
                description: Some(
                    "Summarize current communication state: channels, pending messages, recent activity"
                        .to_string(),
                ),
                arguments: Some(vec![
                    PromptArgument {
                        name: "include_messages".to_string(),
                        description: Some(
                            "Whether to include recent message summaries (default: true)"
                                .to_string(),
                        ),
                        required: false,
                    },
                    PromptArgument {
                        name: "message_limit".to_string(),
                        description: Some(
                            "Number of recent messages per channel to include (default: 10)"
                                .to_string(),
                        ),
                        required: false,
                    },
                    PromptArgument {
                        name: "include_subscriptions".to_string(),
                        description: Some(
                            "Whether to include subscription information (default: true)"
                                .to_string(),
                        ),
                        required: false,
                    },
                ]),
            },
            PromptDefinition {
                name: "channel-health".to_string(),
                description: Some(
                    "Analyze a channel's health: delivery rate, acknowledgment rate, dead letters"
                        .to_string(),
                ),
                arguments: Some(vec![
                    PromptArgument {
                        name: "channel_id".to_string(),
                        description: Some("ID of the channel to analyze".to_string()),
                        required: true,
                    },
                    PromptArgument {
                        name: "time_window".to_string(),
                        description: Some(
                            "Time window for analysis, e.g. 1h, 24h, 7d (default: 24h)"
                                .to_string(),
                        ),
                        required: false,
                    },
                ]),
            },
            PromptDefinition {
                name: "thread-summary".to_string(),
                description: Some(
                    "Summarize a message thread by correlation ID".to_string(),
                ),
                arguments: Some(vec![
                    PromptArgument {
                        name: "message_id".to_string(),
                        description: Some(
                            "ID of any message in the thread to summarize".to_string(),
                        ),
                        required: true,
                    },
                    PromptArgument {
                        name: "include_context".to_string(),
                        description: Some(
                            "Whether to include communication_log context entries (default: true)"
                                .to_string(),
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
            "comm-status" => comm_status::expand(args),
            "channel-health" => channel_health::expand(args),
            "thread-summary" => thread_summary::expand(args),
            _ => Err(McpError::PromptNotFound(name.to_string())),
        }
    }
}
