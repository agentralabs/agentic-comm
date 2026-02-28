//! CommWorkspace — multi-store comparison and cross-referencing.
//!
//! A workspace loads multiple `.acomm` files and provides query, compare,
//! and cross-reference operations across all of them.

use std::collections::HashSet;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::CommStore;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A workspace holding references to multiple `.acomm` stores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommWorkspace {
    /// Unique workspace identifier (deterministic hash of name).
    pub id: String,
    /// Human-readable workspace name.
    pub name: String,
    /// Loaded store contexts.
    pub contexts: Vec<WorkspaceContext>,
}

/// A single loaded store context within a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceContext {
    /// Path to the `.acomm` file.
    pub path: String,
    /// Optional human-readable label.
    pub label: Option<String>,
    /// Role this context plays in the workspace.
    pub role: WorkspaceRole,
    /// Number of messages in the store.
    pub message_count: usize,
    /// Number of channels in the store.
    pub channel_count: usize,
    /// Number of unique agents (participants + trust-level keys).
    pub agent_count: usize,
}

/// Role of a context within a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkspaceRole {
    /// The main store being worked on.
    Primary,
    /// A secondary store for comparison.
    Secondary,
    /// A read-only reference store.
    Reference,
    /// An archived / historical store.
    Archive,
}

impl fmt::Display for WorkspaceRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkspaceRole::Primary => write!(f, "primary"),
            WorkspaceRole::Secondary => write!(f, "secondary"),
            WorkspaceRole::Reference => write!(f, "reference"),
            WorkspaceRole::Archive => write!(f, "archive"),
        }
    }
}

impl FromStr for WorkspaceRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "primary" => Ok(WorkspaceRole::Primary),
            "secondary" => Ok(WorkspaceRole::Secondary),
            "reference" => Ok(WorkspaceRole::Reference),
            "archive" => Ok(WorkspaceRole::Archive),
            other => Err(format!("Unknown workspace role: {other}")),
        }
    }
}

impl Default for WorkspaceRole {
    fn default() -> Self {
        WorkspaceRole::Secondary
    }
}

/// Result of querying across workspace contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceQueryResult {
    /// Label of the context (or path if unlabelled).
    pub context_label: String,
    /// Path to the `.acomm` file.
    pub context_path: String,
    /// Matches found in this context.
    pub matches: Vec<WorkspaceMatch>,
}

/// A single match found in a workspace query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMatch {
    /// Type of matched entity: "message", "channel", or "agent".
    pub match_type: String,
    /// Content or name of the matched entity.
    pub content: String,
    /// Timestamp (epoch millis) of the match, or 0 if not applicable.
    pub timestamp: u64,
}

/// Side-by-side comparison of an item across workspace contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceComparison {
    /// The item being compared.
    pub item: String,
    /// Comparison results per context.
    pub contexts: Vec<WorkspaceComparisonEntry>,
}

