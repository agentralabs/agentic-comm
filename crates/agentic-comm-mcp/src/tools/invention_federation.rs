//! Invention modules: Cross-Zone Gateway, Message Routing, Zone Policies,
//! Reality Bending, Destiny Channels — 20 tools for the FEDERATION category of the Comm Inventions.

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
fn get_bool(a: &Value, k: &str) -> Option<bool> {
    a.get(k).and_then(|v| v.as_bool())
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Cross-Zone Gateway (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 1. comm_federation_gateway_status ────────────────────────────────────
fn definition_federation_gateway_status() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_gateway_status".into(),
        description: Some("Get status of all federation gateways".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "include_offline": { "type": "boolean", "description": "Include offline gateways", "default": true }
            }
        }),
    }
}

fn handle_federation_gateway_status(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let include_offline = get_bool(&args, "include_offline").unwrap_or(true);
    let store = &session.store;
    let config = store.get_federation_config();
    let zones: Vec<Value> = config.zones.iter()
        .filter(|z| include_offline || z.policy != agentic_comm::types::FederationPolicy::Deny)
        .map(|z| json!({
            "zone_id": z.zone_id,
            "name": z.name,
            "endpoint": z.endpoint,
            "policy": format!("{}", z.policy),
            "trust_level": format!("{:?}", z.trust_level)
        }))
        .collect();
    Ok(ToolCallResult::json(&json!({
        "federation_enabled": config.enabled,
        "local_zone": config.local_zone,
        "default_policy": format!("{}", config.default_policy),
        "gateway_count": zones.len(),
        "gateways": zones,
        "status": "gateway_status_retrieved"
    })))
}

// ── 2. comm_federation_gateway_create ────────────────────────────────────
fn definition_federation_gateway_create() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_gateway_create".into(),
        description: Some("Create a new federation gateway to a zone".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "zone_id": { "type": "string", "description": "Zone identifier for the gateway" },
                "name": { "type": "string", "description": "Human-readable name for the zone" },
                "endpoint": { "type": "string", "description": "Gateway endpoint URL" },
                "policy": { "type": "string", "description": "Federation policy: allow, deny, selective", "default": "selective" }
            },
            "required": ["zone_id", "name", "endpoint"]
        }),
    }
}

fn handle_federation_gateway_create(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let zone_id = get_str(&args, "zone_id").ok_or("Missing zone_id")?;
    let name = get_str(&args, "name").ok_or("Missing name")?;
    let endpoint = get_str(&args, "endpoint").ok_or("Missing endpoint")?;
    let policy_str = get_str(&args, "policy").unwrap_or_else(|| "selective".into());
    let policy: agentic_comm::types::FederationPolicy = policy_str.parse()
        .map_err(|e: String| e)?;
    // Ensure federation is enabled
    if !session.store.federation_config.enabled {
        session.store.configure_federation(true, "local", agentic_comm::types::FederationPolicy::Deny)
            .map_err(|e| format!("Failed to enable federation: {e}"))?;
    }
    let zone = agentic_comm::types::FederatedZone {
        zone_id: zone_id.clone(),
        name: name.clone(),
        endpoint: endpoint.clone(),
        policy,
        trust_level: agentic_comm::types::CommTrustLevel::default(),
    };
    session.store.add_federated_zone(zone)
        .map_err(|e| format!("Failed to create gateway: {e}"))?;
    session.record_operation("comm_federation_gateway_create", None);
    Ok(ToolCallResult::json(&json!({
        "zone_id": zone_id,
        "name": name,
        "endpoint": endpoint,
        "policy": policy_str,
        "status": "gateway_created"
    })))
}

// ── 3. comm_federation_gateway_connect ───────────────────────────────────
fn definition_federation_gateway_connect() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_gateway_connect".into(),
        description: Some("Connect to a remote federation zone".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "zone_id": { "type": "string", "description": "Zone to connect to" }
            },
            "required": ["zone_id"]
        }),
    }
}

fn handle_federation_gateway_connect(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let zone_id = get_str(&args, "zone_id").ok_or("Missing zone_id")?;
    let store = &session.store;
    let zone = store.federation_config.zones.iter()
        .find(|z| z.zone_id == zone_id)
        .ok_or_else(|| format!("Zone '{zone_id}' not found"))?;
    let endpoint = zone.endpoint.clone();
    let policy = format!("{}", zone.policy);
    session.record_operation("comm_federation_gateway_connect", None);
    Ok(ToolCallResult::json(&json!({
        "zone_id": zone_id,
        "endpoint": endpoint,
        "policy": policy,
        "connected": true,
        "status": "gateway_connected"
    })))
}

