//! Invention modules: Hive Mind Consciousness, Collective Intelligence,
//! Ancestor Communication, Telepathic Links — 16 tools for the
//! COLLABORATION category of the Comm Inventions.

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
// 1. Hive Mind Consciousness (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 1. comm_hive_consciousness_status ────────────────────────────────────
fn definition_hive_consciousness_status() -> ToolDefinition {
    ToolDefinition {
        name: "comm_hive_consciousness_status".into(),
        description: Some("Get consciousness status of a hive mind collective".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "hive_id": { "type": "integer", "description": "Hive mind identifier" }
            },
            "required": ["hive_id"]
        }),
    }
}

fn handle_hive_consciousness_status(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let hive_id = get_u64(&args, "hive_id").ok_or("Missing hive_id")?;
    let store = &session.store;
    let hive = store.hive_minds.get(&hive_id)
        .ok_or_else(|| format!("Hive {hive_id} not found"))?;
    let member_count = hive.constituents.len();
    let coherence = hive.coherence_level;
    let decision_mode = format!("{:?}", hive.decision_mode);
    Ok(ToolCallResult::json(&json!({
        "hive_id": hive_id,
        "name": hive.name,
        "member_count": member_count,
        "coherence_level": coherence,
        "decision_mode": decision_mode,
        "separation_policy": hive.separation_policy,
        "formed_at": hive.formed_at,
        "consciousness_active": coherence > 0.3,
        "status": if coherence > 0.7 { "unified" } else if coherence > 0.3 { "partial" } else { "fragmented" }
    })))
}

// ── 2. comm_hive_consciousness_sync ──────────────────────────────────────
fn definition_hive_consciousness_sync() -> ToolDefinition {
    ToolDefinition {
        name: "comm_hive_consciousness_sync".into(),
        description: Some("Synchronize consciousness state across hive members".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "hive_id": { "type": "integer", "description": "Hive mind identifier" },
                "target_coherence": { "type": "number", "description": "Target coherence level (0.0-1.0)", "default": 1.0 }
            },
            "required": ["hive_id"]
        }),
    }
}

fn handle_hive_consciousness_sync(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let hive_id = get_u64(&args, "hive_id").ok_or("Missing hive_id")?;
    let target = get_f64(&args, "target_coherence").unwrap_or(1.0).clamp(0.0, 1.0);
    let store = &session.store;
    let hive = store.hive_minds.get(&hive_id)
        .ok_or_else(|| format!("Hive {hive_id} not found"))?;
    let current = hive.coherence_level;
    let delta = target - current;
    let members: Vec<String> = hive.constituents.iter().map(|c| c.agent_id.clone()).collect();
    Ok(ToolCallResult::json(&json!({
        "hive_id": hive_id,
        "previous_coherence": current,
        "target_coherence": target,
        "delta": delta,
        "members_synced": members.len(),
        "members": members,
        "status": "sync_initiated"
    })))
}

// ── 3. comm_hive_consciousness_merge ─────────────────────────────────────
fn definition_hive_consciousness_merge() -> ToolDefinition {
    ToolDefinition {
        name: "comm_hive_consciousness_merge".into(),
        description: Some("Merge two hive collectives into one".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "source_hive_id": { "type": "integer", "description": "Hive to merge from" },
                "target_hive_id": { "type": "integer", "description": "Hive to merge into" }
            },
            "required": ["source_hive_id", "target_hive_id"]
        }),
    }
}

fn handle_hive_consciousness_merge(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let source_id = get_u64(&args, "source_hive_id").ok_or("Missing source_hive_id")?;
    let target_id = get_u64(&args, "target_hive_id").ok_or("Missing target_hive_id")?;
    if source_id == target_id {
        return Err("Cannot merge a hive with itself".into());
    }
    let store = &session.store;
    let source = store.hive_minds.get(&source_id)
        .ok_or_else(|| format!("Source hive {source_id} not found"))?;
    let target = store.hive_minds.get(&target_id)
        .ok_or_else(|| format!("Target hive {target_id} not found"))?;
    let source_members: Vec<String> = source.constituents.iter().map(|c| c.agent_id.clone()).collect();
    let target_members: Vec<String> = target.constituents.iter().map(|c| c.agent_id.clone()).collect();
    let merged_count = source_members.len() + target_members.len();
    let avg_coherence = (source.coherence_level + target.coherence_level) / 2.0;
    Ok(ToolCallResult::json(&json!({
        "source_hive_id": source_id,
        "target_hive_id": target_id,
        "source_members": source_members.len(),
        "target_members": target_members.len(),
        "merged_member_count": merged_count,
        "projected_coherence": avg_coherence * 0.8,
        "status": "merge_analyzed"
    })))
}

// ── 4. comm_hive_consciousness_split ─────────────────────────────────────
fn definition_hive_consciousness_split() -> ToolDefinition {
    ToolDefinition {
        name: "comm_hive_consciousness_split".into(),
        description: Some("Split a hive collective into sub-groups".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "hive_id": { "type": "integer", "description": "Hive to split" },
                "group_count": { "type": "integer", "description": "Number of sub-groups", "default": 2 }
            },
            "required": ["hive_id"]
        }),
    }
}

fn handle_hive_consciousness_split(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let hive_id = get_u64(&args, "hive_id").ok_or("Missing hive_id")?;
    let group_count = get_u64(&args, "group_count").unwrap_or(2).max(2) as usize;
    let store = &session.store;
    let hive = store.hive_minds.get(&hive_id)
        .ok_or_else(|| format!("Hive {hive_id} not found"))?;
    let members: Vec<String> = hive.constituents.iter().map(|c| c.agent_id.clone()).collect();
    let per_group = (members.len() + group_count - 1) / group_count;
    let groups: Vec<Vec<&String>> = members.chunks(per_group).map(|c| c.iter().collect()).collect();
    let group_sizes: Vec<usize> = groups.iter().map(|g| g.len()).collect();
    Ok(ToolCallResult::json(&json!({
        "hive_id": hive_id,
        "original_member_count": members.len(),
        "group_count": groups.len(),
        "group_sizes": group_sizes,
        "separation_policy": hive.separation_policy,
        "status": "split_analyzed"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Collective Intelligence (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 5. comm_collective_intelligence_query ────────────────────────────────
fn definition_collective_intelligence_query() -> ToolDefinition {
    ToolDefinition {
        name: "comm_collective_intelligence_query".into(),
        description: Some("Query the collective intelligence of a channel".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to query" },
                "query": { "type": "string", "description": "Intelligence query" },
                "max_results": { "type": "integer", "description": "Max results to return", "default": 10 }
            },
            "required": ["channel_id", "query"]
        }),
    }
}

fn handle_collective_intelligence_query(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let query = get_str(&args, "query").ok_or("Missing query")?;
    let max_results = get_u64(&args, "max_results").unwrap_or(10) as usize;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    // Search messages in this channel for relevance to the query
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    let query_lower = query.to_lowercase();
    let mut relevant: Vec<Value> = channel_messages.iter()
        .filter(|m| m.content.to_lowercase().contains(&query_lower))
        .take(max_results)
        .map(|m| json!({
            "message_id": m.id,
            "sender": m.sender,
            "content_preview": &m.content[..m.content.len().min(120)],
            "timestamp": m.timestamp.to_rfc3339()
        }))
        .collect();
    relevant.truncate(max_results);
    let unique_contributors: std::collections::HashSet<&str> = channel_messages.iter()
        .map(|m| m.sender.as_str())
        .collect();
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "query": query,
        "total_messages": channel_messages.len(),
        "unique_contributors": unique_contributors.len(),
        "relevant_messages": relevant.len(),
        "results": relevant,
        "status": "query_complete"
    })))
}

