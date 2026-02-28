//! Pipeline Tests
//!
//! Integration tests for the affect contagion pipeline, message forwarding
//! with echo tracking, and conversation summarization features.

use agentic_comm::*;

// ---------------------------------------------------------------------------
// Helper: create a store with a group channel and two participants.
// ---------------------------------------------------------------------------

fn setup_pipeline_store() -> (CommStore, u64) {
    let mut store = CommStore::new();
    let ch = store
        .create_channel("pipeline-test", ChannelType::Group, None)
        .expect("channel creation should succeed");
    store.join_channel(ch.id, "alice").unwrap();
    store.join_channel(ch.id, "bob").unwrap();
    (store, ch.id)
}

/// Helper: send a message with affect metadata (valence, arousal, dominance).
fn send_affect_metadata_message(
    store: &mut CommStore,
    channel_id: u64,
    sender: &str,
    content: &str,
    valence: f64,
    arousal: f64,
    dominance: f64,
) -> u64 {
    let msg = store
        .send_message(channel_id, sender, content, MessageType::Text)
        .expect("send_message should succeed");
    let id = msg.id;
    // Set affect metadata on the stored message
    if let Some(m) = store.messages.get_mut(&id) {
        m.metadata
            .insert("valence".to_string(), valence.to_string());
        m.metadata
            .insert("arousal".to_string(), arousal.to_string());
        m.metadata
            .insert("dominance".to_string(), dominance.to_string());
    }
    id
}

// ===========================================================================
// 1. Affect Contagion — basic
// ===========================================================================

#[test]
fn test_affect_contagion_basic() {
    let (mut store, ch_id) = setup_pipeline_store();

    // Set low resistance so contagion has a large effect
    store.set_affect_resistance(0.0);

    // Alice sends a message with positive valence
    send_affect_metadata_message(&mut store, ch_id, "alice", "Exciting news!", 0.8, 0.9, 0.7);

    // Process contagion
    let results = store.process_affect_contagion(ch_id);

    // Bob should be affected
    assert!(!results.is_empty(), "contagion should produce results");

    let bob_result = results.iter().find(|(agent, _, _, _)| agent == "bob");
    assert!(bob_result.is_some(), "bob should be affected by contagion");

    let (_, valence, arousal, dominance) = bob_result.unwrap();
    // With resistance=0.0, bob's state should move toward alice's values
    assert!(*valence > 0.0, "bob's valence should be positive");
    assert!(*arousal > 0.0, "bob's arousal should be positive");
    assert!(*dominance > 0.0, "bob's dominance should be positive");
}

// ===========================================================================
// 2. Affect History — tracking
// ===========================================================================

#[test]
fn test_affect_history_tracking() {
    let (mut store, ch_id) = setup_pipeline_store();

    // Send messages with affect metadata from alice
    send_affect_metadata_message(&mut store, ch_id, "alice", "Message 1", 0.5, 0.3, 0.6);
    send_affect_metadata_message(&mut store, ch_id, "alice", "Message 2", 0.7, 0.8, 0.4);

    let history = store.get_affect_history("alice");
    assert_eq!(history.agent, "alice");
    // Should have entries from the messages (at least 2 from the affect metadata)
    assert!(
        history.states.len() >= 2,
        "history should have at least 2 entries from messages, got {}",
        history.states.len()
    );

    // Entries from alice's own messages should be "direct"
    let direct_entries: Vec<_> = history
        .states
        .iter()
        .filter(|e| e.source == "direct")
        .collect();
    assert!(
        direct_entries.len() >= 2,
        "should have at least 2 direct entries"
    );
}

// ===========================================================================
// 3. Affect Decay — reduces values
// ===========================================================================

#[test]
fn test_affect_decay_reduces_values() {
    let mut store = CommStore::new();

    // Manually set an affect state
    store.affect_states.insert(
        "alice".to_string(),
        types::AffectState {
            valence: 0.8,
            arousal: 0.6,
            dominance: 0.9,
            ..Default::default()
        },
    );

    // Apply decay with rate 0.5
    store.apply_affect_decay(0.5);

    let state = store.affect_states.get("alice").expect("alice should exist");
    // Values should be halved (multiplied by 1.0 - 0.5 = 0.5)
    assert!(
        (state.valence - 0.4).abs() < 0.01,
        "valence should be ~0.4, got {}",
        state.valence
    );
    assert!(
        (state.arousal - 0.3).abs() < 0.01,
        "arousal should be ~0.3, got {}",
        state.arousal
    );
    assert!(
        (state.dominance - 0.45).abs() < 0.01,
        "dominance should be ~0.45, got {}",
        state.dominance
    );
}