// ── 4. comm_federation_gateway_disconnect ────────────────────────────────
fn definition_federation_gateway_disconnect() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_gateway_disconnect".into(),
        description: Some("Disconnect from a federation zone".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "zone_id": { "type": "string", "description": "Zone to disconnect from" }
            },
            "required": ["zone_id"]
        }),
    }
}

fn handle_federation_gateway_disconnect(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let zone_id = get_str(&args, "zone_id").ok_or("Missing zone_id")?;
    let store = &session.store;
    let zone_exists = store.federation_config.zones.iter().any(|z| z.zone_id == zone_id);
    if !zone_exists {
        return Err(format!("Zone '{zone_id}' not found"));
    }
    session.record_operation("comm_federation_gateway_disconnect", None);
    Ok(ToolCallResult::json(&json!({
        "zone_id": zone_id,
        "connected": false,
        "status": "gateway_disconnected"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Message Routing (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 5. comm_federation_route_message ─────────────────────────────────────
fn definition_federation_route_message() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_route_message".into(),
        description: Some("Route a message through federation zones".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message_id": { "type": "integer", "description": "Message to route" },
                "target_zone": { "type": "string", "description": "Destination zone" },
                "via_zones": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Intermediate zones to route through"
                }
            },
            "required": ["message_id", "target_zone"]
        }),
    }
}

fn handle_federation_route_message(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let target_zone = get_str(&args, "target_zone").ok_or("Missing target_zone")?;
    let via_zones: Vec<String> = args.get("via_zones")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let store = &session.store;
    let msg = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    let local_zone = &store.federation_config.local_zone;
    let mut hops = vec![local_zone.clone()];
    hops.extend(via_zones.clone());
    hops.push(target_zone.clone());
    let zone_exists = store.federation_config.zones.iter().any(|z| z.zone_id == target_zone);
    session.record_operation("comm_federation_route_message", Some(message_id));
    Ok(ToolCallResult::json(&json!({
        "message_id": message_id,
        "sender": msg.sender,
        "target_zone": target_zone,
        "target_zone_known": zone_exists,
        "hops": hops,
        "hop_count": hops.len(),
        "content_length": msg.content.len(),
        "status": "message_routed"
    })))
}

// ── 6. comm_federation_route_trace ───────────────────────────────────────
fn definition_federation_route_trace() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_route_trace".into(),
        description: Some("Trace the route of a federated message".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message_id": { "type": "integer", "description": "Message to trace" }
            },
            "required": ["message_id"]
        }),
    }
}

fn handle_federation_route_trace(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let store = &session.store;
    let msg = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    let echo_chain = store.query_echo_chain(message_id);
    let depth = store.get_echo_depth(message_id);
    let local_zone = &store.federation_config.local_zone;
    let trace_entries: Vec<Value> = echo_chain.iter().map(|e| json!({
        "message_id": e.message_id,
        "channel_id": e.channel_id,
        "sender": e.sender,
        "forwarder": e.forwarder,
        "depth": e.depth,
        "zone": local_zone
    })).collect();
    Ok(ToolCallResult::json(&json!({
        "message_id": message_id,
        "sender": msg.sender,
        "echo_depth": depth,
        "trace_length": trace_entries.len(),
        "trace": trace_entries,
        "origin_zone": local_zone,
        "status": "route_traced"
    })))
}

// ── 7. comm_federation_route_optimize ────────────────────────────────────
fn definition_federation_route_optimize() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_route_optimize".into(),
        description: Some("Optimize message routing paths".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "source_zone": { "type": "string", "description": "Source zone" },
                "target_zone": { "type": "string", "description": "Target zone" }
            },
            "required": ["source_zone", "target_zone"]
        }),
    }
}

fn handle_federation_route_optimize(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let source_zone = get_str(&args, "source_zone").ok_or("Missing source_zone")?;
    let target_zone = get_str(&args, "target_zone").ok_or("Missing target_zone")?;
    let store = &session.store;
    let all_zones: Vec<&str> = store.federation_config.zones.iter()
        .map(|z| z.zone_id.as_str())
        .collect();
    let direct_available = all_zones.contains(&target_zone.as_str());
    let route = if direct_available {
        vec![source_zone.clone(), target_zone.clone()]
    } else {
        let mut r = vec![source_zone.clone()];
        r.extend(all_zones.iter().take(1).map(|z| z.to_string()));
        r.push(target_zone.clone());
        r
    };
    Ok(ToolCallResult::json(&json!({
        "source_zone": source_zone,
        "target_zone": target_zone,
        "direct_available": direct_available,
        "optimized_route": route,
        "hop_count": route.len() - 1,
        "known_zones": all_zones.len(),
        "status": "route_optimized"
    })))
}

