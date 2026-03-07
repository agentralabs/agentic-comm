//! Invention modules: Affective Contagion, Emotional Archaeology, Affect Prophecy,
//! Unspeakable Content, Anticipatory Understanding — 20 tools for the AFFECT category of the Comm Inventions.

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
// 1. Affective Contagion (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 1. comm_affect_contagion_model ───────────────────────────────────────
fn definition_affect_contagion_model() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_contagion_model".into(),
        description: Some("Get the current contagion model parameters".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Optional: scope to a specific channel"
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

fn handle_affect_contagion_model(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id");
    let store = &session.store;
    let resistance = store.affect_resistance;
    let agent_count = store.affect_states.len();
    // If channel_id given, narrow to agents in that channel
    let channel_agents: Option<Vec<String>> = channel_id.and_then(|cid| {
        store.get_channel(cid).map(|ch| ch.participants.clone())
    });
    let scoped_agents = channel_agents.as_ref().map_or(agent_count, |a| a.len());
    // Compute average affect across scoped agents
    let mut total_valence = 0.0f64;
    let mut total_arousal = 0.0f64;
    let mut count = 0u32;
    for (agent_id, state) in &store.affect_states {
        let in_scope = channel_agents.as_ref().map_or(true, |agents| agents.contains(agent_id));
        if in_scope {
            total_valence += state.valence;
            total_arousal += state.arousal;
            count += 1;
        }
    }
    let avg_valence = if count > 0 { total_valence / count as f64 } else { 0.0 };
    let avg_arousal = if count > 0 { total_arousal / count as f64 } else { 0.0 };
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "global_resistance": resistance,
        "total_agents_with_state": agent_count,
        "scoped_agents": scoped_agents,
        "measured_agents": count,
        "average_valence": avg_valence,
        "average_arousal": avg_arousal,
        "contagion_susceptibility": 1.0 - resistance,
        "status": "model_retrieved"
    })))
}

// ── 2. comm_affect_contagion_simulate ────────────────────────────────────
fn definition_affect_contagion_simulate() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_contagion_simulate".into(),
        description: Some("Simulate affect spread across a channel".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to simulate"
                    },
                    "source_agent": {
                        "type": "string",
                        "description": "Agent originating the affect"
                    },
                    "source_valence": {
                        "type": "number",
                        "description": "Valence of the source affect (-1.0 to 1.0)"
                    },
                    "source_arousal": {
                        "type": "number",
                        "description": "Arousal of the source affect (0.0 to 1.0)"
                    },
                    "steps": {
                        "type": "integer",
                        "description": "Simulation steps",
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
                    "channel_id",
                    "source_agent",
                    "source_valence",
                    "source_arousal"
                ]
            }),
    }
}

fn handle_affect_contagion_simulate(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let source_agent = get_str(&args, "source_agent").ok_or("Missing source_agent")?;
    let source_valence = get_f64(&args, "source_valence").ok_or("Missing source_valence")?;
    let source_arousal = get_f64(&args, "source_arousal").ok_or("Missing source_arousal")?;
    let steps = get_u64(&args, "steps").unwrap_or(5) as usize;
    let store = &session.store;
    let channel = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let resistance = store.affect_resistance;
    let susceptibility = 1.0 - resistance;
    // Simulate spread across participants
    let mut simulation: Vec<Value> = Vec::new();
    let mut current_valence = source_valence;
    let mut current_arousal = source_arousal;
    for step in 0..steps {
        let spread_factor = susceptibility * (0.9_f64).powi(step as i32);
        current_valence *= spread_factor;
        current_arousal *= spread_factor;
        let affected_agents: Vec<Value> = channel.participants.iter()
            .filter(|p| *p != &source_agent)
            .map(|p| {
                let existing = store.get_affect_state(p);
                let base_valence = existing.map_or(0.0, |s| s.valence);
                json!({
                    "agent": p,
                    "projected_valence": base_valence + current_valence * spread_factor,
                    "projected_arousal": current_arousal * spread_factor
                })
            })
            .collect();
        simulation.push(json!({
            "step": step + 1,
            "spread_factor": spread_factor,
            "valence_at_step": current_valence,
            "arousal_at_step": current_arousal,
            "affected_agents": affected_agents
        }));
    }
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "source_agent": source_agent,
        "initial_valence": source_valence,
        "initial_arousal": source_arousal,
        "resistance": resistance,
        "total_participants": channel.participants.len(),
        "simulation_steps": simulation.len(),
        "simulation": simulation,
        "status": "simulation_complete"
    })))
}

// ── 3. comm_affect_contagion_immunize ────────────────────────────────────
fn definition_affect_contagion_immunize() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_contagion_immunize".into(),
        description: Some("Set immunity to affect contagion for an agent".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "resistance": {
                        "type": "number",
                        "description": "Global resistance level (0.0 = fully susceptible, 1.0 = immune)"
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
                    "resistance"
                ]
            }),
    }
}

fn handle_affect_contagion_immunize(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let resistance = get_f64(&args, "resistance").ok_or("Missing resistance")?;
    let clamped = resistance.clamp(0.0, 1.0);
    let previous = session.store.set_affect_resistance(clamped);
    session.record_operation("comm_affect_contagion_immunize", None);
    Ok(ToolCallResult::json(&json!({
        "previous_resistance": previous,
        "new_resistance": clamped,
        "susceptibility": 1.0 - clamped,
        "status": "resistance_updated"
    })))
}

