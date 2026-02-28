//! Prompt template: "Help make a collective decision in a hive mind."

use serde_json::Value;

use crate::types::{McpError, McpResult, PromptGetResult, PromptMessage, ToolContent};

/// Expand the `hive_decision` prompt with the given arguments.
pub fn expand(args: Value) -> McpResult<PromptGetResult> {
    let problem = args
        .get("problem")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("'problem' argument is required".to_string()))?;

    let hive_id = args
        .get("hive_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    let system_text = format!(
        "You are facilitating a collective decision-making process within a hive mind. \
         A hive mind enables multiple agents to reach consensus or make collective \
         decisions using various modes (Consensus, Majority, Weighted, Unanimous).\n\n\
         Guidelines:\n\
         - Frame the problem clearly for all participants\n\
         - Choose the appropriate decision mode based on the problem type\n\
         - Ensure all participants have the context they need\n\
         - Respect individual agent autonomy within the collective"
    );

    let user_text = format!(
        "We need to make a collective decision about: {problem}\n\n\
         Hive ID: {hive_id}\n\n\
         Please:\n\
         1. Frame this problem for collective deliberation\n\
         2. Recommend the best decision mode (Consensus, Majority, Weighted, or Unanimous)\n\
         3. Identify what information each participant needs\n\
         4. Use hive_broadcast to share the problem with hive members\n\
         5. Guide the decision process using submit_collective_decision"
    );

    Ok(PromptGetResult {
        description: Some(
            "Guide for making a collective decision in a hive mind".to_string(),
        ),
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
