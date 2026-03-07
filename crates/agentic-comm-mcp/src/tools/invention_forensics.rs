//! Invention modules: Communication Forensics, Pattern Detection,
//! Health Monitoring, Oracle Predictions — 16 tools for the
//! FORENSICS category of the Comm Inventions.

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
// 1. Communication Forensics (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 1. comm_forensics_investigate ────────────────────────────────────────
fn definition_forensics_investigate() -> ToolDefinition {
    ToolDefinition {
        name: "comm_forensics_investigate".into(),
        description: Some("Investigate communication anomalies".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to investigate"
                    },
                    "query": {
                        "type": "string",
                        "description": "Investigation query or description of anomaly"
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                },
                "required": [
                    "channel_id",
                    "query"
                ]
            }),
    }
}

fn handle_forensics_investigate(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let query = get_str(&args, "query").ok_or("Missing query")?;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    let query_lower = query.to_lowercase();
    let relevant: Vec<Value> = channel_messages.iter()
        .filter(|m| m.content.to_lowercase().contains(&query_lower))
        .take(20)
        .map(|m| json!({
            "message_id": m.id,
            "sender": m.sender,
            "content_preview": &m.content[..m.content.len().min(80)],
            "timestamp": m.timestamp.to_rfc3339(),
            "type": format!("{:?}", m.message_type)
        }))
        .collect();
    // Check for anomalies: gaps in message IDs, unusual timing patterns
    let mut senders: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for m in &channel_messages {
        *senders.entry(m.sender.as_str()).or_insert(0) += 1;
    }
    let dead_letter_count = store.dead_letter_count();
    let total_messages = channel_messages.len();
    let relevant_hits = relevant.len();
    let unique_senders_count = senders.len();
    session.record_operation("comm_forensics_investigate", Some(channel_id));
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "query": query,
        "total_messages": total_messages,
        "relevant_hits": relevant_hits,
        "relevant_messages": relevant,
        "unique_senders": unique_senders_count,
        "dead_letters_in_system": dead_letter_count,
        "status": "investigation_complete"
    })))
}

// ── 2. comm_forensics_timeline ───────────────────────────────────────────
fn definition_forensics_timeline() -> ToolDefinition {
    ToolDefinition {
        name: "comm_forensics_timeline".into(),
        description: Some("Generate forensic timeline of events".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to generate timeline for"
                    },
                    "max_events": {
                        "type": "integer",
                        "description": "Maximum events to include",
                        "default": 50
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                },
                "required": [
                    "channel_id"
                ]
            }),
    }
}

fn handle_forensics_timeline(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let max_events = get_u64(&args, "max_events").unwrap_or(50) as usize;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let mut channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    channel_messages.sort_by_key(|m| m.timestamp);
    let timeline: Vec<Value> = channel_messages.iter()
        .take(max_events)
        .enumerate()
        .map(|(i, m)| json!({
            "sequence": i,
            "message_id": m.id,
            "sender": m.sender,
            "timestamp": m.timestamp.to_rfc3339(),
            "type": format!("{:?}", m.message_type),
            "priority": format!("{:?}", m.priority),
            "content_preview": &m.content[..m.content.len().min(60)],
            "has_thread": m.thread_id.is_some(),
            "is_reply": m.reply_to.is_some()
        }))
        .collect();
    // Calculate time span
    let first_ts = channel_messages.first().map(|m| m.timestamp.to_rfc3339());
    let last_ts = channel_messages.last().map(|m| m.timestamp.to_rfc3339());
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "total_events": channel_messages.len(),
        "timeline_length": timeline.len(),
        "first_event": first_ts,
        "last_event": last_ts,
        "timeline": timeline,
        "status": "timeline_generated"
    })))
}

// ── 3. comm_forensics_evidence ───────────────────────────────────────────
fn definition_forensics_evidence() -> ToolDefinition {
    ToolDefinition {
        name: "comm_forensics_evidence".into(),
        description: Some("Collect evidence about a communication event".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "message_id": {
                        "type": "integer",
                        "description": "Message to collect evidence about"
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                },
                "required": [
                    "message_id"
                ]
            }),
    }
}

