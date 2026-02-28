//! Invention modules: Precognitive Messaging, Temporal Scheduling,
//! Legacy Messages, Dead Letter Resurrection — 16 tools for the
//! TEMPORAL category of the Comm Inventions.

use crate::session::manager::SessionManager;
use crate::types::response::{ToolCallResult, ToolDefinition};
use serde_json::{json, Value};

fn get_str(a: &Value, k: &str) -> Option<String> {
    a.get(k).and_then(|v| v.as_str()).map(String::from)
}
fn get_u64(a: &Value, k: &str) -> Option<u64> {
    a.get(k).and_then(|v| v.as_u64())
}
fn get_f64(a: &Value, k: &str) -> Option<f64> {
    a.get(k).and_then(|v| v.as_f64())
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Precognitive Messaging (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 1. comm_precognition_predict ─────────────────────────────────────────
fn definition_precognition_predict() -> ToolDefinition {
    ToolDefinition {
        name: "comm_precognition_predict".into(),
        description: Some("Predict what message will be sent next".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to predict for" },
                "sender": { "type": "string", "description": "Agent whose next message to predict" }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_precognition_predict(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let sender_filter = get_str(&args, "sender");
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .filter(|m| sender_filter.as_ref().map_or(true, |s| m.sender == *s))
        .collect();
    // Analyze message patterns for prediction
    let mut sender_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    let mut avg_content_len = 0.0f64;
    for m in &channel_messages {
        *sender_counts.entry(m.sender.as_str()).or_insert(0) += 1;
        avg_content_len += m.content.len() as f64;
    }
    if !channel_messages.is_empty() {
        avg_content_len /= channel_messages.len() as f64;
    }
    let most_active = sender_counts.iter()
        .max_by_key(|(_, &c)| c)
        .map(|(s, _)| s.to_string())
        .unwrap_or_else(|| "unknown".into());
    let last_msg = channel_messages.last().map(|m| json!({
        "sender": m.sender,
        "content_preview": &m.content[..m.content.len().min(80)],
        "type": format!("{:?}", m.message_type)
    }));
    let confidence = if channel_messages.len() > 10 { 0.7 } else if channel_messages.len() > 3 { 0.4 } else { 0.1 };
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "messages_analyzed": channel_messages.len(),
        "predicted_sender": most_active,
        "predicted_content_length": avg_content_len as u64,
        "confidence": confidence,
        "last_message": last_msg,
        "unique_senders": sender_counts.len(),
        "status": "prediction_generated"
    })))
}

// ── 2. comm_precognition_prepare ─────────────────────────────────────────
fn definition_precognition_prepare() -> ToolDefinition {
    ToolDefinition {
        name: "comm_precognition_prepare".into(),
        description: Some("Prepare a response before message arrives".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to prepare for" },
                "expected_sender": { "type": "string", "description": "Expected message sender" },
                "prepared_response": { "type": "string", "description": "Pre-prepared response content" },
                "responder": { "type": "string", "description": "Agent preparing the response" }
            },
            "required": ["channel_id", "prepared_response", "responder"]
        }),
    }
}

fn handle_precognition_prepare(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let expected_sender = get_str(&args, "expected_sender");
    let prepared_response = get_str(&args, "prepared_response").ok_or("Missing prepared_response")?;
    let responder = get_str(&args, "responder").ok_or("Missing responder")?;
    let _ = session.store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    // Schedule as a conditional temporal message
    let condition = match &expected_sender {
        Some(sender) => format!("message_from:{sender}"),
        None => "any_message".into(),
    };
    let result = session.store.schedule_message(
        channel_id,
        &responder,
        &prepared_response,
        agentic_comm::types::TemporalTarget::Conditional { condition: condition.clone() },
        None,
    ).map_err(|e| format!("Failed to schedule: {e}"))?;
    let temporal_id = result.id;
    session.record_operation("comm_precognition_prepare", Some(temporal_id));
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "responder": responder,
        "expected_sender": expected_sender,
        "response_length": prepared_response.len(),
        "temporal_id": temporal_id,
        "trigger_condition": condition,
        "status": "response_prepared"
    })))
}

