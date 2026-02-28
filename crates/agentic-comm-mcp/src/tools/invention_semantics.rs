//! Invention modules: Semantic Grafting, Echo Chambers, Ghost Conversations,
//! Metamessages — 16 tools for the SEMANTICS category of the Comm Inventions.

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
// 1. Semantic Grafting (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 1. comm_semantic_graft ───────────────────────────────────────────────
fn definition_semantic_graft() -> ToolDefinition {
    ToolDefinition {
        name: "comm_semantic_graft".into(),
        description: Some("Graft a semantic fragment onto a message".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message_id": { "type": "integer", "description": "Message to graft onto" },
                "fragment": { "type": "string", "description": "Semantic fragment to graft" },
                "grafter": { "type": "string", "description": "Identity of the grafting agent" }
            },
            "required": ["message_id", "fragment", "grafter"]
        }),
    }
}

fn handle_semantic_graft(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let fragment = get_str(&args, "fragment").ok_or("Missing fragment")?;
    let grafter = get_str(&args, "grafter").ok_or("Missing grafter")?;
    let store = &session.store;
    let msg = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    // Use graft_semantic on the store
    let channel_id = msg.channel_id;
    match session.store.graft_semantic(message_id, message_id, &fragment) {
        Ok(op) => {
            session.record_operation("comm_semantic_graft", Some(message_id));
            Ok(ToolCallResult::json(&json!({
                "message_id": message_id,
                "channel_id": channel_id,
                "semantic_operation_id": op.id,
                "fragment_length": fragment.len(),
                "grafter": grafter,
                "status": "grafted"
            })))
        }
        Err(e) => Ok(ToolCallResult::error(format!("Graft failed: {e}"))),
    }
}

// ── 2. comm_semantic_extract ─────────────────────────────────────────────
fn definition_semantic_extract() -> ToolDefinition {
    ToolDefinition {
        name: "comm_semantic_extract".into(),
        description: Some("Extract semantic fragments from a message".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message_id": { "type": "integer", "description": "Message to extract from" }
            },
            "required": ["message_id"]
        }),
    }
}

fn handle_semantic_extract(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let store = &session.store;
    match store.extract_semantic(message_id) {
        Ok(op) => {
            Ok(ToolCallResult::json(&json!({
                "message_id": message_id,
                "semantic_operation_id": op.id,
                "topic": op.topic,
                "focus_nodes": op.focus_nodes,
                "depth": op.depth,
                "status": "extracted"
            })))
        }
        Err(e) => Ok(ToolCallResult::error(format!("Extract failed: {e}"))),
    }
}

// ── 3. comm_semantic_merge ───────────────────────────────────────────────
fn definition_semantic_merge() -> ToolDefinition {
    ToolDefinition {
        name: "comm_semantic_merge".into(),
        description: Some("Merge semantic fragments from multiple messages".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message_ids": {
                    "type": "array",
                    "items": { "type": "integer" },
                    "description": "Messages to merge semantics from"
                }
            },
            "required": ["message_ids"]
        }),
    }
}

fn handle_semantic_merge(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_ids: Vec<u64> = args.get("message_ids")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or("Missing or invalid message_ids")?;
    if message_ids.is_empty() {
        return Err("message_ids cannot be empty".into());
    }
    let store = &session.store;
    let mut merged_topics: Vec<String> = Vec::new();
    let mut merged_nodes: Vec<String> = Vec::new();
    let mut total_depth: u64 = 0;
    let mut found_count = 0u32;
    for &mid in &message_ids {
        if let Ok(op) = store.extract_semantic(mid) {
            merged_topics.push(op.topic);
            merged_nodes.extend(op.focus_nodes);
            total_depth += op.depth;
            found_count += 1;
        }
    }
    // Deduplicate focus nodes
    merged_nodes.sort();
    merged_nodes.dedup();
    Ok(ToolCallResult::json(&json!({
        "message_ids": message_ids,
        "messages_found": found_count,
        "merged_topics": merged_topics,
        "merged_focus_nodes": merged_nodes,
        "total_depth": total_depth,
        "average_depth": if found_count > 0 { total_depth as f64 / found_count as f64 } else { 0.0 },
        "status": "merge_complete"
    })))
}