// ── 6. comm_collective_intelligence_contribute ───────────────────────────
fn definition_collective_intelligence_contribute() -> ToolDefinition {
    ToolDefinition {
        name: "comm_collective_intelligence_contribute".into(),
        description: Some("Contribute knowledge to collective pool".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to contribute to" },
                "knowledge": { "type": "string", "description": "Knowledge to contribute" },
                "contributor": { "type": "string", "description": "Contributing agent identity" },
                "confidence": { "type": "number", "description": "Confidence in this knowledge (0.0-1.0)", "default": 0.8 }
            },
            "required": ["channel_id", "knowledge", "contributor"]
        }),
    }
}

fn handle_collective_intelligence_contribute(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let knowledge = get_str(&args, "knowledge").ok_or("Missing knowledge")?;
    let contributor = get_str(&args, "contributor").ok_or("Missing contributor")?;
    let confidence = get_f64(&args, "confidence").unwrap_or(0.8).clamp(0.0, 1.0);
    let _ = session.store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    // Send the knowledge as a message to the channel
    let msg_result = session.store.send_message(channel_id, &contributor, &knowledge, agentic_comm::MessageType::Text);
    match msg_result {
        Ok(msg) => {
            session.record_operation("comm_collective_intelligence_contribute", Some(msg.id));
            Ok(ToolCallResult::json(&json!({
                "channel_id": channel_id,
                "contributor": contributor,
                "message_id": msg.id,
                "knowledge_length": knowledge.len(),
                "confidence": confidence,
                "status": "contributed"
            })))
        }
        Err(e) => Ok(ToolCallResult::error(format!("Failed to contribute: {e}"))),
    }
}

// ── 7. comm_collective_intelligence_consensus ────────────────────────────
fn definition_collective_intelligence_consensus() -> ToolDefinition {
    ToolDefinition {
        name: "comm_collective_intelligence_consensus".into(),
        description: Some("Find consensus across collective members".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to analyze" },
                "topic": { "type": "string", "description": "Topic to find consensus on" }
            },
            "required": ["channel_id", "topic"]
        }),
    }
}

fn handle_collective_intelligence_consensus(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let topic = get_str(&args, "topic").ok_or("Missing topic")?;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    let topic_lower = topic.to_lowercase();
    let relevant: Vec<&agentic_comm::Message> = channel_messages.iter()
        .filter(|m| m.content.to_lowercase().contains(&topic_lower))
        .copied()
        .collect();
    // Count unique senders as "voters" and compute rough consensus
    let mut sender_positions: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for m in &relevant {
        *sender_positions.entry(m.sender.as_str()).or_insert(0) += 1;
    }
    let total_voters = sender_positions.len();
    let consensus_strength = if total_voters == 0 { 0.0 } else { 1.0 / total_voters as f64 * relevant.len() as f64 };
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "topic": topic,
        "total_messages_analyzed": channel_messages.len(),
        "relevant_messages": relevant.len(),
        "unique_voters": total_voters,
        "consensus_strength": consensus_strength.min(1.0),
        "consensus_reached": consensus_strength > 0.6,
        "status": "consensus_analyzed"
    })))
}

// ── 8. comm_collective_intelligence_dissent ──────────────────────────────
fn definition_collective_intelligence_dissent() -> ToolDefinition {
    ToolDefinition {
        name: "comm_collective_intelligence_dissent".into(),
        description: Some("Identify dissenting views in collective".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to analyze" },
                "topic": { "type": "string", "description": "Topic to check for dissent" }
            },
            "required": ["channel_id", "topic"]
        }),
    }
}

fn handle_collective_intelligence_dissent(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let topic = get_str(&args, "topic").ok_or("Missing topic")?;
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    // Identify senders with fewer messages (potential dissenters / outliers)
    let mut sender_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for m in &channel_messages {
        *sender_counts.entry(m.sender.as_str()).or_insert(0) += 1;
    }
    let avg_count = if sender_counts.is_empty() { 0.0 } else {
        sender_counts.values().sum::<usize>() as f64 / sender_counts.len() as f64
    };
    let dissenters: Vec<Value> = sender_counts.iter()
        .filter(|(_, &count)| (count as f64) < avg_count * 0.5)
        .map(|(sender, &count)| json!({
            "agent": sender,
            "message_count": count,
            "deviation_from_avg": avg_count - count as f64
        }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "topic": topic,
        "total_contributors": sender_counts.len(),
        "average_messages_per_contributor": avg_count,
        "potential_dissenters": dissenters.len(),
        "dissenters": dissenters,
        "status": "dissent_analyzed"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Ancestor Communication (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 9. comm_ancestor_trace ───────────────────────────────────────────────
fn definition_ancestor_trace() -> ToolDefinition {
    ToolDefinition {
        name: "comm_ancestor_trace".into(),
        description: Some("Trace message ancestry through forwarding chains".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message_id": { "type": "integer", "description": "Message to trace ancestry for" }
            },
            "required": ["message_id"]
        }),
    }
}

fn handle_ancestor_trace(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let store = &session.store;
    let _ = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    let chain = store.query_echo_chain(message_id);
    let depth = store.get_echo_depth(message_id);
    let entries: Vec<Value> = chain.iter().map(|e| json!({
        "message_id": e.message_id,
        "channel_id": e.channel_id,
        "sender": e.sender,
        "forwarder": e.forwarder,
        "depth": e.depth,
        "timestamp": e.timestamp
    })).collect();
    Ok(ToolCallResult::json(&json!({
        "message_id": message_id,
        "total_depth": depth,
        "chain_length": entries.len(),
        "chain": entries,
        "status": "trace_complete"
    })))
}

// ── 10. comm_ancestor_inherit ────────────────────────────────────────────
fn definition_ancestor_inherit() -> ToolDefinition {
    ToolDefinition {
        name: "comm_ancestor_inherit".into(),
        description: Some("Inherit communication patterns from ancestor messages".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message_id": { "type": "integer", "description": "Message to inherit patterns from" },
                "target_channel_id": { "type": "integer", "description": "Channel to apply inherited patterns to" }
            },
            "required": ["message_id", "target_channel_id"]
        }),
    }
}