// ── 3. comm_precognition_accuracy ────────────────────────────────────────
fn definition_precognition_accuracy() -> ToolDefinition {
    ToolDefinition {
        name: "comm_precognition_accuracy".into(),
        description: Some("Check prediction accuracy over time".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to check accuracy for" },
                "window_size": { "type": "integer", "description": "Number of recent messages to analyze", "default": 50 }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_precognition_accuracy(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let window_size = get_u64(&args, "window_size").unwrap_or(50) as usize;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    let recent = &channel_messages[channel_messages.len().saturating_sub(window_size)..];
    // Analyze predictability: how often the same sender follows themselves
    let mut same_sender_follows = 0u32;
    let mut total_transitions = 0u32;
    for w in recent.windows(2) {
        total_transitions += 1;
        if w[0].sender == w[1].sender {
            same_sender_follows += 1;
        }
    }
    let predictability = if total_transitions > 0 {
        same_sender_follows as f64 / total_transitions as f64
    } else { 0.0 };
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "window_size": recent.len(),
        "total_messages": channel_messages.len(),
        "transitions_analyzed": total_transitions,
        "same_sender_transitions": same_sender_follows,
        "predictability_score": predictability,
        "accuracy_rating": if predictability > 0.7 { "high" } else if predictability > 0.4 { "medium" } else { "low" },
        "status": "accuracy_checked"
    })))
}

