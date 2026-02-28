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
const VALID_URGENCY_LEVELS: &[&str] = &["background", "low", "normal", "high", "urgent", "critical"];

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
                "Validation error: hive_id is required and must be a positive integer"
                    .to_string(),
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
pub fn require_bool(
    params: &Value,
    field: &str,
) -> Result<bool, ToolCallResult> {
    params
        .get(field)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| {
            ToolCallResult::error(format!(
                "Validation error: {field} is required and must be a boolean"
            ))
        })
}

// ---------------------------------------------------------------------------
// New tool validators (Tools 18-26)
// ---------------------------------------------------------------------------

/// Validate manage_consent parameters.
pub fn validate_manage_consent(params: &Value) -> Result<(), ToolCallResult> {
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

/// Validate check_consent parameters.
pub fn validate_check_consent(params: &Value) -> Result<(), ToolCallResult> {
    let grantor = require_string(params, "grantor")?;
    validate_agent_id(grantor)?;
    let grantee = require_string(params, "grantee")?;
    validate_agent_id(grantee)?;
    let scope = require_string(params, "scope")?;
    validate_consent_scope(scope)?;
    Ok(())
}

/// Validate set_trust_level parameters.
pub fn validate_set_trust_level(params: &Value) -> Result<(), ToolCallResult> {
    let agent_id = require_string(params, "agent_id")?;
    validate_agent_id(agent_id)?;
    let level = require_string(params, "level")?;
    validate_trust_level(level)?;
    Ok(())
}

/// Validate get_trust_level parameters.
pub fn validate_get_trust_level(params: &Value) -> Result<(), ToolCallResult> {
    let agent_id = require_string(params, "agent_id")?;
    validate_agent_id(agent_id)?;
    Ok(())
}

/// Validate schedule_message parameters.
pub fn validate_schedule_message(params: &Value) -> Result<(), ToolCallResult> {
    validate_channel_id(params)?;
    let sender = require_string(params, "sender")?;
    validate_sender(sender)?;
    let content = require_string(params, "content")?;
    validate_content(content)?;
    Ok(())
}

/// Validate list_scheduled parameters (no validation needed).
pub fn validate_list_scheduled(_params: &Value) -> Result<(), ToolCallResult> {
    Ok(())
}

/// Validate form_hive parameters.
pub fn validate_form_hive(params: &Value) -> Result<(), ToolCallResult> {
    let name = require_string(params, "name")?;
    if name.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: hive name must not be empty".to_string(),
        ));
    }
    let initiator = require_string(params, "coordinator")?;
    if initiator.is_empty() {
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

/// Validate get_stats parameters (no validation needed).
pub fn validate_get_stats(_params: &Value) -> Result<(), ToolCallResult> {
    Ok(())
}

/// Validate send_affect parameters.
pub fn validate_send_affect(params: &Value) -> Result<(), ToolCallResult> {
    validate_channel_id(params)?;
    let sender = require_string(params, "sender")?;
    validate_sender(sender)?;
    let content = require_string(params, "content")?;
    validate_content(content)?;
    validate_valence(params)?;
    validate_arousal(params)?;
    validate_urgency(params)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// New tool validators (Tools 28-43)
// ---------------------------------------------------------------------------

/// Validate list_consent_gates parameters.
pub fn validate_list_consent_gates(params: &Value) -> Result<(), ToolCallResult> {
    // agent is optional — if present, validate it's non-empty
    if let Some(agent) = params.get("agent").and_then(|v| v.as_str()) {
        validate_agent_id(agent)?;
    }
    Ok(())
}

/// Validate cancel_scheduled parameters.
pub fn validate_cancel_scheduled(params: &Value) -> Result<(), ToolCallResult> {
    validate_temporal_id(params)?;
    Ok(())
}

/// Validate configure_federation parameters.
pub fn validate_configure_federation(params: &Value) -> Result<(), ToolCallResult> {
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

/// Validate add_federated_zone parameters.
pub fn validate_add_federated_zone(params: &Value) -> Result<(), ToolCallResult> {
    let zone_id = require_string(params, "zone_id")?;
    if zone_id.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: zone_id must not be empty".to_string(),
        ));
    }
    // Validate optional policy if present
    if let Some(policy) = params.get("policy").and_then(|v| v.as_str()) {
        validate_federation_policy(policy)?;
    }
    // Validate optional trust_level if present
    if let Some(level) = params.get("trust_level").and_then(|v| v.as_str()) {
        validate_trust_level(level)?;
    }
    Ok(())
}

