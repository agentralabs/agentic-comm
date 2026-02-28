//! Prompt template: "Summarize current communication state."

use serde_json::Value;

use crate::types::{McpResult, PromptGetResult, PromptMessage, ToolContent};

/// Expand the `comm-status` prompt with the given arguments.
pub fn expand(args: Value) -> McpResult<PromptGetResult> {
    let include_messages = args
        .get("include_messages")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let message_limit = args
        .get("message_limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10);

    let include_subscriptions = args
        .get("include_subscriptions")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let system_text = "\
You are analyzing the communication state of an AgenticComm system. \
Use the available MCP resources to gather data before summarizing.\n\n\
Start by reading these resources:\n\
1. `comm://store/stats` — overall store statistics\n\
2. `comm://channels` — list of all channels\n\
3. `comm://dead-letters` — dead letter queue\n\
4. `comm://relationships` — agent relationships\n\
5. `comm://affect` — current affect state"
        .to_string();

    let mut user_text = String::from(
        "Please analyze the communication system and provide:\n\
         1. A summary of the communication topology (who talks to whom, through which channels)\n\
         2. Activity levels per channel (active, idle, stale)\n\
         3. Any potential issues:\n\
            - Channels with no participants\n\
            - Channels with no recent messages (possible abandoned channels)\n\
            - Unacknowledged command messages (possible dropped tasks)\n\
            - Dead letter queue status\n\
         4. Recommendations for communication hygiene (cleanup suggestions)\n",
    );

    if include_messages {
        user_text.push_str(&format!(
            "\nInclude recent message summaries (last {message_limit} per channel)."
        ));
    }

    if include_subscriptions {
        user_text.push_str("\nInclude subscription information.");
    }

    Ok(PromptGetResult {
        description: Some(
            "Summarize current communication state: channels, pending messages, recent activity"
                .to_string(),
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
