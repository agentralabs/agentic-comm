//! Ghost Writer Bridge — Syncs communication context to AI coding assistants.
//!
//! Detects Claude Code, Cursor, Windsurf, and Cody, then periodically writes
//! a communication context summary to each client's memory directory.
//!
//! Uses an async tokio background task with a 5-second sync interval,
//! matching the pattern used by memory and vision sisters.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::Mutex;

use crate::session::manager::SessionManager;

/// Spawn a background tokio task that periodically syncs communication
/// context to all detected AI coding assistant memory directories.
///
/// Returns `None` if no AI clients are detected (comm still works via MCP tools).
pub fn spawn_ghost_writer(
    session: Arc<Mutex<SessionManager>>,
) -> Option<tokio::task::JoinHandle<()>> {
    let clients = detect_all_memory_dirs();
    if clients.is_empty() {
        eprintln!("[ghost_bridge] No AI coding assistants detected. Sync disabled.");
        return None;
    }

    let client_count = clients.len();
    for c in &clients {
        eprintln!("[ghost_bridge] Comm context: {} at {:?}", c.name, c.dir);
    }

    let handle = tokio::spawn(async move {
        // First sync immediately
        sync_once(&session, &clients).await;

        // Then sync every 5 seconds
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        interval.tick().await; // consume the first (immediate) tick
        loop {
            interval.tick().await;
            sync_once(&session, &clients).await;
        }
    });

    eprintln!(
        "[ghost_bridge] Background sync started ({} clients, 5s interval)",
        client_count
    );
    Some(handle)
}

/// Perform one sync cycle — build context from session, write to all clients.
async fn sync_once(session: &Arc<Mutex<SessionManager>>, clients: &[ClientDir]) {
    let markdown = {
        let session = session.lock().await;
        build_comm_context(&session)
    };

    // FNV-1a hash for dedup — we use a static to track the last hash
    // across invocations. This is safe because the ghost writer task is
    // single-threaded.
    static LAST_HASH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    let hash = fnv1a_hash(&markdown);
    let prev = LAST_HASH.load(std::sync::atomic::Ordering::Relaxed);
    if hash == prev {
        return;
    }
    LAST_HASH.store(hash, std::sync::atomic::Ordering::Relaxed);

    for client in clients {
        let target = client.dir.join(&client.filename);
        if let Err(e) = atomic_write(&target, markdown.as_bytes()) {
            eprintln!("[ghost_bridge] Failed to sync to {:?}: {e}", target);
        }
    }
}

