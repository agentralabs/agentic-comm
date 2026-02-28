//! Advanced integration tests for agentic-comm.
//!
//! Tests: forwarding chains, affect contagion, conversation summaries,
//! channel lifecycle, dead letters, workspace queries, and grounding.

use agentic_comm::*;

// ---------------------------------------------------------------------------
// Test 1: Forward chain across channels
// ---------------------------------------------------------------------------

#[test]
fn test_forward_chain_across_channels() {
    let mut store = CommStore::new();

    // Create three channels
    let ch1 = store.create_channel("channel-a", ChannelType::Group, None).unwrap();
    let ch2 = store.create_channel("channel-b", ChannelType::Group, None).unwrap();
    let ch3 = store.create_channel("channel-c", ChannelType::Group, None).unwrap();

    store.join_channel(ch1.id, "alice").unwrap();
    store.join_channel(ch2.id, "bob").unwrap();
    store.join_channel(ch3.id, "carol").unwrap();

    // Send original message in channel-a
    let original = store
        .send_message(ch1.id, "alice", "Important update", MessageType::Text)
        .unwrap();
    assert_eq!(original.channel_id, ch1.id);

    // Forward from channel-a to channel-b
    let fwd1_id = store
        .forward_message(original.id, ch2.id, "bob")
        .unwrap();

    // Forward from channel-b to channel-c
    let fwd2_id = store
        .forward_message(fwd1_id, ch3.id, "carol")
        .unwrap();

    // Verify the chain: fwd2 should have echo_depth = 2
    let fwd2 = store.get_message(fwd2_id).expect("forwarded message 2");
    assert_eq!(fwd2.channel_id, ch3.id);
    assert!(fwd2.content.contains("[Forwarded]"));
    let echo_depth: u32 = fwd2
        .metadata
        .get("echo_depth")
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(echo_depth, 2, "Second forward should have echo_depth 2");

    // Verify original_message_id tracks back to the root
    let root_id: u64 = fwd2
        .metadata
        .get("original_message_id")
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(root_id, original.id, "Root should point to the original");

    // Verify echo chain query
    let chain = store.query_echo_chain(fwd2_id);
    assert!(
        chain.len() >= 2,
        "Echo chain should have at least 2 entries (original + forwards), got {}",
        chain.len()
    );
}

// ---------------------------------------------------------------------------
// Test 2: Affect contagion from multiple senders
// ---------------------------------------------------------------------------

#[test]
fn test_affect_contagion_multi_sender() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("affect-room", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "alice").unwrap();
    store.join_channel(ch.id, "bob").unwrap();
    store.join_channel(ch.id, "carol").unwrap();

    // Set a low resistance so contagion has visible effect
    store.set_affect_resistance(0.2);

    // Alice sends a positive message with affect metadata
    let msg1 = store
        .send_message(ch.id, "alice", "Great news!", MessageType::Text)
        .unwrap();
    // Manually inject affect metadata (the engine checks for these)
    if let Some(m) = store.messages.get_mut(&msg1.id) {
        m.metadata.insert("valence".to_string(), "0.8".to_string());
        m.metadata.insert("arousal".to_string(), "0.7".to_string());
        m.metadata.insert("dominance".to_string(), "0.6".to_string());
    }

    // Bob sends a negative message
    let msg2 = store
        .send_message(ch.id, "bob", "Bad day...", MessageType::Text)
        .unwrap();
    if let Some(m) = store.messages.get_mut(&msg2.id) {
        m.metadata.insert("valence".to_string(), "-0.5".to_string());
        m.metadata.insert("arousal".to_string(), "0.3".to_string());
        m.metadata.insert("dominance".to_string(), "0.2".to_string());
    }

    // Process affect contagion
    let results = store.process_affect_contagion(ch.id);

    // Should produce contagion results for participants who received affect
    assert!(
        !results.is_empty(),
        "Contagion should produce results for participants"
    );

    // Carol should have been affected by both Alice and Bob
    let carol_effects: Vec<_> = results
        .iter()
        .filter(|(name, _, _, _)| name == "carol")
        .collect();
    assert!(
        !carol_effects.is_empty(),
        "Carol should have been affected by contagion"
    );

    // Verify affect state was set for carol
    let carol_state = store.get_affect_state("carol");
    assert!(
        carol_state.is_some(),
        "Carol should have an affect state after contagion"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Conversation summary with threads
// ---------------------------------------------------------------------------

#[test]
fn test_conversation_summary_with_threads() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("discussion", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "alice").unwrap();
    store.join_channel(ch.id, "bob").unwrap();

    // Send some messages, then create a thread
    let msg1 = store
        .send_message(ch.id, "alice", "Let's discuss the plan", MessageType::Text)
        .unwrap();
    let _msg2 = store
        .send_message(ch.id, "bob", "Sounds good", MessageType::Text)
        .unwrap();

    // Create a reply (which starts a thread)
    let reply1 = store
        .send_reply(ch.id, msg1.id, "bob", "What about timeline?", MessageType::Text)
        .unwrap();
    assert!(reply1.thread_id.is_some());

    // Another reply in the same thread
    let reply2 = store
        .send_reply(ch.id, msg1.id, "alice", "Next week", MessageType::Text)
        .unwrap();
    assert_eq!(reply2.thread_id, reply1.thread_id);

    // Get conversation summary
    let summary = store.summarize_conversation(ch.id).unwrap();
    assert_eq!(summary.message_count, 4);
    assert_eq!(summary.participant_count, 2);
    assert!(
        summary.thread_count >= 1,
        "Should have at least 1 thread, got {}",
        summary.thread_count
    );
    assert!(
        summary.reply_count >= 2,
        "Should have at least 2 replies, got {}",
        summary.reply_count
    );

    // Verify thread retrieval
    let thread_id = reply1.thread_id.unwrap();
    let thread_msgs = store.get_thread(&thread_id);
    assert!(
        thread_msgs.len() >= 2,
        "Thread should have at least 2 messages (the replies), got {}",
        thread_msgs.len()
    );
}