// ── 4. comm_affect_contagion_outbreak ────────────────────────────────────
fn definition_affect_contagion_outbreak() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_contagion_outbreak".into(),
        description: Some("Detect affect outbreaks in conversations".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "threshold": {
                        "type": "number",
                        "description": "Arousal threshold for outbreak detection (0.0-1.0)",
                        "default": 0.7
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

fn handle_affect_contagion_outbreak(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let threshold = get_f64(&args, "threshold").unwrap_or(0.7).clamp(0.0, 1.0);
    let store = &session.store;
    // Find agents with arousal above threshold
    let outbreak_agents: Vec<Value> = store.affect_states.iter()
        .filter(|(_, state)| state.arousal >= threshold)
        .map(|(agent, state)| json!({
            "agent": agent,
            "valence": state.valence,
            "arousal": state.arousal,
            "dominance": state.dominance,
            "urgency": format!("{:?}", state.urgency),
            "emotions": state.emotions.iter().map(|e| format!("{:?}", e)).collect::<Vec<_>>()
        }))
        .collect();
    let is_outbreak = outbreak_agents.len() > store.affect_states.len() / 3;
    Ok(ToolCallResult::json(&json!({
        "threshold": threshold,
        "total_agents": store.affect_states.len(),
        "agents_above_threshold": outbreak_agents.len(),
        "is_outbreak": is_outbreak,
        "outbreak_agents": outbreak_agents,
        "status": "outbreak_detection_complete"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Emotional Archaeology (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 5. comm_affect_archaeology_dig ───────────────────────────────────────
fn definition_affect_archaeology_dig() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_archaeology_dig".into(),
        description: Some("Search for historical affect patterns".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "agent": {
                        "type": "string",
                        "description": "Agent to dig affect history for"
                    },
                    "emotion_filter": {
                        "type": "string",
                        "description": "Optional: filter by emotion name"
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
                    "agent"
                ]
            }),
    }
}

fn handle_affect_archaeology_dig(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let emotion_filter = get_str(&args, "emotion_filter");
    let store = &session.store;
    let history = store.get_affect_history(&agent);
    let filtered: Vec<Value> = history.states.iter()
        .filter(|s| emotion_filter.as_ref().map_or(true, |f| {
            s.emotion.to_lowercase().contains(&f.to_lowercase())
        }))
        .map(|s| json!({
            "timestamp": s.timestamp,
            "emotion": s.emotion,
            "intensity": s.intensity,
            "valence": s.valence,
            "arousal": s.arousal,
            "dominance": s.dominance,
            "source": s.source
        }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "agent": agent,
        "emotion_filter": emotion_filter,
        "total_entries": history.states.len(),
        "filtered_entries": filtered.len(),
        "artifacts": filtered,
        "status": "dig_complete"
    })))
}

// ── 6. comm_affect_archaeology_reconstruct ───────────────────────────────
fn definition_affect_archaeology_reconstruct() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_archaeology_reconstruct".into(),
        description: Some("Reconstruct lost affect states".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "agent": {
                        "type": "string",
                        "description": "Agent to reconstruct affect for"
                    },
                    "start_timestamp": {
                        "type": "integer",
                        "description": "Start of reconstruction window (epoch seconds)"
                    },
                    "end_timestamp": {
                        "type": "integer",
                        "description": "End of reconstruction window (epoch seconds)"
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
                    "agent",
                    "start_timestamp",
                    "end_timestamp"
                ]
            }),
    }
}

fn handle_affect_archaeology_reconstruct(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let start_ts = get_u64(&args, "start_timestamp").ok_or("Missing start_timestamp")?;
    let end_ts = get_u64(&args, "end_timestamp").ok_or("Missing end_timestamp")?;
    if start_ts >= end_ts {
        return Err("start_timestamp must be before end_timestamp".into());
    }
    let store = &session.store;
    let history = store.get_affect_history(&agent);
    // Find entries within the window
    let window_entries: Vec<&agentic_comm::types::AffectHistoryEntry> = history.states.iter()
        .filter(|s| s.timestamp >= start_ts && s.timestamp <= end_ts)
        .collect();
    // Reconstruct by interpolating between known states
    let reconstructed: Vec<Value> = if window_entries.is_empty() {
        // No data in window — try to interpolate from nearest known states
        let before: Option<&agentic_comm::types::AffectHistoryEntry> = history.states.iter()
            .filter(|s| s.timestamp < start_ts)
            .last();
        let after: Option<&agentic_comm::types::AffectHistoryEntry> = history.states.iter()
            .filter(|s| s.timestamp > end_ts)
            .next();
        match (before, after) {
            (Some(b), Some(a)) => vec![json!({
                "type": "interpolated",
                "start_state": {"valence": b.valence, "arousal": b.arousal},
                "end_state": {"valence": a.valence, "arousal": a.arousal},
                "interpolated_valence": (b.valence + a.valence) / 2.0,
                "interpolated_arousal": (b.arousal + a.arousal) / 2.0,
                "confidence": 0.3
            })],
            (Some(b), None) => vec![json!({
                "type": "extrapolated_forward",
                "base_state": {"valence": b.valence, "arousal": b.arousal},
                "confidence": 0.2
            })],
            (None, Some(a)) => vec![json!({
                "type": "extrapolated_backward",
                "base_state": {"valence": a.valence, "arousal": a.arousal},
                "confidence": 0.2
            })],
            (None, None) => vec![json!({
                "type": "no_data",
                "confidence": 0.0
            })],
        }
    } else {
        window_entries.iter().map(|s| json!({
            "type": "observed",
            "timestamp": s.timestamp,
            "emotion": s.emotion,
            "valence": s.valence,
            "arousal": s.arousal,
            "dominance": s.dominance,
            "source": s.source,
            "confidence": 1.0
        })).collect()
    };
    Ok(ToolCallResult::json(&json!({
        "agent": agent,
        "start_timestamp": start_ts,
        "end_timestamp": end_ts,
        "window_seconds": end_ts - start_ts,
        "observed_entries": window_entries.len(),
        "reconstruction": reconstructed,
        "status": "reconstruction_complete"
    })))
}