// ── 4. comm_semantic_cluster ─────────────────────────────────────────────
fn definition_semantic_cluster() -> ToolDefinition {
    ToolDefinition {
        name: "comm_semantic_cluster".into(),
        description: Some("Cluster messages by semantic similarity".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to cluster messages from" },
                "max_clusters": { "type": "integer", "description": "Maximum number of clusters", "default": 5 }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_semantic_cluster(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let max_clusters = get_u64(&args, "max_clusters").unwrap_or(5) as usize;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    // Simple word-based clustering: group by most common words
    let mut word_counts: std::collections::HashMap<String, Vec<u64>> = std::collections::HashMap::new();
    for msg in &channel_messages {
        for word in msg.content.to_lowercase().split_whitespace() {
            let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.len() > 3 {
                word_counts.entry(clean).or_default().push(msg.id);
            }
        }
    }
    // Pick the top N words as cluster centers
    let mut clusters: Vec<(&String, &Vec<u64>)> = word_counts.iter().collect();
    clusters.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    clusters.truncate(max_clusters);
    let result_clusters: Vec<Value> = clusters.iter().map(|(word, ids)| json!({
        "keyword": word,
        "message_count": ids.len(),
        "message_ids": &ids[..ids.len().min(10)]
    })).collect();
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "total_messages": channel_messages.len(),
        "cluster_count": result_clusters.len(),
        "clusters": result_clusters,
        "status": "clustering_complete"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Echo Chambers (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 5. comm_echo_chamber_detect ──────────────────────────────────────────
fn definition_echo_chamber_detect() -> ToolDefinition {
    ToolDefinition {
        name: "comm_echo_chamber_detect".into(),
        description: Some("Detect echo chamber patterns in a channel".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to analyze" }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_echo_chamber_detect(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    // Detect echo patterns: high forwarding, repeated content, low diversity
    let mut sender_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    let mut content_hashes: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    for msg in &channel_messages {
        *sender_counts.entry(msg.sender.as_str()).or_insert(0) += 1;
        // Simple content hash (sum of char values)
        let hash: u64 = msg.content.bytes().map(|b| b as u64).sum();
        *content_hashes.entry(hash).or_insert(0) += 1;
    }
    let unique_senders = sender_counts.len();
    let repeated_content = content_hashes.values().filter(|&&c| c > 1).count();
    let total = channel_messages.len();
    let diversity_score = if total > 0 { unique_senders as f64 / total as f64 } else { 1.0 };
    let echo_score = if total > 0 { repeated_content as f64 / total as f64 } else { 0.0 };
    let is_echo_chamber = diversity_score < 0.3 || echo_score > 0.4;
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "total_messages": total,
        "unique_senders": unique_senders,
        "repeated_content_groups": repeated_content,
        "diversity_score": diversity_score,
        "echo_score": echo_score,
        "is_echo_chamber": is_echo_chamber,
        "status": "detection_complete"
    })))
}

// ── 6. comm_echo_chamber_analyze ─────────────────────────────────────────
fn definition_echo_chamber_analyze() -> ToolDefinition {
    ToolDefinition {
        name: "comm_echo_chamber_analyze".into(),
        description: Some("Analyze echo chamber dynamics".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to analyze" },
                "depth": { "type": "integer", "description": "Analysis depth (message count)", "default": 100 }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_echo_chamber_analyze(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let depth = get_u64(&args, "depth").unwrap_or(100) as usize;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let mut channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    channel_messages.sort_by_key(|m| m.timestamp);
    let recent: Vec<&agentic_comm::Message> = channel_messages.into_iter().rev().take(depth).collect();
    // Analyze forwarding chains within this channel
    let forwarded_count = recent.iter()
        .filter(|m| store.get_echo_depth(m.id) > 0)
        .count();
    // Analyze reply patterns
    let reply_count = recent.iter().filter(|m| m.reply_to.is_some()).count();
    let self_reply_count = recent.iter()
        .filter(|m| {
            m.reply_to.and_then(|rid| store.get_message(rid))
                .map_or(false, |parent| parent.sender == m.sender)
        })
        .count();
    let amplification_ratio = if recent.is_empty() { 0.0 } else { forwarded_count as f64 / recent.len() as f64 };
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "messages_analyzed": recent.len(),
        "forwarded_messages": forwarded_count,
        "reply_messages": reply_count,
        "self_replies": self_reply_count,
        "amplification_ratio": amplification_ratio,
        "feedback_loop_detected": self_reply_count > recent.len() / 4,
        "status": "analysis_complete"
    })))
}

// ── 7. comm_echo_chamber_break ───────────────────────────────────────────
fn definition_echo_chamber_break() -> ToolDefinition {
    ToolDefinition {
        name: "comm_echo_chamber_break".into(),
        description: Some("Suggest interventions to break echo chambers".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel with echo chamber" },
                "strategy": { "type": "string", "description": "Strategy: diversify, challenge, bridge", "default": "diversify" }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_echo_chamber_break(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let strategy = get_str(&args, "strategy").unwrap_or_else(|| "diversify".to_string());
    let store = &session.store;
    let channel = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    let current_participants: std::collections::HashSet<&str> = channel_messages.iter()
        .map(|m| m.sender.as_str())
        .collect();
    // Find agents active in other channels but not this one
    let other_agents: Vec<&str> = store.messages.values()
        .filter(|m| m.channel_id != channel_id)
        .map(|m| m.sender.as_str())
        .filter(|s| !current_participants.contains(s))
        .collect::<std::collections::HashSet<&str>>()
        .into_iter()
        .take(5)
        .collect();
    let interventions: Vec<Value> = match strategy.as_str() {
        "diversify" => vec![
            json!({"action": "invite_external_agents", "agents": other_agents, "reason": "Increase sender diversity"}),
            json!({"action": "introduce_new_topics", "reason": "Break content repetition patterns"}),
        ],
        "challenge" => vec![
            json!({"action": "inject_counter_perspective", "reason": "Challenge dominant narrative"}),
            json!({"action": "highlight_assumptions", "reason": "Surface implicit biases"}),
        ],
        "bridge" => vec![
            json!({"action": "create_bridge_channel", "participants": other_agents, "reason": "Connect isolated groups"}),
            json!({"action": "cross_pollinate", "reason": "Share insights across channels"}),
        ],
        _ => vec![json!({"action": "custom", "strategy": strategy, "reason": "User-defined intervention"})],
    };
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "channel_name": channel.name,
        "strategy": strategy,
        "current_participants": current_participants.len(),
        "potential_new_agents": other_agents.len(),
        "interventions": interventions,
        "status": "interventions_suggested"
    })))
}

// ── 8. comm_echo_chamber_health ──────────────────────────────────────────
fn definition_echo_chamber_health() -> ToolDefinition {
    ToolDefinition {
        name: "comm_echo_chamber_health".into(),
        description: Some("Get echo chamber health score".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to assess" }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_echo_chamber_health(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    let total = channel_messages.len();
    let unique_senders: std::collections::HashSet<&str> = channel_messages.iter()
        .map(|m| m.sender.as_str())
        .collect();
    let unique_threads: std::collections::HashSet<Option<&str>> = channel_messages.iter()
        .map(|m| m.thread_id.as_deref())
        .collect();
    let diversity = if total > 0 { unique_senders.len() as f64 / total as f64 } else { 1.0 };
    let topic_variety = if total > 0 { unique_threads.len() as f64 / total.min(20) as f64 } else { 1.0 };
    // Forwarding ratio
    let forwarded = channel_messages.iter().filter(|m| store.get_echo_depth(m.id) > 0).count();
    let forwarding_ratio = if total > 0 { forwarded as f64 / total as f64 } else { 0.0 };
    // Health = high diversity + high topic variety + low forwarding
    let health = ((diversity * 0.4 + topic_variety.min(1.0) * 0.3 + (1.0 - forwarding_ratio) * 0.3) * 100.0).round() / 100.0;
    let rating = if health > 0.7 { "healthy" } else if health > 0.4 { "moderate" } else { "unhealthy" };
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "total_messages": total,
        "unique_senders": unique_senders.len(),
        "unique_threads": unique_threads.len(),
        "diversity_score": diversity,
        "topic_variety": topic_variety.min(1.0),
        "forwarding_ratio": forwarding_ratio,
        "health_score": health,
        "rating": rating,
        "status": "health_assessed"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Ghost Conversations (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 9. comm_ghost_create ─────────────────────────────────────────────────
fn definition_ghost_create() -> ToolDefinition {
    ToolDefinition {
        name: "comm_ghost_create".into(),
        description: Some("Create a ghost conversation invisible to non-participants".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "participants": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Agents who can see this ghost conversation"
                },
                "topic": { "type": "string", "description": "Ghost conversation topic" }
            },
            "required": ["participants", "topic"]
        }),
    }
}

fn handle_ghost_create(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let participants: Vec<String> = args.get("participants")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or("Missing or invalid participants")?;
    let topic = get_str(&args, "topic").ok_or("Missing topic")?;
    if participants.is_empty() {
        return Err("participants cannot be empty".into());
    }
    // Create a channel with ghost metadata
    let config = agentic_comm::ChannelConfig {
        ttl_seconds: 86400,
        ..Default::default()
    };
    let result = session.store.create_channel(
        &format!("ghost:{topic}"),
        agentic_comm::ChannelType::Direct,
        Some(config),
    );
    match result {
        Ok(channel) => {
            // Add participants to the ghost channel
            for p in &participants {
                let _ = session.store.join_channel(channel.id, p);
            }
            session.record_operation("comm_ghost_create", Some(channel.id));
            Ok(ToolCallResult::json(&json!({
                "ghost_channel_id": channel.id,
                "topic": topic,
                "participants": participants,
                "visibility": "ghost",
                "ttl_seconds": 86400,
                "status": "ghost_created"
            })))
        }
        Err(e) => Ok(ToolCallResult::error(format!("Failed to create ghost: {e}"))),
    }
}

// ── 10. comm_ghost_reveal ────────────────────────────────────────────────
fn definition_ghost_reveal() -> ToolDefinition {
    ToolDefinition {
        name: "comm_ghost_reveal".into(),
        description: Some("Reveal a ghost conversation to specified agents".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Ghost channel to reveal" },
                "reveal_to": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Agents to reveal the ghost conversation to"
                }
            },
            "required": ["channel_id", "reveal_to"]
        }),
    }
}

