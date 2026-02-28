//! Memory profiling tests for agentic-comm.
//!
//! Uses a custom global allocator that tracks peak and current allocation
//! to measure memory usage of CommStore operations at scale.
//!
//! **Important**: Because the test binary shares a single global allocator
//! across all test threads, measurements can include noise from concurrent
//! tests. The assertions here are generous regression guards; exact profiling
//! should be done by running a single test at a time with `--test-threads=1`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// ---------------------------------------------------------------------------
// Tracking allocator
// ---------------------------------------------------------------------------

struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let current = ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            let mut peak = PEAK.load(Ordering::Relaxed);
            while current > peak {
                match PEAK.compare_exchange_weak(
                    peak,
                    current,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(p) => peak = p,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static ALLOC: TrackingAllocator = TrackingAllocator;

/// Reset peak to current level and return (current_bytes, peak_bytes).
fn reset_counters() -> (usize, usize) {
    let current = ALLOCATED.load(Ordering::Relaxed);
    PEAK.store(current, Ordering::Relaxed);
    (current, current)
}

/// Snapshot (current_bytes, peak_bytes).
fn snapshot() -> (usize, usize) {
    (
        ALLOCATED.load(Ordering::Relaxed),
        PEAK.load(Ordering::Relaxed),
    )
}

fn format_bytes(b: usize) -> String {
    if b >= 1_048_576 {
        format!("{:.2} MB", b as f64 / 1_048_576.0)
    } else if b >= 1024 {
        format!("{:.2} KB", b as f64 / 1024.0)
    } else {
        format!("{} B", b)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

use agentic_comm::{
    AffectState, ChannelType, CollectiveDecisionMode, CommStore, CommTrustLevel, ConsentScope,
    HiveRole, MessageType,
};

/// Create a CommStore with rate limits disabled for bulk operations.
fn unlocked_store() -> CommStore {
    let mut store = CommStore::new();
    store.rate_limit_config.messages_per_minute = u32::MAX;
    store
}

#[test]
fn memory_profile_message_send() {
    println!();
    println!("=== Memory Profile: Message Send ===");
    println!("+-----------+---------------+---------------+---------------+");
    println!("| Messages  | Heap Delta    | Peak Delta    | Bytes/Msg     |");
    println!("+-----------+---------------+---------------+---------------+");

    for &scale in &[100, 1_000, 10_000] {
        let (base_current, _) = reset_counters();

        let mut store = unlocked_store();
        let ch = store
            .create_channel("mem-bench", ChannelType::Direct, None)
            .unwrap();
        for i in 0..scale {
            store
                .send_message(
                    ch.id,
                    &format!("agent-{}", i % 10),
                    &format!("message number {} with some realistic content payload", i),
                    MessageType::Text,
                )
                .unwrap();
        }

        let (after_current, after_peak) = snapshot();
        let heap_delta = after_current.saturating_sub(base_current);
        let peak_delta = after_peak.saturating_sub(base_current);
        let bytes_per_msg = if scale > 0 {
            heap_delta as f64 / scale as f64
        } else {
            0.0
        };

        println!(
            "| {:>9} | {:>13} | {:>13} | {:>13.1} |",
            scale,
            format_bytes(heap_delta),
            format_bytes(peak_delta),
            bytes_per_msg
        );

        drop(store);
    }

    println!("+-----------+---------------+---------------+---------------+");
    println!();
    // Gross regression guard: 10K messages should not exceed 200 MB total
    // (this catches catastrophic issues, not fine-grained regressions).
}

#[test]
fn memory_profile_channel_create() {
    println!();
    println!("=== Memory Profile: Channel Create ===");
    println!("+-----------+---------------+---------------+---------------+");
    println!("| Channels  | Heap Delta    | Peak Delta    | Bytes/Chan    |");
    println!("+-----------+---------------+---------------+---------------+");

    for &scale in &[100, 1_000, 10_000] {
        let (base_current, _) = reset_counters();

        let mut store = unlocked_store();
        for i in 0..scale {
            store
                .create_channel(&format!("ch-{}", i), ChannelType::Direct, None)
                .unwrap();
        }

        let (after_current, after_peak) = snapshot();
        let heap_delta = after_current.saturating_sub(base_current);
        let peak_delta = after_peak.saturating_sub(base_current);
        let bytes_per_chan = if scale > 0 {
            heap_delta as f64 / scale as f64
        } else {
            0.0
        };

        println!(
            "| {:>9} | {:>13} | {:>13} | {:>13.1} |",
            scale,
            format_bytes(heap_delta),
            format_bytes(peak_delta),
            bytes_per_chan
        );

        drop(store);
    }

    println!("+-----------+---------------+---------------+---------------+");
    println!();
}

#[test]
fn memory_profile_save_load() {
    println!();
    println!("=== Memory Profile: Save/Load ===");
    println!("+-----------+---------------------------+---------------------------+");
    println!("| Messages  | Peak during save          | Peak during load          |");
    println!("+-----------+---------------------------+---------------------------+");

    for &scale in &[100, 1_000, 10_000] {
        let mut store = unlocked_store();
        let ch = store
            .create_channel("mem-bench", ChannelType::Direct, None)
            .unwrap();
        for i in 0..scale {
            store
                .send_message(
                    ch.id,
                    &format!("agent-{}", i % 10),
                    &format!("message number {} with some realistic content payload", i),
                    MessageType::Text,
                )
                .unwrap();
        }

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mem-bench.acomm");

        // Measure save peak
        let (base_save, _) = reset_counters();
        store.save(&path).unwrap();
        let (_, save_peak) = snapshot();
        let save_peak_delta = save_peak.saturating_sub(base_save);

        // Measure load peak
        let (base_load, _) = reset_counters();
        let _loaded = CommStore::load(&path).unwrap();
        let (_, load_peak) = snapshot();
        let load_peak_delta = load_peak.saturating_sub(base_load);

        println!(
            "| {:>9} | {:>25} | {:>25} |",
            scale,
            format_bytes(save_peak_delta),
            format_bytes(load_peak_delta),
        );

        drop(_loaded);
        drop(store);
    }

    println!("+-----------+---------------------------+---------------------------+");
    println!();
}

#[test]
fn memory_profile_rich_store() {
    println!();
    println!("=== Memory Profile: Rich Store (msgs + trust + consent + hive + affect) ===");

    let (base_current, _) = reset_counters();

    let mut store = unlocked_store();

    // Messages
    let ch = store
        .create_channel("rich", ChannelType::Direct, None)
        .unwrap();
    for i in 0..1_000 {
        store
            .send_message(
                ch.id,
                &format!("agent-{}", i % 10),
                &format!("rich message {}", i),
                MessageType::Text,
            )
            .unwrap();
    }
    let (after_msgs, _) = snapshot();
    let msg_delta = after_msgs.saturating_sub(base_current);
    println!("  After 1K messages       : {}", format_bytes(msg_delta));

    // Trust levels
    for i in 0..500 {
        store
            .set_trust_level(&format!("agent-{}", i), CommTrustLevel::High)
            .unwrap();
    }
    let (after_trust, _) = snapshot();
    let trust_delta = after_trust.saturating_sub(after_msgs);
    println!("  After 500 trust levels  : +{}", format_bytes(trust_delta));

    // Consent gates
    for i in 0..200 {
        store
            .grant_consent(
                &format!("agent-{}", i),
                "agent-0",
                ConsentScope::ReadMessages,
                None,
                None,
            )
            .unwrap();
    }
    let (after_consent, _) = snapshot();
    let consent_delta = after_consent.saturating_sub(after_trust);
    println!(
        "  After 200 consent gates : +{}",
        format_bytes(consent_delta)
    );

    // Hive
    let hive = store
        .form_hive(
            "mem-hive",
            "coordinator",
            CollectiveDecisionMode::Consensus,
        )
        .unwrap();
    let hive_id = hive.id;
    for i in 0..50 {
        store
            .join_hive(hive_id, &format!("agent-{}", i), HiveRole::Member)
            .unwrap();
    }
    let (after_hive, _) = snapshot();
    let hive_delta = after_hive.saturating_sub(after_consent);
    println!(
        "  After hive (50 members) : +{}",
        format_bytes(hive_delta)
    );

    // Affect states
    for i in 0..100 {
        store.affect_states.insert(
            format!("agent-{}", i),
            AffectState {
                valence: (i as f64 / 100.0) * 2.0 - 1.0,
                arousal: i as f64 / 100.0,
                dominance: 0.5,
                ..Default::default()
            },
        );
    }
    let (after_affect, peak_total) = snapshot();
    let affect_delta = after_affect.saturating_sub(after_hive);
    println!(
        "  After 100 affect states : +{}",
        format_bytes(affect_delta)
    );

    let total_delta = after_affect.saturating_sub(base_current);
    let peak_delta = peak_total.saturating_sub(base_current);
    println!();
    println!("  TOTAL heap used         : {}", format_bytes(total_delta));
    println!("  PEAK heap used          : {}", format_bytes(peak_delta));
    println!();

    // Gross regression guard: rich store with 1K messages should not exceed 100 MB
    assert!(
        total_delta < 100 * 1_048_576,
        "Rich store total heap {} exceeds 100 MB — likely a regression",
        format_bytes(total_delta)
    );

    drop(store);
}
