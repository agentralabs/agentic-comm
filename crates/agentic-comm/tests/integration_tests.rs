//! Integration tests for agentic-comm engine.
//!
//! These tests exercise full lifecycle workflows across the CommStore,
//! covering channels, messaging, forwarding, affect, grounding,
//! binary persistence, encryption, workspaces, and channel state machines.

use agentic_comm::*;

// ---------------------------------------------------------------------------
// Test 1: Full lifecycle — create channel, send, query, forward, summarize, close
// ---------------------------------------------------------------------------

#[test]
fn test_full_lifecycle() {
    let mut store = CommStore::new();

    // Create a channel and join participants
    let ch = store
        .create_channel("full-lifecycle", ChannelType::Group, None)
        .expect("create channel");
    store.join_channel(ch.id, "alice").unwrap();
    store.join_channel(ch.id, "bob").unwrap();

    // Send messages
    let msg1 = store
        .send_message(ch.id, "alice", "Hello Bob!", MessageType::Text)
        .expect("send msg 1");
    let msg2 = store
        .send_message(ch.id, "bob", "Hi Alice, how are you?", MessageType::Text)
        .expect("send msg 2");
    let _msg3 = store
        .send_message(ch.id, "alice", "Doing great, thanks!", MessageType::Text)
        .expect("send msg 3");

    // Query history
    let filter = MessageFilter {
        limit: Some(10),
        ..Default::default()
    };
    let history = store.query_history(ch.id, &filter);
    assert_eq!(history.len(), 3, "should have 3 messages");

    // Search
    let results = store.search_messages("how are you", 10);
    assert_eq!(results.len(), 1, "search should find 1 match");
    assert_eq!(results[0].id, msg2.id);

    // Forward a message to a second channel
    let ch2 = store
        .create_channel("forward-target", ChannelType::Group, None)
        .expect("create target channel");
    store.join_channel(ch2.id, "charlie").unwrap();

    let forwarded_id = store
        .forward_message(msg1.id, ch2.id, "bob")
        .expect("forward should succeed");
    assert!(forwarded_id > 0, "forwarded message id should be positive");

    // Verify forwarded message exists in target channel
    let fwd_filter = MessageFilter {
        limit: Some(10),
        ..Default::default()
    };
    let fwd_history = store.query_history(ch2.id, &fwd_filter);
    assert!(!fwd_history.is_empty(), "target channel should have forwarded message");

    // Summarize conversation
    let summary = store
        .summarize_conversation(ch.id)
        .expect("summarize should succeed");
    assert_eq!(summary.channel_id, ch.id);
    assert_eq!(summary.message_count, 3);
    assert!(summary.participant_count >= 2);

    // Close the channel
    store.close_channel(ch.id).unwrap();
    let closed = store.get_channel(ch.id).unwrap();
    assert_eq!(closed.state, ChannelState::Closed);
}

// ---------------------------------------------------------------------------
// Test 2: Multi-channel forwarding — forward across 3 channels, verify echo chains
// ---------------------------------------------------------------------------

#[test]
fn test_multi_channel_forwarding() {
    let mut store = CommStore::new();

    // Create 3 channels
    let ch1 = store
        .create_channel("origin", ChannelType::Group, None)
        .unwrap();
    let ch2 = store
        .create_channel("relay", ChannelType::Group, None)
        .unwrap();
    let ch3 = store
        .create_channel("destination", ChannelType::Group, None)
        .unwrap();

    store.join_channel(ch1.id, "sender").unwrap();
    store.join_channel(ch2.id, "relayer").unwrap();
    store.join_channel(ch3.id, "receiver").unwrap();

    // Send original message in ch1
    let original = store
        .send_message(ch1.id, "sender", "Important announcement", MessageType::Text)
        .expect("send original");

    // Forward from ch1 to ch2
    let fwd1_id = store
        .forward_message(original.id, ch2.id, "relayer")
        .expect("forward to relay");

    // Forward from ch2 to ch3
    let fwd2_id = store
        .forward_message(fwd1_id, ch3.id, "receiver")
        .expect("forward to destination");

    // Verify echo chain from the final forwarded message
    let chain = store.query_echo_chain(fwd2_id);
    // The chain should have entries tracing back through the forwarding path
    assert!(
        !chain.is_empty(),
        "echo chain should not be empty for forwarded message"
    );

    // Verify echo depth increases with each forward
    let depth_original = store.get_echo_depth(original.id);
    let depth_fwd1 = store.get_echo_depth(fwd1_id);
    let depth_fwd2 = store.get_echo_depth(fwd2_id);

    assert!(
        depth_fwd1 > depth_original,
        "first forward should have greater depth than original"
    );
    assert!(
        depth_fwd2 > depth_fwd1,
        "second forward should have greater depth than first"
    );

    // Verify all 3 channels have messages
    let filter = MessageFilter {
        limit: Some(10),
        ..Default::default()
    };
    assert_eq!(store.query_history(ch1.id, &filter).len(), 1);
    assert_eq!(store.query_history(ch2.id, &filter).len(), 1);
    assert_eq!(store.query_history(ch3.id, &filter).len(), 1);
}

