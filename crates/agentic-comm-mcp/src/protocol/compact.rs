//! Compact MCP tool facade — collapses 17 consolidated tools into 9 grouped
//! facades. Activated via `ACOM_MCP_TOOL_SURFACE=compact` or the generic
//! `MCP_TOOL_SURFACE=compact` env var.

use serde_json::{json, Value};

// ── Facade group definitions ────────────────────────────────────────────

/// Channel, message, trust, keys, store operations.
const CORE_OPS: &[&str] = &[
    "comm_channel",
    "comm_message",
    "comm_trust",
    "comm_keys",
    "comm_store",
];

/// Semantic messaging and NLP analysis (incl. invention_semantics).
const SEMANTIC_OPS: &[&str] = &["comm_semantic"];

/// Emotional state and affect propagation (incl. invention_affect).
const AFFECT_OPS: &[&str] = &["comm_affect"];

/// Hive consciousness, collective intel, telepathy
/// (incl. invention_collaboration).
const COLLABORATION_OPS: &[&str] = &["comm_hive", "comm_collaboration"];

/// Federation, zones, routing (incl. invention_federation).
const FEDERATION_OPS: &[&str] = &["comm_federation"];

/// Forensics, patterns, health (incl. invention_forensics).
const FORENSICS_OPS: &[&str] = &["comm_forensics"];

/// Scheduled messaging, dead letters (incl. invention_temporal).
const TEMPORAL_OPS: &[&str] = &["comm_temporal"];

/// Session lifecycle, workspace, agent operations.
const SESSION_OPS: &[&str] = &[
    "comm_session",
    "comm_workspace",
    "comm_agent",
];

/// Consent, query, collaboration combined.
const ADVANCED_OPS: &[&str] = &["comm_consent", "comm_query"];

// ── Compact group names ─────────────────────────────────────────────────

const COMPACT_GROUPS: &[(&str, &[&str])] = &[
    ("acom_core", CORE_OPS),
    ("acom_semantic", SEMANTIC_OPS),
    ("acom_affect", AFFECT_OPS),
    ("acom_collaboration", COLLABORATION_OPS),
    ("acom_federation", FEDERATION_OPS),
    ("acom_forensics", FORENSICS_OPS),
    ("acom_temporal", TEMPORAL_OPS),
    ("acom_session", SESSION_OPS),
    ("acom_advanced", ADVANCED_OPS),
];

// ── Public API ──────────────────────────────────────────────────────────

/// Check whether the compact tool surface is active.
pub fn mcp_tool_surface_is_compact() -> bool {
    std::env::var("ACOM_MCP_TOOL_SURFACE")
        .or_else(|_| std::env::var("MCP_TOOL_SURFACE"))
        .map(|v| v.eq_ignore_ascii_case("compact"))
        .unwrap_or(false)
}

/// Build an input schema for a compact facade tool with `operation` enum.
pub fn compact_op_schema(ops: &[&str], description: &str) -> Value {
    json!({
        "type": "object",
        "description": description,
        "properties": {
            "operation": {
                "type": "string",
                "enum": ops,
                "description": "Underlying consolidated tool to invoke"
            },
            "args": {
                "type": "object",
                "description":
                    "Arguments forwarded to the underlying tool \
                     (including its own 'operation' field)",
                "additionalProperties": true
            }
        },
        "required": ["operation"]
    })
}

/// Return the 9 compact tool definitions.
pub fn compact_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "acom_core",
            "description":
                "Manage channels, messages, trust, encryption keys, \
                 and store maintenance",
            "inputSchema": compact_op_schema(
                CORE_OPS,
                "Core communication operations"
            )
        }),
        json!({
            "name": "acom_semantic",
            "description":
                "Semantic messaging, NLP analysis, echo chambers, ghosts, \
                 metamessages, and conversation forks",
            "inputSchema": compact_op_schema(
                SEMANTIC_OPS,
                "Semantic and NLP operations"
            )
        }),
        json!({
            "name": "acom_affect",
            "description":
                "Emotional state, affect propagation, contagion models, \
                 archaeology, prophecy, unspeakable content, anticipation",
            "inputSchema": compact_op_schema(
                AFFECT_OPS,
                "Affect and emotional operations"
            )
        }),
        json!({
            "name": "acom_collaboration",
            "description":
                "Hive consciousness, collective intelligence, ancestry, \
                 telepathy, silence, mind meld, dream collaboration",
            "inputSchema": compact_op_schema(
                COLLABORATION_OPS,
                "Collaboration and hive operations"
            )
        }),
        json!({
            "name": "acom_federation",
            "description":
                "Federation gateways, routing, zones, reality bending, \
                 and destiny alignment",
            "inputSchema": compact_op_schema(
                FEDERATION_OPS,
                "Federation and cross-zone operations"
            )
        }),
        json!({
            "name": "acom_forensics",
            "description":
                "Communication forensics, pattern detection, health \
                 diagnostics, and oracle queries",
            "inputSchema": compact_op_schema(
                FORENSICS_OPS,
                "Forensics and diagnostic operations"
            )
        }),
        json!({
            "name": "acom_temporal",
            "description":
                "Scheduled messaging, dead letters, precognition, legacy \
                 messages, and temporal consensus",
            "inputSchema": compact_op_schema(
                TEMPORAL_OPS,
                "Temporal and scheduling operations"
            )
        }),
        json!({
            "name": "acom_session",
            "description":
                "Session lifecycle, workspace management, and agent \
                 operations",
            "inputSchema": compact_op_schema(
                SESSION_OPS,
                "Session, workspace, and agent operations"
            )
        }),
        json!({
            "name": "acom_advanced",
            "description":
                "Consent management, relationship queries, evidence, and \
                 conversation grounding",
            "inputSchema": compact_op_schema(
                ADVANCED_OPS,
                "Consent, query, and advanced operations"
            )
        }),
    ]
}