fn build_comm_context(session: &SessionManager) -> String {
    let store = &session.store;
    let now = now_utc_string();

    let mut md = String::new();
    md.push_str("# AgenticComm Context\n\n");
    md.push_str(&format!("> Auto-synced by Ghost Writer at {now}\n\n"));

    // Overview stats
    md.push_str("## Overview\n\n");
    md.push_str(&format!(
        "| Metric | Count |\n|--------|-------|\n\
         | Channels | {} |\n\
         | Messages | {} |\n\
         | Subscriptions | {} |\n\
         | Dead Letters | {} |\n\
         | Hive Minds | {} |\n\
         | Trust Levels | {} |\n\
         | Consent Gates | {} |\n\n",
        store.channels.len(),
        store.messages.len(),
        store.subscriptions.len(),
        store.dead_letters.len(),
        store.hive_minds.len(),
        store.trust_levels.len(),
        store.consent_gates.len(),
    ));

    // Active channels
    if !store.channels.is_empty() {
        md.push_str("## Channels\n\n");
        md.push_str("| ID | Name | Type | Participants | State |\n");
        md.push_str("|----|------|------|-------------|-------|\n");
        for ch in store.channels.values().take(15) {
            md.push_str(&format!(
                "| {} | {} | {:?} | {} | {:?} |\n",
                ch.id,
                truncate(&ch.name, 30),
                ch.channel_type,
                ch.participants.len(),
                ch.state,
            ));
        }
        if store.channels.len() > 15 {
            md.push_str(&format!(
                "| | _...and {} more_ | | | |\n",
                store.channels.len() - 15
            ));
        }
        md.push('\n');
    }

    // Recent messages (last 10)
    if !store.messages.is_empty() {
        md.push_str("## Recent Messages\n\n");
        let mut msgs: Vec<_> = store.messages.values().collect();
        msgs.sort_by(|a, b| b.id.cmp(&a.id));
        for m in msgs.iter().take(10) {
            let recipient = m.recipient.as_deref().unwrap_or("broadcast");
            md.push_str(&format!(
                "- [ch:{}] **{}** → {}: {}\n",
                m.channel_id,
                truncate(&m.sender, 20),
                truncate(recipient, 20),
                truncate(&m.content, 60),
            ));
        }
        if store.messages.len() > 10 {
            md.push_str(&format!(
                "- _...and {} more messages_\n",
                store.messages.len() - 10
            ));
        }
        md.push('\n');
    }

    // Active hive minds
    if !store.hive_minds.is_empty() {
        md.push_str("## Hive Minds\n\n");
        for hive in store.hive_minds.values().take(5) {
            md.push_str(&format!(
                "- **{}** (id:{}, {} members)\n",
                truncate(&hive.name, 30),
                hive.id,
                hive.constituents.len(),
            ));
        }
        md.push('\n');
    }

    // Dead letters (if any)
    if !store.dead_letters.is_empty() {
        md.push_str(&format!(
            "## Dead Letters: {} undelivered\n\n",
            store.dead_letters.len()
        ));
    }

    // Operation log (recent tool calls)
    if !session.operation_log.is_empty() {
        md.push_str("## Recent Operations\n\n");
        for record in session.operation_log.iter().rev().take(15) {
            md.push_str(&format!(
                "- `{}` at {}\n",
                record.tool_name, record.timestamp
            ));
        }
        if session.operation_log.len() > 15 {
            md.push_str(&format!(
                "- _...and {} more_\n",
                session.operation_log.len() - 15
            ));
        }
        md.push('\n');
    }

    md.push_str("---\n");
    md.push_str("_Auto-generated by AgenticComm. Do not edit manually._\n");
    md
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max])
    } else {
        s.to_string()
    }
}

fn fnv1a_hash(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn now_utc_string() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let s = secs % 60;
    let min = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let z = (secs / 86400) as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{min:02}:{s:02} UTC")
}

// ═══════════════════════════════════════════════════════════════════
// Multi-client detection
// ═══════════════════════════════════════════════════════════════════

struct ClientDir {
    name: &'static str,
    dir: PathBuf,
    filename: String,
}

fn detect_all_memory_dirs() -> Vec<ClientDir> {
    let home = match std::env::var("HOME").ok().map(PathBuf::from) {
        Some(h) => h,
        None => return vec![],
    };

    let candidates = [
        (
            "Claude Code",
            home.join(".claude").join("memory"),
            "COMM_CONTEXT.md",
        ),
        (
            "Cursor",
            home.join(".cursor").join("memory"),
            "agentic-comm.md",
        ),
        (
            "Windsurf",
            home.join(".windsurf").join("memory"),
            "agentic-comm.md",
        ),
        (
            "Cody",
            home.join(".sourcegraph").join("cody").join("memory"),
            "agentic-comm.md",
        ),
    ];

    let mut dirs = Vec::new();
    for (name, memory_dir, filename) in &candidates {
        if create_if_parent_exists(memory_dir) {
            dirs.push(ClientDir {
                name,
                dir: memory_dir.clone(),
                filename: filename.to_string(),
            });
        }
    }
    dirs
}

fn create_if_parent_exists(memory_dir: &Path) -> bool {
    if memory_dir.exists() {
        return true;
    }
    if let Some(parent) = memory_dir.parent() {
        if parent.exists() {
            return std::fs::create_dir_all(memory_dir).is_ok();
        }
    }
    false
}

fn atomic_write(target: &Path, content: &[u8]) -> Result<(), std::io::Error> {
    let tmp = target.with_extension("tmp");
    let mut f = std::fs::File::create(&tmp)?;
    f.write_all(content)?;
    f.sync_all()?;
    std::fs::rename(&tmp, target)?;
    Ok(())
}