/// A single context's comparison entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceComparisonEntry {
    /// Label of the context.
    pub context_label: String,
    /// Whether the item was found.
    pub found: bool,
    /// Number of occurrences.
    pub count: usize,
    /// Optional details string.
    pub details: Option<String>,
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl CommWorkspace {
    /// Create a new empty workspace with the given name.
    ///
    /// The workspace ID is a deterministic SHA-256-based hex string derived
    /// from the name, giving stable identifiers across sessions.
    pub fn new(name: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        let hash = hasher.finalize();
        let id = hex::encode(&hash[..16]); // 32-char hex

        Self {
            id,
            name: name.to_string(),
            contexts: Vec::new(),
        }
    }

    /// Add a context (`.acomm` file) to the workspace.
    ///
    /// Loads the store to gather statistics. Returns an error if the file
    /// cannot be loaded.
    pub fn add_context(
        &mut self,
        path: &str,
        label: Option<&str>,
        role: WorkspaceRole,
    ) -> Result<(), String> {
        let store = CommStore::load(Path::new(path))
            .map_err(|e| format!("Failed to load {path}: {e}"))?;

        let (message_count, channel_count, agent_count) = store_counts(&store);

        let display_label = label.map(|s| s.to_string());
        self.contexts.push(WorkspaceContext {
            path: path.to_string(),
            label: display_label,
            role,
            message_count,
            channel_count,
            agent_count,
        });

        Ok(())
    }

    /// Return a slice of all loaded contexts.
    pub fn list_contexts(&self) -> &[WorkspaceContext] {
        &self.contexts
    }

    /// Query all contexts for the given text.
    ///
    /// Searches messages (content), channels (name), and agents (participants)
    /// in each loaded store. Returns up to `max_per_context` matches per store.
    pub fn query(&self, query: &str, max_per_context: usize) -> Vec<WorkspaceQueryResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for ctx in &self.contexts {
            let store = match CommStore::load(Path::new(&ctx.path)) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let label = context_label(ctx);
            let mut matches = Vec::new();

            // Search messages
            for msg in store.messages.values() {
                if matches.len() >= max_per_context {
                    break;
                }
                if msg.content.to_lowercase().contains(&query_lower)
                    || msg.sender.to_lowercase().contains(&query_lower)
                {
                    matches.push(WorkspaceMatch {
                        match_type: "message".to_string(),
                        content: format!(
                            "[{}] {}: {}",
                            msg.message_type,
                            msg.sender,
                            truncate(&msg.content, 120)
                        ),
                        timestamp: msg.timestamp.timestamp_millis() as u64,
                    });
                }
            }

            // Search channels
            for ch in store.channels.values() {
                if matches.len() >= max_per_context {
                    break;
                }
                if ch.name.to_lowercase().contains(&query_lower) {
                    matches.push(WorkspaceMatch {
                        match_type: "channel".to_string(),
                        content: format!(
                            "{} ({}, {} participants)",
                            ch.name,
                            ch.channel_type,
                            ch.participants.len()
                        ),
                        timestamp: ch.created_at.timestamp_millis() as u64,
                    });
                }
            }

            // Search agents (participants)
            let mut seen = HashSet::new();
            for ch in store.channels.values() {
                if matches.len() >= max_per_context {
                    break;
                }
                for p in &ch.participants {
                    if p.to_lowercase().contains(&query_lower) && seen.insert(p.clone()) {
                        matches.push(WorkspaceMatch {
                            match_type: "agent".to_string(),
                            content: p.clone(),
                            timestamp: 0,
                        });
                    }
                }
            }

            results.push(WorkspaceQueryResult {
                context_label: label,
                context_path: ctx.path.clone(),
                matches,
            });
        }

        results
    }

    /// Compare the presence of an item across all contexts.
    ///
    /// For each context, searches messages, channels, and agents for the item
    /// and reports whether it was found, how many times, and a brief summary.
    pub fn compare(&self, item: &str, max_per_context: usize) -> WorkspaceComparison {
        let item_lower = item.to_lowercase();
        let mut entries = Vec::new();

        for ctx in &self.contexts {
            let store = match CommStore::load(Path::new(&ctx.path)) {
                Ok(s) => s,
                Err(_) => {
                    entries.push(WorkspaceComparisonEntry {
                        context_label: context_label(ctx),
                        found: false,
                        count: 0,
                        details: Some("Failed to load store".to_string()),
                    });
                    continue;
                }
            };

            let label = context_label(ctx);
            let mut count = 0usize;
            let mut details_parts: Vec<String> = Vec::new();

            // Count message matches
            let msg_count = store
                .messages
                .values()
                .filter(|m| {
                    m.content.to_lowercase().contains(&item_lower)
                        || m.sender.to_lowercase().contains(&item_lower)
                })
                .take(max_per_context)
                .count();
            if msg_count > 0 {
                count += msg_count;
                details_parts.push(format!("{msg_count} message(s)"));
            }

            // Count channel matches
            let ch_count = store
                .channels
                .values()
                .filter(|c| c.name.to_lowercase().contains(&item_lower))
                .count();
            if ch_count > 0 {
                count += ch_count;
                details_parts.push(format!("{ch_count} channel(s)"));
            }

            // Count agent matches
            let mut agent_hits = HashSet::new();
            for ch in store.channels.values() {
                for p in &ch.participants {
                    if p.to_lowercase().contains(&item_lower) {
                        agent_hits.insert(p.clone());
                    }
                }
            }
            for agent in store.trust_levels.keys() {
                if agent.to_lowercase().contains(&item_lower) {
                    agent_hits.insert(agent.clone());
                }
            }
            if !agent_hits.is_empty() {
                count += agent_hits.len();
                details_parts.push(format!("{} agent(s)", agent_hits.len()));
            }

            entries.push(WorkspaceComparisonEntry {
                context_label: label,
                found: count > 0,
                count,
                details: if details_parts.is_empty() {
                    None
                } else {
                    Some(details_parts.join(", "))
                },
            });
        }

        WorkspaceComparison {
            item: item.to_string(),
            contexts: entries,
        }
    }

    /// Cross-reference an item across all contexts.
    ///
    /// Returns a list of (context_label, found, count) tuples.
    pub fn xref(&self, item: &str) -> Vec<(String, bool, usize)> {
        let item_lower = item.to_lowercase();
        let mut results = Vec::new();

        for ctx in &self.contexts {
            let store = match CommStore::load(Path::new(&ctx.path)) {
                Ok(s) => s,
                Err(_) => {
                    results.push((context_label(ctx), false, 0));
                    continue;
                }
            };

            let label = context_label(ctx);
            let mut count = 0usize;

            // Messages
            count += store
                .messages
                .values()
                .filter(|m| {
                    m.content.to_lowercase().contains(&item_lower)
                        || m.sender.to_lowercase().contains(&item_lower)
                })
                .count();

            // Channels
            count += store
                .channels
                .values()
                .filter(|c| c.name.to_lowercase().contains(&item_lower))
                .count();

            // Agents (participants + trust levels)
            let mut agents = HashSet::new();
            for ch in store.channels.values() {
                for p in &ch.participants {
                    if p.to_lowercase().contains(&item_lower) {
                        agents.insert(p.clone());
                    }
                }
            }
            for agent in store.trust_levels.keys() {
                if agent.to_lowercase().contains(&item_lower) {
                    agents.insert(agent.clone());
                }
            }
            count += agents.len();

            results.push((label, count > 0, count));
        }

        results
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Derive the display label for a context.
fn context_label(ctx: &WorkspaceContext) -> String {
    ctx.label
        .clone()
        .unwrap_or_else(|| ctx.path.clone())
}

/// Gather (message_count, channel_count, agent_count) from a loaded store.
fn store_counts(store: &CommStore) -> (usize, usize, usize) {
    let message_count = store.messages.len();
    let channel_count = store.channels.len();

    // Unique agents = union of all channel participants + trust-level keys
    let mut agents = HashSet::new();
    for ch in store.channels.values() {
        for p in &ch.participants {
            agents.insert(p.clone());
        }
    }
    for agent in store.trust_levels.keys() {
        agents.insert(agent.clone());
    }
    let agent_count = agents.len();

    (message_count, channel_count, agent_count)
}

/// Truncate a string to `max` characters, appending "..." if truncated.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max.saturating_sub(3)).collect();
        t.push_str("...");
        t
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChannelType, CommStore, MessageType};

    /// Helper: create a CommStore with channels, messages, and agents, then
    /// save it to a temp file and return the path.
    fn create_test_store(
        _name: &str,
        channels: &[(&str, &[&str])],
        messages: &[(&str, &str)],
    ) -> (tempfile::NamedTempFile, String) {
        let mut store = CommStore::new();

        for (ch_name, participants) in channels {
            let ch = store
                .create_channel(ch_name, ChannelType::Group, None)
                .expect("create channel");
            let ch_id = ch.id;
            for p in *participants {
                store.join_channel(ch_id, p).expect("join channel");
            }
        }

        // Send messages to the first channel (if any)
        if let Some(first_ch) = store.channels.keys().next().copied() {
            for (sender, content) in messages {
                store
                    .send_message(first_ch, sender, content, MessageType::Text)
                    .expect("send message");
            }
        }

        let tmp = tempfile::NamedTempFile::new().expect("create temp file");
        store.save(tmp.path()).expect("save store");
        let path = tmp.path().to_string_lossy().to_string();
        (tmp, path)
    }

    #[test]
    fn test_workspace_new_deterministic_id() {
        let ws1 = CommWorkspace::new("test-workspace");
        let ws2 = CommWorkspace::new("test-workspace");
        let ws3 = CommWorkspace::new("different-workspace");

        assert_eq!(ws1.id, ws2.id, "Same name should produce same ID");
        assert_ne!(ws1.id, ws3.id, "Different names should produce different IDs");
        assert_eq!(ws1.id.len(), 32, "ID should be 32-char hex");
        assert_eq!(ws1.name, "test-workspace");
    }

    #[test]
    fn test_workspace_add_context_and_list() {
        let (_tmp1, path1) = create_test_store(
            "store-a",
            &[("alpha", &["alice", "bob"])],
            &[("alice", "hello"), ("bob", "world")],
        );
        let (_tmp2, path2) = create_test_store(
            "store-b",
            &[("beta", &["charlie"]), ("gamma", &["dave", "eve"])],
            &[("charlie", "test message")],
        );

        let mut ws = CommWorkspace::new("multi-store");
        ws.add_context(&path1, Some("Store A"), WorkspaceRole::Primary)
            .expect("add context 1");
        ws.add_context(&path2, Some("Store B"), WorkspaceRole::Reference)
            .expect("add context 2");

        let contexts = ws.list_contexts();
        assert_eq!(contexts.len(), 2);

        assert_eq!(contexts[0].label, Some("Store A".to_string()));
        assert_eq!(contexts[0].role, WorkspaceRole::Primary);
        assert_eq!(contexts[0].message_count, 2);
        assert_eq!(contexts[0].channel_count, 1);
        assert_eq!(contexts[0].agent_count, 2); // alice, bob

        assert_eq!(contexts[1].label, Some("Store B".to_string()));
        assert_eq!(contexts[1].role, WorkspaceRole::Reference);
        assert_eq!(contexts[1].message_count, 1);
        assert_eq!(contexts[1].channel_count, 2);
        assert_eq!(contexts[1].agent_count, 3); // charlie, dave, eve
    }

    #[test]
    fn test_workspace_query_across_contexts() {
        let (_tmp1, path1) = create_test_store(
            "query-a",
            &[("comms", &["alice", "bob"])],
            &[("alice", "deploy the service"), ("bob", "service deployed")],
        );
        let (_tmp2, path2) = create_test_store(
            "query-b",
            &[("ops", &["carol"])],
            &[("carol", "no service here")],
        );

        let mut ws = CommWorkspace::new("query-test");
        ws.add_context(&path1, Some("Project A"), WorkspaceRole::Primary)
            .unwrap();
        ws.add_context(&path2, Some("Project B"), WorkspaceRole::Secondary)
            .unwrap();

        let results = ws.query("service", 10);
        assert_eq!(results.len(), 2);

        // Project A should have 2 message matches (both mention "service")
        let pa = &results[0];
        assert_eq!(pa.context_label, "Project A");
        assert_eq!(pa.matches.len(), 2);
        assert!(pa.matches.iter().all(|m| m.match_type == "message"));

        // Project B should have 1 message match
        let pb = &results[1];
        assert_eq!(pb.context_label, "Project B");
        assert_eq!(pb.matches.len(), 1);
    }

    #[test]
    fn test_workspace_compare_item() {
        let (_tmp1, path1) = create_test_store(
            "cmp-a",
            &[("deploy-channel", &["deployer", "reviewer"])],
            &[("deployer", "starting deploy"), ("reviewer", "deploy approved")],
        );
        let (_tmp2, path2) = create_test_store(
            "cmp-b",
            &[("chat", &["alice"])],
            &[("alice", "nothing relevant")],
        );

        let mut ws = CommWorkspace::new("compare-test");
        ws.add_context(&path1, Some("Deploy Env"), WorkspaceRole::Primary)
            .unwrap();
        ws.add_context(&path2, Some("Chat Env"), WorkspaceRole::Secondary)
            .unwrap();

        let comparison = ws.compare("deploy", 50);
        assert_eq!(comparison.item, "deploy");
        assert_eq!(comparison.contexts.len(), 2);

        let deploy_env = &comparison.contexts[0];
        assert_eq!(deploy_env.context_label, "Deploy Env");
        assert!(deploy_env.found);
        assert!(deploy_env.count >= 3); // 2 messages + 1 channel name match + agents
        assert!(deploy_env.details.is_some());

        let chat_env = &comparison.contexts[1];
        assert_eq!(chat_env.context_label, "Chat Env");
        assert!(!chat_env.found);
        assert_eq!(chat_env.count, 0);
    }

    #[test]
    fn test_workspace_xref() {
        let (_tmp1, path1) = create_test_store(
            "xref-a",
            &[("general", &["alice", "bob"])],
            &[("alice", "hello bob")],
        );
        let (_tmp2, path2) = create_test_store(
            "xref-b",
            &[("team", &["bob", "charlie"])],
            &[("bob", "message from bob")],
        );
        let (_tmp3, path3) = create_test_store(
            "xref-c",
            &[("private", &["dave"])],
            &[("dave", "no match here")],
        );

        let mut ws = CommWorkspace::new("xref-test");
        ws.add_context(&path1, Some("Store 1"), WorkspaceRole::Primary)
            .unwrap();
        ws.add_context(&path2, Some("Store 2"), WorkspaceRole::Secondary)
            .unwrap();
        ws.add_context(&path3, Some("Store 3"), WorkspaceRole::Archive)
            .unwrap();

        let xrefs = ws.xref("bob");
        assert_eq!(xrefs.len(), 3);

        // Store 1: "bob" is a participant + mentioned in message content
        assert_eq!(xrefs[0].0, "Store 1");
        assert!(xrefs[0].1); // found
        assert!(xrefs[0].2 >= 2); // agent + message

        // Store 2: "bob" is a participant + sender
        assert_eq!(xrefs[1].0, "Store 2");
        assert!(xrefs[1].1);
        assert!(xrefs[1].2 >= 2);

        // Store 3: no "bob"
        assert_eq!(xrefs[2].0, "Store 3");
        assert!(!xrefs[2].1);
        assert_eq!(xrefs[2].2, 0);
    }

    #[test]
    fn test_workspace_role_display_and_parse() {
        assert_eq!(WorkspaceRole::Primary.to_string(), "primary");
        assert_eq!(WorkspaceRole::Secondary.to_string(), "secondary");
        assert_eq!(WorkspaceRole::Reference.to_string(), "reference");
        assert_eq!(WorkspaceRole::Archive.to_string(), "archive");

        assert_eq!("primary".parse::<WorkspaceRole>().unwrap(), WorkspaceRole::Primary);
        assert_eq!("SECONDARY".parse::<WorkspaceRole>().unwrap(), WorkspaceRole::Secondary);
        assert_eq!("Reference".parse::<WorkspaceRole>().unwrap(), WorkspaceRole::Reference);
        assert_eq!("archive".parse::<WorkspaceRole>().unwrap(), WorkspaceRole::Archive);

        assert!("invalid".parse::<WorkspaceRole>().is_err());
    }

    #[test]
    fn test_workspace_add_context_bad_path() {
        let mut ws = CommWorkspace::new("bad-path-test");
        let result = ws.add_context("/nonexistent/path.acomm", None, WorkspaceRole::Primary);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to load"));
    }

    #[test]
    fn test_workspace_empty_query() {
        let ws = CommWorkspace::new("empty-test");
        let results = ws.query("anything", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_workspace_query_max_per_context_limit() {
        let (_tmp, path) = create_test_store(
            "limit-test",
            &[("ch", &["agent"])],
            &[
                ("agent", "alpha msg"),
                ("agent", "alpha two"),
                ("agent", "alpha three"),
                ("agent", "alpha four"),
                ("agent", "alpha five"),
            ],
        );

        let mut ws = CommWorkspace::new("limit-test");
        ws.add_context(&path, Some("Big"), WorkspaceRole::Primary)
            .unwrap();

        // With max_per_context = 2, should get at most 2 matches
        let results = ws.query("alpha", 2);
        assert_eq!(results.len(), 1);
        assert!(results[0].matches.len() <= 2);
    }
}