/// Decode a compact tool call into (operation, inner_args).
pub fn decode_compact_operation(args: Value) -> Result<(String, Value), String> {
    let operation = args
        .get("operation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'operation' field".to_string())?
        .to_string();

    let inner_args = args
        .get("args")
        .cloned()
        .unwrap_or_else(|| json!({}));

    Ok((operation, inner_args))
}

/// Validate that `operation` belongs to the given compact `group`.
pub fn resolve_compact_tool(group: &str, operation: &str) -> Option<String> {
    for &(name, ops) in COMPACT_GROUPS {
        if name == group {
            if ops.contains(&operation) {
                return Some(operation.to_string());
            }
            return None;
        }
    }
    None
}

/// Normalize a tool call: if it targets a compact facade, resolve to the
/// canonical consolidated tool name + inner args. Non-compact tool names
/// pass through unchanged.
pub fn normalize_compact_tool_call(
    tool_name: &str,
    args: Value,
) -> Result<(String, Value), String> {
    // Check if this is a compact facade group name
    let is_compact = COMPACT_GROUPS.iter().any(|&(n, _)| n == tool_name);
    if !is_compact {
        return Ok((tool_name.to_string(), args));
    }

    let (operation, inner_args) = decode_compact_operation(args)?;
    let canonical = resolve_compact_tool(tool_name, &operation).ok_or_else(|| {
        format!(
            "Unknown operation '{}' for compact group '{}'",
            operation, tool_name
        )
    })?;
    Ok((canonical, inner_args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_definitions_count() {
        assert_eq!(compact_tool_definitions().len(), 9);
    }

    #[test]
    fn resolve_core_ops() {
        assert_eq!(
            resolve_compact_tool("acom_core", "comm_channel"),
            Some("comm_channel".into())
        );
        assert_eq!(
            resolve_compact_tool("acom_core", "comm_message"),
            Some("comm_message".into())
        );
        assert_eq!(
            resolve_compact_tool("acom_core", "comm_trust"),
            Some("comm_trust".into())
        );
        assert_eq!(
            resolve_compact_tool("acom_core", "bogus"),
            None
        );
    }

    #[test]
    fn resolve_collaboration_ops() {
        assert_eq!(
            resolve_compact_tool("acom_collaboration", "comm_hive"),
            Some("comm_hive".into())
        );
        assert_eq!(
            resolve_compact_tool("acom_collaboration", "comm_collaboration"),
            Some("comm_collaboration".into())
        );
    }

    #[test]
    fn resolve_unknown_group() {
        assert_eq!(resolve_compact_tool("acom_bogus", "anything"), None);
    }

    #[test]
    fn normalize_compact_call() {
        let args = json!({
            "operation": "comm_channel",
            "args": { "operation": "create", "name": "test-channel" }
        });
        let (name, inner) =
            normalize_compact_tool_call("acom_core", args).unwrap();
        assert_eq!(name, "comm_channel");
        assert_eq!(inner["operation"], "create");
        assert_eq!(inner["name"], "test-channel");
    }

    #[test]
    fn normalize_passthrough() {
        let args = json!({ "operation": "create", "name": "ch" });
        let (name, inner) =
            normalize_compact_tool_call("comm_channel", args.clone()).unwrap();
        assert_eq!(name, "comm_channel");
        assert_eq!(inner, args);
    }

    #[test]
    fn normalize_bad_operation() {
        let args = json!({
            "operation": "comm_bogus",
            "args": {}
        });
        assert!(normalize_compact_tool_call("acom_core", args).is_err());
    }

    #[test]
    fn decode_missing_operation() {
        let args = json!({ "args": {} });
        assert!(decode_compact_operation(args).is_err());
    }

    #[test]
    fn decode_missing_args_defaults_empty() {
        let args = json!({ "operation": "comm_affect" });
        let (op, inner) = decode_compact_operation(args).unwrap();
        assert_eq!(op, "comm_affect");
        assert_eq!(inner, json!({}));
    }

    #[test]
    fn compact_mode_off_by_default() {
        assert!(!mcp_tool_surface_is_compact());
    }
}
