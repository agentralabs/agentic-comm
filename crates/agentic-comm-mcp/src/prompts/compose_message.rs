//! Prompt template: "Help compose a message with appropriate content and affect."

use serde_json::Value;

use crate::types::{McpError, McpResult, PromptGetResult, PromptMessage, ToolContent};

/// Expand the `compose_message` prompt with the given arguments.
pub fn expand(args: Value) -> McpResult<PromptGetResult> {
    let recipient = args
        .get("recipient")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            McpError::InvalidParams("'recipient' argument is required".to_string())
        })?;

    let context = args
        .get("context")
        .and_then(|v| v.as_str())
        .unwrap_or("general communication");

    let mood = args
        .get("mood")
        .and_then(|v| v.as_str())
        .unwrap_or("neutral");

    let system_text = format!(
        "You are helping compose a message for agent-to-agent communication. \
         Consider the recipient's trust level, the communication context, and \
         appropriate affect calibration.\n\n\
         Guidelines:\n\
         - Match the mood/affect to the situation\n\
         - Consider what semantic fragments are most relevant\n\
         - Respect consent boundaries for the recipient\n\
         - Keep messages concise and purposeful"
    );

    let user_text = format!(
        "Please help me compose a message to: {recipient}\n\n\
         Context: {context}\n\
         Desired mood/affect: {mood}\n\n\
         Please:\n\
         1. Suggest appropriate message content for this recipient and context\n\
         2. Recommend the right affect state (valence, arousal, dominance)\n\
         3. Identify the best message type (Direct, Broadcast, SemanticFragment, etc.)\n\
         4. Use send_message tool to deliver the composed message"
    );

    Ok(PromptGetResult {
        description: Some("Guide for composing a message with appropriate affect".to_string()),
        messages: vec![
            PromptMessage {
                role: "assistant".to_string(),
                content: ToolContent::Text { text: system_text },
            },
            PromptMessage {
                role: "user".to_string(),
                content: ToolContent::Text { text: user_text },
            },
        ],
    })
}
