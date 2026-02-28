//! SLA target assertion tests.
//!
//! These tests verify that critical operations meet the performance targets
//! defined in the research paper specification:
//!
//!   - Message send           < 10 us
//!   - Channel create         < 1 ms
//!   - File save  (10K msgs)  < 500 ms
//!   - File load  (10K msgs)  < 200 ms
//!   - Search     (10K msgs)  < 100 ms
//!
//! Each test runs the operation multiple times and checks the median against
//! the target. We use generous warm-up and multiple iterations to reduce
//! flakiness in CI environments.
//!
//! **Debug vs Release**: Debug builds are significantly slower due to lack of
//! optimisations. The SLA multiplier is 10x in debug mode.

use std::time::{Duration, Instant};

use agentic_comm::{ChannelType, CommStore, MessageType};

/// In debug mode, operations are ~10x slower. Apply this multiplier to SLA targets.
#[cfg(debug_assertions)]
const SLA_MULTIPLIER: u32 = 10;
#[cfg(not(debug_assertions))]
const SLA_MULTIPLIER: u32 = 1;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a CommStore with rate limits disabled for benchmark-grade throughput.
fn unlocked_store() -> CommStore {
    let mut store = CommStore::new();
    store.rate_limit_config.messages_per_minute = u32::MAX;
    store
}

/// Run `f` for `iterations` times and return the median duration.
fn median_duration<F: FnMut()>(mut f: F, iterations: usize) -> Duration {
    let mut times = Vec::with_capacity(iterations);
    // Warm up
    for _ in 0..3 {
        f();
    }
    // Measure
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        times.push(start.elapsed());
    }
    times.sort();
    times[times.len() / 2]
}

/// Build a store pre-loaded with `n` messages.
fn store_with_messages(n: usize) -> (CommStore, u64) {
    let mut store = unlocked_store();
    let ch = store
        .create_channel("sla", ChannelType::Direct, None)
        .unwrap();
    let ch_id = ch.id;
    for i in 0..n {
        store
            .send_message(
                ch_id,
                &format!("agent-{}", i % 10),
                &format!("message number {} with some realistic content payload", i),
                MessageType::Text,
            )
            .unwrap();
    }
    (store, ch_id)
}

// ---------------------------------------------------------------------------
// SLA: Message send < 10 us
// ---------------------------------------------------------------------------

#[test]
fn sla_message_send_under_10us() {
    let mut store = unlocked_store();
    let ch = store
        .create_channel("sla-send", ChannelType::Direct, None)
        .unwrap();
    let ch_id = ch.id;
    let mut counter = 0u64;

    let median = median_duration(
        || {
            store
                .send_message(
                    ch_id,
                    "sender",
                    &format!("sla msg {}", counter),
                    MessageType::Text,
                )
                .unwrap();
            counter += 1;
        },
        200,
    );

    let target = Duration::from_micros(10 * SLA_MULTIPLIER as u64);
    println!(
        "SLA message_send: median={:?}, target={:?} (multiplier={}x)",
        median, target, SLA_MULTIPLIER
    );
    assert!(
        median < target,
        "Message send median {:?} exceeds SLA target {:?}",
        median,
        target
    );
}

// ---------------------------------------------------------------------------
// SLA: Channel create < 1 ms
// ---------------------------------------------------------------------------

#[test]
fn sla_channel_create_under_1ms() {
    let mut store = unlocked_store();
    let mut counter = 0u64;

    let median = median_duration(
        || {
            store
                .create_channel(
                    &format!("sla-ch-{}", counter),
                    ChannelType::Direct,
                    None,
                )
                .unwrap();
            counter += 1;
        },
        200,
    );

    let target = Duration::from_millis(1 * SLA_MULTIPLIER as u64);
    println!(
        "SLA channel_create: median={:?}, target={:?} (multiplier={}x)",
        median, target, SLA_MULTIPLIER
    );
    assert!(
        median < target,
        "Channel create median {:?} exceeds SLA target {:?}",
        median,
        target
    );
}

// ---------------------------------------------------------------------------
// SLA: File save (10K messages) < 500 ms
// ---------------------------------------------------------------------------

#[test]
fn sla_file_save_10k_under_500ms() {
    let (store, _) = store_with_messages(10_000);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sla.acomm");

    let median = median_duration(
        || {
            store.save(&path).unwrap();
        },
        5,
    );

    let target = Duration::from_millis(500 * SLA_MULTIPLIER as u64);
    println!(
        "SLA file_save_10k: median={:?}, target={:?} (multiplier={}x)",
        median, target, SLA_MULTIPLIER
    );
    assert!(
        median < target,
        "File save (10K) median {:?} exceeds SLA target {:?}",
        median,
        target
    );
}

// ---------------------------------------------------------------------------
// SLA: File load (10K messages) < 200 ms
// ---------------------------------------------------------------------------

#[test]
fn sla_file_load_10k_under_200ms() {
    let (store, _) = store_with_messages(10_000);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sla.acomm");
    store.save(&path).unwrap();

    let median = median_duration(
        || {
            let _ = CommStore::load(&path).unwrap();
        },
        5,
    );

    let target = Duration::from_millis(200 * SLA_MULTIPLIER as u64);
    println!(
        "SLA file_load_10k: median={:?}, target={:?} (multiplier={}x)",
        median, target, SLA_MULTIPLIER
    );
    assert!(
        median < target,
        "File load (10K) median {:?} exceeds SLA target {:?}",
        median,
        target
    );
}

// ---------------------------------------------------------------------------
// SLA: Search (10K messages) < 100 ms
// ---------------------------------------------------------------------------

#[test]
fn sla_search_10k_under_100ms() {
    let (store, _) = store_with_messages(10_000);

    let median = median_duration(
        || {
            let _ = store.search_messages("number 5000", 10);
        },
        20,
    );

    let target = Duration::from_millis(100 * SLA_MULTIPLIER as u64);
    println!(
        "SLA search_10k: median={:?}, target={:?} (multiplier={}x)",
        median, target, SLA_MULTIPLIER
    );
    assert!(
        median < target,
        "Search (10K) median {:?} exceeds SLA target {:?}",
        median,
        target
    );
}