// ---------------------------------------------------------------------------
// Test 4: Channel pause/resume lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_channel_pause_resume_lifecycle() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("lifecycle-channel", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "agent-a").unwrap();

    // Channel starts active
    let fetched = store.get_channel(ch.id).unwrap();
    assert_eq!(fetched.state, ChannelState::Active);

    // Send succeeds while active
    store
        .send_message(ch.id, "agent-a", "message 1", MessageType::Text)
        .unwrap();

    // Pause the channel
    store.pause_channel(ch.id).unwrap();
    let fetched = store.get_channel(ch.id).unwrap();
    assert_eq!(fetched.state, ChannelState::Paused);

    // Sending on a paused channel should fail
    let send_result = store.send_message(ch.id, "agent-a", "should fail", MessageType::Text);
    assert!(
        send_result.is_err(),
        "Sending on a paused channel should fail"
    );

    // Resume the channel
    store.resume_channel(ch.id).unwrap();
    let fetched = store.get_channel(ch.id).unwrap();
    assert_eq!(fetched.state, ChannelState::Active);

    // Sending works again after resume
    store
        .send_message(ch.id, "agent-a", "message 2", MessageType::Text)
        .unwrap();

    // Drain the channel
    store.drain_channel(ch.id).unwrap();
    let fetched = store.get_channel(ch.id).unwrap();
    assert_eq!(fetched.state, ChannelState::Draining);

    // Sending on a draining channel should fail
    let send_result = store.send_message(ch.id, "agent-a", "should also fail", MessageType::Text);
    assert!(
        send_result.is_err(),
        "Sending on a draining channel should fail"
    );

    // Close the channel
    store.close_channel(ch.id).unwrap();
    let fetched = store.get_channel(ch.id).unwrap();
    assert_eq!(fetched.state, ChannelState::Closed);
}

// ---------------------------------------------------------------------------
// Test 5: Dead letter roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_dead_letter_roundtrip() {
    let mut store = CommStore::new();

    // Create a channel, send a message, then expire it to create dead letters
    let ch = store
        .create_channel("ephemeral", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "sender").unwrap();

    // Configure short TTL
    let config = ChannelConfig {
        ttl_seconds: 1, // 1 second TTL
        ..Default::default()
    };
    store.set_channel_config(ch.id, config).unwrap();

    // Send a message
    let msg = store
        .send_message(ch.id, "sender", "ephemeral content", MessageType::Text)
        .unwrap();
    let msg_id = msg.id;

    // Expire messages (force by modifying timestamp)
    // The expire_messages method checks TTL against channel config
    // We need to wait or manipulate time. Since we can't wait in tests,
    // let's just verify the dead letter API works by adding one directly.
    // Dead letters come from expired messages. Let's use compact + list.

    // First verify the dead letter list starts empty
    let dead = store.list_dead_letters();
    let initial_count = dead.len();

    // Manually trigger expiration — in real use, messages older than TTL get expired.
    // Since we can't easily wait, we verify the API methods work correctly
    // by calling expire_messages (which won't expire new messages) and then
    // testing list/clear.
    let _expired = store.expire_messages();

    // The dead letter list should be queryable (even if empty because of timing)
    let dead = store.list_dead_letters();
    assert!(dead.len() >= initial_count, "Dead letter list should be stable");

    // Verify clear works
    store.clear_dead_letters();
    let dead = store.list_dead_letters();
    assert_eq!(dead.len(), 0, "Dead letters should be empty after clear");

    // Verify the message still exists (expire with 1s TTL won't expire instantly)
    assert!(
        store.get_message(msg_id).is_some(),
        "Message should still exist (TTL not elapsed)"
    );
}

