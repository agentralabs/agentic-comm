//! Input validation for consolidated MCP tools.
//!
//! The public API is a single `validate()` function that routes to domain
//! validators based on the tool name and `operation` parameter. All existing
//! field-level helpers are preserved and reused by the domain validators.
//!
//! Validation failures produce `ToolCallResult::error()` with `isError: true`
//! and a specific error message describing what was wrong.

use serde_json::Value;

use crate::types::response::ToolCallResult;

/// Maximum allowed content size in bytes (1 MB).
const MAX_CONTENT_BYTES: usize = 1_048_576;

/// Maximum allowed name/topic length.
const MAX_NAME_LEN: usize = 128;

/// Maximum allowed limit value.
const MAX_LIMIT: u64 = 10_000;

/// Result type for validation -- Ok means valid, Err holds the error result.
pub type ValidationResult = Result<(), ToolCallResult>;

// ===========================================================================
// Public entry point
// ===========================================================================

/// Validate parameters for a consolidated tool call.
///
/// Routes to the appropriate domain validator based on `tool_name` and the
/// `operation` field inside `params`.
pub fn validate(tool_name: &str, params: &Value) -> ValidationResult {
    let operation = params.get("operation").and_then(|v| v.as_str());

    match tool_name {
        "comm_channel" => validate_channel(operation, params),
        "comm_message" => validate_message(operation, params),
        "comm_semantic" => validate_semantic(operation, params),
        "comm_affect" => validate_affect(operation, params),
        "comm_hive" => validate_hive(operation, params),
        "comm_consent" => validate_consent(operation, params),
        "comm_trust" => validate_trust(operation, params),
        "comm_keys" => validate_keys(operation, params),
        "comm_federation" => validate_federation(operation, params),
        "comm_temporal" => validate_temporal(operation, params),
        "comm_query" => validate_query_tool(operation, params),
        "comm_forensics" => validate_forensics(operation, params),
        "comm_collaboration" => validate_collaboration(operation, params),
        "comm_workspace" => validate_workspace(operation, params),
        "comm_session" | "session_start" | "session_end" => {
            validate_session(tool_name, operation, params)
        }
        "comm_agent" => validate_agent(operation, params),
        "comm_store" => validate_store(operation, params),
        _ => Err(ToolCallResult::error(format!("Unknown tool: {tool_name}"))),
    }
}

// ===========================================================================
// Shared helpers
// ===========================================================================

/// Require the `operation` parameter to be present.
fn require_operation(op: Option<&str>) -> Result<&str, ToolCallResult> {
    op.ok_or_else(|| ToolCallResult::error("Validation error: operation is required".to_string()))
}

/// Require a `channel_id` (positive integer) in params.
fn require_channel_id(params: &Value) -> ValidationResult {
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
    Ok(())
}

/// Require a string field to be present and non-empty.
fn require_str(params: &Value, field: &str) -> ValidationResult {
    let val = params.get(field).and_then(|v| v.as_str()).ok_or_else(|| {
        ToolCallResult::error(format!(
            "Validation error: {field} is required and must be a string"
        ))
    })?;
    if val.is_empty() {
        return Err(ToolCallResult::error(format!(
            "Validation error: {field} must not be empty"
        )));
    }
    Ok(())
}

/// Require a positive integer field.
fn require_positive_id(params: &Value, field: &str) -> ValidationResult {
    let id = params.get(field).and_then(|v| v.as_u64()).ok_or_else(|| {
        ToolCallResult::error(format!(
            "Validation error: {field} is required and must be a positive integer"
        ))
    })?;
    if id == 0 {
        return Err(ToolCallResult::error(format!(
            "Validation error: {field} must be a positive integer (got 0)"
        )));
    }
    Ok(())
}

// ===========================================================================
// Individual field validators (preserved from original)
// ===========================================================================

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
pub fn require_string<'a>(params: &'a Value, field: &str) -> Result<&'a str, ToolCallResult> {
    params.get(field).and_then(|v| v.as_str()).ok_or_else(|| {
        ToolCallResult::error(format!(
            "Validation error: {field} is required and must be a string"
        ))
    })
}

/// Validate an agent_id: must be non-empty.
pub fn validate_agent_id(agent_id: &str) -> ValidationResult {
    if agent_id.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: agent_id must not be empty".to_string(),
        ));
    }
    Ok(())
}

/// Valid consent scopes (must match ConsentScope enum in types.rs).
const VALID_CONSENT_SCOPES: &[&str] = &[
    "read_messages",
    "send_messages",
    "join_channels",
    "view_presence",
    "share_content",
    "schedule_messages",
    "federate",
    "hive_participation",
];

/// Validate a consent scope string.
pub fn validate_consent_scope(scope: &str) -> ValidationResult {
    if !VALID_CONSENT_SCOPES.contains(&scope) {
        return Err(ToolCallResult::error(format!(
            "Validation error: invalid consent scope '{}'. Must be one of: {}",
            scope,
            VALID_CONSENT_SCOPES.join(", ")
        )));
    }
    Ok(())
}