fn handle_ghost_reveal(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let reveal_to: Vec<String> = args.get("reveal_to")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or("Missing or invalid reveal_to")?;
    let channel = session.store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    if !channel.name.starts_with("ghost:") {
        return Err(format!("Channel {channel_id} is not a ghost conversation"));
    }
    // Add agents to the channel
    let mut added = Vec::new();
    for agent in &reveal_to {
        if session.store.join_channel(channel_id, agent).is_ok() {
            added.push(agent.clone());
        }
    }
    session.record_operation("comm_ghost_reveal", Some(channel_id));
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "revealed_to": added,
        "total_requested": reveal_to.len(),
        "total_added": added.len(),
        "status": "ghost_revealed"
    })))
}

// ── 11. comm_ghost_list ──────────────────────────────────────────────────
fn definition_ghost_list() -> ToolDefinition {
    ToolDefinition {
        name: "comm_ghost_list".into(),
        description: Some("List all ghost conversations".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent": { "type": "string", "description": "Optional: filter by participant agent" }
            }
        }),
    }
}

fn handle_ghost_list(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent_filter = get_str(&args, "agent");
    let store = &session.store;
    let ghosts: Vec<Value> = store.list_channels().iter()
        .filter(|ch| ch.name.starts_with("ghost:"))
        .filter(|ch| agent_filter.as_ref().map_or(true, |a| ch.participants.contains(a)))
        .map(|ch| json!({
            "channel_id": ch.id,
            "topic": ch.name.strip_prefix("ghost:").unwrap_or(&ch.name),
            "participants": ch.participants,
            "created_at": ch.created_at.to_rfc3339()
        }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "ghost_count": ghosts.len(),
        "filter_agent": agent_filter,
        "ghosts": ghosts,
        "status": "list_complete"
    })))
}