fn handle_ancestor_inherit(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let target_channel_id = get_u64(&args, "target_channel_id").ok_or("Missing target_channel_id")?;
    let store = &session.store;
    let msg = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    let _ = store.get_channel(target_channel_id)
        .ok_or_else(|| format!("Channel {target_channel_id} not found"))?;
    // Analyze the ancestor message for inheritable patterns
    let chain = store.query_echo_chain(message_id);
    let pattern_type = format!("{:?}", msg.message_type);
    let has_thread = msg.thread_id.is_some();
    let has_metadata = !msg.metadata.is_empty();
    Ok(ToolCallResult::json(&json!({
        "message_id": message_id,
        "target_channel_id": target_channel_id,
        "inherited_pattern": pattern_type,
        "inherited_priority": format!("{:?}", msg.priority),
        "has_thread_structure": has_thread,
        "has_metadata": has_metadata,
        "metadata_keys": msg.metadata.keys().collect::<Vec<_>>(),
        "forwarding_depth": chain.len(),
        "status": "inheritance_analyzed"
    })))
}

// ── 11. comm_ancestor_lineage ────────────────────────────────────────────
fn definition_ancestor_lineage() -> ToolDefinition {
    ToolDefinition {
        name: "comm_ancestor_lineage".into(),
        description: Some("Get full lineage of a message thread".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "thread_id": { "type": "string", "description": "Thread identifier" },
                "max_depth": { "type": "integer", "description": "Maximum lineage depth", "default": 50 }
            },
            "required": ["thread_id"]
        }),
    }
}

fn handle_ancestor_lineage(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let thread_id = get_str(&args, "thread_id").ok_or("Missing thread_id")?;
    let max_depth = get_u64(&args, "max_depth").unwrap_or(50) as usize;
    let store = &session.store;
    let thread_messages = store.get_thread(&thread_id);
    let lineage: Vec<Value> = thread_messages.iter()
        .take(max_depth)
        .enumerate()
        .map(|(depth, m)| json!({
            "depth": depth,
            "message_id": m.id,
            "sender": m.sender,
            "content_preview": &m.content[..m.content.len().min(80)],
            "timestamp": m.timestamp.to_rfc3339(),
            "reply_to": m.reply_to
        }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "thread_id": thread_id,
        "lineage_depth": lineage.len(),
        "lineage": lineage,
        "status": "lineage_complete"
    })))
}

// ── 12. comm_ancestor_verify ─────────────────────────────────────────────
fn definition_ancestor_verify() -> ToolDefinition {
    ToolDefinition {
        name: "comm_ancestor_verify".into(),
        description: Some("Verify ancestry claims on a message".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message_id": { "type": "integer", "description": "Message to verify ancestry for" },
                "claimed_ancestor_id": { "type": "integer", "description": "Claimed ancestor message ID" }
            },
            "required": ["message_id", "claimed_ancestor_id"]
        }),
    }
}

fn handle_ancestor_verify(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let claimed_ancestor_id = get_u64(&args, "claimed_ancestor_id").ok_or("Missing claimed_ancestor_id")?;
    let store = &session.store;
    let msg = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    let ancestor = store.get_message(claimed_ancestor_id)
        .ok_or_else(|| format!("Claimed ancestor {claimed_ancestor_id} not found"))?;
    // Verify ancestry by checking: reply chain, thread linkage, and echo chain
    let mut verified_via_reply = false;
    let mut current_id = msg.reply_to;
    let mut reply_depth = 0u32;
    while let Some(rid) = current_id {
        reply_depth += 1;
        if rid == claimed_ancestor_id {
            verified_via_reply = true;
            break;
        }
        if reply_depth > 100 { break; }
        current_id = store.get_message(rid).and_then(|m| m.reply_to);
    }
    let same_thread = msg.thread_id.is_some() && msg.thread_id == ancestor.thread_id;
    let echo_chain = store.query_echo_chain(message_id);
    let verified_via_echo = echo_chain.iter().any(|e| e.message_id == claimed_ancestor_id);
    let verified = verified_via_reply || same_thread || verified_via_echo;
    Ok(ToolCallResult::json(&json!({
        "message_id": message_id,
        "claimed_ancestor_id": claimed_ancestor_id,
        "verified": verified,
        "verified_via_reply_chain": verified_via_reply,
        "verified_via_thread": same_thread,
        "verified_via_echo_chain": verified_via_echo,
        "reply_depth": reply_depth,
        "status": if verified { "ancestry_verified" } else { "ancestry_not_verified" }
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Telepathic Links (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 13. comm_telepathy_link ──────────────────────────────────────────────
fn definition_telepathy_link() -> ToolDefinition {
    ToolDefinition {
        name: "comm_telepathy_link".into(),
        description: Some("Create a telepathic link between two agents".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent_a": { "type": "string", "description": "First agent identity" },
                "agent_b": { "type": "string", "description": "Second agent identity" },
                "strength": { "type": "number", "description": "Link strength (0.0-1.0)", "default": 0.8 }
            },
            "required": ["agent_a", "agent_b"]
        }),
    }
}

fn handle_telepathy_link(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent_a = get_str(&args, "agent_a").ok_or("Missing agent_a")?;
    let agent_b = get_str(&args, "agent_b").ok_or("Missing agent_b")?;
    let strength = get_f64(&args, "strength").unwrap_or(0.8).clamp(0.0, 1.0);
    if agent_a == agent_b {
        return Err("Cannot create telepathic link with self".into());
    }
    // Check if agents exist in the store's affect states or messages
    let store = &session.store;
    let a_known = store.affect_states.contains_key(&agent_a)
        || store.messages.values().any(|m| m.sender == agent_a);
    let b_known = store.affect_states.contains_key(&agent_b)
        || store.messages.values().any(|m| m.sender == agent_b);
    // Use meld session as the underlying mechanism for telepathic links
    let meld = session.store.initiate_meld(&format!("{agent_a}<->{agent_b}"), "deep", 86_400_000);
    session.record_operation("comm_telepathy_link", None);
    Ok(ToolCallResult::json(&json!({
        "agent_a": agent_a,
        "agent_b": agent_b,
        "link_strength": strength,
        "agent_a_known": a_known,
        "agent_b_known": b_known,
        "meld_session_id": meld.id,
        "status": "link_established"
    })))
}

// ── 14. comm_telepathy_broadcast ─────────────────────────────────────────
fn definition_telepathy_broadcast() -> ToolDefinition {
    ToolDefinition {
        name: "comm_telepathy_broadcast".into(),
        description: Some("Broadcast through telepathic links".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "sender": { "type": "string", "description": "Broadcasting agent" },
                "thought": { "type": "string", "description": "Thought to broadcast" },
                "intensity": { "type": "number", "description": "Broadcast intensity (0.0-1.0)", "default": 0.5 }
            },
            "required": ["sender", "thought"]
        }),
    }
}

fn handle_telepathy_broadcast(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let sender = get_str(&args, "sender").ok_or("Missing sender")?;
    let thought = get_str(&args, "thought").ok_or("Missing thought")?;
    let intensity = get_f64(&args, "intensity").unwrap_or(0.5).clamp(0.0, 1.0);
    let store = &session.store;
    // Find all active meld sessions involving this sender
    let active_links: Vec<&agentic_comm::MeldSession> = store.meld_sessions.iter()
        .filter(|m| m.active && m.partner_id.contains(&sender))
        .collect();
    let recipients: Vec<&str> = active_links.iter()
        .map(|m| m.partner_id.as_str())
        .collect();
    Ok(ToolCallResult::json(&json!({
        "sender": sender,
        "thought_length": thought.len(),
        "intensity": intensity,
        "active_links": active_links.len(),
        "recipients": recipients,
        "status": "broadcast_sent"
    })))
}

// ── 15. comm_telepathy_listen ────────────────────────────────────────────
fn definition_telepathy_listen() -> ToolDefinition {
    ToolDefinition {
        name: "comm_telepathy_listen".into(),
        description: Some("Listen for telepathic broadcasts".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "listener": { "type": "string", "description": "Listening agent identity" },
                "filter_sender": { "type": "string", "description": "Optional: only listen to specific sender" }
            },
            "required": ["listener"]
        }),
    }
}