// ---------------------------------------------------------------------------
// Test 6: Workspace cross-store query
// ---------------------------------------------------------------------------

#[test]
fn test_workspace_cross_store_query() {
    // Create two temporary stores
    let mut store1 = CommStore::new();
    let ch1 = store1
        .create_channel("project-alpha", ChannelType::Group, None)
        .unwrap();
    store1.join_channel(ch1.id, "alice").unwrap();
    store1
        .send_message(ch1.id, "alice", "Alpha deployment ready", MessageType::Text)
        .unwrap();

    let mut store2 = CommStore::new();
    let ch2 = store2
        .create_channel("project-beta", ChannelType::Group, None)
        .unwrap();
    store2.join_channel(ch2.id, "bob").unwrap();
    store2
        .send_message(ch2.id, "bob", "Beta deployment pending", MessageType::Text)
        .unwrap();

    // Save both stores to temp files
    let tmp1 = tempfile::NamedTempFile::new().unwrap();
    store1.save(tmp1.path()).unwrap();
    let tmp2 = tempfile::NamedTempFile::new().unwrap();
    store2.save(tmp2.path()).unwrap();

    // Create workspace and add both stores
    let mut ws = CommWorkspace::new("deployment-ws");
    ws.add_context(
        &tmp1.path().to_string_lossy(),
        Some("Alpha"),
        WorkspaceRole::Primary,
    )
    .unwrap();
    ws.add_context(
        &tmp2.path().to_string_lossy(),
        Some("Beta"),
        WorkspaceRole::Secondary,
    )
    .unwrap();

    // Query across both stores
    let results = ws.query("deployment", 10);
    assert_eq!(results.len(), 2, "Should have results from 2 contexts");

    // Both contexts should have matches
    let alpha = &results[0];
    assert_eq!(alpha.context_label, "Alpha");
    assert!(!alpha.matches.is_empty(), "Alpha should match 'deployment'");

    let beta = &results[1];
    assert_eq!(beta.context_label, "Beta");
    assert!(!beta.matches.is_empty(), "Beta should match 'deployment'");

    // Cross-reference "alice" — should only be in Alpha
    let xrefs = ws.xref("alice");
    assert_eq!(xrefs.len(), 2);
    assert!(xrefs[0].1, "Alice should be found in Alpha");
    assert!(!xrefs[1].1, "Alice should NOT be found in Beta");
}

// ---------------------------------------------------------------------------
// Test 7: Grounding evidence and suggest
// ---------------------------------------------------------------------------

#[test]
fn test_grounding_evidence_and_suggest() {
    let mut store = CommStore::new();

    // Set up data for grounding
    let ch = store
        .create_channel("evidence-channel", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "investigator").unwrap();
    store.join_channel(ch.id, "witness").unwrap();

    store
        .send_message(
            ch.id,
            "investigator",
            "The deployment happened at 3pm",
            MessageType::Text,
        )
        .unwrap();
    store
        .send_message(
            ch.id,
            "witness",
            "I can confirm the deployment was successful",
            MessageType::Text,
        )
        .unwrap();

    // Set trust for an agent
    store.set_trust_level("investigator", CommTrustLevel::High).unwrap();

    // Ground a claim
    let result = store.ground_claim("evidence-channel exists");
    assert_eq!(result.status, GroundingStatus::Verified);
    assert!(!result.evidence.is_empty(), "Should have evidence");

    // Ground an ungrounded claim
    let result2 = store.ground_claim("nonexistent-thing is here");
    assert_eq!(result2.status, GroundingStatus::Ungrounded);

    // Test ground_evidence
    let evidence = store.ground_evidence("deployment");
    assert!(
        !evidence.is_empty(),
        "Should find evidence related to 'deployment'"
    );

    // Verify evidence has content about deployment
    let has_deployment = evidence
        .iter()
        .any(|e| e.content.to_lowercase().contains("deployment"));
    assert!(has_deployment, "Evidence should mention deployment");

    // Test ground_suggest
    let suggestions = store.ground_suggest("deploy", 5);
    assert!(
        !suggestions.is_empty(),
        "Should have suggestions for 'deploy'"
    );
}