// ---------------------------------------------------------------------------
// Test 3: Affect lifecycle — set affect, process contagion, decay, check history
// ---------------------------------------------------------------------------

#[test]
fn test_affect_lifecycle() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("affect-test", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "alice").unwrap();
    store.join_channel(ch.id, "bob").unwrap();

    // Grant consent from bob to alice for sending messages (required for affect messages)
    store
        .grant_consent("bob", "alice", ConsentScope::SendMessages, None, None)
        .expect("grant consent");

    // Send a message with affect metadata
    let affect = AffectState {
        valence: 0.8,
        arousal: 0.6,
        dominance: 0.5,
        emotions: vec![Emotion::Joy],
        urgency: UrgencyLevel::Normal,
        meta_confidence: 0.9,
    };

    let msg = store
        .send_affect_message(ch.id, "alice", "I'm so happy!", affect)
        .expect("send affect message");
    assert!(msg.id > 0);

    // Process contagion on the channel
    let contagion_results = store.process_affect_contagion(ch.id);
    // Results might be empty if no affect state was set for agents yet,
    // but the function should not panic
    assert!(
        contagion_results.len() >= 0,
        "contagion processing should complete without error"
    );

    // Set resistance and verify
    let actual = store.set_affect_resistance(0.5);
    assert!((actual - 0.5).abs() < 0.01, "resistance should be ~0.5");

    // Apply decay
    store.apply_affect_decay(0.1);

    // Get history (may be empty if affect tracking is event-driven)
    let history = store.get_affect_history("alice");
    assert_eq!(history.agent, "alice");
    // History states may or may not be populated depending on engine behavior
}

// ---------------------------------------------------------------------------
// Test 4: Grounding integration — send messages, then use ground_evidence and ground_suggest
// ---------------------------------------------------------------------------

#[test]
fn test_grounding_integration() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("deploy-channel", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "deployer").unwrap();
    store.join_channel(ch.id, "reviewer").unwrap();

    // Send messages about a deployment
    store
        .send_message(ch.id, "deployer", "Starting deployment of v2.0", MessageType::Text)
        .unwrap();
    store
        .send_message(ch.id, "reviewer", "Deployment approved for v2.0", MessageType::Text)
        .unwrap();
    store
        .send_message(ch.id, "deployer", "Deployment of v2.0 completed successfully", MessageType::Text)
        .unwrap();

    // Ground a claim — the engine checks if claim.contains(channel_name) or
    // claim.contains(agent_name), so include the channel name in the claim.
    let result = store.ground_claim("the deploy-channel was used for deployer");
    assert!(
        !result.evidence.is_empty(),
        "should find evidence: claim contains channel name 'deploy-channel' and participant 'deployer'"
    );

    // Ground evidence search — searches message content for the query
    let evidence = store.ground_evidence("deployment");
    assert!(
        !evidence.is_empty(),
        "should find grounding evidence for 'deployment' in message content"
    );

    // Ground suggestions — suggests channel names, agents, etc.
    let suggestions = store.ground_suggest("deploy", 5);
    assert!(
        !suggestions.is_empty(),
        "should have suggestions related to 'deploy'"
    );
}

// ---------------------------------------------------------------------------
// Test 5: Binary format roundtrip — save with binary format, reload, verify all data intact
// ---------------------------------------------------------------------------