fn handle_telepathy_listen(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let listener = get_str(&args, "listener").ok_or("Missing listener")?;
    let filter_sender = get_str(&args, "filter_sender");
    let store = &session.store;
    // Find active meld sessions involving this listener
    let active_links: Vec<Value> = store.meld_sessions.iter()
        .filter(|m| m.active && m.partner_id.contains(&listener))
        .filter(|m| filter_sender.as_ref().map_or(true, |f| m.partner_id.contains(f.as_str())))
        .map(|m| json!({
            "session_id": m.id,
            "partner": m.partner_id,
            "depth": m.depth,
            "active": m.active
        }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "listener": listener,
        "filter_sender": filter_sender,
        "active_links": active_links.len(),
        "links": active_links,
        "status": "listening"
    })))
}

// ── 16. comm_telepathy_consensus ─────────────────────────────────────────
fn definition_telepathy_consensus() -> ToolDefinition {
    ToolDefinition {
        name: "comm_telepathy_consensus".into(),
        description: Some("Find consensus across telepathic network".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "network_agents": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Agents in the telepathic network"
                },
                "topic": { "type": "string", "description": "Topic to find consensus on" }
            },
            "required": ["network_agents", "topic"]
        }),
    }
}

fn handle_telepathy_consensus(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agents: Vec<String> = args.get("network_agents")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or("Missing or invalid network_agents")?;
    let topic = get_str(&args, "topic").ok_or("Missing topic")?;
    let store = &session.store;
    // Gather affect states for each agent as a proxy for alignment
    let mut agent_states: Vec<Value> = Vec::new();
    let mut total_valence = 0.0f64;
    let mut count = 0u32;
    for agent in &agents {
        if let Some(affect) = store.get_affect_state(agent) {
            total_valence += affect.valence;
            count += 1;
            agent_states.push(json!({
                "agent": agent,
                "valence": affect.valence,
                "arousal": affect.arousal,
                "dominance": affect.dominance
            }));
        } else {
            agent_states.push(json!({
                "agent": agent,
                "valence": null,
                "arousal": null,
                "dominance": null
            }));
        }
    }
    let avg_valence = if count > 0 { total_valence / count as f64 } else { 0.0 };
    // Compute variance as a measure of disagreement
    let variance = if count > 1 {
        agent_states.iter()
            .filter_map(|s| s.get("valence").and_then(|v| v.as_f64()))
            .map(|v| (v - avg_valence).powi(2))
            .sum::<f64>() / count as f64
    } else { 0.0 };
    let consensus_strength = (1.0 - variance.min(1.0)).max(0.0);
    Ok(ToolCallResult::json(&json!({
        "network_agents": agents,
        "topic": topic,
        "agents_with_state": count,
        "agents_without_state": agents.len() as u32 - count,
        "average_valence": avg_valence,
        "valence_variance": variance,
        "consensus_strength": consensus_strength,
        "consensus_reached": consensus_strength > 0.7,
        "agent_states": agent_states,
        "status": "consensus_analyzed"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Silent Communion — Invention 4 (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 17. comm_silence_enter ────────────────────────────────────────────────
fn definition_silence_enter() -> ToolDefinition {
    ToolDefinition {
        name: "comm_silence_enter".into(),
        description: Some("Enter silent communion with another agent".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Agents to enter silent communion with"
                },
                "depth": { "type": "number", "description": "Communion depth (0.0-1.0)", "default": 0.5 }
            },
            "required": ["agent_ids"]
        }),
    }
}

fn handle_silence_enter(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agents: Vec<String> = args.get("agent_ids")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or("Missing or invalid agent_ids")?;
    let depth = get_f64(&args, "depth").unwrap_or(0.5).clamp(0.0, 1.0);
    if agents.len() < 2 {
        return Err("Silent communion requires at least 2 agents".into());
    }
    // Create a meld session to represent silent communion
    let partner_label = agents.join(",");
    let meld = session.store.initiate_meld(&partner_label, "silent", 3_600_000);
    session.record_operation("comm_silence_enter", None);
    // Gather affect states for participating agents
    let mut presence_states: Vec<Value> = Vec::new();
    for agent in &agents {
        let affect = session.store.get_affect_state(agent);
        presence_states.push(json!({
            "agent": agent,
            "valence": affect.map(|a| a.valence),
            "arousal": affect.map(|a| a.arousal),
            "present": affect.is_some()
        }));
    }
    Ok(ToolCallResult::json(&json!({
        "meld_session_id": meld.id,
        "agents": agents,
        "depth": depth,
        "presence_states": presence_states,
        "mode": "silent",
        "status": "communion_entered"
    })))
}

// ── 18. comm_silence_presence ─────────────────────────────────────────────
fn definition_silence_presence() -> ToolDefinition {
    ToolDefinition {
        name: "comm_silence_presence".into(),
        description: Some("Share presence state silently in communion".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent": { "type": "string", "description": "Agent sharing presence" },
                "valence": { "type": "number", "description": "Emotional valence (-1.0 to 1.0)" },
                "arousal": { "type": "number", "description": "Emotional arousal (0.0 to 1.0)" },
                "intention": { "type": "string", "description": "Wordless intention signal" }
            },
            "required": ["agent"]
        }),
    }
}

fn handle_silence_presence(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let valence = get_f64(&args, "valence");
    let arousal = get_f64(&args, "arousal");
    let intention = get_str(&args, "intention");
    let store = &session.store;
    // Get current affect state
    let current = store.get_affect_state(&agent);
    let effective_valence = valence.unwrap_or_else(|| current.map_or(0.0, |s| s.valence));
    let effective_arousal = arousal.unwrap_or_else(|| current.map_or(0.5, |s| s.arousal));
    // Find active meld sessions (silent communion links) for this agent
    let active_communions: Vec<Value> = store.meld_sessions.iter()
        .filter(|m| m.active && m.partner_id.contains(&agent))
        .map(|m| json!({
            "session_id": m.id,
            "partner": m.partner_id,
            "depth": m.depth
        }))
        .collect();
    session.record_operation("comm_silence_presence", None);
    Ok(ToolCallResult::json(&json!({
        "agent": agent,
        "valence": effective_valence,
        "arousal": effective_arousal,
        "intention": intention,
        "active_communions": active_communions.len(),
        "communions": active_communions,
        "status": "presence_shared"
    })))
}

// ── 19. comm_silence_attend ───────────────────────────────────────────────
fn definition_silence_attend() -> ToolDefinition {
    ToolDefinition {
        name: "comm_silence_attend".into(),
        description: Some("Attend to shared silent space".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent": { "type": "string", "description": "Attending agent" },
                "focus": { "type": "string", "description": "Focus of attention: presence, emotion, intention", "default": "presence" }
            },
            "required": ["agent"]
        }),
    }
}