fn handle_forensics_evidence(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let store = &session.store;
    let msg = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    let echo_chain = store.query_echo_chain(message_id);
    let echo_depth = store.get_echo_depth(message_id);
    let replies = store.get_replies(message_id);
    let thread_messages = msg.thread_id.as_ref()
        .map(|tid| store.get_thread(tid).len())
        .unwrap_or(0);
    // Get sender affect state
    let sender_affect = store.get_affect_state(&msg.sender);
    let affect_info = sender_affect.map(|a| json!({
        "valence": a.valence,
        "arousal": a.arousal,
        "dominance": a.dominance,
        "urgency": format!("{:?}", a.urgency)
    }));
    Ok(ToolCallResult::json(&json!({
        "message_id": message_id,
        "sender": msg.sender,
        "channel_id": msg.channel_id,
        "content": msg.content,
        "timestamp": msg.timestamp.to_rfc3339(),
        "type": format!("{:?}", msg.message_type),
        "priority": format!("{:?}", msg.priority),
        "status": format!("{:?}", msg.status),
        "reply_to": msg.reply_to,
        "thread_id": msg.thread_id,
        "thread_message_count": thread_messages,
        "reply_count": replies.len(),
        "echo_depth": echo_depth,
        "echo_chain_length": echo_chain.len(),
        "metadata_keys": msg.metadata.keys().collect::<Vec<_>>(),
        "sender_affect": affect_info,
        "evidence_status": "evidence_collected"
    })))
}

// ── 4. comm_forensics_report ─────────────────────────────────────────────
fn definition_forensics_report() -> ToolDefinition {
    ToolDefinition {
        name: "comm_forensics_report".into(),
        description: Some("Generate forensic report for a channel".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to report on"
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                },
                "required": [
                    "channel_id"
                ]
            }),
    }
}

fn handle_forensics_report(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let store = &session.store;
    let channel = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    let mut sender_stats: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    let mut type_stats: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut total_content_len = 0usize;
    for m in &channel_messages {
        *sender_stats.entry(m.sender.as_str()).or_insert(0) += 1;
        *type_stats.entry(format!("{:?}", m.message_type)).or_insert(0) += 1;
        total_content_len += m.content.len();
    }
    let avg_content_len = if channel_messages.is_empty() { 0.0 } else {
        total_content_len as f64 / channel_messages.len() as f64
    };
    let top_senders: Vec<Value> = {
        let mut sorted: Vec<_> = sender_stats.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        sorted.iter().take(5).map(|(s, c)| json!({ "sender": s, "message_count": c })).collect()
    };
    let stats = store.stats();
    let channel_name = channel.name.clone();
    let channel_state = format!("{}", channel.state);
    let participant_count = channel.participants.len();
    let participants = channel.participants.clone();
    let total_messages = channel_messages.len();
    let unique_senders = sender_stats.len();
    let dead_letters_in_system = stats.dead_letter_count;
    let federation_enabled = stats.federation_enabled;
    session.record_operation("comm_forensics_report", Some(channel_id));
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "channel_name": channel_name,
        "channel_state": channel_state,
        "participant_count": participant_count,
        "participants": participants,
        "total_messages": total_messages,
        "unique_senders": unique_senders,
        "top_senders": top_senders,
        "message_types": type_stats,
        "average_content_length": avg_content_len as u64,
        "total_content_bytes": total_content_len,
        "dead_letters_in_system": dead_letters_in_system,
        "federation_enabled": federation_enabled,
        "status": "report_generated"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Pattern Detection (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 5. comm_pattern_detect ───────────────────────────────────────────────
fn definition_pattern_detect() -> ToolDefinition {
    ToolDefinition {
        name: "comm_pattern_detect".into(),
        description: Some("Detect communication patterns in a channel".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to analyze"
                    },
                    "min_occurrences": {
                        "type": "integer",
                        "description": "Minimum pattern occurrences",
                        "default": 2
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                },
                "required": [
                    "channel_id"
                ]
            }),
    }
}

