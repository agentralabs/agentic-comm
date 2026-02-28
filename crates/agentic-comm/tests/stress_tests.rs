//! SPEC-PART4 §2.3 — Stress tests for the CommStore engine.
//!
//! Validates performance and correctness under high-volume scenarios:
//! large message counts, many channels, deep threads, bulk consent,
//! large hives, federation zones, temporal scheduling, and save/load.

use agentic_comm::*;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Stress 01: 10,000 messages in a single channel
// ---------------------------------------------------------------------------

#[test]
fn stress_10k_messages_single_channel() {
    let mut store = CommStore::new();
    store.rate_limit_config.messages_per_minute = u32::MAX;
    let ch = store
        .create_channel("stress-ch", ChannelType::Direct, None)
        .unwrap();

    let start = Instant::now();
    for i in 0..10_000 {
        store
            .send_message(
                ch.id,
                "stressor",
                &format!("stress msg {}", i),
                MessageType::Text,
            )
            .unwrap();
    }
    let elapsed = start.elapsed();

    let msgs = store.receive_messages(ch.id, None, None).unwrap();
    assert_eq!(msgs.len(), 10_000);
    println!("10K messages: {:?}", elapsed);
}

// ---------------------------------------------------------------------------
// Stress 02: 1,000 channels
// ---------------------------------------------------------------------------

#[test]
fn stress_1k_channels() {
    let mut store = CommStore::new();

    let start = Instant::now();
    for i in 0..1_000 {
        store
            .create_channel(&format!("stress-ch-{}", i), ChannelType::Direct, None)
            .unwrap();
    }
    let elapsed = start.elapsed();

    let channels = store.list_channels();
    assert_eq!(channels.len(), 1_000);
    println!("1K channels: {:?}", elapsed);
}

// ---------------------------------------------------------------------------
// Stress 03: 10 senders x 100 messages each = 1,000
// ---------------------------------------------------------------------------

#[test]
fn stress_concurrent_senders_100_each() {
    let mut store = CommStore::new();
    store.rate_limit_config.messages_per_minute = u32::MAX;
    let ch = store
        .create_channel("multi-sender", ChannelType::Direct, None)
        .unwrap();

    for sender_id in 0..10 {
        for msg_id in 0..100 {
            store
                .send_message(
                    ch.id,
                    &format!("sender_{}", sender_id),
                    &format!("s{}-m{}", sender_id, msg_id),
                    MessageType::Text,
                )
                .unwrap();
        }
    }

    let msgs = store.receive_messages(ch.id, None, None).unwrap();
    assert_eq!(msgs.len(), 1_000);
}

// ---------------------------------------------------------------------------
// Stress 04: 1 MB message
// ---------------------------------------------------------------------------

#[test]
fn stress_large_message_1mb() {
    let mut store = CommStore::new();
    let ch = store
        .create_channel("large-msg", ChannelType::Direct, None)
        .unwrap();

    let large_content = "x".repeat(1_000_000);
    let msg = store
        .send_message(ch.id, "sender", &large_content, MessageType::Text)
        .unwrap();

    let fetched = store.get_message(msg.id).expect("message should exist");
    assert_eq!(fetched.content.len(), 1_000_000);
}

// ---------------------------------------------------------------------------
// Stress 05: 1,000 consent gates + lookup performance
// ---------------------------------------------------------------------------

#[test]
fn stress_many_consent_gates() {
    let mut store = CommStore::new();

    for i in 0..1_000 {
        store
            .grant_consent(
                &format!("agent_{}", i),
                "reader",
                ConsentScope::ReadMessages,
                None,
                None,
            )
            .unwrap();
    }

    let gates = store.list_consent_gates(None);
    assert_eq!(gates.len(), 1_000);

    // Check performance of consent lookup
    let start = Instant::now();
    for i in 0..1_000 {
        store.check_consent(
            &format!("agent_{}", i),
            "reader",
            &ConsentScope::ReadMessages,
        );
    }
    let elapsed = start.elapsed();
    println!("1K consent checks: {:?}", elapsed);
}

// ---------------------------------------------------------------------------
// Stress 06: Hive with 50 members
// ---------------------------------------------------------------------------