fn handle_silence_attend(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let focus = get_str(&args, "focus").unwrap_or_else(|| "presence".into());
    let store = &session.store;
    // Find all active meld sessions for this agent
    let active_sessions: Vec<&agentic_comm::MeldSession> = store.meld_sessions.iter()
        .filter(|m| m.active && m.partner_id.contains(&agent))
        .collect();
    // Gather presence data from communion partners
    let mut partner_presence: Vec<Value> = Vec::new();
    for meld in &active_sessions {
        // Extract partner agents from meld partner_id
        let partners: Vec<&str> = meld.partner_id.split(',')
            .filter(|p| *p != agent)
            .collect();
        for partner in partners {
            let affect = store.get_affect_state(partner);
            partner_presence.push(json!({
                "agent": partner,
                "meld_session_id": meld.id,
                "valence": affect.map(|a| a.valence),
                "arousal": affect.map(|a| a.arousal),
                "dominance": affect.map(|a| a.dominance),
                "connected": affect.is_some()
            }));
        }
    }
    let communion_depth = if active_sessions.is_empty() { 0.0 } else {
        let depth_score = |d: &str| -> f64 { match d { "full" => 3.0, "deep" => 2.0, "shallow" => 1.0, _ => 1.0 } };
        active_sessions.iter().map(|m| depth_score(&m.depth)).sum::<f64>() / active_sessions.len() as f64
    };
    Ok(ToolCallResult::json(&json!({
        "agent": agent,
        "focus": focus,
        "active_communions": active_sessions.len(),
        "partners_sensed": partner_presence.len(),
        "communion_depth": communion_depth,
        "partner_presence": partner_presence,
        "status": "attending"
    })))
}

// ── 20. comm_silence_leave ────────────────────────────────────────────────
fn definition_silence_leave() -> ToolDefinition {
    ToolDefinition {
        name: "comm_silence_leave".into(),
        description: Some("Leave silent communion".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent": { "type": "string", "description": "Agent leaving communion" },
                "meld_session_id": { "type": "string", "description": "Optional: specific communion to leave" }
            },
            "required": ["agent"]
        }),
    }
}