/// Valid trust levels.
const VALID_TRUST_LEVELS: &[&str] = &[
    "none", "minimal", "basic", "standard", "high", "full", "absolute",
];

/// Validate a trust level string.
pub fn validate_trust_level(level: &str) -> ValidationResult {
    if !VALID_TRUST_LEVELS.contains(&level) {
        return Err(ToolCallResult::error(format!(
            "Validation error: invalid trust level '{}'. Must be one of: {}",
            level,
            VALID_TRUST_LEVELS.join(", ")
        )));
    }
    Ok(())
}

/// Validate a valence value: must be -1.0 to 1.0.
pub fn validate_valence(params: &Value) -> Result<Option<f64>, ToolCallResult> {
    match params.get("valence").and_then(|v| v.as_f64()) {
        Some(v) if !(-1.0..=1.0).contains(&v) => Err(ToolCallResult::error(format!(
            "Validation error: valence must be between -1.0 and 1.0 (got {v})"
        ))),
        Some(v) => Ok(Some(v)),
        None => Ok(None),
    }
}

/// Validate an arousal value: must be 0.0 to 1.0.
pub fn validate_arousal(params: &Value) -> Result<Option<f64>, ToolCallResult> {
    match params.get("arousal").and_then(|v| v.as_f64()) {
        Some(v) if !(0.0..=1.0).contains(&v) => Err(ToolCallResult::error(format!(
            "Validation error: arousal must be between 0.0 and 1.0 (got {v})"
        ))),
        Some(v) => Ok(Some(v)),
        None => Ok(None),
    }
}

/// Valid urgency levels (must match UrgencyLevel enum in types.rs).
const VALID_URGENCY_LEVELS: &[&str] =
    &["background", "low", "normal", "high", "urgent", "critical"];

/// Validate an urgency string if present.
pub fn validate_urgency(params: &Value) -> ValidationResult {
    if let Some(urgency) = params.get("urgency").and_then(|v| v.as_str()) {
        if !VALID_URGENCY_LEVELS.contains(&urgency) {
            return Err(ToolCallResult::error(format!(
                "Validation error: invalid urgency '{}'. Must be one of: {}",
                urgency,
                VALID_URGENCY_LEVELS.join(", ")
            )));
        }
    }
    Ok(())
}

/// Valid federation policies.
const VALID_FEDERATION_POLICIES: &[&str] = &["allow", "deny", "selective"];

/// Validate a federation policy string.
pub fn validate_federation_policy(policy: &str) -> ValidationResult {
    if !VALID_FEDERATION_POLICIES.contains(&policy) {
        return Err(ToolCallResult::error(format!(
            "Validation error: invalid federation policy '{}'. Must be one of: {}",
            policy,
            VALID_FEDERATION_POLICIES.join(", ")
        )));
    }
    Ok(())
}

/// Valid hive roles.
const VALID_HIVE_ROLES: &[&str] = &["coordinator", "member", "observer"];

/// Validate a hive role string.
pub fn validate_hive_role(role: &str) -> ValidationResult {
    if !VALID_HIVE_ROLES.contains(&role) {
        return Err(ToolCallResult::error(format!(
            "Validation error: invalid hive role '{}'. Must be one of: {}",
            role,
            VALID_HIVE_ROLES.join(", ")
        )));
    }
    Ok(())
}

/// Validate a temporal_id: must be a positive integer.
pub fn validate_temporal_id(params: &Value) -> Result<u64, ToolCallResult> {
    let id = params
        .get("temporal_id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            ToolCallResult::error(
                "Validation error: temporal_id is required and must be a positive integer"
                    .to_string(),
            )
        })?;
    if id == 0 {
        return Err(ToolCallResult::error(
            "Validation error: temporal_id must be a positive integer (got 0)".to_string(),
        ));
    }
    Ok(id)
}

/// Validate a hive_id: must be a positive integer.
pub fn validate_hive_id(params: &Value) -> Result<u64, ToolCallResult> {
    let id = params
        .get("hive_id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            ToolCallResult::error(
                "Validation error: hive_id is required and must be a positive integer".to_string(),
            )
        })?;
    if id == 0 {
        return Err(ToolCallResult::error(
            "Validation error: hive_id must be a positive integer (got 0)".to_string(),
        ));
    }
    Ok(id)
}

/// Validate a required boolean field from params, returning the value.
pub fn require_bool(params: &Value, field: &str) -> Result<bool, ToolCallResult> {
    params.get(field).and_then(|v| v.as_bool()).ok_or_else(|| {
        ToolCallResult::error(format!(
            "Validation error: {field} is required and must be a boolean"
        ))
    })
}

/// Valid priority levels.
const VALID_PRIORITIES: &[&str] = &["low", "normal", "high", "urgent", "critical"];