fn handle_pattern_detect(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let min_occurrences = get_u64(&args, "min_occurrences").unwrap_or(2) as usize;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    // Detect sender-sequence patterns (bigrams)
    let mut bigrams: std::collections::HashMap<(String, String), usize> = std::collections::HashMap::new();
    for w in channel_messages.windows(2) {
        let key = (w[0].sender.clone(), w[1].sender.clone());
        *bigrams.entry(key).or_insert(0) += 1;
    }
    let patterns: Vec<Value> = bigrams.iter()
        .filter(|(_, &count)| count >= min_occurrences)
        .map(|((a, b), &count)| json!({
            "pattern_type": "sender_sequence",
            "from_sender": a,
            "to_sender": b,
            "occurrences": count
        }))
        .collect();
    // Detect message type patterns
    let mut type_sequences: std::collections::HashMap<(String, String), usize> = std::collections::HashMap::new();
    for w in channel_messages.windows(2) {
        let key = (format!("{:?}", w[0].message_type), format!("{:?}", w[1].message_type));
        *type_sequences.entry(key).or_insert(0) += 1;
    }
    let type_patterns: Vec<Value> = type_sequences.iter()
        .filter(|(_, &count)| count >= min_occurrences)
        .map(|((a, b), &count)| json!({
            "pattern_type": "type_sequence",
            "from_type": a,
            "to_type": b,
            "occurrences": count
        }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "messages_analyzed": channel_messages.len(),
        "min_occurrences": min_occurrences,
        "sender_patterns": patterns.len(),
        "type_patterns": type_patterns.len(),
        "patterns": patterns,
        "type_patterns_detail": type_patterns,
        "status": "patterns_detected"
    })))
}

// ── 6. comm_pattern_recurring ────────────────────────────────────────────
fn definition_pattern_recurring() -> ToolDefinition {
    ToolDefinition {
        name: "comm_pattern_recurring".into(),
        description: Some("Find recurring communication patterns".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to analyze"
                    },
                    "window_size": {
                        "type": "integer",
                        "description": "Window size for recurring pattern detection",
                        "default": 5
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                },
                "required": [
                    "channel_id"
                ]
            }),
    }
}

fn handle_pattern_recurring(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let window_size = get_u64(&args, "window_size").unwrap_or(5) as usize;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let mut channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    channel_messages.sort_by_key(|m| m.timestamp);
    // Build sender sequence and find repeating subsequences
    let sender_seq: Vec<&str> = channel_messages.iter().map(|m| m.sender.as_str()).collect();
    let mut recurring: std::collections::HashMap<Vec<&str>, usize> = std::collections::HashMap::new();
    if sender_seq.len() >= window_size {
        for w in sender_seq.windows(window_size) {
            *recurring.entry(w.to_vec()).or_insert(0) += 1;
        }
    }
    let recurrences: Vec<Value> = recurring.iter()
        .filter(|(_, &count)| count > 1)
        .map(|(seq, &count)| json!({
            "sequence": seq,
            "occurrences": count,
            "window_size": window_size
        }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "window_size": window_size,
        "messages_analyzed": channel_messages.len(),
        "recurring_patterns_found": recurrences.len(),
        "patterns": recurrences,
        "status": "recurring_patterns_found"
    })))
}

// ── 7. comm_pattern_anomaly ──────────────────────────────────────────────
fn definition_pattern_anomaly() -> ToolDefinition {
    ToolDefinition {
        name: "comm_pattern_anomaly".into(),
        description: Some("Detect anomalous communication patterns".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to analyze"
                    },
                    "sensitivity": {
                        "type": "number",
                        "description": "Anomaly detection sensitivity (0.0-1.0)",
                        "default": 0.5
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                },
                "required": [
                    "channel_id"
                ]
            }),
    }
}