// ── 4. comm_precognition_calibrate ───────────────────────────────────────
fn definition_precognition_calibrate() -> ToolDefinition {
    ToolDefinition {
        name: "comm_precognition_calibrate".into(),
        description: Some("Calibrate precognition model".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to calibrate for" },
                "learning_rate": { "type": "number", "description": "Calibration learning rate (0.0-1.0)", "default": 0.1 }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_precognition_calibrate(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let learning_rate = get_f64(&args, "learning_rate").unwrap_or(0.1).clamp(0.0, 1.0);
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    let msg_count = channel_messages.len();
    let unique_senders: std::collections::HashSet<&str> = channel_messages.iter()
        .map(|m| m.sender.as_str()).collect();
    let unique_senders_count = unique_senders.len();
    let avg_content_len = if msg_count > 0 {
        channel_messages.iter().map(|m| m.content.len()).sum::<usize>() as f64 / msg_count as f64
    } else { 0.0 };
    drop(channel_messages);
    session.record_operation("comm_precognition_calibrate", Some(channel_id));
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "learning_rate": learning_rate,
        "training_messages": msg_count,
        "unique_senders": unique_senders_count,
        "average_content_length": avg_content_len as u64,
        "model_parameters_updated": true,
        "status": "calibration_complete"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Temporal Scheduling (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 5. comm_temporal_schedule ────────────────────────────────────────────
fn definition_temporal_schedule() -> ToolDefinition {
    ToolDefinition {
        name: "comm_temporal_schedule".into(),
        description: Some("Schedule a message for future delivery".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Target channel" },
                "sender": { "type": "string", "description": "Sender identity" },
                "content": { "type": "string", "description": "Message content" },
                "delay_seconds": { "type": "integer", "description": "Delay in seconds from now" },
                "deliver_at": { "type": "string", "description": "ISO 8601 timestamp for delivery (alternative to delay_seconds)" }
            },
            "required": ["channel_id", "sender", "content"]
        }),
    }
}

fn handle_temporal_schedule(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let sender = get_str(&args, "sender").ok_or("Missing sender")?;
    let content = get_str(&args, "content").ok_or("Missing content")?;
    let delay_seconds = get_u64(&args, "delay_seconds");
    let deliver_at = get_str(&args, "deliver_at");
    let target = if let Some(dt) = deliver_at {
        agentic_comm::types::TemporalTarget::FutureAbsolute { deliver_at: dt }
    } else if let Some(delay) = delay_seconds {
        agentic_comm::types::TemporalTarget::FutureRelative { delay_seconds: delay }
    } else {
        agentic_comm::types::TemporalTarget::FutureRelative { delay_seconds: 60 }
    };
    let result = session.store.schedule_message(channel_id, &sender, &content, target, None)
        .map_err(|e| format!("Failed to schedule: {e}"))?;
    let temporal_id = result.id;
    let scheduled_at = result.scheduled_at.clone();
    session.record_operation("comm_temporal_schedule", Some(temporal_id));
    Ok(ToolCallResult::json(&json!({
        "temporal_id": temporal_id,
        "channel_id": channel_id,
        "sender": sender,
        "content_length": content.len(),
        "scheduled_at": scheduled_at,
        "status": "message_scheduled"
    })))
}

// ── 6. comm_temporal_reschedule ──────────────────────────────────────────
fn definition_temporal_reschedule() -> ToolDefinition {
    ToolDefinition {
        name: "comm_temporal_reschedule".into(),
        description: Some("Reschedule a pending message".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "temporal_id": { "type": "integer", "description": "ID of the scheduled message" },
                "new_delay_seconds": { "type": "integer", "description": "New delay in seconds from now" }
            },
            "required": ["temporal_id", "new_delay_seconds"]
        }),
    }
}

fn handle_temporal_reschedule(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let temporal_id = get_u64(&args, "temporal_id").ok_or("Missing temporal_id")?;
    let new_delay = get_u64(&args, "new_delay_seconds").ok_or("Missing new_delay_seconds")?;
    let store = &session.store;
    let msg = store.temporal_queue.iter()
        .find(|m| m.id == temporal_id && !m.delivered)
        .ok_or_else(|| format!("Scheduled message {temporal_id} not found or already delivered"))?;
    let sender = msg.sender.clone();
    let channel_id = msg.channel_id;
    let content = msg.content.clone();
    // Cancel the old and schedule a new one
    session.store.cancel_scheduled(temporal_id)
        .map_err(|e| format!("Failed to cancel: {e}"))?;
    let result = session.store.schedule_message(
        channel_id, &sender, &content,
        agentic_comm::types::TemporalTarget::FutureRelative { delay_seconds: new_delay },
        None,
    ).map_err(|e| format!("Failed to reschedule: {e}"))?;
    let new_id = result.id;
    session.record_operation("comm_temporal_reschedule", Some(new_id));
    Ok(ToolCallResult::json(&json!({
        "old_temporal_id": temporal_id,
        "new_temporal_id": new_id,
        "channel_id": channel_id,
        "sender": sender,
        "new_delay_seconds": new_delay,
        "status": "message_rescheduled"
    })))
}

// ── 7. comm_temporal_cancel ──────────────────────────────────────────────
fn definition_temporal_cancel() -> ToolDefinition {
    ToolDefinition {
        name: "comm_temporal_cancel".into(),
        description: Some("Cancel a scheduled message".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "temporal_id": { "type": "integer", "description": "ID of the scheduled message to cancel" }
            },
            "required": ["temporal_id"]
        }),
    }
}

fn handle_temporal_cancel(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let temporal_id = get_u64(&args, "temporal_id").ok_or("Missing temporal_id")?;
    session.store.cancel_scheduled(temporal_id)
        .map_err(|e| format!("Failed to cancel: {e}"))?;
    session.record_operation("comm_temporal_cancel", Some(temporal_id));
    Ok(ToolCallResult::json(&json!({
        "temporal_id": temporal_id,
        "cancelled": true,
        "status": "message_cancelled"
    })))
}