// ── 12. comm_ghost_dissolve ──────────────────────────────────────────────
fn definition_ghost_dissolve() -> ToolDefinition {
    ToolDefinition {
        name: "comm_ghost_dissolve".into(),
        description: Some("Dissolve a ghost conversation".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Ghost channel to dissolve" }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_ghost_dissolve(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let channel = session.store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    if !channel.name.starts_with("ghost:") {
        return Err(format!("Channel {channel_id} is not a ghost conversation"));
    }
    let topic = channel.name.strip_prefix("ghost:").unwrap_or(&channel.name).to_string();
    let participant_count = channel.participants.len();
    // Close the ghost channel
    match session.store.close_channel(channel_id) {
        Ok(()) => {
            session.record_operation("comm_ghost_dissolve", Some(channel_id));
            Ok(ToolCallResult::json(&json!({
                "channel_id": channel_id,
                "topic": topic,
                "participants_removed": participant_count,
                "status": "ghost_dissolved"
            })))
        }
        Err(e) => Ok(ToolCallResult::error(format!("Failed to dissolve ghost: {e}"))),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Metamessages (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 13. comm_metamessage_encode ──────────────────────────────────────────
fn definition_metamessage_encode() -> ToolDefinition {
    ToolDefinition {
        name: "comm_metamessage_encode".into(),
        description: Some("Encode a metamessage (message about messages)".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "target_message_id": { "type": "integer", "description": "Message this metamessage is about" },
                "meta_type": { "type": "string", "description": "Metamessage type: annotation, correction, context, sentiment" },
                "content": { "type": "string", "description": "Metamessage content" },
                "author": { "type": "string", "description": "Metamessage author" }
            },
            "required": ["target_message_id", "meta_type", "content", "author"]
        }),
    }
}

fn handle_metamessage_encode(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let target_id = get_u64(&args, "target_message_id").ok_or("Missing target_message_id")?;
    let meta_type = get_str(&args, "meta_type").ok_or("Missing meta_type")?;
    let content = get_str(&args, "content").ok_or("Missing content")?;
    let author = get_str(&args, "author").ok_or("Missing author")?;
    let store = &session.store;
    let msg = store.get_message(target_id)
        .ok_or_else(|| format!("Target message {target_id} not found"))?;
    let channel_id = msg.channel_id;
    // Encode metamessage as a reply with special metadata
    let meta_content = format!("[meta:{meta_type}] {content}");
    match session.store.send_reply(channel_id, target_id, &author, &meta_content, agentic_comm::MessageType::Text) {
        Ok(reply) => {
            session.record_operation("comm_metamessage_encode", Some(reply.id));
            Ok(ToolCallResult::json(&json!({
                "metamessage_id": reply.id,
                "target_message_id": target_id,
                "channel_id": channel_id,
                "meta_type": meta_type,
                "author": author,
                "status": "metamessage_encoded"
            })))
        }
        Err(e) => Ok(ToolCallResult::error(format!("Failed to encode metamessage: {e}"))),
    }
}

// ── 14. comm_metamessage_decode ──────────────────────────────────────────
fn definition_metamessage_decode() -> ToolDefinition {
    ToolDefinition {
        name: "comm_metamessage_decode".into(),
        description: Some("Decode metamessages from a conversation".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message_id": { "type": "integer", "description": "Message to decode metamessages for" }
            },
            "required": ["message_id"]
        }),
    }
}

