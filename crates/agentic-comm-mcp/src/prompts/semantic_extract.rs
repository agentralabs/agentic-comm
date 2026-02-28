//! Prompt template: "Help extract the right semantic fragment to share."

use serde_json::Value;

use crate::types::{McpError, McpResult, PromptGetResult, PromptMessage, ToolContent};

/// Expand the `semantic_extract` prompt with the given arguments.
pub fn expand(args: Value) -> McpResult<PromptGetResult> {
    let topic = args
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("'topic' argument is required".to_string()))?;

    let audience = args
        .get("audience")
        .and_then(|v| v.as_str())
        .unwrap_or("any agent");

    let system_text = format!(
        "You are helping extract a semantic fragment from a larger body of knowledge. \
         A semantic fragment is a self-contained piece of meaning that can be shared \
         across agent boundaries without losing its essential context.\n\n\
         Guidelines:\n\
         - Extract the minimal viable fragment that conveys the meaning\n\
         - Preserve enough context for the fragment to stand alone\n\
         - Consider the audience's trust level and consent boundaries\n\
         - Tag with appropriate metadata for searchability"
    );

    let user_text = format!(
        "I need to extract a semantic fragment about: {topic}\n\n\
         Target audience: {audience}\n\n\
         Please:\n\
         1. Identify the core meaning to extract about this topic\n\
         2. Determine what context is needed for the fragment to be self-contained\n\
         3. Consider what should be redacted or abstracted for the audience\n\
         4. Use send_message with message_type SemanticFragment to share it"
    );

    Ok(PromptGetResult {
        description: Some("Guide for extracting a semantic fragment to share".to_string()),
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
