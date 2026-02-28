//! Input validation for all MCP tool parameters.
//!
//! Every tool parameter is validated before dispatch. Validation failures
//! produce `ToolCallResult::error()` with `isError: true` and a specific
//! error message describing what was wrong.

use serde_json::Value;

use crate::types::response::ToolCallResult;

/// Maximum allowed content size in bytes (1 MB).
const MAX_CONTENT_BYTES: usize = 1_048_576;

/// Maximum allowed name/topic length.
const MAX_NAME_LEN: usize = 128;

/// Maximum allowed limit value.
const MAX_LIMIT: u64 = 10_000;

/// Result type for validation — Ok means valid, Err holds the error result.
pub type ValidationResult = Result<(), ToolCallResult>;

// ---------------------------------------------------------------------------
// Individual field validators
// ---------------------------------------------------------------------------

/// Validate a channel name: 1-128 chars, alphanumeric + hyphen + underscore only.
pub fn validate_channel_name(name: &str) -> ValidationResult {
    if name.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: channel name must not be empty".to_string(),
        ));
    }
    if name.len() > MAX_NAME_LEN {
        return Err(ToolCallResult::error(format!(
            "Validation error: channel name exceeds maximum length of {MAX_NAME_LEN} characters (got {})",
            name.len()
        )));
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ToolCallResult::error(
            "Validation error: channel name must contain only alphanumeric characters, hyphens, or underscores".to_string(),
        ));
    }
    Ok(())
}

/// Validate message content: non-empty, max 1 MB.
pub fn validate_content(content: &str) -> ValidationResult {
    if content.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: content must not be empty".to_string(),
        ));
    }
    if content.len() > MAX_CONTENT_BYTES {
        return Err(ToolCallResult::error(format!(
            "Validation error: content exceeds maximum size of {} bytes (got {} bytes)",
            MAX_CONTENT_BYTES,
            content.len()
        )));
    }
    Ok(())
}

/// Validate a sender identity: must be non-empty.
pub fn validate_sender(sender: &str) -> ValidationResult {
    if sender.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: sender must not be empty".to_string(),
        ));
    }
    Ok(())
}

/// Validate a topic: 1-128 chars, alphanumeric + hyphen + underscore + dot.
pub fn validate_topic(topic: &str) -> ValidationResult {
    if topic.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: topic must not be empty".to_string(),
        ));
    }
    if topic.len() > MAX_NAME_LEN {
        return Err(ToolCallResult::error(format!(
            "Validation error: topic exceeds maximum length of {MAX_NAME_LEN} characters (got {})",
            topic.len()
        )));
    }
    if !topic
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(ToolCallResult::error(
            "Validation error: topic must contain only alphanumeric characters, hyphens, underscores, or dots".to_string(),
        ));
    }
    Ok(())
}

/// Validate a channel_id: must be a positive integer.
pub fn validate_channel_id(params: &Value) -> Result<u64, ToolCallResult> {
    let id = params
        .get("channel_id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            ToolCallResult::error(
                "Validation error: channel_id is required and must be a positive integer"
                    .to_string(),
            )
        })?;
    if id == 0 {
        return Err(ToolCallResult::error(
            "Validation error: channel_id must be a positive integer (got 0)".to_string(),
        ));
    }
    Ok(id)
}

/// Validate a message_id: must be a positive integer.
pub fn validate_message_id(params: &Value) -> Result<u64, ToolCallResult> {
    let id = params
        .get("message_id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            ToolCallResult::error(
                "Validation error: message_id is required and must be a positive integer"
                    .to_string(),
            )
        })?;
    if id == 0 {
        return Err(ToolCallResult::error(
            "Validation error: message_id must be a positive integer (got 0)".to_string(),
        ));
    }
    Ok(id)
}

/// Validate a subscription_id: must be a positive integer.
pub fn validate_subscription_id(params: &Value) -> Result<u64, ToolCallResult> {
    let id = params
        .get("subscription_id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            ToolCallResult::error(
                "Validation error: subscription_id is required and must be a positive integer"
                    .to_string(),
            )
        })?;
    if id == 0 {
        return Err(ToolCallResult::error(
            "Validation error: subscription_id must be a positive integer (got 0)".to_string(),
        ));
    }
    Ok(id)
}