// ── 7. comm_affect_archaeology_timeline ──────────────────────────────────
fn definition_affect_archaeology_timeline() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_archaeology_timeline".into(),
        description: Some("Get affect timeline for a channel".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to get timeline for"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max entries to return",
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

fn handle_affect_archaeology_timeline(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let limit = get_u64(&args, "limit").unwrap_or(50) as usize;
    let store = &session.store;
    let channel = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    // Build timeline from affect histories of channel participants
    let mut timeline: Vec<Value> = Vec::new();
    for participant in &channel.participants {
        let history = store.get_affect_history(participant);
        for entry in &history.states {
            timeline.push(json!({
                "agent": participant,
                "timestamp": entry.timestamp,
                "emotion": entry.emotion,
                "intensity": entry.intensity,
                "valence": entry.valence,
                "arousal": entry.arousal,
                "source": entry.source
            }));
        }
    }
    // Sort by timestamp
    timeline.sort_by(|a, b| {
        let ta = a.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0);
        let tb = b.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0);
        ta.cmp(&tb)
    });
    timeline.truncate(limit);
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "participants": channel.participants,
        "timeline_entries": timeline.len(),
        "timeline": timeline,
        "status": "timeline_complete"
    })))
}

// ── 8. comm_affect_archaeology_artifacts ─────────────────────────────────
fn definition_affect_archaeology_artifacts() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_archaeology_artifacts".into(),
        description: Some("Find affect artifacts near a message".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "message_id": {
                        "type": "integer",
                        "description": "Message to find nearby affect artifacts for"
                    },
                    "window_seconds": {
                        "type": "integer",
                        "description": "Time window around the message (seconds)",
                        "default": 300
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

fn handle_affect_archaeology_artifacts(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let window_secs = get_u64(&args, "window_seconds").unwrap_or(300);
    let store = &session.store;
    let msg = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    let msg_ts = msg.timestamp.timestamp() as u64;
    let start = msg_ts.saturating_sub(window_secs);
    let end = msg_ts + window_secs;
    // Find affect history entries near this timestamp for the sender
    let sender_history = store.get_affect_history(&msg.sender);
    let nearby_artifacts: Vec<Value> = sender_history.states.iter()
        .filter(|s| s.timestamp >= start && s.timestamp <= end)
        .map(|s| json!({
            "agent": msg.sender,
            "timestamp": s.timestamp,
            "emotion": s.emotion,
            "intensity": s.intensity,
            "valence": s.valence,
            "arousal": s.arousal,
            "source": s.source,
            "offset_from_message_secs": (s.timestamp as i64) - (msg_ts as i64)
        }))
        .collect();
    // Also check other agents in the same channel
    let channel = store.get_channel(msg.channel_id);
    let mut other_artifacts: Vec<Value> = Vec::new();
    if let Some(ch) = channel {
        for participant in &ch.participants {
            if participant == &msg.sender { continue; }
            let ph = store.get_affect_history(participant);
            for s in &ph.states {
                if s.timestamp >= start && s.timestamp <= end {
                    other_artifacts.push(json!({
                        "agent": participant,
                        "timestamp": s.timestamp,
                        "emotion": s.emotion,
                        "intensity": s.intensity,
                        "valence": s.valence,
                        "offset_from_message_secs": (s.timestamp as i64) - (msg_ts as i64)
                    }));
                }
            }
        }
    }
    Ok(ToolCallResult::json(&json!({
        "message_id": message_id,
        "message_timestamp": msg_ts,
        "window_seconds": window_secs,
        "sender_artifacts": nearby_artifacts.len(),
        "other_artifacts": other_artifacts.len(),
        "sender_affect": nearby_artifacts,
        "channel_affect": other_artifacts,
        "status": "artifacts_found"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Affect Prophecy (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 9. comm_affect_prophecy_predict ──────────────────────────────────────
fn definition_affect_prophecy_predict() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_prophecy_predict".into(),
        description: Some("Predict future affect states".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "agent": {
                        "type": "string",
                        "description": "Agent to predict affect for"
                    },
                    "horizon_seconds": {
                        "type": "integer",
                        "description": "Prediction horizon in seconds",
                        "default": 3600
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
                    "agent"
                ]
            }),
    }
}

fn handle_affect_prophecy_predict(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let horizon = get_u64(&args, "horizon_seconds").unwrap_or(3600);
    let store = &session.store;
    let current = store.get_affect_state(&agent);
    let history = store.get_affect_history(&agent);
    // Simple linear prediction based on trend in history
    let recent: Vec<&agentic_comm::types::AffectHistoryEntry> = history.states.iter().rev().take(5).collect();
    let (predicted_valence, predicted_arousal, confidence) = if recent.len() >= 2 {
        // Safe: recent.len() >= 2 guarantees first() and last() are Some
        let first = &recent[recent.len() - 1];
        let last = &recent[0];
        let valence_trend = last.valence - first.valence;
        let arousal_trend = last.arousal - first.arousal;
        let time_span = if last.timestamp > first.timestamp { (last.timestamp - first.timestamp) as f64 } else { 1.0 };
        let valence_rate = valence_trend / time_span;
        let arousal_rate = arousal_trend / time_span;
        let pv = (last.valence + valence_rate * horizon as f64).clamp(-1.0, 1.0);
        let pa = (last.arousal + arousal_rate * horizon as f64).clamp(0.0, 1.0);
        let conf = (0.8 - (horizon as f64 / 86400.0) * 0.5).clamp(0.1, 0.9);
        (pv, pa, conf)
    } else if let Some(cur) = current {
        // No trend data — predict same as current with low confidence
        (cur.valence, cur.arousal, 0.3)
    } else {
        (0.0, 0.0, 0.1)
    };
    Ok(ToolCallResult::json(&json!({
        "agent": agent,
        "horizon_seconds": horizon,
        "current_valence": current.map(|c| c.valence),
        "current_arousal": current.map(|c| c.arousal),
        "predicted_valence": predicted_valence,
        "predicted_arousal": predicted_arousal,
        "confidence": confidence,
        "data_points_used": recent.len(),
        "status": "prediction_complete"
    })))
}

// ── 10. comm_affect_prophecy_warn ────────────────────────────────────────
fn definition_affect_prophecy_warn() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_prophecy_warn".into(),
        description: Some("Get early warnings for affect crises".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to monitor"
                    },
                    "valence_threshold": {
                        "type": "number",
                        "description": "Negative valence threshold for crisis (-1.0 to 0.0)",
                        "default": -0.5
                    },
                    "arousal_threshold": {
                        "type": "number",
                        "description": "Arousal threshold for crisis (0.0 to 1.0)",
                        "default": 0.8
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

fn handle_affect_prophecy_warn(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let valence_threshold = get_f64(&args, "valence_threshold").unwrap_or(-0.5);
    let arousal_threshold = get_f64(&args, "arousal_threshold").unwrap_or(0.8);
    let store = &session.store;
    let channel = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let mut warnings: Vec<Value> = Vec::new();
    for participant in &channel.participants {
        if let Some(state) = store.get_affect_state(participant) {
            let mut agent_warnings = Vec::new();
            if state.valence < valence_threshold {
                agent_warnings.push(format!("Negative valence ({:.2}) below threshold ({:.2})", state.valence, valence_threshold));
            }
            if state.arousal > arousal_threshold {
                agent_warnings.push(format!("High arousal ({:.2}) above threshold ({:.2})", state.arousal, arousal_threshold));
            }
            if !agent_warnings.is_empty() {
                warnings.push(json!({
                    "agent": participant,
                    "valence": state.valence,
                    "arousal": state.arousal,
                    "urgency": format!("{:?}", state.urgency),
                    "warnings": agent_warnings
                }));
            }
        }
    }
    let crisis_level = if warnings.len() > channel.participants.len() / 2 {
        "critical"
    } else if !warnings.is_empty() {
        "elevated"
    } else {
        "normal"
    };
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "participants": channel.participants.len(),
        "agents_with_warnings": warnings.len(),
        "crisis_level": crisis_level,
        "valence_threshold": valence_threshold,
        "arousal_threshold": arousal_threshold,
        "warnings": warnings,
        "status": "warning_check_complete"
    })))
}

// ── 11. comm_affect_prophecy_track ───────────────────────────────────────
fn definition_affect_prophecy_track() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_prophecy_track".into(),
        description: Some("Track prophecy accuracy over time".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "agent": {
                        "type": "string",
                        "description": "Agent whose prophecy accuracy to track"
                    },
                    "lookback_entries": {
                        "type": "integer",
                        "description": "Number of history entries to analyze",
                        "default": 20
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
                    "agent"
                ]
            }),
    }
}