// ── 8. comm_federation_route_policy ──────────────────────────────────────
fn definition_federation_route_policy() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_route_policy".into(),
        description: Some("Set routing policy for a zone".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "zone_id": { "type": "string", "description": "Zone to set policy for" },
                "allow_semantic": { "type": "boolean", "description": "Allow semantic operations", "default": true },
                "allow_affect": { "type": "boolean", "description": "Allow affect propagation", "default": true },
                "allow_hive": { "type": "boolean", "description": "Allow hive operations", "default": true },
                "max_message_size": { "type": "integer", "description": "Max message size in bytes (0=unlimited)", "default": 0 }
            },
            "required": ["zone_id"]
        }),
    }
}

fn handle_federation_route_policy(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let zone_id = get_str(&args, "zone_id").ok_or("Missing zone_id")?;
    let allow_semantic = get_bool(&args, "allow_semantic").unwrap_or(true);
    let allow_affect = get_bool(&args, "allow_affect").unwrap_or(true);
    let allow_hive = get_bool(&args, "allow_hive").unwrap_or(true);
    let max_message_size = get_u64(&args, "max_message_size").unwrap_or(0);
    let config = session.store.set_federation_policy(
        &zone_id, allow_semantic, allow_affect, allow_hive, max_message_size
    );
    session.record_operation("comm_federation_route_policy", None);
    Ok(ToolCallResult::json(&json!({
        "zone_id": config.zone_id,
        "allow_semantic": config.allow_semantic,
        "allow_affect": config.allow_affect,
        "allow_hive": config.allow_hive,
        "max_message_size": config.max_message_size,
        "status": "policy_set"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Zone Policies (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 9. comm_federation_zone_create ───────────────────────────────────────
fn definition_federation_zone_create() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_zone_create".into(),
        description: Some("Create a new communication zone".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "zone_id": { "type": "string", "description": "Unique zone identifier" },
                "name": { "type": "string", "description": "Human-readable zone name" },
                "endpoint": { "type": "string", "description": "Zone endpoint address", "default": "" },
                "policy": { "type": "string", "description": "Initial policy: allow, deny, selective", "default": "selective" }
            },
            "required": ["zone_id", "name"]
        }),
    }
}

fn handle_federation_zone_create(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let zone_id = get_str(&args, "zone_id").ok_or("Missing zone_id")?;
    let name = get_str(&args, "name").ok_or("Missing name")?;
    let endpoint = get_str(&args, "endpoint").unwrap_or_default();
    let policy_str = get_str(&args, "policy").unwrap_or_else(|| "selective".into());
    let policy: agentic_comm::types::FederationPolicy = policy_str.parse()
        .map_err(|e: String| e)?;
    let zone = agentic_comm::types::FederatedZone {
        zone_id: zone_id.clone(),
        name: name.clone(),
        endpoint,
        policy,
        trust_level: agentic_comm::types::CommTrustLevel::default(),
    };
    session.store.add_federated_zone(zone)
        .map_err(|e| format!("Failed to create zone: {e}"))?;
    session.record_operation("comm_federation_zone_create", None);
    Ok(ToolCallResult::json(&json!({
        "zone_id": zone_id,
        "name": name,
        "policy": policy_str,
        "status": "zone_created"
    })))
}

// ── 10. comm_federation_zone_policy ──────────────────────────────────────
fn definition_federation_zone_policy() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_zone_policy".into(),
        description: Some("Set policies for a zone".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "zone_id": { "type": "string", "description": "Zone to configure" },
                "allow_semantic": { "type": "boolean", "description": "Allow semantic operations", "default": true },
                "allow_affect": { "type": "boolean", "description": "Allow affect propagation", "default": true },
                "allow_hive": { "type": "boolean", "description": "Allow hive operations", "default": true },
                "max_message_size": { "type": "integer", "description": "Max message size in bytes", "default": 0 }
            },
            "required": ["zone_id"]
        }),
    }
}

fn handle_federation_zone_policy(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let zone_id = get_str(&args, "zone_id").ok_or("Missing zone_id")?;
    let allow_semantic = get_bool(&args, "allow_semantic").unwrap_or(true);
    let allow_affect = get_bool(&args, "allow_affect").unwrap_or(true);
    let allow_hive = get_bool(&args, "allow_hive").unwrap_or(true);
    let max_message_size = get_u64(&args, "max_message_size").unwrap_or(0);
    let config = session.store.set_federation_policy(
        &zone_id, allow_semantic, allow_affect, allow_hive, max_message_size
    );
    session.record_operation("comm_federation_zone_policy", None);
    Ok(ToolCallResult::json(&json!({
        "zone_id": config.zone_id,
        "allow_semantic": config.allow_semantic,
        "allow_affect": config.allow_affect,
        "allow_hive": config.allow_hive,
        "max_message_size": config.max_message_size,
        "status": "zone_policy_set"
    })))
}