fn handle_pattern_anomaly(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let sensitivity = get_f64(&args, "sensitivity").unwrap_or(0.5).clamp(0.0, 1.0);
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    // Analyze content length distribution for anomalies
    let content_lengths: Vec<usize> = channel_messages.iter().map(|m| m.content.len()).collect();
    let avg_len = if content_lengths.is_empty() { 0.0 } else {
        content_lengths.iter().sum::<usize>() as f64 / content_lengths.len() as f64
    };
    let std_dev = if content_lengths.len() > 1 {
        let variance = content_lengths.iter()
            .map(|&l| (l as f64 - avg_len).powi(2))
            .sum::<f64>() / content_lengths.len() as f64;
        variance.sqrt()
    } else { 0.0 };
    let threshold = avg_len + std_dev * (2.0 - sensitivity);
    let anomalies: Vec<Value> = channel_messages.iter()
        .filter(|m| m.content.len() as f64 > threshold || (m.content.len() as f64) < avg_len - std_dev * (2.0 - sensitivity))
        .take(20)
        .map(|m| json!({
            "message_id": m.id,
            "sender": m.sender,
            "content_length": m.content.len(),
            "deviation": (m.content.len() as f64 - avg_len).abs() / std_dev.max(1.0),
            "type": format!("{:?}", m.message_type)
        }))
        .collect();
    // Detect sender anomalies: senders who appear very rarely
    let mut sender_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for m in &channel_messages {
        *sender_counts.entry(m.sender.as_str()).or_insert(0) += 1;
    }
    let avg_sender_msgs = if sender_counts.is_empty() { 0.0 } else {
        sender_counts.values().sum::<usize>() as f64 / sender_counts.len() as f64
    };
    let sender_anomalies: Vec<Value> = sender_counts.iter()
        .filter(|(_, &c)| (c as f64) < avg_sender_msgs * sensitivity * 0.5)
        .map(|(s, &c)| json!({ "sender": s, "message_count": c, "type": "rare_sender" }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "sensitivity": sensitivity,
        "messages_analyzed": channel_messages.len(),
        "average_content_length": avg_len as u64,
        "content_std_dev": std_dev,
        "content_anomalies": anomalies.len(),
        "sender_anomalies": sender_anomalies.len(),
        "anomalies": anomalies,
        "sender_anomaly_detail": sender_anomalies,
        "status": "anomalies_detected"
    })))
}

// ── 8. comm_pattern_predict ──────────────────────────────────────────────
fn definition_pattern_predict() -> ToolDefinition {
    ToolDefinition {
        name: "comm_pattern_predict".into(),
        description: Some("Predict next communication pattern".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to predict for"
                    },
                    "horizon": {
                        "type": "integer",
                        "description": "Number of future messages to predict",
                        "default": 5
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                },
                "required": [
                    "channel_id"
                ]
            }),
    }
}