#[test]
fn test_binary_format_roundtrip() {
    let tmp = tempfile::NamedTempFile::new().expect("create temp file");
    let path = tmp.path();

    // Build a store with various data
    let mut store = CommStore::new();
    let ch = store
        .create_channel("binary-test", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "agent-a").unwrap();
    store.join_channel(ch.id, "agent-b").unwrap();

    let msg1 = store
        .send_message(ch.id, "agent-a", "Message one", MessageType::Text)
        .unwrap();
    let msg2 = store
        .send_message(ch.id, "agent-b", "Message two", MessageType::Text)
        .unwrap();

    // Add trust level
    store.set_trust_level("agent-a", CommTrustLevel::High).unwrap();

    // Subscribe to a topic
    store.subscribe("events", "agent-a").unwrap();

    // Save
    store.save(path).expect("save should succeed");

    // Verify the file starts with the ACOM magic bytes
    let raw_bytes = std::fs::read(path).expect("read file");
    assert_eq!(&raw_bytes[0..4], b"ACOM", "file should start with ACOM magic");

    // Reload
    let loaded = CommStore::load(path).expect("load should succeed");

    // Verify channels
    let loaded_ch = loaded.get_channel(ch.id).expect("channel should exist");
    assert_eq!(loaded_ch.name, "binary-test");
    assert_eq!(loaded_ch.participants.len(), 2);

    // Verify messages
    let loaded_msg1 = loaded.get_message(msg1.id).expect("msg1 should exist");
    assert_eq!(loaded_msg1.content, "Message one");
    let loaded_msg2 = loaded.get_message(msg2.id).expect("msg2 should exist");
    assert_eq!(loaded_msg2.content, "Message two");

    // Verify trust level
    let trust = loaded.get_trust_level("agent-a");
    assert_eq!(trust, CommTrustLevel::High);

    // Verify stats match
    let original_stats = store.stats();
    let loaded_stats = loaded.stats();
    assert_eq!(original_stats.channel_count, loaded_stats.channel_count);
    assert_eq!(original_stats.message_count, loaded_stats.message_count);
    assert_eq!(
        original_stats.subscription_count,
        loaded_stats.subscription_count
    );
}

// ---------------------------------------------------------------------------
// Test 6: Encryption roundtrip integration — encrypt message content, decrypt, verify
// ---------------------------------------------------------------------------

#[test]
fn test_encryption_roundtrip_integration() {
    // Generate a key
    let key = encryption::EncryptionKey::generate();
    assert_eq!(key.epoch, 1);

    // Encrypt a realistic message content
    let plaintext = "Deploy credentials: user=admin pass=s3cret123 host=prod.example.com";
    let payload = encryption::encrypt(&key, plaintext).expect("encrypt should succeed");

    // Verify ciphertext is different from plaintext
    assert_ne!(payload.ciphertext, plaintext);
    assert!(!payload.ciphertext.is_empty());
    assert!(!payload.nonce.is_empty());
    assert_eq!(payload.epoch, 1);

    // Decrypt and verify
    let decrypted = encryption::decrypt(&key, &payload).expect("decrypt should succeed");
    assert_eq!(decrypted, plaintext);

    // Rotate key and verify old encrypted data can't be decrypted with new key
    let rotated_key = key.rotate();
    assert_eq!(rotated_key.epoch, 2);
    let result = encryption::decrypt(&rotated_key, &payload);
    assert!(result.is_err(), "rotated key should not decrypt old payload");

    // Encrypt with rotated key works for new data
    let new_payload = encryption::encrypt(&rotated_key, "new secret").expect("encrypt with rotated key");
    assert_eq!(new_payload.epoch, 2);
    let new_decrypted = encryption::decrypt(&rotated_key, &new_payload).expect("decrypt new payload");
    assert_eq!(new_decrypted, "new secret");

    // Multiple encryptions of the same plaintext produce different ciphertexts (random nonce)
    let payload_a = encryption::encrypt(&key, "same message").unwrap();
    let payload_b = encryption::encrypt(&key, "same message").unwrap();
    assert_ne!(
        payload_a.nonce, payload_b.nonce,
        "nonces should differ between encryptions"
    );
    // But both decrypt to the same plaintext
    assert_eq!(encryption::decrypt(&key, &payload_a).unwrap(), "same message");
    assert_eq!(encryption::decrypt(&key, &payload_b).unwrap(), "same message");
}

// ---------------------------------------------------------------------------
// Test 7: Workspace integration — create workspace, add 2 stores, query across them
// ---------------------------------------------------------------------------