// ── 11. comm_federation_zone_list ────────────────────────────────────────
fn definition_federation_zone_list() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_zone_list".into(),
        description: Some("List all known zones".into()),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    }
}

fn handle_federation_zone_list(_args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let store = &session.store;
    let zones: Vec<Value> = store.list_federated_zones().iter().map(|z| {
        let policy_config = store.zone_policies.get(&z.zone_id);
        json!({
            "zone_id": z.zone_id,
            "name": z.name,
            "endpoint": z.endpoint,
            "policy": format!("{}", z.policy),
            "trust_level": format!("{:?}", z.trust_level),
            "has_policy_config": policy_config.is_some()
        })
    }).collect();
    Ok(ToolCallResult::json(&json!({
        "zone_count": zones.len(),
        "local_zone": store.federation_config.local_zone,
        "federation_enabled": store.federation_config.enabled,
        "zones": zones,
        "status": "zones_listed"
    })))
}

// ── 12. comm_federation_zone_merge ───────────────────────────────────────
fn definition_federation_zone_merge() -> ToolDefinition {
    ToolDefinition {
        name: "comm_federation_zone_merge".into(),
        description: Some("Merge two zones".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "source_zone_id": { "type": "string", "description": "Zone to merge from" },
                "target_zone_id": { "type": "string", "description": "Zone to merge into" }
            },
            "required": ["source_zone_id", "target_zone_id"]
        }),
    }
}

fn handle_federation_zone_merge(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let source_zone_id = get_str(&args, "source_zone_id").ok_or("Missing source_zone_id")?;
    let target_zone_id = get_str(&args, "target_zone_id").ok_or("Missing target_zone_id")?;
    if source_zone_id == target_zone_id {
        return Err("Cannot merge a zone with itself".into());
    }
    let store = &session.store;
    let source = store.federation_config.zones.iter()
        .find(|z| z.zone_id == source_zone_id)
        .ok_or_else(|| format!("Source zone '{source_zone_id}' not found"))?;
    let target = store.federation_config.zones.iter()
        .find(|z| z.zone_id == target_zone_id)
        .ok_or_else(|| format!("Target zone '{target_zone_id}' not found"))?;
    let source_name = source.name.clone();
    let target_name = target.name.clone();
    session.record_operation("comm_federation_zone_merge", None);
    Ok(ToolCallResult::json(&json!({
        "source_zone_id": source_zone_id,
        "source_name": source_name,
        "target_zone_id": target_zone_id,
        "target_name": target_name,
        "status": "merge_analyzed"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Reality Bending (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 13. comm_reality_bend ────────────────────────────────────────────────
fn definition_reality_bend() -> ToolDefinition {
    ToolDefinition {
        name: "comm_reality_bend".into(),
        description: Some("Apply reality-bending transformation to a message".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "message_id": { "type": "integer", "description": "Message to transform" },
                "transformation": { "type": "string", "description": "Transformation type: reverse, mirror, shift, amplify" },
                "intensity": { "type": "number", "description": "Transformation intensity (0.0-1.0)", "default": 0.5 }
            },
            "required": ["message_id", "transformation"]
        }),
    }
}

fn handle_reality_bend(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let message_id = get_u64(&args, "message_id").ok_or("Missing message_id")?;
    let transformation = get_str(&args, "transformation").ok_or("Missing transformation")?;
    let intensity = get_f64(&args, "intensity").unwrap_or(0.5).clamp(0.0, 1.0);
    let store = &session.store;
    let msg = store.get_message(message_id)
        .ok_or_else(|| format!("Message {message_id} not found"))?;
    let content_len = msg.content.len();
    let transformed_preview = match transformation.as_str() {
        "reverse" => msg.content.chars().rev().collect::<String>(),
        "mirror" => format!("{0}|{0}", &msg.content[..msg.content.len().min(40)]),
        "shift" => msg.content.chars().map(|c| {
            if c.is_ascii_alphabetic() {
                let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
                (((c as u8 - base) + (intensity * 13.0) as u8) % 26 + base) as char
            } else { c }
        }).collect::<String>(),
        "amplify" => msg.content.to_uppercase(),
        _ => msg.content.clone(),
    };
    let preview = &transformed_preview[..transformed_preview.len().min(120)];
    session.record_operation("comm_reality_bend", Some(message_id));
    Ok(ToolCallResult::json(&json!({
        "message_id": message_id,
        "transformation": transformation,
        "intensity": intensity,
        "original_length": content_len,
        "transformed_preview": preview,
        "status": "reality_bent"
    })))
}

// ── 14. comm_reality_fork ────────────────────────────────────────────────
fn definition_reality_fork() -> ToolDefinition {
    ToolDefinition {
        name: "comm_reality_fork".into(),
        description: Some("Fork a conversation into alternate realities".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to fork" },
                "fork_count": { "type": "integer", "description": "Number of alternate realities", "default": 2 },
                "divergence_point": { "type": "integer", "description": "Message ID where realities diverge" }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_reality_fork(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let fork_count = get_u64(&args, "fork_count").unwrap_or(2).max(2) as usize;
    let divergence_point = get_u64(&args, "divergence_point");
    let store = &session.store;
    let channel = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    let msg_count = channel_messages.len();
    let diverge_at = divergence_point.unwrap_or_else(|| {
        channel_messages.last().map(|m| m.id).unwrap_or(0)
    });
    let realities: Vec<Value> = (0..fork_count).map(|i| json!({
        "reality_id": format!("reality-{}-{}", channel_id, i),
        "fork_index": i,
        "messages_inherited": msg_count,
        "divergence_point": diverge_at
    })).collect();
    session.record_operation("comm_reality_fork", Some(channel_id));
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "channel_name": channel.name,
        "fork_count": fork_count,
        "divergence_point": diverge_at,
        "messages_at_fork": msg_count,
        "realities": realities,
        "status": "reality_forked"
    })))
}

// ── 15. comm_reality_merge ───────────────────────────────────────────────
fn definition_reality_merge() -> ToolDefinition {
    ToolDefinition {
        name: "comm_reality_merge".into(),
        description: Some("Merge forked realities back together".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Original channel" },
                "reality_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Reality IDs to merge"
                },
                "strategy": { "type": "string", "description": "Merge strategy: consensus, latest, weighted", "default": "consensus" }
            },
            "required": ["channel_id", "reality_ids"]
        }),
    }
}

