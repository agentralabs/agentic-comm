//! Prompt template: "Analyze a channel's health."

use serde_json::Value;

use crate::types::{McpError, McpResult, PromptGetResult, PromptMessage, ToolContent};

/// Expand the `channel-health` prompt with the given arguments.
pub fn expand(args: Value) -> McpResult<PromptGetResult> {
    let channel_id = args
        .get("channel_id")
        .and_then(|v| v.as_str().or_else(|| v.as_u64().map(|_| "")))
        .ok_or_else(|| {
            McpError::InvalidParams("'channel_id' argument is required".to_string())
        })?;

    // Accept channel_id as either string or number
    let channel_id_str = if channel_id.is_empty() {
        args.get("channel_id")
            .and_then(|v| v.as_u64())
            .map(|n| n.to_string())
            .unwrap_or_default()
    } else {
        channel_id.to_string()
    };

    let time_window = args
        .get("time_window")
        .and_then(|v| v.as_str())
        .unwrap_or("24h");

    let system_text = format!(
        "You are performing a health check on communication channel {channel_id_str}.\n\n\
         Use these MCP resources to gather data:\n\
         1. `comm://channels/{channel_id_str}` \u{2014} channel details and configuration\n\
         2. `comm://channels/{channel_id_str}/messages` \u{2014} message history\n\
         3. `comm://dead-letters` \u{2014} check for dead letters related to this channel\n\n\
         Analysis time window: {time_window}"
    );

    let user_text = format!(
        "Please analyze channel {channel_id_str} and provide:\n\
         1. **Delivery health:** What percentage of messages were delivered? \
            Are there delivery failures?\n\
         2. **Acknowledgment rate:** For command and query messages, what percentage \
            received timely acknowledgments?\n\
         3. **Participant balance:** Are all participants active, or are some silent? \
            Is communication balanced or dominated by one sender?\n\
         4. **Message type distribution:** What types of messages flow through this \
            channel? Is the distribution healthy for the channel type?\n\
         5. **Latency patterns:** Are there gaps in communication? Long periods of \
            silence followed by bursts?\n\
         6. **Configuration review:** Is the channel configuration appropriate?\n\
            - Is the max_participants limit reasonable?\n\
            - Is the TTL appropriate for the message types?\n\
            - Should encryption be enabled?\n\
         7. **Recommendations:** Specific actions to improve channel health."
    );

    Ok(PromptGetResult {
        description: Some(format!(
            "Health analysis for channel {channel_id_str}"
        )),
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