#[test]
fn stress_many_hive_members() {
    let mut store = CommStore::new();
    let hive = store
        .form_hive("mega-hive", "coordinator_0", CollectiveDecisionMode::Consensus)
        .unwrap();
    let hive_id = hive.id;

    for i in 0..50 {
        store
            .join_hive(hive_id, &format!("agent_{}", i), HiveRole::Member)
            .unwrap();
    }

    let h = store.get_hive(hive_id).expect("hive should exist");
    assert_eq!(h.constituents.len(), 51); // 50 members + 1 coordinator
}

// ---------------------------------------------------------------------------
// Stress 07: Save & load a large store (100 channels, 50 msgs each)
// ---------------------------------------------------------------------------

#[test]
fn stress_save_load_large_store() {
    let mut store = CommStore::new();
    store.rate_limit_config.messages_per_minute = u32::MAX;

    for ch_i in 0..100 {
        let ch = store
            .create_channel(&format!("ch-{}", ch_i), ChannelType::Direct, None)
            .unwrap();
        for msg_i in 0..50 {
            store
                .send_message(
                    ch.id,
                    "sender",
                    &format!("ch{}-msg{}", ch_i, msg_i),
                    MessageType::Text,
                )
                .unwrap();
        }
    }

    // Save and load
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("stress.acomm");

    let start = Instant::now();
    store.save(&path).unwrap();
    let save_elapsed = start.elapsed();

    let start = Instant::now();
    let loaded = CommStore::load(&path).unwrap();
    let load_elapsed = start.elapsed();

    assert_eq!(loaded.list_channels().len(), 100);
    println!(
        "Save 5K msgs: {:?}, Load: {:?}",
        save_elapsed, load_elapsed
    );
}

// ---------------------------------------------------------------------------
// Stress 08: 100-deep reply chain
// ---------------------------------------------------------------------------

#[test]
fn stress_deep_thread_chain() {
    let mut store = CommStore::new();
    let ch = store
        .create_channel("deep-thread", ChannelType::Direct, None)
        .unwrap();

    let root = store
        .send_message(ch.id, "agent_a", "root message", MessageType::Text)
        .unwrap();
    let mut prev_id = root.id;

    for i in 1..100 {
        let sender = if i % 2 == 0 { "agent_a" } else { "agent_b" };
        let reply = store
            .send_reply(
                ch.id,
                prev_id,
                sender,
                &format!("reply depth {}", i),
                MessageType::Response,
            )
            .unwrap();
        prev_id = reply.id;
    }

    // The thread_id is derived from the root message: "thread-{root_id}"
    let thread_id = format!("thread-{}", root.id);
    let thread = store.get_thread(&thread_id);
    // Root + 99 replies = 100 messages in the thread
    assert_eq!(thread.len(), 100);
}

// ---------------------------------------------------------------------------
// Stress 09: 20 federated zones
// ---------------------------------------------------------------------------

#[test]
fn stress_federation_many_zones() {
    let mut store = CommStore::new();
    store
        .configure_federation(true, "test-net", FederationPolicy::Allow)
        .unwrap();

    for i in 0..20 {
        store
            .add_federated_zone(FederatedZone {
                zone_id: format!("zone_{}", i),
                name: format!("Zone {}", i),
                endpoint: format!("gateway_{}.example.com", i),
                policy: FederationPolicy::Allow,
                trust_level: CommTrustLevel::Standard,
            })
            .unwrap();
    }

    let zones = store.list_federated_zones();
    assert_eq!(zones.len(), 20);
}

// ---------------------------------------------------------------------------
// Stress 10: 100 scheduled (temporal) messages
// ---------------------------------------------------------------------------

#[test]
fn stress_temporal_queue_100_scheduled() {
    let mut store = CommStore::new();
    let ch = store
        .create_channel("temporal-stress", ChannelType::Direct, None)
        .unwrap();

    for i in 0..100 {
        store
            .schedule_message(
                ch.id,
                "sender",
                &format!("scheduled {}", i),
                TemporalTarget::Immediate,
                None,
            )
            .unwrap();
    }

    let scheduled = store.list_scheduled();
    assert!(scheduled.len() >= 100);
}