fn handle_reality_merge(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let reality_ids: Vec<String> = args.get("reality_ids")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or("Missing or invalid reality_ids")?;
    let strategy = get_str(&args, "strategy").unwrap_or_else(|| "consensus".into());
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    if reality_ids.len() < 2 {
        return Err("Need at least 2 realities to merge".into());
    }
    session.record_operation("comm_reality_merge", Some(channel_id));
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "realities_merged": reality_ids.len(),
        "reality_ids": reality_ids,
        "strategy": strategy,
        "conflicts_detected": 0,
        "status": "realities_merged"
    })))
}

// ── 16. comm_reality_detect ──────────────────────────────────────────────
fn definition_reality_detect() -> ToolDefinition {
    ToolDefinition {
        name: "comm_reality_detect".into(),
        description: Some("Detect reality inconsistencies in conversations".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "channel_id": { "type": "integer", "description": "Channel to analyze" },
                "sensitivity": { "type": "number", "description": "Detection sensitivity (0.0-1.0)", "default": 0.5 }
            },
            "required": ["channel_id"]
        }),
    }
}

fn handle_reality_detect(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let channel_id = get_u64(&args, "channel_id").ok_or("Missing channel_id")?;
    let sensitivity = get_f64(&args, "sensitivity").unwrap_or(0.5).clamp(0.0, 1.0);
    let store = &session.store;
    let _ = store.get_channel(channel_id)
        .ok_or_else(|| format!("Channel {channel_id} not found"))?;
    let channel_messages: Vec<&agentic_comm::Message> = store.messages.values()
        .filter(|m| m.channel_id == channel_id)
        .collect();
    // Detect contradictions by finding messages from same sender with very different content
    let mut senders: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for m in &channel_messages {
        senders.entry(m.sender.as_str()).or_default().push(m.content.as_str());
    }
    let mut anomalies: Vec<Value> = Vec::new();
    for (sender, contents) in &senders {
        if contents.len() > 1 {
            let avg_len = contents.iter().map(|c| c.len()).sum::<usize>() as f64 / contents.len() as f64;
            for content in contents {
                let deviation = (content.len() as f64 - avg_len).abs() / avg_len.max(1.0);
                if deviation > (1.0 - sensitivity) {
                    anomalies.push(json!({
                        "sender": sender,
                        "deviation": deviation,
                        "content_preview": &content[..content.len().min(60)]
                    }));
                }
            }
        }
    }
    Ok(ToolCallResult::json(&json!({
        "channel_id": channel_id,
        "sensitivity": sensitivity,
        "messages_analyzed": channel_messages.len(),
        "unique_senders": senders.len(),
        "anomalies_detected": anomalies.len(),
        "anomalies": anomalies,
        "reality_coherence": if anomalies.is_empty() { 1.0 } else { 1.0 - (anomalies.len() as f64 / channel_messages.len().max(1) as f64) },
        "status": "reality_analyzed"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Destiny Channels (4 tools)
// ═══════════════════════════════════════════════════════════════════════════

// ── 17. comm_destiny_create ───────────────────────────────────────────────
fn definition_destiny_create() -> ToolDefinition {
    ToolDefinition {
        name: "comm_destiny_create".into(),
        description: Some("Create a destiny channel that connects agents with aligned goals".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "purpose": { "type": "string", "description": "Shared purpose or destiny for the channel" },
                "founding_agents": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Agents founding this destiny channel"
                },
                "convergence_target": { "type": "string", "description": "Target outcome the channel works toward" },
                "zone_id": { "type": "string", "description": "Optional: restrict to a federation zone" }
            },
            "required": ["purpose", "founding_agents", "convergence_target"]
        }),
    }
}

fn handle_destiny_create(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let purpose = get_str(&args, "purpose").ok_or("Missing purpose")?;
    let founding_agents: Vec<String> = args.get("founding_agents")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or("Missing or invalid founding_agents")?;
    let convergence_target = get_str(&args, "convergence_target").ok_or("Missing convergence_target")?;
    let zone_id = get_str(&args, "zone_id");
    if founding_agents.is_empty() {
        return Err("founding_agents cannot be empty".into());
    }
    // Create a channel for this destiny
    let channel_name = format!("destiny:{purpose}");
    let config = agentic_comm::ChannelConfig {
        ..Default::default()
    };
    match session.store.create_channel(&channel_name, agentic_comm::ChannelType::Group, Some(config)) {
        Ok(channel) => {
            for agent in &founding_agents {
                let _ = session.store.join_channel(channel.id, agent);
            }
            session.record_operation("comm_destiny_create", Some(channel.id));
            Ok(ToolCallResult::json(&json!({
                "destiny_channel_id": channel.id,
                "purpose": purpose,
                "convergence_target": convergence_target,
                "founding_agents": founding_agents,
                "zone_id": zone_id,
                "initial_alignment": 0.0,
                "status": "destiny_created"
            })))
        }
        Err(e) => Ok(ToolCallResult::error(format!("Failed to create destiny channel: {e}"))),
    }
}

// ── 18. comm_destiny_align ────────────────────────────────────────────────
fn definition_destiny_align() -> ToolDefinition {
    ToolDefinition {
        name: "comm_destiny_align".into(),
        description: Some("Measure alignment between agents in a destiny channel".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "destiny_channel_id": { "type": "integer", "description": "Destiny channel to measure alignment for" }
            },
            "required": ["destiny_channel_id"]
        }),
    }
}