/// Validate a limit value: must be 1-10000 if present.
pub fn validate_limit(params: &Value, field: &str) -> Result<Option<u64>, ToolCallResult> {
    match params.get(field).and_then(|v| v.as_u64()) {
        Some(n) if n == 0 => Err(ToolCallResult::error(format!(
            "Validation error: {field} must be at least 1 (got 0)"
        ))),
        Some(n) if n > MAX_LIMIT => Err(ToolCallResult::error(format!(
            "Validation error: {field} must not exceed {MAX_LIMIT} (got {n})"
        ))),
        Some(n) => Ok(Some(n)),
        None => {
            // Check if the field exists but isn't a valid number
            if params.get(field).is_some() {
                Err(ToolCallResult::error(format!(
                    "Validation error: {field} must be a positive integer"
                )))
            } else {
                Ok(None)
            }
        }
    }
}

/// Validate a search query: must be non-empty.
pub fn validate_query(query: &str) -> ValidationResult {
    if query.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: query must not be empty".to_string(),
        ));
    }
    Ok(())
}

/// Validate a participant/recipient name: must be non-empty.
pub fn validate_participant(name: &str) -> ValidationResult {
    if name.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: participant name must not be empty".to_string(),
        ));
    }
    Ok(())
}

/// Validate a required string field from params, returning the value.
pub fn require_string<'a>(
    params: &'a Value,
    field: &str,
) -> Result<&'a str, ToolCallResult> {
    params
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ToolCallResult::error(format!(
                "Validation error: {field} is required and must be a string"
            ))
        })
}

// ---------------------------------------------------------------------------
// Composite validators for each tool
// ---------------------------------------------------------------------------

/// Validate send_message parameters.
pub fn validate_send_message(params: &Value) -> Result<(), ToolCallResult> {
    validate_channel_id(params)?;
    let sender = require_string(params, "sender")?;
    validate_sender(sender)?;
    let content = require_string(params, "content")?;
    validate_content(content)?;
    Ok(())
}

/// Validate receive_messages parameters.
pub fn validate_receive_messages(params: &Value) -> Result<(), ToolCallResult> {
    validate_channel_id(params)?;
    Ok(())
}

/// Validate create_channel parameters.
pub fn validate_create_channel(params: &Value) -> Result<(), ToolCallResult> {
    let name = require_string(params, "name")?;
    validate_channel_name(name)?;
    Ok(())
}

/// Validate join_channel parameters.
pub fn validate_join_channel(params: &Value) -> Result<(), ToolCallResult> {
    validate_channel_id(params)?;
    let participant = require_string(params, "participant")?;
    validate_participant(participant)?;
    Ok(())
}

/// Validate leave_channel parameters.
pub fn validate_leave_channel(params: &Value) -> Result<(), ToolCallResult> {
    validate_channel_id(params)?;
    let participant = require_string(params, "participant")?;
    validate_participant(participant)?;
    Ok(())
}

/// Validate get_channel_info parameters.
pub fn validate_get_channel_info(params: &Value) -> Result<(), ToolCallResult> {
    validate_channel_id(params)?;
    Ok(())
}

/// Validate subscribe parameters.
pub fn validate_subscribe(params: &Value) -> Result<(), ToolCallResult> {
    let topic = require_string(params, "topic")?;
    validate_topic(topic)?;
    let subscriber = require_string(params, "subscriber")?;
    validate_sender(subscriber)?;
    Ok(())
}

/// Validate unsubscribe parameters.
pub fn validate_unsubscribe(params: &Value) -> Result<(), ToolCallResult> {
    validate_subscription_id(params)?;
    Ok(())
}

/// Validate publish parameters.
pub fn validate_publish(params: &Value) -> Result<(), ToolCallResult> {
    let topic = require_string(params, "topic")?;
    validate_topic(topic)?;
    let sender = require_string(params, "sender")?;
    validate_sender(sender)?;
    let content = require_string(params, "content")?;
    validate_content(content)?;
    Ok(())
}

/// Validate broadcast parameters.
pub fn validate_broadcast(params: &Value) -> Result<(), ToolCallResult> {
    validate_channel_id(params)?;
    let sender = require_string(params, "sender")?;
    validate_sender(sender)?;
    let content = require_string(params, "content")?;
    validate_content(content)?;
    Ok(())
}

/// Validate query_history parameters.
pub fn validate_query_history(params: &Value) -> Result<(), ToolCallResult> {
    validate_channel_id(params)?;
    validate_limit(params, "limit")?;
    Ok(())
}

/// Validate search_messages parameters.
pub fn validate_search_messages(params: &Value) -> Result<(), ToolCallResult> {
    let query = require_string(params, "query")?;
    validate_query(query)?;
    validate_limit(params, "max_results")?;
    Ok(())
}

/// Validate get_message parameters.
pub fn validate_get_message(params: &Value) -> Result<(), ToolCallResult> {
    validate_message_id(params)?;
    Ok(())
}