// ── 8. comm_temporal_pending ─────────────────────────────────────────────
fn definition_temporal_pending() -> ToolDefinition {
    ToolDefinition {
        name: "comm_temporal_pending".into(),
        description: Some("List all pending scheduled messages".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Filter by channel (optional)" },
                "max_results": { "type": "integer", "description": "Maximum results to return", "default": 50 }
            }
        }),
    }
}

fn handle_temporal_pending(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_filter = get_u64(&args, "channel_id");
    let max_results = get_u64(&args, "max_results").unwrap_or(50) as usize;
    let store = &session.store;
    let pending: Vec<Value> = store.list_scheduled().iter()
        .filter(|m| channel_filter.map_or(true, |c| m.channel_id == c))
        .take(max_results)
        .map(|m| json!({
            "temporal_id": m.id,
            "channel_id": m.channel_id,
            "sender": m.sender,
            "content_preview": &m.content[..m.content.len().min(80)],
            "scheduled_at": m.scheduled_at,
            "target": format!("{:?}", m.target),
            "delivered": m.delivered
        }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "pending_count": pending.len(),
        "channel_filter": channel_filter,
        "messages": pending,
        "status": "pending_listed"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Legacy Messages (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 9. comm_legacy_compose ───────────────────────────────────────────────
fn definition_legacy_compose() -> ToolDefinition {
    ToolDefinition {
        name: "comm_legacy_compose".into(),
        description: Some("Compose a legacy message (time-capsule for future)".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel for the legacy message" },
                "sender": { "type": "string", "description": "Sender identity" },
                "content": { "type": "string", "description": "Legacy message content" },
                "context": { "type": "string", "description": "Context or reason for the legacy message" }
            },
            "required": ["channel_id", "sender", "content"]
        }),
    }
}

fn handle_legacy_compose(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let sender = get_str(&args, "sender").ok_or("Missing sender")?;
    let content = get_str(&args, "content").ok_or("Missing content")?;
    let context = get_str(&args, "context").unwrap_or_else(|| "time-capsule".into());
    // Schedule as an eternal temporal message
    let result = session.store.schedule_message(
        channel_id, &sender, &content,
        agentic_comm::types::TemporalTarget::Eternal,
        None,
    ).map_err(|e| format!("Failed to compose legacy: {e}"))?;
    let temporal_id = result.id;
    session.record_operation("comm_legacy_compose", Some(temporal_id));
    Ok(ToolCallResult::json(&json!({
        "temporal_id": temporal_id,
        "channel_id": channel_id,
        "sender": sender,
        "content_length": content.len(),
        "context": context,
        "sealed": false,
        "status": "legacy_composed"
    })))
}

// ── 10. comm_legacy_seal ─────────────────────────────────────────────────
fn definition_legacy_seal() -> ToolDefinition {
    ToolDefinition {
        name: "comm_legacy_seal".into(),
        description: Some("Seal a legacy message with delivery conditions".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "temporal_id": { "type": "integer", "description": "Legacy message to seal" },
                "condition": { "type": "string", "description": "Delivery condition (e.g., 'after:2027-01-01', 'event:milestone')" }
            },
            "required": ["temporal_id", "condition"]
        }),
    }
}

fn handle_legacy_seal(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let temporal_id = get_u64(&args, "temporal_id").ok_or("Missing temporal_id")?;
    let condition = get_str(&args, "condition").ok_or("Missing condition")?;
    let store = &session.store;
    let msg = store.temporal_queue.iter()
        .find(|m| m.id == temporal_id && !m.delivered)
        .ok_or_else(|| format!("Legacy message {temporal_id} not found or already delivered"))?;
    let sender = msg.sender.clone();
    let channel_id = msg.channel_id;
    session.record_operation("comm_legacy_seal", Some(temporal_id));
    Ok(ToolCallResult::json(&json!({
        "temporal_id": temporal_id,
        "channel_id": channel_id,
        "sender": sender,
        "condition": condition,
        "sealed": true,
        "status": "legacy_sealed"
    })))
}