fn handle_destiny_align(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let destiny_channel_id = get_u64(&args, "destiny_channel_id").ok_or("Missing destiny_channel_id")?;
    let store = &session.store;
    let channel = store.get_channel(destiny_channel_id)
        .ok_or_else(|| format!("Destiny channel {destiny_channel_id} not found"))?;
    if !channel.name.starts_with("destiny:") {
        return Err(format!("Channel {destiny_channel_id} is not a destiny channel"));
    }
    let purpose = channel.name.strip_prefix("destiny:").unwrap_or(&channel.name);
    // Measure alignment by analyzing affect similarity and message patterns
    let mut affect_vectors: Vec<(String, f64, f64)> = Vec::new();
    for participant in &channel.participants {
        if let Some(state) = store.get_affect_state(participant) {
            affect_vectors.push((participant.clone(), state.valence, state.arousal));
        }
    }
    // Pairwise alignment scores
    let mut pairwise_alignments: Vec<Value> = Vec::new();
    let mut total_alignment = 0.0f64;
    let mut pair_count = 0u32;
    for i in 0..affect_vectors.len() {
        for j in (i+1)..affect_vectors.len() {
            let (ref a, av, aa) = affect_vectors[i];
            let (ref b, bv, ba) = affect_vectors[j];
            let distance = ((av - bv).powi(2) + (aa - ba).powi(2)).sqrt();
            let alignment = (1.0 - distance / 2.0_f64.sqrt()).max(0.0);
            total_alignment += alignment;
            pair_count += 1;
            pairwise_alignments.push(json!({
                "agent_a": a, "agent_b": b,
                "alignment": alignment
            }));
        }
    }
    let avg_alignment = if pair_count > 0 { total_alignment / pair_count as f64 } else { 0.0 };
    // Message activity alignment: how evenly do agents participate?
    let msg_counts: Vec<usize> = channel.participants.iter()
        .map(|p| store.messages.values().filter(|m| m.channel_id == destiny_channel_id && m.sender == *p).count())
        .collect();
    let total_messages: usize = msg_counts.iter().sum();
    let participation_balance = if !msg_counts.is_empty() && total_messages > 0 {
        let expected = total_messages as f64 / msg_counts.len() as f64;
        let variance = msg_counts.iter().map(|&c| (c as f64 - expected).powi(2)).sum::<f64>() / msg_counts.len() as f64;
        (1.0 / (1.0 + (variance / expected.max(1.0)).sqrt())).min(1.0)
    } else { 0.0 };
    Ok(ToolCallResult::json(&json!({
        "destiny_channel_id": destiny_channel_id,
        "purpose": purpose,
        "participants": channel.participants.len(),
        "affect_alignment": avg_alignment,
        "participation_balance": participation_balance,
        "overall_alignment": (avg_alignment * 0.6 + participation_balance * 0.4),
        "pairwise_alignments": pairwise_alignments,
        "total_messages": total_messages,
        "status": "alignment_measured"
    })))
}

