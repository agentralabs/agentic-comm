//! Prompt template: "Summarize a message thread."

use serde_json::Value;

use crate::types::{McpError, McpResult, PromptGetResult, PromptMessage, ToolContent};

/// Expand the `thread-summary` prompt with the given arguments.
pub fn expand(args: Value) -> McpResult<PromptGetResult> {
    let message_id = args
        .get("message_id")
        .and_then(|v| v.as_str().or_else(|| v.as_u64().map(|_| "")))
        .ok_or_else(|| {
            McpError::InvalidParams("'message_id' argument is required".to_string())
        })?;

    // Accept message_id as either string or number
    let message_id_str = if message_id.is_empty() {
        args.get("message_id")
            .and_then(|v| v.as_u64())
            .map(|n| n.to_string())
            .unwrap_or_default()
    } else {
        message_id.to_string()
    };

    let include_context = args
        .get("include_context")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let system_text = format!(
        "You are summarizing a message thread from the communication history.\n\n\
         Use these MCP resources and tools to gather data:\n\
         1. `comm://messages/{message_id_str}` \u{2014} starting message details\n\
         2. Use `comm_thread_trace` tool to find all related messages in the thread\n\
         3. Read the channel via `comm://channels/{{channel_id}}` for context"
    );

    let mut user_text = format!(
        "Please trace and summarize the message thread starting from message {message_id_str}.\n\n\
         Provide:\n\
         1. **Thread overview:** What was the purpose of this conversation?\n\
         2. **Participants:** Who was involved and what role did each play?\n\
         3. **Timeline:** Key events in chronological order with timestamps\n\
         4. **Outcome:** Was the thread resolved? If it was a command, was it \
            acknowledged and completed? If it was a query, was it answered?\n\
         5. **Open items:** Any unresolved questions, unacknowledged commands, \
            or pending follow-ups\n\
         6. **Key decisions:** Any decisions made during the thread"
    );

    if include_context {
        user_text.push_str(
            "\n\nAlso include any communication_log context entries \
             (intent and observations) logged during this thread.",
        );
    }

    Ok(PromptGetResult {
        description: Some(format!(
            "Summary of message thread starting from message {message_id_str}"
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