/// Validate remove_federated_zone parameters.
pub fn validate_remove_federated_zone(params: &Value) -> Result<(), ToolCallResult> {
    let zone_id = require_string(params, "zone_id")?;
    if zone_id.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: zone_id must not be empty".to_string(),
        ));
    }
    Ok(())
}

/// Validate dissolve_hive parameters.
pub fn validate_dissolve_hive(params: &Value) -> Result<(), ToolCallResult> {
    validate_hive_id(params)?;
    Ok(())
}

/// Validate join_hive parameters.
pub fn validate_join_hive(params: &Value) -> Result<(), ToolCallResult> {
    validate_hive_id(params)?;
    let agent_id = require_string(params, "agent_id")?;
    validate_agent_id(agent_id)?;
    // Validate optional role if present
    if let Some(role) = params.get("role").and_then(|v| v.as_str()) {
        validate_hive_role(role)?;
    }
    Ok(())
}

/// Validate leave_hive parameters.
pub fn validate_leave_hive(params: &Value) -> Result<(), ToolCallResult> {
    validate_hive_id(params)?;
    let agent_id = require_string(params, "agent_id")?;
    validate_agent_id(agent_id)?;
    Ok(())
}

/// Validate get_hive parameters.
pub fn validate_get_hive(params: &Value) -> Result<(), ToolCallResult> {
    validate_hive_id(params)?;
    Ok(())
}

/// Validate log_communication parameters.
pub fn validate_log_communication(params: &Value) -> Result<(), ToolCallResult> {
    let content = require_string(params, "content")?;
    validate_content(content)?;
    let role = require_string(params, "role")?;
    if role.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: role must not be empty".to_string(),
        ));
    }
    // Validate optional topic if present
    if let Some(topic) = params.get("topic").and_then(|v| v.as_str()) {
        validate_topic(topic)?;
    }
    Ok(())
}

/// Validate get_comm_log parameters.
pub fn validate_get_comm_log(params: &Value) -> Result<(), ToolCallResult> {
    validate_limit(params, "limit")?;
    Ok(())
}

/// Validate get_audit_log parameters.
pub fn validate_get_audit_log(params: &Value) -> Result<(), ToolCallResult> {
    validate_limit(params, "limit")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Semantic tool validators
// ---------------------------------------------------------------------------

/// Validate send_semantic parameters.
pub fn validate_send_semantic(params: &Value) -> ValidationResult {
    validate_channel_id(params)?;
    let sender = require_string(params, "sender")?;
    validate_sender(sender)?;
    let topic = require_string(params, "topic")?;
    validate_topic(topic)?;
    // focus_nodes is optional array, depth is optional integer — no strict validation needed
    Ok(())
}

/// Validate extract_semantic parameters.
pub fn validate_extract_semantic(params: &Value) -> ValidationResult {
    validate_message_id(params)?;
    Ok(())
}

/// Validate graft_semantic parameters.
pub fn validate_graft_semantic(params: &Value) -> ValidationResult {
    let source_id = params
        .get("source_id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            ToolCallResult::error(
                "Validation error: source_id is required and must be a positive integer"
                    .to_string(),
            )
        })?;
    if source_id == 0 {
        return Err(ToolCallResult::error(
            "Validation error: source_id must be a positive integer (got 0)".to_string(),
        ));
    }
    let target_id = params
        .get("target_id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            ToolCallResult::error(
                "Validation error: target_id is required and must be a positive integer"
                    .to_string(),
            )
        })?;
    if target_id == 0 {
        return Err(ToolCallResult::error(
            "Validation error: target_id must be a positive integer (got 0)".to_string(),
        ));
    }
    // strategy is optional string, no strict validation needed
    Ok(())
}