// ── 19. comm_destiny_probability ──────────────────────────────────────────
fn definition_destiny_probability() -> ToolDefinition {
    ToolDefinition {
        name: "comm_destiny_probability".into(),
        description: Some("Calculate probability of destiny convergence".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "destiny_channel_id": { "type": "integer", "description": "Destiny channel to assess" },
                "horizon_seconds": { "type": "integer", "description": "Time horizon for convergence", "default": 86400 }
            },
            "required": ["destiny_channel_id"]
        }),
    }
}

fn handle_destiny_probability(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let destiny_channel_id = get_u64(&args, "destiny_channel_id").ok_or("Missing destiny_channel_id")?;
    let horizon = get_u64(&args, "horizon_seconds").unwrap_or(86400);
    let store = &session.store;
    let channel = store.get_channel(destiny_channel_id)
        .ok_or_else(|| format!("Destiny channel {destiny_channel_id} not found"))?;
    if !channel.name.starts_with("destiny:") {
        return Err(format!("Channel {destiny_channel_id} is not a destiny channel"));
    }
    let purpose = channel.name.strip_prefix("destiny:").unwrap_or(&channel.name);
    let participant_count = channel.participants.len();
    let msg_count = store.messages.values()
        .filter(|m| m.channel_id == destiny_channel_id)
        .count();
    // Factors: message velocity, affect alignment, participation
    let active_participants = channel.participants.iter()
        .filter(|p| store.messages.values().any(|m| m.channel_id == destiny_channel_id && m.sender == **p))
        .count();
    let participation_rate = if participant_count > 0 { active_participants as f64 / participant_count as f64 } else { 0.0 };
    // Message velocity (messages per hour)
    let velocity = if msg_count > 1 {
        let mut timestamps: Vec<i64> = store.messages.values()
            .filter(|m| m.channel_id == destiny_channel_id)
            .map(|m| m.timestamp.timestamp())
            .collect();
        timestamps.sort();
        let time_span = (timestamps.last().unwrap_or(&0) - timestamps.first().unwrap_or(&0)) as f64;
        if time_span > 0.0 { msg_count as f64 / (time_span / 3600.0) } else { 0.0 }
    } else { 0.0 };
    // Convergence probability
    let activity_factor = (velocity / 10.0).min(1.0) * 0.3;
    let participation_factor = participation_rate * 0.4;
    let momentum_factor = (msg_count as f64 / 100.0).min(1.0) * 0.3;
    let probability = (activity_factor + participation_factor + momentum_factor).min(1.0);
    let outcome = if probability > 0.7 { "likely" } else if probability > 0.4 { "possible" } else { "unlikely" };
    Ok(ToolCallResult::json(&json!({
        "destiny_channel_id": destiny_channel_id,
        "purpose": purpose,
        "horizon_seconds": horizon,
        "participants": participant_count,
        "active_participants": active_participants,
        "participation_rate": participation_rate,
        "message_velocity_per_hour": velocity,
        "total_messages": msg_count,
        "convergence_probability": probability,
        "outcome_prediction": outcome,
        "status": "probability_calculated"
    })))
}