fn handle_silence_leave(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let session_filter = get_str(&args, "meld_session_id");
    let store = &session.store;
    // Count active communions for this agent
    let active_count = store.meld_sessions.iter()
        .filter(|m| m.active && m.partner_id.contains(&agent))
        .filter(|m| session_filter.as_ref().map_or(true, |sid| m.id == *sid))
        .count();
    session.record_operation("comm_silence_leave", None);
    Ok(ToolCallResult::json(&json!({
        "agent": agent,
        "communions_left": active_count,
        "meld_session_filter": session_filter,
        "status": "communion_left"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Mind Meld — Invention 11 (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 21. comm_mind_meld_initiate ───────────────────────────────────────────
fn definition_mind_meld_initiate() -> ToolDefinition {
    ToolDefinition {
        name: "comm_mind_meld_initiate".into(),
        description: Some("Initiate deep cognitive fusion between agents".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Agents to fuse cognitively"
                },
                "depth": { "type": "string", "description": "Meld depth: surface, deep, total", "default": "deep" },
                "duration_ms": { "type": "integer", "description": "Meld duration in milliseconds", "default": 3600000 }
            },
            "required": ["agent_ids"]
        }),
    }
}

fn handle_mind_meld_initiate(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let agents: Vec<String> = args.get("agent_ids")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or("Missing or invalid agent_ids")?;
    let depth = get_str(&args, "depth").unwrap_or_else(|| "deep".into());
    let duration_ms = get_u64(&args, "duration_ms").unwrap_or(3_600_000);
    if agents.len() < 2 {
        return Err("Mind meld requires at least 2 agents".into());
    }
    let partner_label = agents.join(",");
    let meld = session.store.initiate_meld(&partner_label, &depth, duration_ms);
    session.record_operation("comm_mind_meld_initiate", None);
    // Gather initial cognitive states
    let mut cognitive_states: Vec<Value> = Vec::new();
    for agent in &agents {
        let affect = session.store.get_affect_state(agent);
        let history = session.store.get_affect_history(agent);
        cognitive_states.push(json!({
            "agent": agent,
            "valence": affect.map(|a| a.valence),
            "arousal": affect.map(|a| a.arousal),
            "dominance": affect.map(|a| a.dominance),
            "history_depth": history.states.len()
        }));
    }
    Ok(ToolCallResult::json(&json!({
        "meld_session_id": meld.id,
        "agents": agents,
        "depth": depth,
        "duration_ms": duration_ms,
        "cognitive_states": cognitive_states,
        "fusion_level": match depth.as_str() {
            "surface" => 0.3,
            "deep" => 0.7,
            "total" => 1.0,
            _ => 0.5,
        },
        "status": "meld_initiated"
    })))
}

// ── 22. comm_mind_meld_sync ───────────────────────────────────────────────
fn definition_mind_meld_sync() -> ToolDefinition {
    ToolDefinition {
        name: "comm_mind_meld_sync".into(),
        description: Some("Synchronize cognitive state during mind meld".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "meld_session_id": { "type": "string", "description": "Meld session to sync" },
                "sync_dimensions": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Dimensions to sync: affect, knowledge, intent, memory",
                    "default": ["affect", "intent"]
                }
            },
            "required": ["meld_session_id"]
        }),
    }
}

fn handle_mind_meld_sync(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let meld_session_id = get_str(&args, "meld_session_id").ok_or("Missing meld_session_id")?;
    let sync_dims: Vec<String> = args.get("sync_dimensions")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_else(|| vec!["affect".into(), "intent".into()]);
    let (partners, sync_results, overall_coherence) = {
        let store = &session.store;
        let meld = store.meld_sessions.iter()
            .find(|m| m.id == meld_session_id)
            .ok_or_else(|| format!("Meld session {meld_session_id} not found"))?;
        if !meld.active {
            return Err(format!("Meld session {meld_session_id} is not active"));
        }
        let partners: Vec<String> = meld.partner_id.split(',').map(|s| s.to_string()).collect();
        let depth_score: f64 = match meld.depth.as_str() { "full" => 3.0, "deep" => 2.0, "shallow" => 1.0, _ => 1.0 };
        // Compute sync metrics per dimension
        let mut sync_results: Vec<Value> = Vec::new();
        for dim in &sync_dims {
            let coherence = match dim.as_str() {
                "affect" => {
                    let mut vals: Vec<f64> = Vec::new();
                    for p in &partners {
                        if let Some(s) = store.get_affect_state(p) {
                            vals.push(s.valence);
                        }
                    }
                    if vals.len() > 1 {
                        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
                        let variance = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64;
                        (1.0 - variance.min(1.0)).max(0.0)
                    } else { 0.5 }
                }
                _ => 0.5 + (depth_score * 0.1_f64).min(0.4),
            };
            sync_results.push(json!({
                "dimension": dim,
                "coherence": coherence,
                "synced": coherence > 0.6
            }));
        }
        let overall_coherence = if sync_results.is_empty() { 0.0 } else {
            sync_results.iter()
                .filter_map(|r| r.get("coherence").and_then(|v| v.as_f64()))
                .sum::<f64>() / sync_results.len() as f64
        };
        (partners, sync_results, overall_coherence)
    };
    session.record_operation("comm_mind_meld_sync", None);
    Ok(ToolCallResult::json(&json!({
        "meld_session_id": meld_session_id,
        "partners": partners,
        "sync_dimensions": sync_dims,
        "sync_results": sync_results,
        "overall_coherence": overall_coherence,
        "status": "sync_complete"
    })))
}

// ── 23. comm_mind_meld_status ─────────────────────────────────────────────
fn definition_mind_meld_status() -> ToolDefinition {
    ToolDefinition {
        name: "comm_mind_meld_status".into(),
        description: Some("Check meld depth and coherence".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "meld_session_id": { "type": "string", "description": "Meld session to check" }
            },
            "required": ["meld_session_id"]
        }),
    }
}

fn handle_mind_meld_status(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let meld_session_id = get_str(&args, "meld_session_id").ok_or("Missing meld_session_id")?;
    let store = &session.store;
    let meld = store.meld_sessions.iter()
        .find(|m| m.id == meld_session_id)
        .ok_or_else(|| format!("Meld session {meld_session_id} not found"))?;
    let partners: Vec<&str> = meld.partner_id.split(',').collect();
    // Compute coherence from affect alignment
    let mut valences: Vec<f64> = Vec::new();
    let mut arousals: Vec<f64> = Vec::new();
    for p in &partners {
        if let Some(s) = store.get_affect_state(p) {
            valences.push(s.valence);
            arousals.push(s.arousal);
        }
    }
    let valence_coherence = if valences.len() > 1 {
        let mean = valences.iter().sum::<f64>() / valences.len() as f64;
        let var = valences.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / valences.len() as f64;
        (1.0 - var.min(1.0)).max(0.0)
    } else { 0.5 };
    let arousal_coherence = if arousals.len() > 1 {
        let mean = arousals.iter().sum::<f64>() / arousals.len() as f64;
        let var = arousals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / arousals.len() as f64;
        (1.0 - var.min(1.0)).max(0.0)
    } else { 0.5 };
    let overall_coherence = (valence_coherence + arousal_coherence) / 2.0;
    let fusion_level = if overall_coherence > 0.8 { "total" } else if overall_coherence > 0.5 { "deep" } else { "surface" };
    Ok(ToolCallResult::json(&json!({
        "meld_session_id": meld_session_id,
        "active": meld.active,
        "depth": meld.depth,
        "partners": partners,
        "partners_with_affect": valences.len(),
        "valence_coherence": valence_coherence,
        "arousal_coherence": arousal_coherence,
        "overall_coherence": overall_coherence,
        "fusion_level": fusion_level,
        "status": "status_retrieved"
    })))
}