/// Validate list_semantic_conflicts parameters.
pub fn validate_list_semantic_conflicts(params: &Value) -> ValidationResult {
    // channel_id and severity are both optional
    if params.get("channel_id").is_some() {
        validate_channel_id(params)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Affect tool validators
// ---------------------------------------------------------------------------

/// Validate get_affect_state parameters.
pub fn validate_get_affect_state(params: &Value) -> ValidationResult {
    let agent_id = require_string(params, "agent_id")?;
    validate_agent_id(agent_id)?;
    Ok(())
}

/// Validate set_affect_resistance parameters.
pub fn validate_set_affect_resistance(params: &Value) -> ValidationResult {
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

// ---------------------------------------------------------------------------
// Hive extension validators
// ---------------------------------------------------------------------------

/// Validate hive_think parameters.
pub fn validate_hive_think(params: &Value) -> ValidationResult {
    validate_hive_id(params)?;
    let question = require_string(params, "question")?;
    validate_content(question)?;
    // timeout_ms is optional
    Ok(())
}

/// Validate initiate_meld parameters.
pub fn validate_initiate_meld(params: &Value) -> ValidationResult {
    let partner_id = require_string(params, "partner_id")?;
    validate_agent_id(partner_id)?;
    // depth and duration_ms are optional
    Ok(())
}

// ---------------------------------------------------------------------------
// Consent flow validators
// ---------------------------------------------------------------------------

/// Validate list_pending_consent parameters.
pub fn validate_list_pending_consent(params: &Value) -> ValidationResult {
    // agent_id and consent_type are both optional
    if let Some(agent_id) = params.get("agent_id").and_then(|v| v.as_str()) {
        validate_agent_id(agent_id)?;
    }
    Ok(())
}

/// Validate respond_consent parameters.
pub fn validate_respond_consent(params: &Value) -> ValidationResult {
    let request_id = require_string(params, "request_id")?;
    if request_id.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: request_id must not be empty".to_string(),
        ));
    }
    let response = require_string(params, "response")?;
    if response.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: response must not be empty".to_string(),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Query tool validators
// ---------------------------------------------------------------------------

/// Validate query_relationships parameters.
pub fn validate_query_relationships(params: &Value) -> ValidationResult {
    let agent_id = require_string(params, "agent_id")?;
    validate_agent_id(agent_id)?;
    // relationship_type and depth are optional
    Ok(())
}

/// Validate query_echoes parameters.
pub fn validate_query_echoes(params: &Value) -> ValidationResult {
    validate_message_id(params)?;
    // depth is optional
    Ok(())
}

/// Validate query_conversations parameters.
pub fn validate_query_conversations(params: &Value) -> ValidationResult {
    // All parameters are optional
    if params.get("channel_id").is_some() {
        validate_channel_id(params)?;
    }
    validate_limit(params, "limit")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Federation extension validators
// ---------------------------------------------------------------------------

/// Validate get_federation_status parameters.
pub fn validate_get_federation_status(_params: &Value) -> ValidationResult {
    // No required params
    Ok(())
}

/// Validate set_federation_policy parameters.
pub fn validate_set_federation_policy(params: &Value) -> ValidationResult {
    let zone_id = require_string(params, "zone_id")?;
    if zone_id.is_empty() {
        return Err(ToolCallResult::error(
            "Validation error: zone_id must not be empty".to_string(),
        ));
    }
    // allow_semantic, allow_affect, allow_hive, max_message_size are all optional with defaults
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

    #[test]
    fn test_validate_manage_consent_valid() {
        let valid = json!({
            "action": "grant",
            "grantor": "agent-001",
            "grantee": "agent-002",
            "scope": "read_messages"
        });
        assert!(validate_manage_consent(&valid).is_ok());
    }

    #[test]
    fn test_validate_set_trust_level_valid() {
        let valid = json!({"agent_id": "agent-001", "level": "basic"});
        assert!(validate_set_trust_level(&valid).is_ok());
    }

    #[test]
    fn test_validate_schedule_message_valid() {
        let valid = json!({"channel_id": 1, "sender": "alice", "content": "hello"});
        assert!(validate_schedule_message(&valid).is_ok());
    }

    #[test]
    fn test_validate_form_hive_valid() {
        let valid = json!({
            "name": "hive-alpha",
            "coordinator": "agent-001",
            "members": ["agent-001", "agent-002"]
        });
        assert!(validate_form_hive(&valid).is_ok());
    }

    #[test]
    fn test_validate_send_affect_valid() {
        let valid = json!({
            "channel_id": 1, "sender": "alice",
            "content": "hello", "valence": 0.5,
            "arousal": 0.3, "urgency": "low"
        });
        assert!(validate_send_affect(&valid).is_ok());
    }

    #[test]
    fn test_validate_get_stats() {
        assert!(validate_get_stats(&json!({})).is_ok());
    }

    #[test]
    fn test_validate_list_scheduled() {
        assert!(validate_list_scheduled(&json!({})).is_ok());
    }


}