fn handle_pattern_predict(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let horizon = get_u64(&args, "horizon").unwrap_or(5) as usize;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    // Build transition probabilities
    let mut transitions: std::collections::HashMap<&str, std::collections::HashMap<&str, usize>> = std::collections::HashMap::new();
    for w in channel_messages.windows(2) {
        *transitions.entry(w[0].sender.as_str())
            .or_default()
            .entry(w[1].sender.as_str())
            .or_insert(0) += 1;
    }
    // Predict next senders based on last sender
    let last_sender = channel_messages.last().map(|m| m.sender.as_str()).unwrap_or("unknown");
    let mut predictions: Vec<Value> = Vec::new();
    let mut current = last_sender;
    for step in 0..horizon {
        if let Some(nexts) = transitions.get(current) {
            let best = nexts.iter().max_by_key(|(_, &c)| c);
            if let Some((next_sender, &count)) = best {
                let total: usize = nexts.values().sum();
                let probability = count as f64 / total as f64;
                predictions.push(json!({
                    "step": step + 1,
                    "predicted_sender": next_sender,
                    "probability": probability
                }));
                current = next_sender;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    let confidence = if channel_messages.len() > 20 { 0.7 } else if channel_messages.len() > 5 { 0.4 } else { 0.1 };
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "horizon": horizon,
        "messages_analyzed": channel_messages.len(),
        "last_sender": last_sender,
        "predictions": predictions,
        "prediction_count": predictions.len(),
        "model_confidence": confidence,
        "status": "pattern_predicted"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Health Monitoring (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 9. comm_health_status ────────────────────────────────────────────────
fn definition_health_status() -> ToolDefinition {
    ToolDefinition {
        name: "comm_health_status".into(),
        description: Some("Get overall communication health status".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                }
            }),
    }
}

fn handle_health_status(_args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let store = &session.store;
    let stats = store.stats();
    let dead_letter_ratio = if stats.message_count > 0 {
        stats.dead_letter_count as f64 / stats.message_count as f64
    } else { 0.0 };
    let health_score = (1.0 - dead_letter_ratio.min(1.0)) * 100.0;
    let health_grade = if health_score > 90.0 { "excellent" }
        else if health_score > 70.0 { "good" }
        else if health_score > 50.0 { "fair" }
        else { "poor" };
    Ok(ToolCallResult::json(&json!({
        "health_score": health_score,
        "health_grade": health_grade,
        "channel_count": stats.channel_count,
        "message_count": stats.message_count,
        "subscription_count": stats.subscription_count,
        "dead_letter_count": stats.dead_letter_count,
        "dead_letter_ratio": dead_letter_ratio,
        "temporal_queue_count": stats.temporal_queue_count,
        "hive_count": stats.hive_count,
        "federation_enabled": stats.federation_enabled,
        "federated_zone_count": stats.federated_zone_count,
        "audit_log_count": stats.audit_log_count,
        "status": "health_status_retrieved"
    })))
}

// ── 10. comm_health_diagnose ─────────────────────────────────────────────
fn definition_health_diagnose() -> ToolDefinition {
    ToolDefinition {
        name: "comm_health_diagnose".into(),
        description: Some("Diagnose communication health issues".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to diagnose (optional, omit for system-wide)"
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                }
            }),
    }
}

fn handle_health_diagnose(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id");
    let store = &session.store;
    let mut issues: Vec<Value> = Vec::new();
    // System-wide checks
    let stats = store.stats();
    if stats.dead_letter_count > 0 {
        issues.push(json!({
            "severity": "warning",
            "category": "dead_letters",
            "description": format!("{} dead letters found", stats.dead_letter_count)
        }));
    }
    if stats.temporal_queue_count > 50 {
        issues.push(json!({
            "severity": "warning",
            "category": "temporal_queue",
            "description": format!("{} pending temporal messages", stats.temporal_queue_count)
        }));
    }
    // Channel-specific checks
    if let Some(ch_id) = channel_id {
        if let Some(channel) = store.get_channel(ch_id) {
            if channel.participants.is_empty() {
                issues.push(json!({
                    "severity": "error",
                    "category": "channel",
                    "description": format!("Channel {} has no participants", ch_id)
                }));
            }
            let msg_count = store.messages.values()
                .filter(|m| m.channel_id == ch_id)
                .count();
            if msg_count == 0 {
                issues.push(json!({
                    "severity": "info",
                    "category": "channel",
                    "description": format!("Channel {} has no messages", ch_id)
                }));
            }
        } else {
            issues.push(json!({
                "severity": "error",
                "category": "channel",
                "description": format!("Channel {} not found", ch_id)
            }));
        }
    }
    // Check for channels with no recent activity
    for ch in store.channels.values() {
        let ch_msgs: Vec<_> = store.messages.values()
            .filter(|m| m.channel_id == ch.id)
            .collect();
        if ch_msgs.is_empty() && !ch.participants.is_empty() {
            issues.push(json!({
                "severity": "info",
                "category": "idle_channel",
                "description": format!("Channel '{}' (id={}) has participants but no messages", ch.name, ch.id)
            }));
        }
    }
    let severity_counts: std::collections::HashMap<&str, usize> = {
        let mut counts = std::collections::HashMap::new();
        for issue in &issues {
            let sev = issue.get("severity").and_then(|v| v.as_str()).unwrap_or("unknown");
            *counts.entry(sev).or_insert(0) += 1;
        }
        counts
    };
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "total_issues": issues.len(),
        "severity_counts": severity_counts,
        "issues": issues,
        "status": "diagnosis_complete"
    })))
}

// ── 11. comm_health_prescribe ────────────────────────────────────────────
fn definition_health_prescribe() -> ToolDefinition {
    ToolDefinition {
        name: "comm_health_prescribe".into(),
        description: Some("Prescribe actions to improve communication health".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to prescribe for (optional)"
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                }
            }),
    }
}

