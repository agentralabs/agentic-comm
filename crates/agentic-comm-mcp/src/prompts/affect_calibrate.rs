//! Prompt template: "Help calibrate affect for communication."

use serde_json::Value;

use crate::types::{McpError, McpResult, PromptGetResult, PromptMessage, ToolContent};

/// Expand the `affect_calibrate` prompt with the given arguments.
pub fn expand(args: Value) -> McpResult<PromptGetResult> {
    let situation = args
        .get("situation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            McpError::InvalidParams("'situation' argument is required".to_string())
        })?;

    let relationship = args
        .get("relationship")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let system_text = format!(
        "You are helping calibrate the emotional affect for an agent communication. \
         Affect in AgenticComm uses a three-dimensional model:\n\
         - Valence: positive/negative sentiment (-1.0 to 1.0)\n\
         - Arousal: energy/intensity level (0.0 to 1.0)\n\
         - Dominance: assertiveness/confidence (0.0 to 1.0)\n\n\
         Guidelines:\n\
         - Match affect to the communication situation\n\
         - Consider the relationship between communicating agents\n\
         - Avoid affect that could be misinterpreted\n\
         - Use urgency levels appropriately (Low, Normal, High, Critical)"
    );

    let user_text = format!(
        "I need to calibrate the affect for this situation: {situation}\n\n\
         Relationship context: {relationship}\n\n\
         Please:\n\
         1. Analyze the emotional requirements of this situation\n\
         2. Recommend valence, arousal, and dominance values\n\
         3. Suggest the appropriate urgency level\n\
         4. Explain how this affect calibration supports the communication goal\n\
         5. Use set_affect to configure the affect state before sending"
    );

    Ok(PromptGetResult {
        description: Some("Guide for calibrating affect for communication".to_string()),
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