// ===========================================================================
// 4. Affect Decay — zero rate = no change
// ===========================================================================

#[test]
fn test_affect_decay_zero_rate_no_change() {
    let mut store = CommStore::new();

    store.affect_states.insert(
        "bob".to_string(),
        types::AffectState {
            valence: 0.5,
            arousal: 0.7,
            dominance: 0.3,
            ..Default::default()
        },
    );

    store.apply_affect_decay(0.0);

    let state = store.affect_states.get("bob").expect("bob should exist");
    assert!(
        (state.valence - 0.5).abs() < f64::EPSILON,
        "valence should not change"
    );
    assert!(
        (state.arousal - 0.7).abs() < f64::EPSILON,
        "arousal should not change"
    );
    assert!(
        (state.dominance - 0.3).abs() < f64::EPSILON,
        "dominance should not change"
    );
}

// ===========================================================================
// 5. Forward Message — basic
// ===========================================================================

#[test]
fn test_forward_message_basic() {
    let (mut store, ch_id) = setup_pipeline_store();

    // Create a second channel
    let ch2 = store
        .create_channel("target-channel", ChannelType::Group, None)
        .expect("create target channel");
    store.join_channel(ch2.id, "charlie").unwrap();

    // Send original message
    let original = store
        .send_message(ch_id, "alice", "Important announcement", MessageType::Text)
        .expect("send original");

    // Forward it
    let new_id = store
        .forward_message(original.id, ch2.id, "bob")
        .expect("forward should succeed");

    // New message should exist
    let forwarded = store.get_message(new_id).expect("forwarded message should exist");
    assert_eq!(forwarded.channel_id, ch2.id);
    assert_eq!(forwarded.sender, "bob");
    assert!(new_id != original.id, "new message should have a different ID");
}

// ===========================================================================
// 6. Forward Message — preserves content
// ===========================================================================

#[test]
fn test_forward_message_preserves_content() {
    let (mut store, ch_id) = setup_pipeline_store();

    let ch2 = store
        .create_channel("target", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch2.id, "charlie").unwrap();

    let original = store
        .send_message(ch_id, "alice", "Secret data payload", MessageType::Text)
        .unwrap();

    let new_id = store
        .forward_message(original.id, ch2.id, "bob")
        .unwrap();

    let forwarded = store.get_message(new_id).unwrap();
    // Content should include original content (prefixed with [Forwarded])
    assert!(
        forwarded.content.contains("Secret data payload"),
        "forwarded message should contain original content, got: {}",
        forwarded.content
    );
    assert!(
        forwarded.content.starts_with("[Forwarded]"),
        "forwarded message should start with [Forwarded] prefix"
    );
}

// ===========================================================================
// 7. Forward Metadata — tracked
// ===========================================================================

#[test]
fn test_forward_metadata_tracked() {
    let (mut store, ch_id) = setup_pipeline_store();

    let ch2 = store
        .create_channel("target", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch2.id, "charlie").unwrap();

    let original = store
        .send_message(ch_id, "alice", "Trackable message", MessageType::Text)
        .unwrap();

    let new_id = store.forward_message(original.id, ch2.id, "bob").unwrap();

    let forwarded = store.get_message(new_id).unwrap();
    assert_eq!(
        forwarded.metadata.get("forwarded_from"),
        Some(&original.id.to_string()),
        "forwarded_from metadata should point to original"
    );
    assert_eq!(
        forwarded.metadata.get("echo_depth"),
        Some(&"1".to_string()),
        "echo_depth should be 1 for first forward"
    );
    assert_eq!(
        forwarded.metadata.get("forwarder"),
        Some(&"bob".to_string()),
        "forwarder metadata should be set"
    );
}

// ===========================================================================
// 8. Echo Chain — single hop
// ===========================================================================

#[test]
fn test_echo_chain_single_hop() {
    let (mut store, ch_id) = setup_pipeline_store();

    let ch2 = store
        .create_channel("target", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch2.id, "charlie").unwrap();

    let original = store
        .send_message(ch_id, "alice", "Chain start", MessageType::Text)
        .unwrap();

    let fwd_id = store.forward_message(original.id, ch2.id, "bob").unwrap();

    // Query chain from the forwarded message
    let chain = store.query_echo_chain(fwd_id);
    assert!(
        chain.len() >= 2,
        "chain should include original and forwarded, got {}",
        chain.len()
    );

    // Root should be first (depth 0)
    assert_eq!(chain[0].message_id, original.id);
    assert_eq!(chain[0].depth, 0);

    // Forwarded should be second (depth 1)
    let fwd_entry = chain.iter().find(|e| e.message_id == fwd_id);
    assert!(fwd_entry.is_some(), "forwarded message should be in chain");
    assert_eq!(fwd_entry.unwrap().depth, 1);
}