fn handle_affect_prophecy_track(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let lookback = get_u64(&args, "lookback_entries").unwrap_or(20) as usize;
    let store = &session.store;
    let history = store.get_affect_history(&agent);
    let entries: Vec<&agentic_comm::types::AffectHistoryEntry> = history.states.iter().rev().take(lookback).collect();
    // Measure how well each state predicts the next (stability)
    let mut prediction_errors: Vec<f64> = Vec::new();
    for window in entries.windows(2) {
        let current = window[0];
        let previous = window[1];
        let error = ((current.valence - previous.valence).powi(2) + (current.arousal - previous.arousal).powi(2)).sqrt();
        prediction_errors.push(error);
    }
    let avg_error = if prediction_errors.is_empty() { 0.0 } else {
        prediction_errors.iter().sum::<f64>() / prediction_errors.len() as f64
    };
    let stability = (1.0 - avg_error.min(1.0)).max(0.0);
    let predictability = if stability > 0.8 { "highly_predictable" } else if stability > 0.5 { "moderately_predictable" } else { "volatile" };
    Ok(ToolCallResult::json(&json!({
        "agent": agent,
        "entries_analyzed": entries.len(),
        "prediction_pairs": prediction_errors.len(),
        "average_prediction_error": avg_error,
        "stability_score": stability,
        "predictability": predictability,
        "status": "tracking_complete"
    })))
}

// ── 12. comm_affect_prophecy_similar ─────────────────────────────────────
fn definition_affect_prophecy_similar() -> ToolDefinition {
    ToolDefinition {
        name: "comm_affect_prophecy_similar".into(),
        description: Some("Find similar affect trajectories".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "agent": {
                        "type": "string",
                        "description": "Reference agent"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Max similar agents to return",
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
                    "cursor": {
                        "type": "string",
                        "description": "Pagination cursor for next page"
                    }
                },
                "required": [
                    "agent"
                ]
            }),
    }
}