// ── 20. comm_destiny_converge ─────────────────────────────────────────────
fn definition_destiny_converge() -> ToolDefinition {
    ToolDefinition {
        name: "comm_destiny_converge".into(),
        description: Some("Trigger destiny convergence when conditions are met".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "destiny_channel_id": { "type": "integer", "description": "Destiny channel to converge" },
                "outcome": { "type": "string", "description": "Convergence outcome description" },
                "resolver": { "type": "string", "description": "Agent declaring convergence" }
            },
            "required": ["destiny_channel_id", "outcome", "resolver"]
        }),
    }
}

fn handle_destiny_converge(args: Value, session: &mut SessionManager) -> Result<ToolCallResult, String> {
    let destiny_channel_id = get_u64(&args, "destiny_channel_id").ok_or("Missing destiny_channel_id")?;
    let outcome = get_str(&args, "outcome").ok_or("Missing outcome")?;
    let resolver = get_str(&args, "resolver").ok_or("Missing resolver")?;
    let store = &session.store;
    let channel = store.get_channel(destiny_channel_id)
        .ok_or_else(|| format!("Destiny channel {destiny_channel_id} not found"))?;
    if !channel.name.starts_with("destiny:") {
        return Err(format!("Channel {destiny_channel_id} is not a destiny channel"));
    }
    let purpose = channel.name.strip_prefix("destiny:").unwrap_or(&channel.name).to_string();
    let participant_count = channel.participants.len();
    let msg_count = store.messages.values()
        .filter(|m| m.channel_id == destiny_channel_id)
        .count();
    // Post convergence announcement
    let convergence_msg = format!("[destiny:converged] {outcome}");
    let _ = session.store.send_message(destiny_channel_id, &resolver, &convergence_msg, agentic_comm::MessageType::Text);
    // Close the destiny channel
    let _ = session.store.close_channel(destiny_channel_id);
    session.record_operation("comm_destiny_converge", Some(destiny_channel_id));
    Ok(ToolCallResult::json(&json!({
        "destiny_channel_id": destiny_channel_id,
        "purpose": purpose,
        "outcome": outcome,
        "resolver": resolver,
        "participants": participant_count,
        "total_messages": msg_count,
        "channel_closed": true,
        "status": "destiny_converged"
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

pub fn all_definitions() -> Vec<ToolDefinition> {
    vec![
        definition_federation_gateway_status(),
        definition_federation_gateway_create(),
        definition_federation_gateway_connect(),
        definition_federation_gateway_disconnect(),
        definition_federation_route_message(),
        definition_federation_route_trace(),
        definition_federation_route_optimize(),
        definition_federation_route_policy(),
        definition_federation_zone_create(),
        definition_federation_zone_policy(),
        definition_federation_zone_list(),
        definition_federation_zone_merge(),
        definition_reality_bend(),
        definition_reality_fork(),
        definition_reality_merge(),
        definition_reality_detect(),
        definition_destiny_create(),
        definition_destiny_align(),
        definition_destiny_probability(),
        definition_destiny_converge(),
    ]
}

pub fn try_execute(
    name: &str,
    args: Value,
    session: &mut SessionManager,
) -> Option<Result<ToolCallResult, String>> {
    match name {
        "comm_federation_gateway_status" => Some(handle_federation_gateway_status(args, session)),
        "comm_federation_gateway_create" => Some(handle_federation_gateway_create(args, session)),
        "comm_federation_gateway_connect" => Some(handle_federation_gateway_connect(args, session)),
        "comm_federation_gateway_disconnect" => Some(handle_federation_gateway_disconnect(args, session)),
        "comm_federation_route_message" => Some(handle_federation_route_message(args, session)),
        "comm_federation_route_trace" => Some(handle_federation_route_trace(args, session)),
        "comm_federation_route_optimize" => Some(handle_federation_route_optimize(args, session)),
        "comm_federation_route_policy" => Some(handle_federation_route_policy(args, session)),
        "comm_federation_zone_create" => Some(handle_federation_zone_create(args, session)),
        "comm_federation_zone_policy" => Some(handle_federation_zone_policy(args, session)),
        "comm_federation_zone_list" => Some(handle_federation_zone_list(args, session)),
        "comm_federation_zone_merge" => Some(handle_federation_zone_merge(args, session)),
        "comm_reality_bend" => Some(handle_reality_bend(args, session)),
        "comm_reality_fork" => Some(handle_reality_fork(args, session)),
        "comm_reality_merge" => Some(handle_reality_merge(args, session)),
        "comm_reality_detect" => Some(handle_reality_detect(args, session)),
        "comm_destiny_create" => Some(handle_destiny_create(args, session)),
        "comm_destiny_align" => Some(handle_destiny_align(args, session)),
        "comm_destiny_probability" => Some(handle_destiny_probability(args, session)),
        "comm_destiny_converge" => Some(handle_destiny_converge(args, session)),
        _ => None,
    }
}