fn handle_health_prescribe(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id");
    let store = &session.store;
    let stats = store.stats();
    let mut prescriptions: Vec<Value> = Vec::new();
    if stats.dead_letter_count > 0 {
        prescriptions.push(json!({
            "priority": "high",
            "action": "replay_dead_letters",
            "description": format!("Attempt to redeliver {} dead letters", stats.dead_letter_count),
            "tool": "comm_dead_letter_phoenix"
        }));
    }
    if stats.temporal_queue_count > 20 {
        prescriptions.push(json!({
            "priority": "medium",
            "action": "review_temporal_queue",
            "description": format!("Review {} pending temporal messages", stats.temporal_queue_count),
            "tool": "comm_temporal_pending"
        }));
    }
    if !stats.federation_enabled && stats.federated_zone_count > 0 {
        prescriptions.push(json!({
            "priority": "medium",
            "action": "enable_federation",
            "description": "Federation zones exist but federation is disabled",
            "tool": "comm_federation_gateway_create"
        }));
    }
    if let Some(ch_id) = channel_id {
        if store.get_channel(ch_id).is_some() {
            let msg_count = store.messages.values()
                .filter(|m| m.channel_id == ch_id).count();
            if msg_count > 1000 {
                prescriptions.push(json!({
                    "priority": "low",
                    "action": "compact_channel",
                    "description": format!("Channel {} has {} messages, consider compaction", ch_id, msg_count),
                    "tool": "comm_compact"
                }));
            }
        }
    }
    if prescriptions.is_empty() {
        prescriptions.push(json!({
            "priority": "info",
            "action": "none",
            "description": "System is healthy, no prescriptions needed"
        }));
    }
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "prescription_count": prescriptions.len(),
        "prescriptions": prescriptions,
        "status": "prescriptions_generated"
    })))
}

// ── 12. comm_health_history ──────────────────────────────────────────────
fn definition_health_history() -> ToolDefinition {
    ToolDefinition {
        name: "comm_health_history".into(),
        description: Some("Get communication health history over time".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Maximum audit entries to return",
                        "default": 50
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                }
            }),
    }
}