fn handle_metamessage_decode(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let store = &session.store;
    let _ = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    // Find replies that are metamessages (start with [meta:...])
    let replies = store.get_replies(message_id);
    let metamessages: Vec<Value> = replies.iter()
        .filter(|r| r.content.starts_with("[meta:"))
        .map(|r| {
            let meta_type = r.content.strip_prefix("[meta:")
                .and_then(|s| s.split(']').next())
                .unwrap_or("unknown");
            let meta_content = r.content.split(']')
                .nth(1)
                .unwrap_or("")
                .trim();
            json!({
                "metamessage_id": r.id,
                "meta_type": meta_type,
                "content": meta_content,
                "author": r.sender,
                "timestamp": r.timestamp.to_rfc3339()
            })
        })
        .collect();
    Ok(ToolCallResult::json(&json!({
        "message_id": message_id,
        "total_replies": replies.len(),
        "metamessage_count": metamessages.len(),
        "metamessages": metamessages,
        "status": "decode_complete"
    })))
}

// ── 15. comm_metamessage_trace ───────────────────────────────────────────
fn definition_metamessage_trace() -> ToolDefinition {
    ToolDefinition {
        name: "comm_metamessage_trace".into(),
        description: Some("Trace metamessage patterns".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to trace metamessage patterns in" },
                "meta_type": { "type": "string", "description": "Optional: filter by metamessage type" }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_metamessage_trace(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let meta_type_filter = get_str(&args, "meta_type");
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    // Find all metamessages in this channel
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id && m.content.starts_with("[meta:"))
        .collect();
    let traces: Vec<Value> = channel_messages.iter()
        .filter(|m| {
            meta_type_filter.as_ref().map_or(true, |ft| {
                m.content.strip_prefix("[meta:")
                    .and_then(|s| s.split(']').next())
                    .map_or(false, |mt| mt == ft.as_str())
            })
        })
        .map(|m| {
            let meta_type = m.content.strip_prefix("[meta:")
                .and_then(|s| s.split(']').next())
                .unwrap_or("unknown");
            json!({
                "message_id": m.id,
                "meta_type": meta_type,
                "sender": m.sender,
                "target_message_id": m.reply_to,
                "timestamp": m.timestamp.to_rfc3339()
            })
        })
        .collect();
    // Count by type
    let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for trace in &traces {
        let mt = trace.get("meta_type").and_then(|v| v.as_str()).unwrap_or("unknown");
        *type_counts.entry(mt.to_string()).or_insert(0) += 1;
    }
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "total_metamessages": traces.len(),
        "type_distribution": type_counts,
        "filter": meta_type_filter,
        "traces": traces,
        "status": "trace_complete"
    })))
}