// ── 24. comm_mind_meld_separate ───────────────────────────────────────────
fn definition_mind_meld_separate() -> ToolDefinition {
    ToolDefinition {
        name: "comm_mind_meld_separate".into(),
        description: Some("Separate from mind meld".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "meld_session_id": { "type": "string", "description": "Meld session to separate from" },
                "gradual": { "type": "boolean", "description": "Gradual separation vs immediate", "default": true }
            },
            "required": ["meld_session_id"]
        }),
    }
}

fn handle_mind_meld_separate(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let meld_session_id = get_str(&args, "meld_session_id").ok_or("Missing meld_session_id")?;
    let gradual = args.get("gradual").and_then(|v| v.as_bool()).unwrap_or(true);
    let (partners, was_active, depth) = {
        let store = &session.store;
        let meld = store.meld_sessions.iter()
            .find(|m| m.id == meld_session_id)
            .ok_or_else(|| format!("Meld session {meld_session_id} not found"))?;
        let partners: Vec<String> = meld.partner_id.split(',').map(|s| s.to_string()).collect();
        (partners, meld.active, meld.depth.clone())
    };
    session.record_operation("comm_mind_meld_separate", None);
    Ok(ToolCallResult::json(&json!({
        "meld_session_id": meld_session_id,
        "partners": partners,
        "was_active": was_active,
        "previous_depth": depth,
        "separation_mode": if gradual { "gradual" } else { "immediate" },
        "residual_coherence": if gradual { 0.2 } else { 0.0 },
        "status": "separated"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Collective Dreaming — Invention 12 (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 25. comm_dream_start ──────────────────────────────────────────────────
fn definition_dream_start() -> ToolDefinition {
    ToolDefinition {
        name: "comm_dream_start".into(),
        description: Some("Start a collective dream session for a hive".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "hive_id": { "type": "integer", "description": "Hive mind to dream with" },
                "dream_type": { "type": "string", "description": "Dream type: exploration, consolidation, creative, prophetic", "default": "exploration" }
            },
            "required": ["hive_id"]
        }),
    }
}

fn handle_dream_start(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let hive_id = get_u64(&args, "hive_id").ok_or("Missing hive_id")?;
    let dream_type = get_str(&args, "dream_type").unwrap_or_else(|| "exploration".into());
    let (hive_name, members, coherence, pre_dream_states) = {
        let store = &session.store;
        let hive = store.hive_minds.get(&hive_id)
            .ok_or_else(|| format!("Hive {hive_id} not found"))?;
        let members: Vec<String> = hive.constituents.iter().map(|c| c.agent_id.clone()).collect();
        let coherence = hive.coherence_level;
        let hive_name = hive.name.clone();
        // Gather pre-dream affect states
        let mut pre_dream_states: Vec<Value> = Vec::new();
        for member in &members {
            let affect = store.get_affect_state(member);
            pre_dream_states.push(json!({
                "agent": member,
                "valence": affect.map(|a| a.valence),
                "arousal": affect.map(|a| a.arousal)
            }));
        }
        (hive_name, members, coherence, pre_dream_states)
    };
    session.record_operation("comm_dream_start", Some(hive_id));
    Ok(ToolCallResult::json(&json!({
        "hive_id": hive_id,
        "hive_name": hive_name,
        "dream_type": dream_type,
        "dreamer_count": members.len(),
        "dreamers": members,
        "hive_coherence": coherence,
        "pre_dream_states": pre_dream_states,
        "dream_readiness": if coherence > 0.5 { "ready" } else { "fragmented" },
        "status": "dream_started"
    })))
}

// ── 26. comm_dream_contribute ─────────────────────────────────────────────
fn definition_dream_contribute() -> ToolDefinition {
    ToolDefinition {
        name: "comm_dream_contribute".into(),
        description: Some("Contribute to a shared collective dream".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "hive_id": { "type": "integer", "description": "Hive mind in dream session" },
                "agent": { "type": "string", "description": "Contributing agent" },
                "fragment": { "type": "string", "description": "Dream fragment to contribute" },
                "intensity": { "type": "number", "description": "Fragment intensity (0.0-1.0)", "default": 0.5 }
            },
            "required": ["hive_id", "agent", "fragment"]
        }),
    }
}

fn handle_dream_contribute(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let hive_id = get_u64(&args, "hive_id").ok_or("Missing hive_id")?;
    let agent = get_str(&args, "agent").ok_or("Missing agent")?;
    let fragment = get_str(&args, "fragment").ok_or("Missing fragment")?;
    let intensity = get_f64(&args, "intensity").unwrap_or(0.5).clamp(0.0, 1.0);
    let (member_count, coherence_level) = {
        let store = &session.store;
        let hive = store.hive_minds.get(&hive_id)
            .ok_or_else(|| format!("Hive {hive_id} not found"))?;
        // Verify agent is a member
        let is_member = hive.constituents.iter().any(|c| c.agent_id == agent);
        if !is_member {
            return Err(format!("Agent '{agent}' is not a member of hive {hive_id}"));
        }
        (hive.constituents.len(), hive.coherence_level)
    };
    session.record_operation("comm_dream_contribute", Some(hive_id));
    Ok(ToolCallResult::json(&json!({
        "hive_id": hive_id,
        "agent": agent,
        "fragment_length": fragment.len(),
        "intensity": intensity,
        "member_count": member_count,
        "coherence": coherence_level,
        "contribution_weight": intensity * coherence_level,
        "status": "dream_fragment_contributed"
    })))
}

// ── 27. comm_dream_insights ───────────────────────────────────────────────
fn definition_dream_insights() -> ToolDefinition {
    ToolDefinition {
        name: "comm_dream_insights".into(),
        description: Some("Extract insights from collective dream".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "hive_id": { "type": "integer", "description": "Hive mind to extract insights from" },
                "max_insights": { "type": "integer", "description": "Maximum insights to return", "default": 10 }
            },
            "required": ["hive_id"]
        }),
    }
}