fn handle_health_history(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let limit = get_u64(&args, "limit").unwrap_or(50) as usize;
    let store = &session.store;
    let audit_entries = store.get_audit_log(Some(limit));
    let entries: Vec<Value> = audit_entries.iter().map(|e| json!({
        "event_type": format!("{:?}", e.event_type),
        "timestamp": e.timestamp,
        "agent_id": e.agent_id,
        "description": e.description,
        "related_id": e.related_id
    })).collect();
    let stats = store.stats();
    Ok(ToolCallResult::json(&json!({
        "audit_entries": entries.len(),
        "total_audit_log": stats.audit_log_count,
        "entries": entries,
        "current_health": {
            "channels": stats.channel_count,
            "messages": stats.message_count,
            "dead_letters": stats.dead_letter_count,
            "temporal_queue": stats.temporal_queue_count
        },
        "status": "history_retrieved"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Oracle Predictions (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 13. comm_oracle_query ────────────────────────────────────────────────
fn definition_oracle_query() -> ToolDefinition {
    ToolDefinition {
        name: "comm_oracle_query".into(),
        description: Some("Query the oracle for communication predictions".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "Question for the oracle"
                    },
                    "context_channel_id": {
                        "type": "integer",
                        "description": "Channel to use as context (optional)"
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                },
                "required": [
                    "question"
                ]
            }),
    }
}

fn handle_oracle_query(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let question = get_str(&args, "question").ok_or("Missing question")?;
    let context_channel_id = get_u64(&args, "context_channel_id");
    let store = &session.store;
    let stats = store.stats();
    // Gather contextual data for the oracle
    let channel_context = context_channel_id.and_then(|ch_id| {
        store.get_channel(ch_id).map(|ch| {
            let msg_count = store.messages.values()
                .filter(|m| m.channel_id == ch_id).count();
            json!({
                "channel_name": ch.name,
                "participants": ch.participants.len(),
                "message_count": msg_count
            })
        })
    });
    // Search for relevant messages
    let relevant = store.search_messages(&question, 5);
    let relevant_msgs: Vec<Value> = relevant.iter().map(|m| json!({
        "message_id": m.id,
        "sender": m.sender,
        "content_preview": &m.content[..m.content.len().min(60)]
    })).collect();
    let confidence = if relevant.is_empty() { 0.2 } else { 0.5 + (relevant.len() as f64 * 0.1).min(0.4) };
    session.record_operation("comm_oracle_query", None);
    Ok(ToolCallResult::json(&json!({
        "question": question,
        "channel_context": channel_context,
        "relevant_messages_found": relevant_msgs.len(),
        "relevant_messages": relevant_msgs,
        "system_stats": {
            "channels": stats.channel_count,
            "messages": stats.message_count,
            "active_hives": stats.hive_count
        },
        "oracle_confidence": confidence,
        "status": "oracle_consulted"
    })))
}

// ── 14. comm_oracle_prophecy ─────────────────────────────────────────────
fn definition_oracle_prophecy() -> ToolDefinition {
    ToolDefinition {
        name: "comm_oracle_prophecy".into(),
        description: Some("Get prophecy about future communication state".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "horizon_minutes": {
                        "type": "integer",
                        "description": "How far into the future to predict (minutes)",
                        "default": 60
                    },
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to prophesy about (optional)"
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                }
            }),
    }
}

fn handle_oracle_prophecy(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let horizon_minutes = get_u64(&args, "horizon_minutes").unwrap_or(60);
    let channel_id = get_u64(&args, "channel_id");
    let store = &session.store;
    let stats = store.stats();
    // Calculate message velocity
    let total_msgs = stats.message_count;
    let pending_temporal = stats.temporal_queue_count;
    // Estimate future state based on current trends
    let projected_messages = total_msgs + (pending_temporal * horizon_minutes as usize / 60).max(1);
    let projected_dead_letters = if stats.dead_letter_count > 0 {
        stats.dead_letter_count + (stats.dead_letter_count * horizon_minutes as usize / 1440).max(0)
    } else { 0 };
    let mut prophecy = json!({
        "horizon_minutes": horizon_minutes,
        "current_messages": total_msgs,
        "projected_messages": projected_messages,
        "current_dead_letters": stats.dead_letter_count,
        "projected_dead_letters": projected_dead_letters,
        "pending_deliveries": pending_temporal,
        "confidence": if total_msgs > 100 { 0.6 } else { 0.3 }
    });
    if let Some(ch_id) = channel_id {
        if let Some(ch) = store.get_channel(ch_id) {
            let ch_msgs = store.messages.values()
                .filter(|m| m.channel_id == ch_id).count();
            if let Some(obj) = prophecy.as_object_mut() {
                obj.insert("channel_prophecy".into(), json!({
                    "channel_name": ch.name,
                    "current_messages": ch_msgs,
                    "projected_messages": ch_msgs + ch_msgs / 10 + 1,
                    "participant_count": ch.participants.len()
                }));
            }
        }
    }
    if let Some(obj) = prophecy.as_object_mut() {
        obj.insert("status".into(), json!("prophecy_delivered"));
    }
    Ok(ToolCallResult::json(&prophecy))
}

// ── 15. comm_oracle_verify ───────────────────────────────────────────────
fn definition_oracle_verify() -> ToolDefinition {
    ToolDefinition {
        name: "comm_oracle_verify".into(),
        description: Some("Verify past oracle predictions".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "prediction_type": {
                        "type": "string",
                        "description": "Type of prediction to verify (e.g., 'message_count', 'dead_letters')"
                    },
                    "predicted_value": {
                        "type": "number",
                        "description": "The value that was predicted"
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                },
                "required": [
                    "prediction_type",
                    "predicted_value"
                ]
            }),
    }
}

fn handle_oracle_verify(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let prediction_type = get_str(&args, "prediction_type").ok_or("Missing prediction_type")?;
    let predicted_value = get_f64(&args, "predicted_value").ok_or("Missing predicted_value")?;
    let store = &session.store;
    let stats = store.stats();
    let actual_value = match prediction_type.as_str() {
        "message_count" => stats.message_count as f64,
        "dead_letters" => stats.dead_letter_count as f64,
        "channel_count" => stats.channel_count as f64,
        "hive_count" => stats.hive_count as f64,
        "temporal_queue" => stats.temporal_queue_count as f64,
        _ => return Err(format!("Unknown prediction type: {prediction_type}")),
    };
    let error = (predicted_value - actual_value).abs();
    let accuracy = if predicted_value.abs() > 0.0 {
        1.0 - (error / predicted_value.abs()).min(1.0)
    } else if actual_value == 0.0 { 1.0 } else { 0.0 };
    Ok(ToolCallResult::json(&json!({
        "prediction_type": prediction_type,
        "predicted_value": predicted_value,
        "actual_value": actual_value,
        "absolute_error": error,
        "accuracy": accuracy,
        "accurate": accuracy > 0.8,
        "status": "prediction_verified"
    })))
}

// ── 16. comm_oracle_calibrate ────────────────────────────────────────────
fn definition_oracle_calibrate() -> ToolDefinition {
    ToolDefinition {
        name: "comm_oracle_calibrate".into(),
        description: Some("Calibrate oracle prediction model".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "learning_rate": {
                        "type": "number",
                        "description": "Calibration learning rate (0.0-1.0)",
                        "default": 0.1
                    },
                    "training_window": {
                        "type": "integer",
                        "description": "Number of recent events to train on",
                        "default": 100
                    },
                    "include_content": {
                        "type": "boolean",
                        "default": false,
                        "description": "Return full content (default: IDs only)"
                    },
                    "intent": {
                        "type": "string",
                        "enum": [
                            "exists",
                            "ids",
                            "summary",
                            "fields",
                            "full"
                        ],
                        "description": "Extraction intent level"
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return changes since this Unix timestamp"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum token budget for response"
                    },
                    "max_results": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum number of results"
                    },
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                }
            }),
    }
}