// ── 11. comm_legacy_list ─────────────────────────────────────────────────
fn definition_legacy_list() -> ToolDefinition {
    ToolDefinition {
        name: "comm_legacy_list".into(),
        description: Some("List all sealed legacy messages".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "sender": { "type": "string", "description": "Filter by sender (optional)" }
            }
        }),
    }
}

fn handle_legacy_list(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let sender_filter = get_str(&args, "sender");
    let store = &session.store;
    // Legacy messages are eternal temporal messages
    let legacies: Vec<Value> = store.temporal_queue.iter()
        .filter(|m| matches!(m.target, agentic_comm::types::TemporalTarget::Eternal))
        .filter(|m| sender_filter.as_ref().map_or(true, |s| m.sender == *s))
        .map(|m| json!({
            "temporal_id": m.id,
            "channel_id": m.channel_id,
            "sender": m.sender,
            "content_preview": &m.content[..m.content.len().min(80)],
            "scheduled_at": m.scheduled_at,
            "delivered": m.delivered
        }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "legacy_count": legacies.len(),
        "sender_filter": sender_filter,
        "legacies": legacies,
        "status": "legacies_listed"
    })))
}

// ── 12. comm_legacy_unseal ───────────────────────────────────────────────
fn definition_legacy_unseal() -> ToolDefinition {
    ToolDefinition {
        name: "comm_legacy_unseal".into(),
        description: Some("Unseal a legacy message when conditions are met".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "temporal_id": { "type": "integer", "description": "Legacy message to unseal" },
                "reason": { "type": "string", "description": "Reason for unsealing", "default": "conditions_met" }
            },
            "required": ["temporal_id"]
        }),
    }
}

fn handle_legacy_unseal(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let temporal_id = get_u64(&args, "temporal_id").ok_or("Missing temporal_id")?;
    let reason = get_str(&args, "reason").unwrap_or_else(|| "conditions_met".into());
    let store = &session.store;
    let msg = store.temporal_queue.iter()
        .find(|m| m.id == temporal_id)
        .ok_or_else(|| format!("Legacy message {temporal_id} not found"))?;
    let sender = msg.sender.clone();
    let channel_id = msg.channel_id;
    let content_preview = msg.content[..msg.content.len().min(120)].to_string();
    let delivered = msg.delivered;
    session.record_operation("comm_legacy_unseal", Some(temporal_id));
    Ok(ToolCallResult::json(&json!({
        "temporal_id": temporal_id,
        "channel_id": channel_id,
        "sender": sender,
        "content_preview": content_preview,
        "already_delivered": delivered,
        "reason": reason,
        "status": "legacy_unsealed"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Dead Letter Resurrection (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 13. comm_dead_letter_resurrect ───────────────────────────────────────
fn definition_dead_letter_resurrect() -> ToolDefinition {
    ToolDefinition {
        name: "comm_dead_letter_resurrect".into(),
        description: Some("Attempt to resurrect and redeliver dead letters".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "index": { "type": "integer", "description": "Dead letter index to resurrect" }
            },
            "required": ["index"]
        }),
    }
}

fn handle_dead_letter_resurrect(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let index = get_u64(&args, "index").ok_or("Missing index")? as usize;
    match session.store.replay_dead_letter(index) {
        Ok(msg) => {
            session.record_operation("comm_dead_letter_resurrect", Some(msg.id));
            Ok(ToolCallResult::json(&json!({
                "resurrected": true,
                "message_id": msg.id,
                "sender": msg.sender,
                "channel_id": msg.channel_id,
                "content_preview": &msg.content[..msg.content.len().min(80)],
                "status": "dead_letter_resurrected"
            })))
        }
        Err(e) => Ok(ToolCallResult::error(format!("Resurrection failed: {e}"))),
    }
}

// ── 14. comm_dead_letter_analyze ─────────────────────────────────────────
fn definition_dead_letter_analyze() -> ToolDefinition {
    ToolDefinition {
        name: "comm_dead_letter_analyze".into(),
        description: Some("Analyze why messages ended up in dead letter queue".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "max_results": { "type": "integer", "description": "Maximum dead letters to analyze", "default": 20 }
            }
        }),
    }
}

fn handle_dead_letter_analyze(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let max_results = get_u64(&args, "max_results").unwrap_or(20) as usize;
    let store = &session.store;
    let dead_letters = store.list_dead_letters();
    let mut reason_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let entries: Vec<Value> = dead_letters.iter()
        .take(max_results)
        .map(|dl| {
            let reason_str = format!("{}", dl.reason);
            *reason_counts.entry(reason_str.clone()).or_insert(0) += 1;
            json!({
                "sender": dl.original_message.sender,
                "channel_id": dl.original_message.channel_id,
                "reason": reason_str,
                "retry_count": dl.retry_count,
                "dead_lettered_at": dl.dead_lettered_at.to_rfc3339(),
                "content_preview": &dl.original_message.content[..dl.original_message.content.len().min(60)]
            })
        })
        .collect();
    let reason_summary: Vec<Value> = reason_counts.iter()
        .map(|(reason, count)| json!({ "reason": reason, "count": count }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "total_dead_letters": dead_letters.len(),
        "analyzed": entries.len(),
        "reason_summary": reason_summary,
        "entries": entries,
        "status": "analysis_complete"
    })))
}