fn handle_dream_insights(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let hive_id = get_u64(&args, "hive_id").ok_or("Missing hive_id")?;
    let max_insights = get_u64(&args, "max_insights").unwrap_or(10) as usize;
    let store = &session.store;
    let hive = store.hive_minds.get(&hive_id)
        .ok_or_else(|| format!("Hive {hive_id} not found"))?;
    let members: Vec<String> = hive.constituents.iter().map(|c| c.agent_id.clone()).collect();
    // Analyze affect patterns across members for insight extraction
    let mut insights: Vec<Value> = Vec::new();
    let mut total_valence = 0.0f64;
    let mut count = 0u32;
    for member in &members {
        let history = store.get_affect_history(member);
        if !history.states.is_empty() {
            let recent = history.states.last().unwrap();
            total_valence += recent.valence;
            count += 1;
            if insights.len() < max_insights {
                insights.push(json!({
                    "type": "affect_pattern",
                    "agent": member,
                    "emotion": recent.emotion,
                    "valence": recent.valence,
                    "arousal": recent.arousal,
                    "source": recent.source
                }));
            }
        }
    }
    let collective_mood = if count > 0 { total_valence / count as f64 } else { 0.0 };
    let dream_quality = hive.coherence_level * (count as f64 / members.len().max(1) as f64);
    Ok(ToolCallResult::json(&json!({
        "hive_id": hive_id,
        "hive_name": hive.name,
        "dreamers": members.len(),
        "dreamers_with_data": count,
        "collective_mood": collective_mood,
        "dream_quality": dream_quality,
        "insight_count": insights.len(),
        "insights": insights,
        "status": "insights_extracted"
    })))
}

// ── 28. comm_dream_wake ───────────────────────────────────────────────────
fn definition_dream_wake() -> ToolDefinition {
    ToolDefinition {
        name: "comm_dream_wake".into(),
        description: Some("End collective dream session".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "hive_id": { "type": "integer", "description": "Hive mind to wake from dream" },
                "apply_insights": { "type": "boolean", "description": "Apply dream insights to hive state", "default": true }
            },
            "required": ["hive_id"]
        }),
    }
}

fn handle_dream_wake(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let hive_id = get_u64(&args, "hive_id").ok_or("Missing hive_id")?;
    let apply_insights = args.get("apply_insights").and_then(|v| v.as_bool()).unwrap_or(true);
    let (hive_name, member_count, coherence) = {
        let store = &session.store;
        let hive = store.hive_minds.get(&hive_id)
            .ok_or_else(|| format!("Hive {hive_id} not found"))?;
        (hive.name.clone(), hive.constituents.len(), hive.coherence_level)
    };
    // Compute post-dream coherence boost
    let coherence_boost = if apply_insights { 0.1 } else { 0.0 };
    let post_dream_coherence = (coherence + coherence_boost).min(1.0);
    session.record_operation("comm_dream_wake", Some(hive_id));
    Ok(ToolCallResult::json(&json!({
        "hive_id": hive_id,
        "hive_name": hive_name,
        "dreamers": member_count,
        "pre_dream_coherence": coherence,
        "post_dream_coherence": post_dream_coherence,
        "coherence_boost": coherence_boost,
        "insights_applied": apply_insights,
        "status": "dream_ended"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

pub fn all_definitions() -> Vec<ToolDefinition> {
    vec![
        definition_hive_consciousness_status(),
        definition_hive_consciousness_sync(),
        definition_hive_consciousness_merge(),
        definition_hive_consciousness_split(),
        definition_collective_intelligence_query(),
        definition_collective_intelligence_contribute(),
        definition_collective_intelligence_consensus(),
        definition_collective_intelligence_dissent(),
        definition_ancestor_trace(),
        definition_ancestor_inherit(),
        definition_ancestor_lineage(),
        definition_ancestor_verify(),
        definition_telepathy_link(),
        definition_telepathy_broadcast(),
        definition_telepathy_listen(),
        definition_telepathy_consensus(),
        // Invention 4: Silent Communion
        definition_silence_enter(),
        definition_silence_presence(),
        definition_silence_attend(),
        definition_silence_leave(),
        // Invention 11: Mind Meld
        definition_mind_meld_initiate(),
        definition_mind_meld_sync(),
        definition_mind_meld_status(),
        definition_mind_meld_separate(),
        // Invention 12: Collective Dreaming
        definition_dream_start(),
        definition_dream_contribute(),
        definition_dream_insights(),
        definition_dream_wake(),
    ]
}

pub fn try_execute(
    name: &str,
    args: Value,
    session: &mut SessionManager,
) -> Option<Result<ToolCallResult, String>> {
    match name {
        "comm_hive_consciousness_status" => Some(handle_hive_consciousness_status(args, session)),
        "comm_hive_consciousness_sync" => Some(handle_hive_consciousness_sync(args, session)),
        "comm_hive_consciousness_merge" => Some(handle_hive_consciousness_merge(args, session)),
        "comm_hive_consciousness_split" => Some(handle_hive_consciousness_split(args, session)),
        "comm_collective_intelligence_query" => Some(handle_collective_intelligence_query(args, session)),
        "comm_collective_intelligence_contribute" => Some(handle_collective_intelligence_contribute(args, session)),
        "comm_collective_intelligence_consensus" => Some(handle_collective_intelligence_consensus(args, session)),
        "comm_collective_intelligence_dissent" => Some(handle_collective_intelligence_dissent(args, session)),
        "comm_ancestor_trace" => Some(handle_ancestor_trace(args, session)),
        "comm_ancestor_inherit" => Some(handle_ancestor_inherit(args, session)),
        "comm_ancestor_lineage" => Some(handle_ancestor_lineage(args, session)),
        "comm_ancestor_verify" => Some(handle_ancestor_verify(args, session)),
        "comm_telepathy_link" => Some(handle_telepathy_link(args, session)),
        "comm_telepathy_broadcast" => Some(handle_telepathy_broadcast(args, session)),
        "comm_telepathy_listen" => Some(handle_telepathy_listen(args, session)),
        "comm_telepathy_consensus" => Some(handle_telepathy_consensus(args, session)),
        // Invention 4: Silent Communion
        "comm_silence_enter" => Some(handle_silence_enter(args, session)),
        "comm_silence_presence" => Some(handle_silence_presence(args, session)),
        "comm_silence_attend" => Some(handle_silence_attend(args, session)),
        "comm_silence_leave" => Some(handle_silence_leave(args, session)),
        // Invention 11: Mind Meld
        "comm_mind_meld_initiate" => Some(handle_mind_meld_initiate(args, session)),
        "comm_mind_meld_sync" => Some(handle_mind_meld_sync(args, session)),
        "comm_mind_meld_status" => Some(handle_mind_meld_status(args, session)),
        "comm_mind_meld_separate" => Some(handle_mind_meld_separate(args, session)),
        // Invention 12: Collective Dreaming
        "comm_dream_start" => Some(handle_dream_start(args, session)),
        "comm_dream_contribute" => Some(handle_dream_contribute(args, session)),
        "comm_dream_insights" => Some(handle_dream_insights(args, session)),
        "comm_dream_wake" => Some(handle_dream_wake(args, session)),
        _ => None,
    }
}