/// Validate acknowledge_message parameters.
pub fn validate_acknowledge_message(params: &Value) -> Result<(), ToolCallResult> {
    validate_message_id(params)?;
    let recipient = require_string(params, "recipient")?;
    validate_participant(recipient)?;
    Ok(())
}

/// Validate set_channel_config parameters.
pub fn validate_set_channel_config(params: &Value) -> Result<(), ToolCallResult> {
    validate_channel_id(params)?;
    Ok(())
}

/// Validate communication_log parameters.
pub fn validate_communication_log(params: &Value) -> Result<(), ToolCallResult> {
    let intent = require_string(params, "intent")?;
    if intent.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: intent must not be empty".to_string(),
        ));
    }
    // Validate topic if present
    if let Some(topic) = params.get("topic").and_then(|v| v.as_str()) {
        validate_topic(topic)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_channel_name_valid() {
        assert!(validate_channel_name("my-channel").is_ok());
        assert!(validate_channel_name("test_123").is_ok());
        assert!(validate_channel_name("a").is_ok());
        assert!(validate_channel_name(&"x".repeat(128)).is_ok());
    }

    #[test]
    fn test_validate_channel_name_invalid() {
        assert!(validate_channel_name("").is_err());
        assert!(validate_channel_name(&"x".repeat(129)).is_err());
        assert!(validate_channel_name("bad name!").is_err());
        assert!(validate_channel_name("has.dot").is_err());
        assert!(validate_channel_name("has space").is_err());
    }

    #[test]
    fn test_validate_content_valid() {
        assert!(validate_content("hello").is_ok());
        assert!(validate_content(&"x".repeat(MAX_CONTENT_BYTES)).is_ok());
    }

    #[test]
    fn test_validate_content_invalid() {
        assert!(validate_content("").is_err());
        assert!(validate_content(&"x".repeat(MAX_CONTENT_BYTES + 1)).is_err());
    }

    #[test]
    fn test_validate_sender_valid() {
        assert!(validate_sender("alice").is_ok());
    }

    #[test]
    fn test_validate_sender_empty() {
        assert!(validate_sender("").is_err());
    }

    #[test]
    fn test_validate_topic_valid() {
        assert!(validate_topic("weather").is_ok());
        assert!(validate_topic("system.alerts.cpu").is_ok());
        assert!(validate_topic("my-topic_v2").is_ok());
    }

    #[test]
    fn test_validate_topic_invalid() {
        assert!(validate_topic("").is_err());
        assert!(validate_topic(&"t".repeat(129)).is_err());
        assert!(validate_topic("bad topic!").is_err());
    }

    #[test]
    fn test_validate_channel_id() {
        assert!(validate_channel_id(&json!({"channel_id": 1})).is_ok());
        assert!(validate_channel_id(&json!({"channel_id": 0})).is_err());
        assert!(validate_channel_id(&json!({})).is_err());
    }

    #[test]
    fn test_validate_message_id() {
        assert!(validate_message_id(&json!({"message_id": 1})).is_ok());
        assert!(validate_message_id(&json!({"message_id": 0})).is_err());
        assert!(validate_message_id(&json!({})).is_err());
    }

    #[test]
    fn test_validate_subscription_id() {
        assert!(validate_subscription_id(&json!({"subscription_id": 5})).is_ok());
        assert!(validate_subscription_id(&json!({"subscription_id": 0})).is_err());
        assert!(validate_subscription_id(&json!({})).is_err());
    }

    #[test]
    fn test_validate_limit() {
        assert!(validate_limit(&json!({"limit": 1}), "limit").is_ok());
        assert!(validate_limit(&json!({"limit": 10000}), "limit").is_ok());
        assert!(validate_limit(&json!({"limit": 0}), "limit").is_err());
        assert!(validate_limit(&json!({"limit": 10001}), "limit").is_err());
        assert!(validate_limit(&json!({}), "limit").is_ok()); // absent is OK
    }

    #[test]
    fn test_validate_query() {
        assert!(validate_query("search term").is_ok());
        assert!(validate_query("").is_err());
    }

    #[test]
    fn test_validate_send_message_composite() {
        let valid = json!({
            "channel_id": 1,
            "sender": "alice",
            "content": "hello"
        });
        assert!(validate_send_message(&valid).is_ok());

        let no_sender = json!({"channel_id": 1, "content": "hello"});
        assert!(validate_send_message(&no_sender).is_err());

        let empty_content = json!({"channel_id": 1, "sender": "alice", "content": ""});
        assert!(validate_send_message(&empty_content).is_err());
    }
}