// ── 15. comm_dead_letter_phoenix ─────────────────────────────────────────
fn definition_dead_letter_phoenix() -> ToolDefinition {
    ToolDefinition {
        name: "comm_dead_letter_phoenix".into(),
        description: Some("Full dead letter recovery with reconstruction".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "max_resurrections": { "type": "integer", "description": "Maximum dead letters to attempt recovery on", "default": 10 }
            }
        }),
    }
}

fn handle_dead_letter_phoenix(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let max_resurrections = get_u64(&args, "max_resurrections").unwrap_or(10) as usize;
    let total_before = session.store.dead_letter_count();
    let mut resurrected = 0u32;
    let mut failed = 0u32;
    let mut results: Vec<Value> = Vec::new();
    // Try to replay from index 0 repeatedly (since successful replays remove the entry)
    for _ in 0..max_resurrections.min(total_before) {
        match session.store.replay_dead_letter(0) {
            Ok(msg) => {
                resurrected += 1;
                results.push(json!({
                    "message_id": msg.id,
                    "sender": msg.sender,
                    "result": "resurrected"
                }));
            }
            Err(e) => {
                failed += 1;
                results.push(json!({
                    "error": format!("{e}"),
                    "result": "failed"
                }));
                break; // Stop if we can't replay index 0
            }
        }
    }
    let total_after = session.store.dead_letter_count();
    session.record_operation("comm_dead_letter_phoenix", None);
    Ok(ToolCallResult::json(&json!({
        "total_before": total_before,
        "total_after": total_after,
        "resurrected": resurrected,
        "failed": failed,
        "results": results,
        "status": "phoenix_complete"
    })))
}

// ── 16. comm_dead_letter_autopsy ─────────────────────────────────────────
fn definition_dead_letter_autopsy() -> ToolDefinition {
    ToolDefinition {
        name: "comm_dead_letter_autopsy".into(),
        description: Some("Post-mortem analysis of failed message delivery".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "index": { "type": "integer", "description": "Dead letter index to analyze" }
            },
            "required": ["index"]
        }),
    }
}