// ===========================================================================
// Domain validators (1 per consolidated tool)
// ===========================================================================

// ---------------------------------------------------------------------------
// 1. comm_channel
// ---------------------------------------------------------------------------

fn validate_channel(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "create" => {
            require_str(params, "name")?;
            if let Some(name) = params.get("name").and_then(|v| v.as_str()) {
                validate_channel_name(name)?;
            }
            Ok(())
        }
        "join" | "leave" => {
            require_channel_id(params)?;
            require_str(params, "participant")?;
            Ok(())
        }
        "info" | "pause" | "resume" | "drain" | "close" | "expire" | "compact" => {
            require_channel_id(params)?;
            Ok(())
        }
        "config" => {
            require_channel_id(params)?;
            Ok(())
        }
        "list" | "stats" | "list_channels" => Ok(()),
        "subscribe" => {
            let topic = require_string(params, "topic")?;
            validate_topic(topic)?;
            let subscriber = require_string(params, "subscriber")?;
            validate_sender(subscriber)?;
            Ok(())
        }
        "unsubscribe" => {
            validate_subscription_id(params)?;
            Ok(())
        }
        "publish" => {
            let topic = require_string(params, "topic")?;
            validate_topic(topic)?;
            let sender = require_string(params, "sender")?;
            validate_sender(sender)?;
            let content = require_string(params, "content")?;
            validate_content(content)?;
            Ok(())
        }
        "broadcast" => {
            require_channel_id(params)?;
            let sender = require_string(params, "sender")?;
            validate_sender(sender)?;
            let content = require_string(params, "content")?;
            validate_content(content)?;
            Ok(())
        }
        _ => Err(ToolCallResult::error(format!(
            "Unknown channel operation: {op}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// 2. comm_message
// ---------------------------------------------------------------------------

fn validate_message(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "send" => {
            require_channel_id(params)?;
            let sender = require_string(params, "sender")?;
            validate_sender(sender)?;
            let content = require_string(params, "content")?;
            validate_content(content)?;
            Ok(())
        }
        "receive" => {
            require_channel_id(params)?;
            Ok(())
        }
        "search" => {
            let query = require_string(params, "query")?;
            validate_query(query)?;
            validate_limit(params, "max_results")?;
            Ok(())
        }
        "get" | "ack" | "delete" => {
            require_positive_id(params, "message_id")?;
            Ok(())
        }
        "forward" => {
            require_positive_id(params, "message_id")?;
            require_channel_id(params)?;
            if let Some(forwarder) = params.get("forwarder").and_then(|v| v.as_str()) {
                validate_sender(forwarder)?;
            }
            Ok(())
        }
        "reply" => {
            require_channel_id(params)?;
            let sender = require_string(params, "sender")?;
            validate_sender(sender)?;
            let content = require_string(params, "content")?;
            validate_content(content)?;
            // reply_to_id or parent_id
            if params.get("reply_to_id").and_then(|v| v.as_u64()).is_none()
                && params.get("parent_id").and_then(|v| v.as_u64()).is_none()
            {
                return Err(ToolCallResult::error(
                    "Validation error: reply_to_id or parent_id is required".to_string(),
                ));
            }
            Ok(())
        }
        "thread" | "replies" => {
            if params.get("thread_id").and_then(|v| v.as_str()).is_some()
                || params.get("thread_id").and_then(|v| v.as_u64()).is_some()
                || params.get("message_id").and_then(|v| v.as_u64()).is_some()
            {
                Ok(())
            } else {
                Err(ToolCallResult::error(
                    "Validation error: thread_id or message_id is required".to_string(),
                ))
            }
        }
        "rich_send" => {
            require_channel_id(params)?;
            let sender = require_string(params, "sender")?;
            validate_sender(sender)?;
            let content_type = require_string(params, "content_type")?;
            let valid_types = [
                "text",
                "semantic",
                "affect",
                "full",
                "temporal",
                "precognitive",
                "meta",
                "unspeakable",
            ];
            if !valid_types.contains(&content_type) {
                return Err(ToolCallResult::error(format!(
                    "Validation error: content_type must be one of: {}",
                    valid_types.join(", ")
                )));
            }
            if params.get("content_data").is_none() {
                return Err(ToolCallResult::error(
                    "Validation error: content_data is required".to_string(),
                ));
            }
            Ok(())
        }
        "rich_get" => {
            require_positive_id(params, "message_id")?;
            Ok(())
        }
        "get_by_comm_id" => {
            let comm_id = require_string(params, "comm_id")?;
            // Basic UUID format check
            if comm_id.len() != 36 || comm_id.chars().filter(|c| *c == '-').count() != 4 {
                return Err(ToolCallResult::error(
                    "Validation error: comm_id must be a valid UUID string (e.g., 550e8400-e29b-41d4-a716-446655440000)".to_string(),
                ));
            }
            Ok(())
        }
        "priority_send" => {
            require_channel_id(params)?;
            let sender = require_string(params, "sender")?;
            validate_sender(sender)?;
            let content = require_string(params, "content")?;
            validate_content(content)?;
            // Validate optional priority
            if let Some(priority) = params.get("priority").and_then(|v| v.as_str()) {
                if !VALID_PRIORITIES.contains(&priority) {
                    return Err(ToolCallResult::error(format!(
                        "Validation error: invalid priority '{}'. Must be one of: {}",
                        priority,
                        VALID_PRIORITIES.join(", ")
                    )));
                }
            }
            Ok(())
        }
        "history" => {
            require_channel_id(params)?;
            validate_limit(params, "limit")?;
            Ok(())
        }
        "summarize" => {
            require_channel_id(params)?;
            Ok(())
        }
        "dead_letters" | "list_dead_letters" => Ok(()),
        "replay_dead_letter" => {
            params
                .get("index")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| {
                    ToolCallResult::error(
                        "Validation error: index is required and must be a non-negative integer"
                            .to_string(),
                    )
                })?;
            Ok(())
        }
        "echo_chain" | "echo_depth" => {
            require_positive_id(params, "message_id")?;
            Ok(())
        }
        _ => Err(ToolCallResult::error(format!(
            "Unknown message operation: {op}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// 3. comm_semantic
// ---------------------------------------------------------------------------

fn validate_semantic(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "send" => {
            require_channel_id(params)?;
            let sender = require_string(params, "sender")?;
            validate_sender(sender)?;
            let topic = require_string(params, "topic")?;
            validate_topic(topic)?;
            Ok(())
        }
        "extract" => {
            require_positive_id(params, "message_id")?;
            Ok(())
        }
        "graft" => {
            require_positive_id(params, "source_id")?;
            require_positive_id(params, "target_id")?;
            Ok(())
        }
        "conflicts" => {
            // channel_id is optional
            if params.get("channel_id").is_some() {
                require_channel_id(params)?;
            }
            Ok(())
        }
        // Invention-level semantic ops -- handlers validate internally
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// 4. comm_affect
// ---------------------------------------------------------------------------

fn validate_affect(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "send" => {
            require_channel_id(params)?;
            let sender = require_string(params, "sender")?;
            validate_sender(sender)?;
            let content = require_string(params, "content")?;
            validate_content(content)?;
            validate_valence(params)?;
            validate_arousal(params)?;
            validate_urgency(params)?;
            Ok(())
        }
        "get_state" => {
            let agent_id = require_string(params, "agent_id")?;
            validate_agent_id(agent_id)?;
            Ok(())
        }
        "set_resistance" => {
            let resistance = params
                .get("resistance")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| {
                    ToolCallResult::error(
                        "Validation error: resistance is required and must be a number".to_string(),
                    )
                })?;
            if !(0.0..=1.0).contains(&resistance) {
                return Err(ToolCallResult::error(format!(
                    "Validation error: resistance must be between 0.0 and 1.0 (got {resistance})"
                )));
            }
            Ok(())
        }
        "contagion" => {
            require_channel_id(params)?;
            Ok(())
        }
        "decay" => {
            let rate = params.get("decay_rate").ok_or_else(|| {
                ToolCallResult::error("Validation error: decay_rate is required".to_string())
            })?;
            let rate_val = rate.as_f64().ok_or_else(|| {
                ToolCallResult::error("Validation error: decay_rate must be a number".to_string())
            })?;
            if !(0.0..=1.0).contains(&rate_val) {
                return Err(ToolCallResult::error(
                    "Validation error: decay_rate must be between 0.0 and 1.0".to_string(),
                ));
            }
            Ok(())
        }
        "history" => {
            let agent = require_string(params, "agent")?;
            validate_agent_id(agent)?;
            Ok(())
        }
        "blend" => {
            let agent = require_string(params, "agent_id")?;
            validate_agent_id(agent)?;
            Ok(())
        }
        // Invention-level affect ops -- handlers validate internally
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// 5. comm_hive
// ---------------------------------------------------------------------------

fn validate_hive(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "form" | "create" => {
            let name = require_string(params, "name")?;
            if name.is_empty() {
                return Err(ToolCallResult::error(
                    "Validation error: hive name must not be empty".to_string(),
                ));
            }
            let coordinator = require_string(params, "coordinator")?;
            if coordinator.is_empty() {
                return Err(ToolCallResult::error(
                    "Validation error: coordinator must not be empty".to_string(),
                ));
            }
            let members = params
                .get("members")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    ToolCallResult::error(
                        "Validation error: members is required and must be an array".to_string(),
                    )
                })?;
            if members.is_empty() {
                return Err(ToolCallResult::error(
                    "Validation error: members array must not be empty".to_string(),
                ));
            }
            for (i, member) in members.iter().enumerate() {
                match member.as_str() {
                    Some(s) if s.is_empty() => {
                        return Err(ToolCallResult::error(format!(
                            "Validation error: member at index {i} must not be empty"
                        )));
                    }
                    Some(_) => {}
                    None => {
                        return Err(ToolCallResult::error(format!(
                            "Validation error: member at index {i} must be a string"
                        )));
                    }
                }
            }
            Ok(())
        }
        "join" => {
            validate_hive_id(params)?;
            let agent_id = require_string(params, "agent_id")?;
            validate_agent_id(agent_id)?;
            if let Some(role) = params.get("role").and_then(|v| v.as_str()) {
                validate_hive_role(role)?;
            }
            Ok(())
        }
        "leave" => {
            validate_hive_id(params)?;
            let agent_id = require_string(params, "agent_id")?;
            validate_agent_id(agent_id)?;
            Ok(())
        }
        "dissolve" => {
            validate_hive_id(params)?;
            Ok(())
        }
        "get" | "info" | "status" => {
            validate_hive_id(params)?;
            Ok(())
        }
        "think" => {
            validate_hive_id(params)?;
            let question = require_string(params, "question")?;
            validate_content(question)?;
            Ok(())
        }
        "meld" => {
            let partner_id = require_string(params, "partner_id")?;
            validate_agent_id(partner_id)?;
            Ok(())
        }
        "list" => Ok(()),
        // Invention-level hive ops -- handlers validate internally
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// 6. comm_consent
// ---------------------------------------------------------------------------

fn validate_consent(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "grant" | "revoke" => {
            let grantor = require_string(params, "grantor")?;
            validate_agent_id(grantor)?;
            let grantee = require_string(params, "grantee")?;
            validate_agent_id(grantee)?;
            let scope = require_string(params, "scope")?;
            validate_consent_scope(scope)?;
            Ok(())
        }
        "check" => {
            let grantor = require_string(params, "grantor")?;
            validate_agent_id(grantor)?;
            let grantee = require_string(params, "grantee")?;
            validate_agent_id(grantee)?;
            let scope = require_string(params, "scope")?;
            validate_consent_scope(scope)?;
            Ok(())
        }
        "list_gates" => {
            // agent is optional
            if let Some(agent) = params.get("agent").and_then(|v| v.as_str()) {
                validate_agent_id(agent)?;
            }
            Ok(())
        }
        "list_pending" => {
            if let Some(agent_id) = params.get("agent_id").and_then(|v| v.as_str()) {
                validate_agent_id(agent_id)?;
            }
            Ok(())
        }
        "respond" => {
            require_str(params, "request_id")?;
            require_str(params, "response")?;
            Ok(())
        }
        "manage" => {
            let action = require_string(params, "action")?;
            if action != "grant" && action != "revoke" {
                return Err(ToolCallResult::error(
                    "Validation error: action must be 'grant' or 'revoke'".to_string(),
                ));
            }
            let grantor = require_string(params, "grantor")?;
            validate_agent_id(grantor)?;
            let grantee = require_string(params, "grantee")?;
            validate_agent_id(grantee)?;
            let scope = require_string(params, "scope")?;
            validate_consent_scope(scope)?;
            Ok(())
        }
        _ => Err(ToolCallResult::error(format!(
            "Unknown consent operation: {op}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// 7. comm_trust
// ---------------------------------------------------------------------------

fn validate_trust(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "set" => {
            let agent_id = require_string(params, "agent_id")?;
            validate_agent_id(agent_id)?;
            let level = require_string(params, "level")?;
            validate_trust_level(level)?;
            Ok(())
        }
        "get" => {
            let agent_id = require_string(params, "agent_id")?;
            validate_agent_id(agent_id)?;
            Ok(())
        }
        "list" => Ok(()),
        _ => Err(ToolCallResult::error(format!(
            "Unknown trust operation: {op}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// 8. comm_keys
// ---------------------------------------------------------------------------

fn validate_keys(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "generate" => {
            require_str(params, "algorithm")?;
            Ok(())
        }
        "rotate" | "revoke" | "export" | "get" => {
            params
                .get("key_id")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| {
                    ToolCallResult::error(
                        "Validation error: key_id is required and must be a non-negative integer"
                            .to_string(),
                    )
                })?;
            Ok(())
        }
        "list" => Ok(()),
        "encrypt" => {
            let content = require_string(params, "content")?;
            validate_content(content)?;
            Ok(())
        }
        "decrypt" => {
            let ciphertext = require_string(params, "ciphertext")?;
            if ciphertext.is_empty() {
                return Err(ToolCallResult::error(
                    "Validation error: ciphertext must not be empty".to_string(),
                ));
            }
            let nonce = require_string(params, "nonce")?;
            if nonce.is_empty() {
                return Err(ToolCallResult::error(
                    "Validation error: nonce must not be empty".to_string(),
                ));
            }
            // epoch is optional
            if let Some(epoch_val) = params.get("epoch") {
                if epoch_val.as_u64().is_none() {
                    return Err(ToolCallResult::error(
                        "Validation error: epoch must be a non-negative integer".to_string(),
                    ));
                }
            }
            Ok(())
        }
        "verify_signature" => {
            require_str(params, "public_key")?;
            require_str(params, "content")?;
            require_str(params, "signature")?;
            Ok(())
        }
        _ => Err(ToolCallResult::error(format!(
            "Unknown keys operation: {op}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// 9. comm_federation
// ---------------------------------------------------------------------------

fn validate_federation(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "configure" => {
            require_bool(params, "enabled")?;
            let local_zone = require_string(params, "local_zone")?;
            if local_zone.is_empty() {
                return Err(ToolCallResult::error(
                    "Validation error: local_zone must not be empty".to_string(),
                ));
            }
            let policy = require_string(params, "policy")?;
            validate_federation_policy(policy)?;
            Ok(())
        }
        "add_zone" => {
            let zone_id = require_string(params, "zone_id")?;
            if zone_id.is_empty() {
                return Err(ToolCallResult::error(
                    "Validation error: zone_id must not be empty".to_string(),
                ));
            }
            if let Some(policy) = params.get("policy").and_then(|v| v.as_str()) {
                validate_federation_policy(policy)?;
            }
            if let Some(level) = params.get("trust_level").and_then(|v| v.as_str()) {
                validate_trust_level(level)?;
            }
            Ok(())
        }
        "remove_zone" => {
            require_str(params, "zone_id")?;
            Ok(())
        }
        "status" => Ok(()),
        "set_policy" => {
            require_str(params, "zone_id")?;
            Ok(())
        }
        "set_zone_policy" => {
            require_str(params, "zone")?;
            Ok(())
        }
        // Invention-level federation ops -- handlers validate internally
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// 10. comm_temporal
// ---------------------------------------------------------------------------

fn validate_temporal(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "schedule" => {
            require_channel_id(params)?;
            let sender = require_string(params, "sender")?;
            validate_sender(sender)?;
            let content = require_string(params, "content")?;
            validate_content(content)?;
            Ok(())
        }
        "cancel" => {
            validate_temporal_id(params)?;
            Ok(())
        }
        "list" | "pending" | "list_scheduled" => Ok(()),
        // Invention-level temporal ops -- handlers validate internally
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// 11. comm_query
// ---------------------------------------------------------------------------

fn validate_query_tool(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "relationships" => {
            let agent_id = require_string(params, "agent_id")?;
            validate_agent_id(agent_id)?;
            Ok(())
        }
        "echoes" => {
            require_positive_id(params, "message_id")?;
            Ok(())
        }
        "conversations" => {
            if params.get("channel_id").is_some() {
                require_channel_id(params)?;
            }
            validate_limit(params, "limit")?;
            Ok(())
        }
        "claims" => {
            let claim = require_string(params, "claim")?;
            validate_query(claim)?;
            Ok(())
        }
        "evidence" => {
            let q = require_string(params, "query")?;
            validate_query(q)?;
            Ok(())
        }
        "suggest" => {
            let q = require_string(params, "query")?;
            validate_query(q)?;
            Ok(())
        }
        "audit_log" | "comm_log" => {
            validate_limit(params, "limit")?;
            Ok(())
        }
        "stats" => Ok(()),
        _ => Err(ToolCallResult::error(format!(
            "Unknown query operation: {op}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// 12. comm_forensics (invention module -- minimal validation)
// ---------------------------------------------------------------------------

fn validate_forensics(op: Option<&str>, _params: &Value) -> ValidationResult {
    let _op = require_operation(op)?;
    // Invention handlers validate internally
    Ok(())
}

// ---------------------------------------------------------------------------
// 13. comm_collaboration (invention module -- minimal validation)
// ---------------------------------------------------------------------------

fn validate_collaboration(op: Option<&str>, _params: &Value) -> ValidationResult {
    let _op = require_operation(op)?;
    // Invention handlers validate internally
    Ok(())
}

// ---------------------------------------------------------------------------
// 14. comm_workspace
// ---------------------------------------------------------------------------

fn validate_workspace(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "create" => {
            let name = require_string(params, "name")?;
            if name.is_empty() {
                return Err(ToolCallResult::error(
                    "Validation error: workspace name must not be empty".to_string(),
                ));
            }
            if name.len() > MAX_NAME_LEN {
                return Err(ToolCallResult::error(format!(
                    "Validation error: workspace name exceeds maximum length of {MAX_NAME_LEN} characters (got {})",
                    name.len()
                )));
            }
            Ok(())
        }
        "add" => {
            require_str(params, "workspace_id")?;
            require_str(params, "path")?;
            Ok(())
        }
        "list" => {
            require_str(params, "workspace_id")?;
            Ok(())
        }
        "query" => {
            require_str(params, "workspace_id")?;
            let q = require_string(params, "query")?;
            validate_query(q)?;
            Ok(())
        }
        "compare" => {
            require_str(params, "workspace_id")?;
            let item = require_string(params, "item")?;
            validate_query(item)?;
            Ok(())
        }
        "xref" => {
            require_str(params, "workspace_id")?;
            let item = require_string(params, "item")?;
            validate_query(item)?;
            Ok(())
        }
        _ => Err(ToolCallResult::error(format!(
            "Unknown workspace operation: {op}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// 15. comm_session / session_start / session_end
// ---------------------------------------------------------------------------

fn validate_session(tool_name: &str, _op: Option<&str>, params: &Value) -> ValidationResult {
    match tool_name {
        "session_start" => Ok(()),
        "session_end" => Ok(()),
        "comm_session" => {
            // Operation-based session tool
            let op = params
                .get("operation")
                .and_then(|v| v.as_str())
                .unwrap_or("resume");
            match op {
                "start" => Ok(()),
                "end" => Ok(()),
                "resume" => {
                    if let Some(limit) = params.get("limit") {
                        if let Some(n) = limit.as_u64() {
                            if n == 0 {
                                return Err(ToolCallResult::error(
                                    "Validation error: limit must be greater than 0".to_string(),
                                ));
                            }
                            if n > MAX_LIMIT {
                                return Err(ToolCallResult::error(format!(
                                    "Validation error: limit exceeds maximum of {MAX_LIMIT} (got {n})"
                                )));
                            }
                        } else if let Some(n) = limit.as_i64() {
                            if n <= 0 {
                                return Err(ToolCallResult::error(
                                    "Validation error: limit must be a positive integer"
                                        .to_string(),
                                ));
                            }
                        }
                    }
                    Ok(())
                }
                _ => Err(ToolCallResult::error(format!(
                    "Unknown session operation: {op}"
                ))),
            }
        }
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// 16. comm_agent
// ---------------------------------------------------------------------------

fn validate_agent(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "register" => {
            require_str(params, "agent_id")?;
            Ok(())
        }
        "info" | "status" | "presence" => {
            require_str(params, "agent_id")?;
            Ok(())
        }
        "set_presence" => {
            require_str(params, "agent_id")?;
            require_str(params, "status")?;
            Ok(())
        }
        "list" => Ok(()),
        "log" => {
            let content = require_string(params, "content")?;
            validate_content(content)?;
            let role = require_string(params, "role")?;
            if role.is_empty() {
                return Err(ToolCallResult::error(
                    "Validation error: role must not be empty".to_string(),
                ));
            }
            if let Some(topic) = params.get("topic").and_then(|v| v.as_str()) {
                validate_topic(topic)?;
            }
            Ok(())
        }
        "communication_log" => {
            let intent = require_string(params, "intent")?;
            if intent.is_empty() {
                return Err(ToolCallResult::error(
                    "Validation error: intent must not be empty".to_string(),
                ));
            }
            if let Some(topic) = params.get("topic").and_then(|v| v.as_str()) {
                validate_topic(topic)?;
            }
            Ok(())
        }
        _ => Err(ToolCallResult::error(format!(
            "Unknown agent operation: {op}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// 17. comm_store
// ---------------------------------------------------------------------------

fn validate_store(op: Option<&str>, params: &Value) -> ValidationResult {
    let op = require_operation(op)?;
    match op {
        "save" => {
            require_str(params, "path")?;
            Ok(())
        }
        "load" => {
            require_str(params, "path")?;
            Ok(())
        }
        "export" => {
            require_str(params, "path")?;
            Ok(())
        }
        "import" => {
            require_str(params, "path")?;
            Ok(())
        }
        "stats" | "status" => Ok(()),
        _ => Err(ToolCallResult::error(format!(
            "Unknown store operation: {op}"
        ))),
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── Field-level helper tests ─────────────────────────────────────────

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

    // ── Consolidated entry-point tests ───────────────────────────────────

    #[test]
    fn test_validate_unknown_tool() {
        assert!(validate("unknown_tool", &json!({"operation": "x"})).is_err());
    }

    #[test]
    fn test_validate_channel_create() {
        let valid = json!({"operation": "create", "name": "my-channel"});
        assert!(validate("comm_channel", &valid).is_ok());

        let no_name = json!({"operation": "create"});
        assert!(validate("comm_channel", &no_name).is_err());
    }

    #[test]
    fn test_validate_channel_join() {
        let valid = json!({"operation": "join", "channel_id": 1, "participant": "alice"});
        assert!(validate("comm_channel", &valid).is_ok());

        let no_channel = json!({"operation": "join", "participant": "alice"});
        assert!(validate("comm_channel", &no_channel).is_err());
    }

    #[test]
    fn test_validate_message_send() {
        let valid = json!({
            "operation": "send",
            "channel_id": 1,
            "sender": "alice",
            "content": "hello"
        });
        assert!(validate("comm_message", &valid).is_ok());

        let no_sender = json!({"operation": "send", "channel_id": 1, "content": "hello"});
        assert!(validate("comm_message", &no_sender).is_err());

        let empty_content = json!({
            "operation": "send",
            "channel_id": 1,
            "sender": "alice",
            "content": ""
        });
        assert!(validate("comm_message", &empty_content).is_err());
    }

    #[test]
    fn test_validate_message_search() {
        let valid = json!({"operation": "search", "query": "hello"});
        assert!(validate("comm_message", &valid).is_ok());

        let empty_query = json!({"operation": "search", "query": ""});
        assert!(validate("comm_message", &empty_query).is_err());
    }

    #[test]
    fn test_validate_consent_grant() {
        let valid = json!({
            "operation": "grant",
            "grantor": "agent-001",
            "grantee": "agent-002",
            "scope": "read_messages"
        });
        assert!(validate("comm_consent", &valid).is_ok());
    }

    #[test]
    fn test_validate_trust_set() {
        let valid = json!({
            "operation": "set",
            "agent_id": "agent-001",
            "level": "basic"
        });
        assert!(validate("comm_trust", &valid).is_ok());

        let bad_level = json!({
            "operation": "set",
            "agent_id": "agent-001",
            "level": "superdupermax"
        });
        assert!(validate("comm_trust", &bad_level).is_err());
    }

    #[test]
    fn test_validate_hive_form() {
        let valid = json!({
            "operation": "form",
            "name": "hive-alpha",
            "coordinator": "agent-001",
            "members": ["agent-001", "agent-002"]
        });
        assert!(validate("comm_hive", &valid).is_ok());
    }

    #[test]
    fn test_validate_affect_send() {
        let valid = json!({
            "operation": "send",
            "channel_id": 1,
            "sender": "alice",
            "content": "hello",
            "valence": 0.5,
            "arousal": 0.3,
            "urgency": "low"
        });
        assert!(validate("comm_affect", &valid).is_ok());
    }

    #[test]
    fn test_validate_workspace_create() {
        let valid = json!({"operation": "create", "name": "test-ws"});
        assert!(validate("comm_workspace", &valid).is_ok());

        let empty_name = json!({"operation": "create", "name": ""});
        assert!(validate("comm_workspace", &empty_name).is_err());
    }

    #[test]
    fn test_validate_keys_generate() {
        let valid = json!({"operation": "generate", "algorithm": "ed25519"});
        assert!(validate("comm_keys", &valid).is_ok());

        let no_algo = json!({"operation": "generate"});
        assert!(validate("comm_keys", &no_algo).is_err());
    }

    #[test]
    fn test_validate_session_resume() {
        let valid = json!({"operation": "resume", "limit": 10});
        assert!(validate("comm_session", &valid).is_ok());

        let zero_limit = json!({"operation": "resume", "limit": 0});
        assert!(validate("comm_session", &zero_limit).is_err());
    }

    #[test]
    fn test_validate_forensics_requires_operation() {
        let no_op = json!({});
        assert!(validate("comm_forensics", &no_op).is_err());

        let with_op = json!({"operation": "investigate"});
        assert!(validate("comm_forensics", &with_op).is_ok());
    }

    #[test]
    fn test_validate_collaboration_requires_operation() {
        let no_op = json!({});
        assert!(validate("comm_collaboration", &no_op).is_err());

        let with_op = json!({"operation": "start"});
        assert!(validate("comm_collaboration", &with_op).is_ok());
    }

    #[test]
    fn test_validate_missing_operation() {
        let no_op = json!({"channel_id": 1});
        assert!(validate("comm_channel", &no_op).is_err());
        assert!(validate("comm_message", &no_op).is_err());
        assert!(validate("comm_trust", &no_op).is_err());
    }

    #[test]
    fn test_validate_federation_configure() {
        let valid = json!({
            "operation": "configure",
            "enabled": true,
            "local_zone": "zone-a",
            "policy": "allow"
        });
        assert!(validate("comm_federation", &valid).is_ok());
    }

    #[test]
    fn test_validate_temporal_schedule() {
        let valid = json!({
            "operation": "schedule",
            "channel_id": 1,
            "sender": "alice",
            "content": "later"
        });
        assert!(validate("comm_temporal", &valid).is_ok());
    }

    #[test]
    fn test_validate_query_relationships() {
        let valid = json!({"operation": "relationships", "agent_id": "agent-001"});
        assert!(validate("comm_query", &valid).is_ok());
    }

    #[test]
    fn test_validate_agent_register() {
        let valid = json!({"operation": "register", "agent_id": "agent-001"});
        assert!(validate("comm_agent", &valid).is_ok());
    }

    #[test]
    fn test_validate_store_save() {
        let valid = json!({"operation": "save", "path": "/tmp/test.acomm"});
        assert!(validate("comm_store", &valid).is_ok());

        let no_path = json!({"operation": "save"});
        assert!(validate("comm_store", &no_path).is_err());
    }
}