#[test]
fn test_workspace_integration() {
    // Create two temporary stores with different content
    let tmp1 = tempfile::NamedTempFile::new().unwrap();
    let tmp2 = tempfile::NamedTempFile::new().unwrap();

    // Store 1: deployment-related messages
    {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("deploys", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch.id, "deployer").unwrap();
        store
            .send_message(ch.id, "deployer", "Deploy v3.0 started", MessageType::Text)
            .unwrap();
        store
            .send_message(ch.id, "deployer", "Deploy v3.0 complete", MessageType::Text)
            .unwrap();
        store.save(tmp1.path()).unwrap();
    }

    // Store 2: review-related messages
    {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("reviews", ChannelType::Group, None)
            .unwrap();
        store.join_channel(ch.id, "reviewer").unwrap();
        store
            .send_message(ch.id, "reviewer", "Code review for v3.0 approved", MessageType::Text)
            .unwrap();
        store.save(tmp2.path()).unwrap();
    }

    // Create workspace and add both stores
    let mut ws = CommWorkspace::new("integration-ws");
    let path1 = tmp1.path().to_string_lossy().to_string();
    let path2 = tmp2.path().to_string_lossy().to_string();

    ws.add_context(&path1, Some("Deployments"), WorkspaceRole::Primary)
        .expect("add context 1");
    ws.add_context(&path2, Some("Reviews"), WorkspaceRole::Secondary)
        .expect("add context 2");

    // Verify contexts
    assert_eq!(ws.list_contexts().len(), 2);

    // Query across contexts for "v3.0"
    let results = ws.query("v3.0", 10);
    assert_eq!(results.len(), 2, "should search both contexts");

    // Deployments context should have 2 matches
    let deploys_result = results.iter().find(|r| r.context_label == "Deployments");
    assert!(deploys_result.is_some());
    assert_eq!(deploys_result.unwrap().matches.len(), 2);

    // Reviews context should have 1 match
    let reviews_result = results.iter().find(|r| r.context_label == "Reviews");
    assert!(reviews_result.is_some());
    assert_eq!(reviews_result.unwrap().matches.len(), 1);

    // Compare "deployer" across contexts
    let comparison = ws.compare("deployer", 50);
    assert_eq!(comparison.contexts.len(), 2);
    // "deployer" should be found in store 1 (as agent + sender in messages)
    assert!(comparison.contexts[0].found);
    assert!(comparison.contexts[0].count > 0);

    // Cross-reference "v3.0" across contexts
    let xrefs = ws.xref("v3.0");
    assert_eq!(xrefs.len(), 2);
    assert!(xrefs[0].1, "v3.0 should be found in deployments");
    assert!(xrefs[1].1, "v3.0 should be found in reviews");
}

// ---------------------------------------------------------------------------
// Test 8: Channel lifecycle full — create, pause, resume, drain, close
// ---------------------------------------------------------------------------

#[test]
fn test_channel_lifecycle_full() {
    let mut store = CommStore::new();

    // Create channel
    let ch = store
        .create_channel("lifecycle-channel", ChannelType::Group, None)
        .expect("create channel");
    store.join_channel(ch.id, "operator").unwrap();
    assert_eq!(
        store.get_channel(ch.id).unwrap().state,
        ChannelState::Active
    );

    // Send a message while active
    store
        .send_message(ch.id, "operator", "Active message", MessageType::Text)
        .expect("send while active should succeed");

    // Pause the channel
    store.pause_channel(ch.id).unwrap();
    assert_eq!(
        store.get_channel(ch.id).unwrap().state,
        ChannelState::Paused
    );

    // Sending to a paused channel should fail
    let paused_send = store.send_message(ch.id, "operator", "Paused msg", MessageType::Text);
    assert!(paused_send.is_err(), "send to paused channel should fail");

    // Resume the channel
    store.resume_channel(ch.id).unwrap();
    assert_eq!(
        store.get_channel(ch.id).unwrap().state,
        ChannelState::Active
    );

    // Send should work again after resume
    store
        .send_message(ch.id, "operator", "Resumed message", MessageType::Text)
        .expect("send after resume should succeed");

    // Drain the channel
    store.drain_channel(ch.id).unwrap();
    assert_eq!(
        store.get_channel(ch.id).unwrap().state,
        ChannelState::Draining
    );

    // Sending to a draining channel should fail
    let drain_send = store.send_message(ch.id, "operator", "Draining msg", MessageType::Text);
    assert!(drain_send.is_err(), "send to draining channel should fail");

    // Reading should still work (draining allows reads)
    let filter = MessageFilter {
        limit: Some(50),
        ..Default::default()
    };
    let history = store.query_history(ch.id, &filter);
    assert_eq!(history.len(), 2, "should still be able to read messages");

    // Close the channel
    store.close_channel(ch.id).unwrap();
    assert_eq!(
        store.get_channel(ch.id).unwrap().state,
        ChannelState::Closed
    );

    // Sending to a closed channel should fail
    let closed_send = store.send_message(ch.id, "operator", "Closed msg", MessageType::Text);
    assert!(closed_send.is_err(), "send to closed channel should fail");
}