fn handle_affect_prophecy_similar(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let max_results = get_u64(&args, "max_results").unwrap_or(5) as usize;
    let store = &session.store;
    let ref_state = store.get_affect_state(&agent);
    let (ref_valence, ref_arousal) = ref_state.map_or((0.0, 0.0), |s| (s.valence, s.arousal));
    // Compare with all other agents
    let mut similarities: Vec<(String, f64)> = store.affect_states.iter()
        .filter(|(id, _)| *id != &agent)
        .map(|(id, state)| {
            let distance = ((state.valence - ref_valence).powi(2) + (state.arousal - ref_arousal).powi(2)).sqrt();
            let similarity = (1.0 - distance / 2.0_f64.sqrt()).max(0.0); // Normalize: max distance is sqrt(2)
            (id.clone(), similarity)
        })
        .collect();
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    similarities.truncate(max_results);
    let results: Vec<Value> = similarities.iter().map(|(id, sim)| {
        let state = store.get_affect_state(id);
        json!({
            "agent": id,
            "similarity": sim,
            "valence": state.map(|s| s.valence),
            "arousal": state.map(|s| s.arousal)
        })
    }).collect();
    Ok(ToolCallResult::json(&json!({
        "reference_agent": agent,
        "reference_valence": ref_valence,
        "reference_arousal": ref_arousal,
        "similar_count": results.len(),
        "similar_agents": results,
        "status": "similarity_search_complete"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Unspeakable Content (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 13. comm_unspeakable_encode ──────────────────────────────────────────
fn definition_unspeakable_encode() -> ToolDefinition {
    ToolDefinition {
        name: "comm_unspeakable_encode".into(),
        description: Some("Encode unspeakable content beyond text representation".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to encode into"
                    },
                    "sender": {
                        "type": "string",
                        "description": "Encoding agent"
                    },
                    "approximation": {
                        "type": "string",
                        "description": "Closest textual approximation"
                    },
                    "dimensions": {
                        "type": "object",
                        "description": "Non-textual dimensions: valence, arousal, complexity, urgency (all 0.0-1.0)"
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
                    "sender",
                    "approximation"
                ]
            }),
    }
}

fn handle_unspeakable_encode(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let sender = get_str(&args, "sender").ok_or("Missing sender")?;
    let approximation = get_str(&args, "approximation").ok_or("Missing approximation")?;
    let dimensions = args.get("dimensions").cloned().unwrap_or_else(|| json!({}));
    let _ = session.store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    // Encode as a message with special metadata marker
    let encoded_content = format!("[unspeakable] {approximation}");
    match session.store.send_message(channel_id, &sender, &encoded_content, agentic_comm::MessageType::Text) {
        Ok(msg) => {
            session.record_operation("comm_unspeakable_encode", Some(msg.id));
            Ok(ToolCallResult::json(&json!({
                "message_id": msg.id,
                "channel_id": channel_id,
                "sender": sender,
                "approximation_length": approximation.len(),
                "dimensions": dimensions,
                "fidelity": "lossy",
                "encoding": "textual_approximation_with_dimensions",
                "status": "encoded"
            })))
        }
        Err(e) => Ok(ToolCallResult::error(format!("Encoding failed: {e}"))),
    }
}

// ── 14. comm_unspeakable_decode ──────────────────────────────────────────
fn definition_unspeakable_decode() -> ToolDefinition {
    ToolDefinition {
        name: "comm_unspeakable_decode".into(),
        description: Some("Decode unspeakable content for an agent".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "message_id": {
                        "type": "integer",
                        "description": "Message containing unspeakable content"
                    },
                    "decoder": {
                        "type": "string",
                        "description": "Agent decoding the content"
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
                    "message_id",
                    "decoder"
                ]
            }),
    }
}

fn handle_unspeakable_decode(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let decoder = get_str(&args, "decoder").ok_or("Missing decoder")?;
    let store = &session.store;
    let msg = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    if !msg.content.starts_with("[unspeakable]") {
        return Err(format!("Message {message_id} is not unspeakable content"));
    }
    let approximation = msg.content.strip_prefix("[unspeakable] ").unwrap_or(&msg.content);
    // Check decoder's affect state for context
    let decoder_state = store.get_affect_state(&decoder);
    let interpretation_bias = decoder_state.map_or(0.0, |s| s.valence);
    Ok(ToolCallResult::json(&json!({
        "message_id": message_id,
        "decoder": decoder,
        "original_sender": msg.sender,
        "approximation": approximation,
        "interpretation_bias": interpretation_bias,
        "fidelity_warning": "This is a lossy approximation of unspeakable content",
        "status": "decoded"
    })))
}

// ── 15. comm_unspeakable_detect ──────────────────────────────────────────
fn definition_unspeakable_detect() -> ToolDefinition {
    ToolDefinition {
        name: "comm_unspeakable_detect".into(),
        description: Some("Detect unspeakable content in messages".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel to scan"
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

fn handle_unspeakable_detect(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let unspeakable: Vec<Value> = store.messages.values()
        .filter(|m| m.channel_id == channel_id && m.content.starts_with("[unspeakable]"))
        .map(|m| json!({
            "message_id": m.id,
            "sender": m.sender,
            "approximation_preview": &m.content[..m.content.len().min(80)],
            "timestamp": m.timestamp.to_rfc3339()
        }))
        .collect();
    let total_messages = store.messages.values().filter(|m| m.channel_id == channel_id).count();
    let ratio = if total_messages > 0 { unspeakable.len() as f64 / total_messages as f64 } else { 0.0 };
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "total_messages": total_messages,
        "unspeakable_count": unspeakable.len(),
        "unspeakable_ratio": ratio,
        "unspeakable_messages": unspeakable,
        "status": "detection_complete"
    })))
}

// ── 16. comm_unspeakable_translate ───────────────────────────────────────
fn definition_unspeakable_translate() -> ToolDefinition {
    ToolDefinition {
        name: "comm_unspeakable_translate".into(),
        description: Some("Translate unspeakable content to approximation".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "message_id": {
                        "type": "integer",
                        "description": "Message containing unspeakable content"
                    },
                    "target_modality": {
                        "type": "string",
                        "description": "Target modality: text, affect, metaphor",
                        "default": "text"
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

fn handle_unspeakable_translate(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let target_modality = get_str(&args, "target_modality").unwrap_or_else(|| "text".to_string());
    let store = &session.store;
    let msg = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    if !msg.content.starts_with("[unspeakable]") {
        return Err(format!("Message {message_id} is not unspeakable content"));
    }
    let approximation = msg.content.strip_prefix("[unspeakable] ").unwrap_or(&msg.content);
    // Translate based on target modality
    let translation = match target_modality.as_str() {
        "text" => json!({
            "modality": "text",
            "translation": approximation,
            "fidelity": 0.3
        }),
        "affect" => {
            // Try to express as affect dimensions
            let sender_state = store.get_affect_state(&msg.sender);
            json!({
                "modality": "affect",
                "valence": sender_state.map_or(0.0, |s| s.valence),
                "arousal": sender_state.map_or(0.5, |s| s.arousal),
                "dominance": sender_state.map_or(0.5, |s| s.dominance),
                "fidelity": 0.5
            })
        }
        "metaphor" => json!({
            "modality": "metaphor",
            "translation": format!("Like trying to describe a color to someone who has never seen: {}", &approximation[..approximation.len().min(60)]),
            "fidelity": 0.2
        }),
        _ => json!({
            "modality": target_modality,
            "translation": approximation,
            "fidelity": 0.1,
            "warning": "Unknown modality — falling back to text"
        }),
    };
    Ok(ToolCallResult::json(&json!({
        "message_id": message_id,
        "original_sender": msg.sender,
        "target_modality": target_modality,
        "translation": translation,
        "status": "translation_complete"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Anticipatory Understanding (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 17. comm_anticipate_predict ───────────────────────────────────────────
fn definition_anticipate_predict() -> ToolDefinition {
    ToolDefinition {
        name: "comm_anticipate_predict".into(),
        description: Some("Predict what an agent needs before they ask".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "agent": {
                        "type": "string",
                        "description": "Agent to anticipate needs for"
                    },
                    "channel_id": {
                        "type": "integer",
                        "description": "Optional: scope prediction to a channel"
                    },
                    "horizon_messages": {
                        "type": "integer",
                        "description": "Number of future messages to predict for",
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
                    "agent"
                ]
            }),
    }
}

fn handle_anticipate_predict(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let channel_id = get_u64(&args, "channel_id");
    let horizon = get_u64(&args, "horizon_messages").unwrap_or(5) as usize;
    let store = &session.store;
    // Analyze agent's message patterns
    let agent_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.sender == agent)
        .filter(|m| channel_id.map_or(true, |cid| m.channel_id == cid))
        .collect();
    // Analyze affect state
    let affect = store.get_affect_state(&agent);
    let affect_history = store.get_affect_history(&agent);
    // Pattern analysis: message frequency, topic keywords, response patterns
    let mut word_freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut reply_ratio = 0.0f64;
    let mut replies = 0u32;
    for msg in &agent_messages {
        for word in msg.content.to_lowercase().split_whitespace() {
            let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.len() > 3 {
                *word_freq.entry(clean).or_insert(0) += 1;
            }
        }
        if msg.reply_to.is_some() {
            replies += 1;
        }
    }
    if !agent_messages.is_empty() {
        reply_ratio = replies as f64 / agent_messages.len() as f64;
    }
    // Top topics
    let mut topics: Vec<(String, usize)> = word_freq.into_iter().collect();
    topics.sort_by(|a, b| b.1.cmp(&a.1));
    topics.truncate(5);
    let predicted_topics: Vec<Value> = topics.iter().map(|(word, count)| json!({
        "topic": word,
        "frequency": count,
        "likelihood": (*count as f64 / agent_messages.len().max(1) as f64).min(1.0)
    })).collect();
    let current_valence = affect.map_or(0.0, |a| a.valence);
    let current_arousal = affect.map_or(0.0, |a| a.arousal);
    let emotional_trajectory = if affect_history.states.len() >= 2 {
        let recent = &affect_history.states[affect_history.states.len()-1];
        let prev = &affect_history.states[affect_history.states.len()-2];
        if recent.valence > prev.valence { "improving" } else if recent.valence < prev.valence { "declining" } else { "stable" }
    } else { "unknown" };
    let confidence = if agent_messages.len() > 20 { 0.7 } else if agent_messages.len() > 5 { 0.4 } else { 0.15 };
    Ok(ToolCallResult::json(&json!({
        "agent": agent,
        "channel_id": channel_id,
        "horizon_messages": horizon,
        "messages_analyzed": agent_messages.len(),
        "reply_ratio": reply_ratio,
        "predicted_topics": predicted_topics,
        "current_valence": current_valence,
        "current_arousal": current_arousal,
        "emotional_trajectory": emotional_trajectory,
        "confidence": confidence,
        "status": "anticipation_complete"
    })))
}

// ── 18. comm_anticipate_prepare ───────────────────────────────────────────
fn definition_anticipate_prepare() -> ToolDefinition {
    ToolDefinition {
        name: "comm_anticipate_prepare".into(),
        description: Some("Prepare proactive content based on anticipated needs".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "agent": {
                        "type": "string",
                        "description": "Agent to prepare content for"
                    },
                    "channel_id": {
                        "type": "integer",
                        "description": "Channel for the prepared content"
                    },
                    "prepared_content": {
                        "type": "string",
                        "description": "Content prepared in anticipation"
                    },
                    "trigger_condition": {
                        "type": "string",
                        "description": "Condition to trigger delivery: message_from, topic_mention, affect_shift",
                        "default": "message_from"
                    },
                    "preparer": {
                        "type": "string",
                        "description": "Agent preparing the content"
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
                    "agent",
                    "channel_id",
                    "prepared_content",
                    "preparer"
                ]
            }),
    }
}

fn handle_anticipate_prepare(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let prepared_content = get_str(&args, "prepared_content").ok_or("Missing prepared_content")?;
    let trigger_condition = get_str(&args, "trigger_condition").unwrap_or_else(|| "message_from".into());
    let preparer = get_str(&args, "preparer").ok_or("Missing preparer")?;
    let _ = session.store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let condition = format!("{}:{}", trigger_condition, agent);
    let result = session.store.schedule_message(
        channel_id, &preparer, &prepared_content,
        agentic_comm::types::TemporalTarget::Conditional { condition: condition.clone() },
        None,
    ).map_err(|e| format!("Failed to prepare: {e}"))?;
    let temporal_id = result.id;
    session.record_operation("comm_anticipate_prepare", Some(temporal_id));
    Ok(ToolCallResult::json(&json!({
        "temporal_id": temporal_id,
        "target_agent": agent,
        "channel_id": channel_id,
        "preparer": preparer,
        "trigger_condition": condition,
        "content_length": prepared_content.len(),
        "status": "anticipation_prepared"
    })))
}

// ── 19. comm_anticipate_calibrate ─────────────────────────────────────────
fn definition_anticipate_calibrate() -> ToolDefinition {
    ToolDefinition {
        name: "comm_anticipate_calibrate".into(),
        description: Some("Calibrate anticipatory understanding for an agent".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "agent": {
                        "type": "string",
                        "description": "Agent to calibrate anticipation for"
                    },
                    "feedback": {
                        "type": "string",
                        "description": "Feedback on last anticipation: accurate, missed, wrong"
                    },
                    "learning_rate": {
                        "type": "number",
                        "description": "Calibration learning rate (0.0-1.0)",
                        "default": 0.1
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
                    "agent",
                    "feedback"
                ]
            }),
    }
}

fn handle_anticipate_calibrate(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let feedback = get_str(&args, "feedback").ok_or("Missing feedback")?;
    let learning_rate = get_f64(&args, "learning_rate").unwrap_or(0.1).clamp(0.0, 1.0);
    if !["accurate", "missed", "wrong"].contains(&feedback.as_str()) {
        return Err("feedback must be one of: accurate, missed, wrong".into());
    }
    let store = &session.store;
    // Analyze current model quality based on affect history depth
    let history = store.get_affect_history(&agent);
    let data_points = history.states.len();
    let message_count = store.messages.values().filter(|m| m.sender == agent).count();
    let model_maturity = if message_count > 50 { "mature" } else if message_count > 10 { "developing" } else { "nascent" };
    let accuracy_adjustment = match feedback.as_str() {
        "accurate" => learning_rate,
        "missed" => -learning_rate * 0.5,
        "wrong" => -learning_rate,
        _ => 0.0,
    };
    session.record_operation("comm_anticipate_calibrate", None);
    Ok(ToolCallResult::json(&json!({
        "agent": agent,
        "feedback": feedback,
        "learning_rate": learning_rate,
        "accuracy_adjustment": accuracy_adjustment,
        "data_points": data_points,
        "message_count": message_count,
        "model_maturity": model_maturity,
        "calibrated": true,
        "status": "calibration_complete"
    })))
}

// ── 20. comm_anticipate_accuracy ──────────────────────────────────────────
fn definition_anticipate_accuracy() -> ToolDefinition {
    ToolDefinition {
        name: "comm_anticipate_accuracy".into(),
        description: Some("Check accuracy of anticipatory predictions for an agent".into()),
        input_schema: json!({
                "type": "object",
                "properties": {
                    "agent": {
                        "type": "string",
                        "description": "Agent to check anticipation accuracy for"
                    },
                    "window_messages": {
                        "type": "integer",
                        "description": "Number of recent messages to evaluate",
                        "default": 30
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
                    "agent"
                ]
            }),
    }
}

fn handle_anticipate_accuracy(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let window = get_u64(&args, "window_messages").unwrap_or(30) as usize;
    let store = &session.store;
    // Analyze pattern consistency: how predictable is this agent's behavior?
    let mut agent_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.sender == agent)
        .collect();
    agent_messages.sort_by_key(|m| m.timestamp);
    let recent = &agent_messages[agent_messages.len().saturating_sub(window)..];
    // Topic consistency: do they talk about the same things?
    let mut all_words: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for msg in recent {
        for word in msg.content.to_lowercase().split_whitespace() {
            let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.len() > 3 {
                *all_words.entry(clean).or_insert(0) += 1;
            }
        }
    }
    let total_unique = all_words.len();
    let total_words: usize = all_words.values().sum();
    let repetition_rate = if total_unique > 0 { total_words as f64 / total_unique as f64 } else { 0.0 };
    // Timing consistency: are intervals between messages consistent?
    let mut intervals: Vec<i64> = Vec::new();
    for w in recent.windows(2) {
        let diff = w[1].timestamp.signed_duration_since(w[0].timestamp).num_seconds();
        intervals.push(diff);
    }
    let avg_interval = if !intervals.is_empty() { intervals.iter().sum::<i64>() as f64 / intervals.len() as f64 } else { 0.0 };
    let interval_variance = if intervals.len() > 1 {
        let mean = avg_interval;
        intervals.iter().map(|&i| (i as f64 - mean).powi(2)).sum::<f64>() / intervals.len() as f64
    } else { 0.0 };
    let timing_predictability = (1.0 / (1.0 + (interval_variance / 10000.0).sqrt())).min(1.0);
    // Affect consistency
    let affect_history = store.get_affect_history(&agent);
    let affect_stability = if affect_history.states.len() >= 2 {
        let mut valence_diffs: Vec<f64> = Vec::new();
        for w in affect_history.states.windows(2) {
            valence_diffs.push((w[1].valence - w[0].valence).abs());
        }
        let avg_diff = valence_diffs.iter().sum::<f64>() / valence_diffs.len() as f64;
        (1.0 - avg_diff).max(0.0)
    } else { 0.5 };
    let overall_predictability = (repetition_rate.min(5.0) / 5.0 * 0.3 + timing_predictability * 0.3 + affect_stability * 0.4).min(1.0);
    let rating = if overall_predictability > 0.7 { "high" } else if overall_predictability > 0.4 { "medium" } else { "low" };
    Ok(ToolCallResult::json(&json!({
        "agent": agent,
        "messages_analyzed": recent.len(),
        "unique_words": total_unique,
        "repetition_rate": repetition_rate,
        "average_message_interval_seconds": avg_interval,
        "timing_predictability": timing_predictability,
        "affect_stability": affect_stability,
        "overall_predictability": overall_predictability,
        "accuracy_rating": rating,
        "status": "accuracy_assessed"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

pub fn all_definitions() -> Vec<ToolDefinition> {
    vec![
        definition_affect_contagion_model(),
        definition_affect_contagion_simulate(),
        definition_affect_contagion_immunize(),
        definition_affect_contagion_outbreak(),
        definition_affect_archaeology_dig(),
        definition_affect_archaeology_reconstruct(),
        definition_affect_archaeology_timeline(),
        definition_affect_archaeology_artifacts(),
        definition_affect_prophecy_predict(),
        definition_affect_prophecy_warn(),
        definition_affect_prophecy_track(),
        definition_affect_prophecy_similar(),
        definition_unspeakable_encode(),
        definition_unspeakable_decode(),
        definition_unspeakable_detect(),
        definition_unspeakable_translate(),
        definition_anticipate_predict(),
        definition_anticipate_prepare(),
        definition_anticipate_calibrate(),
        definition_anticipate_accuracy(),
    ]
}

pub fn try_execute(
    name: &str,
    args: Value,
    session: &mut SessionManager,
) -> Option<Result<ToolCallResult, String>> {
    match name {
        "comm_affect_contagion_model" => Some(handle_affect_contagion_model(args, session)),
        "comm_affect_contagion_simulate" => Some(handle_affect_contagion_simulate(args, session)),
        "comm_affect_contagion_immunize" => Some(handle_affect_contagion_immunize(args, session)),
        "comm_affect_contagion_outbreak" => Some(handle_affect_contagion_outbreak(args, session)),
        "comm_affect_archaeology_dig" => Some(handle_affect_archaeology_dig(args, session)),
        "comm_affect_archaeology_reconstruct" => Some(handle_affect_archaeology_reconstruct(args, session)),
        "comm_affect_archaeology_timeline" => Some(handle_affect_archaeology_timeline(args, session)),
        "comm_affect_archaeology_artifacts" => Some(handle_affect_archaeology_artifacts(args, session)),
        "comm_affect_prophecy_predict" => Some(handle_affect_prophecy_predict(args, session)),
        "comm_affect_prophecy_warn" => Some(handle_affect_prophecy_warn(args, session)),
        "comm_affect_prophecy_track" => Some(handle_affect_prophecy_track(args, session)),
        "comm_affect_prophecy_similar" => Some(handle_affect_prophecy_similar(args, session)),
        "comm_unspeakable_encode" => Some(handle_unspeakable_encode(args, session)),
        "comm_unspeakable_decode" => Some(handle_unspeakable_decode(args, session)),
        "comm_unspeakable_detect" => Some(handle_unspeakable_detect(args, session)),
        "comm_unspeakable_translate" => Some(handle_unspeakable_translate(args, session)),
        "comm_anticipate_predict" => Some(handle_anticipate_predict(args, session)),
        "comm_anticipate_prepare" => Some(handle_anticipate_prepare(args, session)),
        "comm_anticipate_calibrate" => Some(handle_anticipate_calibrate(args, session)),
        "comm_anticipate_accuracy" => Some(handle_anticipate_accuracy(args, session)),
        _ => None,
    }
}