// ===========================================================================
// 9. Echo Chain — multi-hop (3 forwards)
// ===========================================================================

#[test]
fn test_echo_chain_multi_hop() {
    let mut store = CommStore::new();

    // Create 4 channels
    let ch1 = store.create_channel("ch1", ChannelType::Group, None).unwrap();
    store.join_channel(ch1.id, "alice").unwrap();
    let ch2 = store.create_channel("ch2", ChannelType::Group, None).unwrap();
    store.join_channel(ch2.id, "bob").unwrap();
    let ch3 = store.create_channel("ch3", ChannelType::Group, None).unwrap();
    store.join_channel(ch3.id, "charlie").unwrap();
    let ch4 = store.create_channel("ch4", ChannelType::Group, None).unwrap();
    store.join_channel(ch4.id, "dave").unwrap();

    // Original message
    let original = store
        .send_message(ch1.id, "alice", "Viral message", MessageType::Text)
        .unwrap();

    // Forward chain: alice -> bob -> charlie -> dave
    let fwd1 = store.forward_message(original.id, ch2.id, "bob").unwrap();
    let fwd2 = store.forward_message(fwd1, ch3.id, "charlie").unwrap();
    let fwd3 = store.forward_message(fwd2, ch4.id, "dave").unwrap();

    // Query chain from the deepest forward
    let chain = store.query_echo_chain(fwd3);

    // Should have at least 4 entries (original + 3 forwards)
    assert!(
        chain.len() >= 4,
        "chain should have at least 4 entries, got {}",
        chain.len()
    );

    // Verify depths are increasing
    assert_eq!(chain[0].depth, 0, "first entry should be depth 0");
    let max_depth = chain.iter().map(|e| e.depth).max().unwrap_or(0);
    assert!(max_depth >= 3, "max depth should be at least 3, got {}", max_depth);
}

// ===========================================================================
// 10. Echo Depth — root is zero
// ===========================================================================

#[test]
fn test_echo_depth_root_is_zero() {
    let (mut store, ch_id) = setup_pipeline_store();

    let original = store
        .send_message(ch_id, "alice", "Root message", MessageType::Text)
        .unwrap();

    let depth = store.get_echo_depth(original.id);
    assert_eq!(depth, 0, "original message should have depth 0");
}

// ===========================================================================
// 11. Summarize Conversation — basic
// ===========================================================================

#[test]
fn test_summarize_conversation_basic() {
    let (mut store, ch_id) = setup_pipeline_store();

    // Send some messages
    store
        .send_message(ch_id, "alice", "Hello Bob!", MessageType::Text)
        .unwrap();
    store
        .send_message(ch_id, "bob", "Hi Alice!", MessageType::Text)
        .unwrap();
    store
        .send_message(ch_id, "alice", "How are you?", MessageType::Text)
        .unwrap();

    let summary = store
        .summarize_conversation(ch_id)
        .expect("summarize should succeed");

    assert_eq!(summary.channel_id, ch_id);
    assert_eq!(summary.channel_name, "pipeline-test");
    assert_eq!(summary.message_count, 3);
    assert_eq!(summary.participant_count, 2);
    assert!(summary.avg_message_length > 0.0);
    assert!(
        summary.most_active_participant.is_some(),
        "should have a most active participant"
    );
    // Alice sent 2, bob sent 1 — alice should be most active
    assert_eq!(
        summary.most_active_participant.as_deref(),
        Some("alice"),
        "alice should be most active"
    );
    assert_eq!(summary.most_active_count, 2);
}

// ===========================================================================
// 12. Summarize Conversation — empty channel
// ===========================================================================

#[test]
fn test_summarize_conversation_empty() {
    let (store, ch_id) = setup_pipeline_store();

    let summary = store
        .summarize_conversation(ch_id)
        .expect("summarize empty channel should succeed");

    assert_eq!(summary.channel_id, ch_id);
    assert_eq!(summary.message_count, 0);
    assert_eq!(summary.avg_message_length, 0.0);
    assert_eq!(summary.thread_count, 0);
    assert_eq!(summary.reply_count, 0);
    assert!(!summary.has_affect_data);
    assert!(summary.most_active_participant.is_none());
}