// ── 16. comm_metamessage_inject ──────────────────────────────────────────
fn definition_metamessage_inject() -> ToolDefinition {
    ToolDefinition {
        name: "comm_metamessage_inject".into(),
        description: Some("Inject metamessage context into a conversation".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to inject into" },
                "context": { "type": "string", "description": "Context to inject" },
                "meta_type": { "type": "string", "description": "Type of context: background, correction, clarification", "default": "context" },
                "injector": { "type": "string", "description": "Agent injecting the context" }
            },
            "required": ["channel_id", "context", "injector"]
        }),
    }
}

fn handle_metamessage_inject(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let context = get_str(&args, "context").ok_or("Missing context")?;
    let meta_type = get_str(&args, "meta_type").unwrap_or_else(|| "context".to_string());
    let injector = get_str(&args, "injector").ok_or("Missing injector")?;
    let _ = session.store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let meta_content = format!("[meta:{meta_type}] {context}");
    match session.store.send_message(channel_id, &injector, &meta_content, agentic_comm::MessageType::Text) {
        Ok(msg) => {
            session.record_operation("comm_metamessage_inject", Some(msg.id));
            Ok(ToolCallResult::json(&json!({
                "message_id": msg.id,
                "channel_id": channel_id,
                "meta_type": meta_type,
                "injector": injector,
                "context_length": context.len(),
                "status": "context_injected"
            })))
        }
        Err(e) => Ok(ToolCallResult::error(format!("Failed to inject context: {e}"))),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

pub fn all_definitions() -> Vec<ToolDefinition> {
    vec![
        definition_semantic_graft(),
        definition_semantic_extract(),
        definition_semantic_merge(),
        definition_semantic_cluster(),
        definition_echo_chamber_detect(),
        definition_echo_chamber_analyze(),
        definition_echo_chamber_break(),
        definition_echo_chamber_health(),
        definition_ghost_create(),
        definition_ghost_reveal(),
        definition_ghost_list(),
        definition_ghost_dissolve(),
        definition_metamessage_encode(),
        definition_metamessage_decode(),
        definition_metamessage_trace(),
        definition_metamessage_inject(),
    ]
}

pub fn try_execute(
    name: &str,
    args: Value,
    session: &mut SessionManager,
) -> Option<Result<ToolCallResult, String>> {
    match name {
        "comm_semantic_graft" => Some(handle_semantic_graft(args, session)),
        "comm_semantic_extract" => Some(handle_semantic_extract(args, session)),
        "comm_semantic_merge" => Some(handle_semantic_merge(args, session)),
        "comm_semantic_cluster" => Some(handle_semantic_cluster(args, session)),
        "comm_echo_chamber_detect" => Some(handle_echo_chamber_detect(args, session)),
        "comm_echo_chamber_analyze" => Some(handle_echo_chamber_analyze(args, session)),
        "comm_echo_chamber_break" => Some(handle_echo_chamber_break(args, session)),
        "comm_echo_chamber_health" => Some(handle_echo_chamber_health(args, session)),
        "comm_ghost_create" => Some(handle_ghost_create(args, session)),
        "comm_ghost_reveal" => Some(handle_ghost_reveal(args, session)),
        "comm_ghost_list" => Some(handle_ghost_list(args, session)),
        "comm_ghost_dissolve" => Some(handle_ghost_dissolve(args, session)),
        "comm_metamessage_encode" => Some(handle_metamessage_encode(args, session)),
        "comm_metamessage_decode" => Some(handle_metamessage_decode(args, session)),
        "comm_metamessage_trace" => Some(handle_metamessage_trace(args, session)),
        "comm_metamessage_inject" => Some(handle_metamessage_inject(args, session)),
        _ => None,
    }
}