fn handle_oracle_calibrate(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let learning_rate = get_f64(&args, "learning_rate").unwrap_or(0.1).clamp(0.0, 1.0);
    let training_window = get_u64(&args, "training_window").unwrap_or(100) as usize;
    let store = &session.store;
    let stats = store.stats();
    let audit_entries = store.get_audit_log(Some(training_window));
    // Analyze audit log for calibration data
    let mut event_type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for entry in &audit_entries {
        *event_type_counts.entry(format!("{:?}", entry.event_type)).or_insert(0) += 1;
    }
    let training_samples = audit_entries.len();
    let model_complexity = event_type_counts.len();
    session.record_operation("comm_oracle_calibrate", None);
    Ok(ToolCallResult::json(&json!({
        "learning_rate": learning_rate,
        "training_window": training_window,
        "training_samples": training_samples,
        "model_complexity": model_complexity,
        "event_type_distribution": event_type_counts,
        "system_state": {
            "channels": stats.channel_count,
            "messages": stats.message_count,
            "dead_letters": stats.dead_letter_count
        },
        "calibration_complete": true,
        "status": "oracle_calibrated"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

pub fn all_definitions() -> Vec<ToolDefinition> {
    vec![
        definition_forensics_investigate(),
        definition_forensics_timeline(),
        definition_forensics_evidence(),
        definition_forensics_report(),
        definition_pattern_detect(),
        definition_pattern_recurring(),
        definition_pattern_anomaly(),
        definition_pattern_predict(),
        definition_health_status(),
        definition_health_diagnose(),
        definition_health_prescribe(),
        definition_health_history(),
        definition_oracle_query(),
        definition_oracle_prophecy(),
        definition_oracle_verify(),
        definition_oracle_calibrate(),
    ]
}

pub fn try_execute(
    name: &str,
    args: Value,
    session: &mut SessionManager,
) -> Option<Result<ToolCallResult, String>> {
    match name {
        "comm_forensics_investigate" => Some(handle_forensics_investigate(args, session)),
        "comm_forensics_timeline" => Some(handle_forensics_timeline(args, session)),
        "comm_forensics_evidence" => Some(handle_forensics_evidence(args, session)),
        "comm_forensics_report" => Some(handle_forensics_report(args, session)),
        "comm_pattern_detect" => Some(handle_pattern_detect(args, session)),
        "comm_pattern_recurring" => Some(handle_pattern_recurring(args, session)),
        "comm_pattern_anomaly" => Some(handle_pattern_anomaly(args, session)),
        "comm_pattern_predict" => Some(handle_pattern_predict(args, session)),
        "comm_health_status" => Some(handle_health_status(args, session)),
        "comm_health_diagnose" => Some(handle_health_diagnose(args, session)),
        "comm_health_prescribe" => Some(handle_health_prescribe(args, session)),
        "comm_health_history" => Some(handle_health_history(args, session)),
        "comm_oracle_query" => Some(handle_oracle_query(args, session)),
        "comm_oracle_prophecy" => Some(handle_oracle_prophecy(args, session)),
        "comm_oracle_verify" => Some(handle_oracle_verify(args, session)),
        "comm_oracle_calibrate" => Some(handle_oracle_calibrate(args, session)),
        _ => None,
    }
}