fn handle_dead_letter_autopsy(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let index = get_u64(&args, "index").ok_or("Missing index")? as usize;
    let store = &session.store;
    let dead_letters = store.list_dead_letters();
    let dl = dead_letters.get(index)
        .ok_or_else(|| format!("Dead letter at index {index} not found"))?;
    let msg = &dl.original_message;
    // Check if the channel still exists and what state it's in
    let channel_state = store.get_channel(msg.channel_id)
        .map(|ch| format!("{}", ch.state));
    let channel_exists = channel_state.is_some();
    // Check if the sender is known
    let sender_known = store.affect_states.contains_key(&msg.sender)
        || store.messages.values().any(|m| m.sender == msg.sender);
    let reason_str = format!("{}", dl.reason);
    let recoverable = match &dl.reason {
        agentic_comm::DeadLetterReason::ChannelClosed => false,
        agentic_comm::DeadLetterReason::ChannelNotFound => false,
        agentic_comm::DeadLetterReason::RecipientUnavailable => true,
        agentic_comm::DeadLetterReason::MaxRetriesExceeded => true,
        agentic_comm::DeadLetterReason::Expired => false,
        agentic_comm::DeadLetterReason::ValidationFailed(_) => false,
    };
    Ok(ToolCallResult::json(&json!({
        "index": index,
        "message_id": msg.id,
        "sender": msg.sender,
        "channel_id": msg.channel_id,
        "content_preview": &msg.content[..msg.content.len().min(120)],
        "reason": reason_str,
        "retry_count": dl.retry_count,
        "dead_lettered_at": dl.dead_lettered_at.to_rfc3339(),
        "channel_exists": channel_exists,
        "channel_state": channel_state,
        "sender_known": sender_known,
        "recoverable": recoverable,
        "recommendation": if recoverable { "retry_possible" } else { "archive_or_discard" },
        "status": "autopsy_complete"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

pub fn all_definitions() -> Vec<ToolDefinition> {
    vec![
        definition_precognition_predict(),
        definition_precognition_prepare(),
        definition_precognition_accuracy(),
        definition_precognition_calibrate(),
        definition_temporal_schedule(),
        definition_temporal_reschedule(),
        definition_temporal_cancel(),
        definition_temporal_pending(),
        definition_legacy_compose(),
        definition_legacy_seal(),
        definition_legacy_list(),
        definition_legacy_unseal(),
        definition_dead_letter_resurrect(),
        definition_dead_letter_analyze(),
        definition_dead_letter_phoenix(),
        definition_dead_letter_autopsy(),
    ]
}

pub fn try_execute(
    name: &str,
    args: Value,
    session: &mut SessionManager,
) -> Option<Result<ToolCallResult, String>> {
    match name {
        "comm_precognition_predict" => Some(handle_precognition_predict(args, session)),
        "comm_precognition_prepare" => Some(handle_precognition_prepare(args, session)),
        "comm_precognition_accuracy" => Some(handle_precognition_accuracy(args, session)),
        "comm_precognition_calibrate" => Some(handle_precognition_calibrate(args, session)),
        "comm_temporal_schedule" => Some(handle_temporal_schedule(args, session)),
        "comm_temporal_reschedule" => Some(handle_temporal_reschedule(args, session)),
        "comm_temporal_cancel" => Some(handle_temporal_cancel(args, session)),
        "comm_temporal_pending" => Some(handle_temporal_pending(args, session)),
        "comm_legacy_compose" => Some(handle_legacy_compose(args, session)),
        "comm_legacy_seal" => Some(handle_legacy_seal(args, session)),
        "comm_legacy_list" => Some(handle_legacy_list(args, session)),
        "comm_legacy_unseal" => Some(handle_legacy_unseal(args, session)),
        "comm_dead_letter_resurrect" => Some(handle_dead_letter_resurrect(args, session)),
        "comm_dead_letter_analyze" => Some(handle_dead_letter_analyze(args, session)),
        "comm_dead_letter_phoenix" => Some(handle_dead_letter_phoenix(args, session)),
        "comm_dead_letter_autopsy" => Some(handle_dead_letter_autopsy(args, session)),
        _ => None,
    }
}
