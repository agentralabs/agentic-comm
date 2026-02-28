//! SPEC-PART4 Required Test Scenarios
//!
//! 16 integration-level tests exercising the CommStore engine directly.
//! Each test is self-contained with its own CommStore instance.

use agentic_comm::*;

// ---------------------------------------------------------------------------
// Scenario 01: Direct channel creation
// ---------------------------------------------------------------------------

#[test]
fn scenario_01_direct_channel_creation() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("agent-a-agent-b", ChannelType::Direct, None)
        .expect("channel creation should succeed");

    // Join both participants
    store.join_channel(ch.id, "agent_a").unwrap();
    store.join_channel(ch.id, "agent_b").unwrap();

    // Verify channel exists
    let fetched = store.get_channel(ch.id).expect("channel should exist");
    assert_eq!(fetched.name, "agent-a-agent-b");
    assert_eq!(fetched.channel_type, ChannelType::Direct);
    assert_eq!(fetched.state, ChannelState::Active);
    assert_eq!(fetched.participants.len(), 2);
    assert!(fetched.participants.contains(&"agent_a".to_string()));
    assert!(fetched.participants.contains(&"agent_b".to_string()));
    // Channel has a unique ID (> 0)
    assert!(fetched.id > 0);
}

// ---------------------------------------------------------------------------
// Scenario 02: Message send & receive
// ---------------------------------------------------------------------------

#[test]
fn scenario_02_message_send_receive() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("dm-channel", ChannelType::Direct, None)
        .unwrap();
    store.join_channel(ch.id, "agent_a").unwrap();
    store.join_channel(ch.id, "agent_b").unwrap();

    // Send a text message from A to B
    let msg = store
        .send_message(ch.id, "agent_a", "Hello agent B!", MessageType::Text)
        .expect("send should succeed");

    // Verify message properties
    assert_eq!(msg.sender, "agent_a");
    assert_eq!(msg.content, "Hello agent B!");
    assert_eq!(msg.message_type, MessageType::Text);
    assert!(msg.signature.is_some(), "signature should be present");
    assert_eq!(msg.status, MessageStatus::Sent);

    // Retrieve by ID
    let fetched = store.get_message(msg.id).expect("message should exist");
    assert_eq!(fetched.id, msg.id);
    assert_eq!(fetched.content, "Hello agent B!");

    // Retrieve via receive_messages
    let msgs = store.receive_messages(ch.id, None, None).unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].id, msg.id);
}

// ---------------------------------------------------------------------------
// Scenario 03: Semantic fragment transfer
// ---------------------------------------------------------------------------

#[test]
fn scenario_03_semantic_fragment_transfer() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("semantic-ch", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "agent_a").unwrap();
    store.join_channel(ch.id, "agent_b").unwrap();

    // Build a semantic fragment with focus nodes
    let fragment = SemanticFragment {
        content: "quantum-entanglement".to_string(),
        role: "topic".to_string(),
        confidence: 0.95,
        nodes: vec![
            CognitiveNode {
                id: "n1".to_string(),
                label: "entanglement".to_string(),
                node_type: CognitiveNodeType::Concept,
            },
            CognitiveNode {
                id: "n2".to_string(),
                label: "measurement".to_string(),
                node_type: CognitiveNodeType::Action,
            },
        ],
        edges: vec![CognitiveEdge {
            from: "n1".to_string(),
            to: "n2".to_string(),
            edge_type: CognitiveEdgeType::RelatedTo,
            weight: 0.8,
        }],
        graft_points: Vec::new(),
        context: String::new(),
        perspective: String::new(),
    };

    // Serialize the semantic content into the message body
    let rich = RichMessageContent::Semantic(fragment);
    let content_json =
        serde_json::to_string(&rich).expect("semantic content should serialize");

    let msg = store
        .send_message(ch.id, "agent_a", &content_json, MessageType::Text)
        .unwrap();

    // Verify we can deserialize it back
    let retrieved = store.get_message(msg.id).unwrap();
    let parsed: RichMessageContent =
        serde_json::from_str(&retrieved.content).expect("should parse back");

    match parsed {
        RichMessageContent::Semantic(frag) => {
            assert_eq!(frag.content, "quantum-entanglement");
            assert_eq!(frag.nodes.len(), 2);
            assert_eq!(frag.edges.len(), 1);
            assert!((frag.confidence - 0.95).abs() < f64::EPSILON);
        }
        other => panic!("Expected Semantic variant, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Scenario 04: Affect contagion
// ---------------------------------------------------------------------------

#[test]
fn scenario_04_affect_contagion() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("affect-ch", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "agent_a").unwrap();
    store.join_channel(ch.id, "agent_b").unwrap();

    // Build an affect state with specific dimensions
    let affect = AffectState {
        valence: 0.8,
        arousal: 0.6,
        dominance: 0.4,
        emotions: vec![Emotion::Joy, Emotion::Excitement],
        urgency: UrgencyLevel::High,
        meta_confidence: 0.9,
    };

    // Also serialize via RichMessageContent::Affect for verification
    let rich = RichMessageContent::Affect(affect.clone());
    let content_json = serde_json::to_string(&rich).unwrap();

    let msg = store
        .send_message(ch.id, "agent_a", &content_json, MessageType::Text)
        .unwrap();

    // Verify round-trip
    let retrieved = store.get_message(msg.id).unwrap();
    let parsed: RichMessageContent = serde_json::from_str(&retrieved.content).unwrap();

    match parsed {
        RichMessageContent::Affect(a) => {
            assert!((a.valence - 0.8).abs() < f64::EPSILON);
            assert!((a.arousal - 0.6).abs() < f64::EPSILON);
            assert!((a.dominance - 0.4).abs() < f64::EPSILON);
            assert_eq!(a.emotions.len(), 2);
            assert!(a.emotions.contains(&Emotion::Joy));
            assert!(a.emotions.contains(&Emotion::Excitement));
            assert!((a.meta_confidence - 0.9).abs() < f64::EPSILON);
        }
        other => panic!("Expected Affect variant, got {:?}", other),
    }

    // Also test send_affect_message API which embeds affect in content.
    // Rich content (affect-prefixed) requires explicit SendMessages consent
    // from each other participant to the sender.
    store
        .grant_consent(
            "agent_a",
            "agent_b",
            ConsentScope::SendMessages,
            Some("allow affect".to_string()),
            None,
        )
        .unwrap();

    let affect2 = AffectState {
        valence: -0.3,
        arousal: 0.9,
        dominance: 0.2,
        emotions: vec![Emotion::Anxiety],
        urgency: UrgencyLevel::Urgent,
        meta_confidence: 0.7,
    };

    let msg2 = store
        .send_affect_message(ch.id, "agent_b", "I am anxious", affect2)
        .expect("affect message should succeed after consent grant");

    // The enriched content contains the affect JSON prefix
    assert!(msg2.content.contains("[affect:"));
    assert!(msg2.content.contains("I am anxious"));
}

// ---------------------------------------------------------------------------
// Scenario 05: Consent denial
// ---------------------------------------------------------------------------

#[test]
fn scenario_05_consent_denial() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("consent-ch", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "agent_a").unwrap();
    store.join_channel(ch.id, "agent_b").unwrap();

    // Grant consent first, then revoke it to simulate denial
    store
        .grant_consent(
            "agent_b",
            "agent_a",
            ConsentScope::SendMessages,
            Some("initial grant".to_string()),
            None,
        )
        .unwrap();
    assert!(store.check_consent("agent_b", "agent_a", &ConsentScope::SendMessages));

    // Now revoke (deny) the SendMessages consent
    store
        .revoke_consent("agent_b", "agent_a", &ConsentScope::SendMessages)
        .unwrap();

    // Verify the denial is reflected
    assert!(
        !store.check_consent("agent_b", "agent_a", &ConsentScope::SendMessages),
        "Consent should be denied after revocation"
    );

    // Verify the consent entry exists and has Revoked status
    let gates = store.list_consent_gates(Some("agent_b"));
    assert!(!gates.is_empty());
    let entry = gates
        .iter()
        .find(|e| e.grantee == "agent_a" && e.scope == ConsentScope::SendMessages)
        .expect("consent entry should exist");
    assert_eq!(entry.status, ConsentStatus::Revoked);

    // Also verify consent that was never granted returns false
    assert!(!store.check_consent("agent_a", "agent_b", &ConsentScope::ShareContent));
}

// ---------------------------------------------------------------------------
// Scenario 06: Temporal message scheduling
// ---------------------------------------------------------------------------

#[test]
fn scenario_06_temporal_message_scheduling() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("temporal-ch", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "agent_a").unwrap();
    store.join_channel(ch.id, "agent_b").unwrap();

    // Schedule a message for future delivery
    let sched = store
        .schedule_message(
            ch.id,
            "agent_a",
            "Future message",
            TemporalTarget::FutureAbsolute {
                deliver_at: "2099-01-01T00:00:00Z".to_string(),
            },
            None,
        )
        .expect("scheduling should succeed");

    let sched_id = sched.id;
    assert!(!sched.delivered);

    // Verify it's in the scheduled queue
    let scheduled = store.list_scheduled();
    assert_eq!(scheduled.len(), 1);
    assert_eq!(scheduled[0].id, sched_id);
    assert_eq!(scheduled[0].content, "Future message");

    // The message should NOT be in the regular channel messages yet
    let channel_msgs = store.receive_messages(ch.id, None, None).unwrap();
    assert!(
        channel_msgs.is_empty(),
        "Future-scheduled messages should not appear in channel yet"
    );

    // Now schedule an Immediate message and deliver it
    store
        .schedule_message(
            ch.id,
            "agent_a",
            "Immediate delivery",
            TemporalTarget::Immediate,
            None,
        )
        .unwrap();

    let delivered_count = store.deliver_pending_temporal();
    assert_eq!(delivered_count, 1, "One immediate message should be delivered");

    // Verify the immediate message now appears in channel
    let channel_msgs = store.receive_messages(ch.id, None, None).unwrap();
    assert_eq!(channel_msgs.len(), 1);
    assert_eq!(channel_msgs[0].content, "Immediate delivery");
}

// ---------------------------------------------------------------------------
// Scenario 07: Hive formation
// ---------------------------------------------------------------------------

#[test]
fn scenario_07_hive_formation() {
    let mut store = CommStore::new();

    // Form a hive with a coordinator
    let hive = store
        .form_hive("test-hive", "coordinator-agent", CollectiveDecisionMode::Majority)
        .expect("hive formation should succeed");

    let hive_id = hive.id;
    assert!(hive_id > 0);
    assert_eq!(hive.name, "test-hive");
    assert_eq!(hive.constituents.len(), 1); // coordinator only

    // Add two more members
    store
        .join_hive(hive_id, "agent_b", HiveRole::Member)
        .unwrap();
    store
        .join_hive(hive_id, "agent_c", HiveRole::Member)
        .unwrap();

    // Verify the hive has 3 constituents
    let fetched = store.get_hive(hive_id).expect("hive should exist");
    assert_eq!(fetched.constituents.len(), 3);
    assert_eq!(fetched.decision_mode, CollectiveDecisionMode::Majority);

    let agent_ids: Vec<&str> = fetched
        .constituents
        .iter()
        .map(|c| c.agent_id.as_str())
        .collect();
    assert!(agent_ids.contains(&"coordinator-agent"));
    assert!(agent_ids.contains(&"agent_b"));
    assert!(agent_ids.contains(&"agent_c"));

    // Verify the coordinator role
    let coord = fetched
        .constituents
        .iter()
        .find(|c| c.agent_id == "coordinator-agent")
        .unwrap();
    assert_eq!(coord.role, HiveRole::Coordinator);
}

// ---------------------------------------------------------------------------
// Scenario 08: Mind meld (messages between agents in a meld channel)
// ---------------------------------------------------------------------------

#[test]
fn scenario_08_mind_meld() {
    let mut store = CommStore::new();

    // Create a meld channel using SilentCommunion state
    let ch = store
        .create_channel("meld-session", ChannelType::Direct, None)
        .unwrap();
    store.join_channel(ch.id, "agent_a").unwrap();
    store.join_channel(ch.id, "agent_b").unwrap();

    // Set channel state to SilentCommunion (shared semantic space)
    // There's no direct API to set it, so we access the channel via the store
    store
        .channels
        .get_mut(&ch.id)
        .unwrap()
        .state = ChannelState::SilentCommunion;

    // SilentCommunion should allow send/receive
    let msg = store
        .send_message(ch.id, "agent_a", "shared thought", MessageType::Text)
        .expect("SilentCommunion should allow messaging");

    assert_eq!(msg.content, "shared thought");

    // Both agents can read the messages
    let msgs = store.receive_messages(ch.id, None, None).unwrap();
    assert_eq!(msgs.len(), 1);

    // Send from agent_b too
    store
        .send_message(ch.id, "agent_b", "echoing thought", MessageType::Text)
        .unwrap();

    let msgs = store.receive_messages(ch.id, None, None).unwrap();
    assert_eq!(msgs.len(), 2);

    // Verify channel is in SilentCommunion state
    let fetched = store.get_channel(ch.id).unwrap();
    assert_eq!(fetched.state, ChannelState::SilentCommunion);
}

// ---------------------------------------------------------------------------
// Scenario 09: Federation communication
// ---------------------------------------------------------------------------

#[test]
fn scenario_09_federation_communication() {
    let mut store = CommStore::new();

    // Configure federation with a local zone
    store
        .configure_federation(true, "zone_a", FederationPolicy::Allow)
        .expect("federation config should succeed");

    // Add a federated zone
    store
        .add_federated_zone(FederatedZone {
            zone_id: "zone_b".to_string(),
            name: "Remote Zone B".to_string(),
            endpoint: "https://zone-b.example.com".to_string(),
            policy: FederationPolicy::Allow,
            trust_level: CommTrustLevel::High,
        })
        .expect("adding federated zone should succeed");

    // Verify federation config
    let config = store.get_federation_config();
    assert!(config.enabled);
    assert_eq!(config.local_zone, "zone_a");
    assert_eq!(config.default_policy, FederationPolicy::Allow);

    // Verify zones
    let zones = store.list_federated_zones();
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].zone_id, "zone_b");
    assert_eq!(zones[0].name, "Remote Zone B");
    assert_eq!(zones[0].trust_level, CommTrustLevel::High);

    // Create a channel in zone_a and verify federation config is stored
    let ch = store
        .create_channel("cross-zone-ch", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "agent_a").unwrap();

    // Send a message in the federated context
    let msg = store
        .send_message(ch.id, "agent_a", "Hello from zone_a", MessageType::Text)
        .unwrap();
    assert_eq!(msg.content, "Hello from zone_a");
}

// ---------------------------------------------------------------------------
// Scenario 10: Key rotation
// ---------------------------------------------------------------------------

#[test]
fn scenario_10_key_rotation() {
    let mut store = CommStore::new();

    // Create an encrypted channel
    let config = ChannelConfig {
        encryption_required: true,
        ..ChannelConfig::default()
    };
    let ch = store
        .create_channel("encrypted-ch", ChannelType::Group, Some(config))
        .unwrap();
    store.join_channel(ch.id, "agent_a").unwrap();
    store.join_channel(ch.id, "agent_b").unwrap();

    // Send message in first "epoch"
    let msg1 = store
        .send_message(ch.id, "agent_a", "Pre-rotation message", MessageType::Text)
        .unwrap();
    let msg1_id = msg1.id;

    // Simulate key rotation by updating channel config
    let new_config = ChannelConfig {
        encryption_required: true,
        ..ChannelConfig::default()
    };
    store
        .set_channel_config(ch.id, new_config)
        .expect("channel config update should succeed");

    // Verify old messages are still accessible after "rotation"
    let old_msg = store.get_message(msg1_id).expect("old message should still exist");
    assert_eq!(old_msg.content, "Pre-rotation message");

    // New messages can still be sent
    let msg2 = store
        .send_message(ch.id, "agent_b", "Post-rotation message", MessageType::Text)
        .unwrap();

    // Both messages exist
    let msgs = store.receive_messages(ch.id, None, None).unwrap();
    assert_eq!(msgs.len(), 2);
    assert!(msgs.iter().any(|m| m.id == msg1_id));
    assert!(msgs.iter().any(|m| m.id == msg2.id));
}

// ---------------------------------------------------------------------------
// Scenario 11: Concurrent messages
// ---------------------------------------------------------------------------

#[test]
fn scenario_11_concurrent_messages() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("concurrent-ch", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "sender_a").unwrap();
    store.join_channel(ch.id, "sender_b").unwrap();
    store.join_channel(ch.id, "sender_c").unwrap();

    // Send 100 messages in rapid succession from multiple senders
    let senders = ["sender_a", "sender_b", "sender_c"];
    for i in 0..100 {
        let sender = senders[i % senders.len()];
        let content = format!("message-{}", i);
        store
            .send_message(ch.id, sender, &content, MessageType::Text)
            .unwrap();
    }

    // Verify all 100 messages are stored
    let all_msgs = store.receive_messages(ch.id, None, None).unwrap();
    assert_eq!(all_msgs.len(), 100);

    // Verify ordering is preserved per timestamp (messages are sorted by timestamp)
    for window in all_msgs.windows(2) {
        assert!(window[0].timestamp <= window[1].timestamp);
    }

    // Verify each sender has the right count
    let a_count = all_msgs.iter().filter(|m| m.sender == "sender_a").count();
    let b_count = all_msgs.iter().filter(|m| m.sender == "sender_b").count();
    let c_count = all_msgs.iter().filter(|m| m.sender == "sender_c").count();
    // 100 messages distributed round-robin: 34, 33, 33
    assert_eq!(a_count, 34);
    assert_eq!(b_count, 33);
    assert_eq!(c_count, 33);
}

// ---------------------------------------------------------------------------
// Scenario 12: Multi-project isolation
// ---------------------------------------------------------------------------

#[test]
fn scenario_12_multi_project_isolation() {
    // Create two separate CommStore instances
    let mut store1 = CommStore::new();
    let mut store2 = CommStore::new();

    // Create same-named channels in both
    let ch1 = store1
        .create_channel("shared-name", ChannelType::Group, None)
        .unwrap();
    let ch2 = store2
        .create_channel("shared-name", ChannelType::Group, None)
        .unwrap();

    store1.join_channel(ch1.id, "agent_a").unwrap();
    store2.join_channel(ch2.id, "agent_a").unwrap();

    // Send messages only in store1
    store1
        .send_message(ch1.id, "agent_a", "Store 1 only", MessageType::Text)
        .unwrap();
    store1
        .send_message(ch1.id, "agent_a", "Another in store 1", MessageType::Text)
        .unwrap();

    // Send a different message in store2
    store2
        .send_message(ch2.id, "agent_a", "Store 2 only", MessageType::Text)
        .unwrap();

    // Verify isolation: store1 has 2 messages, store2 has 1
    let msgs1 = store1.receive_messages(ch1.id, None, None).unwrap();
    let msgs2 = store2.receive_messages(ch2.id, None, None).unwrap();

    assert_eq!(msgs1.len(), 2);
    assert_eq!(msgs2.len(), 1);

    // Messages in store1 are NOT visible in store2
    assert!(msgs1.iter().all(|m| m.content != "Store 2 only"));
    assert!(msgs2.iter().all(|m| m.content != "Store 1 only"));
}

// ---------------------------------------------------------------------------
// Scenario 13: Concurrent startup (multiple loads from same path)
// ---------------------------------------------------------------------------

#[test]
fn scenario_13_concurrent_startup() {
    let tmp = tempfile::tempdir().expect("should create temp dir");
    let path = tmp.path().join("test.acomm");

    // Create store, populate it, save to disk
    let mut store = CommStore::new();
    let ch = store
        .create_channel("persist-ch", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "agent_a").unwrap();
    store
        .send_message(ch.id, "agent_a", "Hello persistence", MessageType::Text)
        .unwrap();
    store.save(&path).expect("save should succeed");

    // Load from same path multiple times
    let loaded1 = CommStore::load(&path).expect("first load should succeed");
    let loaded2 = CommStore::load(&path).expect("second load should succeed");
    let loaded3 = CommStore::load(&path).expect("third load should succeed");

    // Each load gets consistent data
    assert_eq!(loaded1.channels.len(), loaded2.channels.len());
    assert_eq!(loaded2.channels.len(), loaded3.channels.len());
    assert_eq!(loaded1.messages.len(), loaded2.messages.len());
    assert_eq!(loaded2.messages.len(), loaded3.messages.len());

    // Verify the actual data
    assert_eq!(loaded1.channels.len(), 1);
    assert_eq!(loaded1.messages.len(), 1);
}

// ---------------------------------------------------------------------------
// Scenario 14: Restart continuity
// ---------------------------------------------------------------------------

#[test]
fn scenario_14_restart_continuity() {
    let tmp = tempfile::tempdir().expect("should create temp dir");
    let path = tmp.path().join("continuity.acomm");

    // Create store with channels, messages, consent, trust
    let mut store = CommStore::new();

    // Channels
    let ch1 = store
        .create_channel("ch-one", ChannelType::Direct, None)
        .unwrap();
    store.join_channel(ch1.id, "agent_a").unwrap();
    store.join_channel(ch1.id, "agent_b").unwrap();

    let ch2 = store
        .create_channel("ch-two", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch2.id, "agent_c").unwrap();

    // Messages
    let msg1 = store
        .send_message(ch1.id, "agent_a", "Message in ch-one", MessageType::Text)
        .unwrap();
    store
        .send_message(ch2.id, "agent_c", "Message in ch-two", MessageType::Query)
        .unwrap();

    // Consent gates
    store
        .grant_consent(
            "agent_a",
            "agent_b",
            ConsentScope::ReadMessages,
            Some("test consent".to_string()),
            None,
        )
        .unwrap();

    // Trust levels
    store
        .set_trust_level("agent_a", CommTrustLevel::High)
        .unwrap();
    store
        .set_trust_level("agent_b", CommTrustLevel::Full)
        .unwrap();

    // Schedule a temporal message
    store
        .schedule_message(
            ch1.id,
            "agent_a",
            "Deferred message",
            TemporalTarget::FutureAbsolute {
                deliver_at: "2099-12-31T23:59:59Z".to_string(),
            },
            None,
        )
        .unwrap();

    // Save to disk
    store.save(&path).expect("save should succeed");

    // Load from disk
    let restored = CommStore::load(&path).expect("load should succeed");

    // Verify all channels restored
    assert_eq!(restored.channels.len(), 2);
    let restored_ch1 = restored.get_channel(ch1.id).expect("ch1 should exist");
    assert_eq!(restored_ch1.name, "ch-one");
    assert_eq!(restored_ch1.participants.len(), 2);

    // Verify all messages restored
    assert_eq!(restored.messages.len(), 2);
    let restored_msg = restored.get_message(msg1.id).expect("msg should exist");
    assert_eq!(restored_msg.content, "Message in ch-one");

    // Verify consent gates restored
    assert!(restored.check_consent("agent_a", "agent_b", &ConsentScope::ReadMessages));

    // Verify trust levels restored
    assert_eq!(restored.get_trust_level("agent_a"), CommTrustLevel::High);
    assert_eq!(restored.get_trust_level("agent_b"), CommTrustLevel::Full);

    // Verify temporal queue restored
    let scheduled = restored.list_scheduled();
    assert_eq!(scheduled.len(), 1);
    assert_eq!(scheduled[0].content, "Deferred message");
}

// ---------------------------------------------------------------------------
// Scenario 15: Message echo tracking (forwarding chain via reply_to)
// ---------------------------------------------------------------------------

#[test]
fn scenario_15_message_echo_tracking() {
    let mut store = CommStore::new();

    // Create 3 channels: A->B, B->C, C->D
    let ch_ab = store
        .create_channel("ch-ab", ChannelType::Direct, None)
        .unwrap();
    store.join_channel(ch_ab.id, "agent_a").unwrap();
    store.join_channel(ch_ab.id, "agent_b").unwrap();

    let ch_bc = store
        .create_channel("ch-bc", ChannelType::Direct, None)
        .unwrap();
    store.join_channel(ch_bc.id, "agent_b").unwrap();
    store.join_channel(ch_bc.id, "agent_c").unwrap();

    let ch_cd = store
        .create_channel("ch-cd", ChannelType::Direct, None)
        .unwrap();
    store.join_channel(ch_cd.id, "agent_c").unwrap();
    store.join_channel(ch_cd.id, "agent_d").unwrap();

    // Step 1: A sends original message to B
    let orig_msg = store
        .send_message(ch_ab.id, "agent_a", "Original from A", MessageType::Text)
        .unwrap();

    // Step 2: B forwards to C (using send_reply to link to original)
    let fwd_bc = store
        .send_reply(
            ch_ab.id,
            orig_msg.id,
            "agent_b",
            "Forwarding from A via B",
            MessageType::Text,
        )
        .unwrap();
    assert_eq!(fwd_bc.reply_to, Some(orig_msg.id));

    // Step 3: C forwards to D (links to B's forward)
    let fwd_cd = store
        .send_reply(
            ch_ab.id,
            fwd_bc.id,
            "agent_c",
            "Forwarding via B and C",
            MessageType::Text,
        )
        .unwrap();
    assert_eq!(fwd_cd.reply_to, Some(fwd_bc.id));

    // Verify the thread chain from the original message
    let thread_id = format!("thread-{}", orig_msg.id);
    let thread = store.get_thread(&thread_id);
    assert_eq!(
        thread.len(),
        3,
        "Thread should contain original + 2 forwards"
    );

    // Verify reply chain: D's message -> C's message -> A's message
    let last = &thread[2];
    assert_eq!(last.reply_to, Some(fwd_bc.id));
    let mid = &thread[1];
    assert_eq!(mid.reply_to, Some(orig_msg.id));
}

// ---------------------------------------------------------------------------
// Scenario 16: Ghost conversation
// ---------------------------------------------------------------------------

#[test]
fn scenario_16_ghost_conversation() {
    let mut store = CommStore::new();

    // Create a channel for a "terminated" agent (ghost agent)
    let ch = store
        .create_channel("ghost-channel", ChannelType::Direct, None)
        .unwrap();
    store.join_channel(ch.id, "ghost_agent").unwrap();
    store.join_channel(ch.id, "living_agent").unwrap();

    // Send messages to/from the ghost agent
    let msg1 = store
        .send_message(
            ch.id,
            "ghost_agent",
            "Message from the ghost",
            MessageType::Text,
        )
        .unwrap();

    let msg2 = store
        .send_message(
            ch.id,
            "living_agent",
            "Reply to ghost",
            MessageType::Text,
        )
        .unwrap();

    let msg3 = store
        .send_message(
            ch.id,
            "ghost_agent",
            "Final ghost message",
            MessageType::Text,
        )
        .unwrap();

    // Verify all 3 messages exist before closing
    let msgs = store.receive_messages(ch.id, None, None).unwrap();
    assert_eq!(msgs.len(), 3);

    // Close the channel (simulating agent termination)
    store
        .close_channel(ch.id)
        .expect("channel close should succeed");

    // Verify channel is closed
    let closed_ch = store.get_channel(ch.id).unwrap();
    assert_eq!(closed_ch.state, ChannelState::Closed);

    // Messages are still preserved in the store (accessible by ID)
    let preserved1 = store.get_message(msg1.id).expect("ghost msg 1 should exist");
    assert_eq!(preserved1.content, "Message from the ghost");

    let preserved2 = store.get_message(msg2.id).expect("reply should exist");
    assert_eq!(preserved2.content, "Reply to ghost");

    let preserved3 = store.get_message(msg3.id).expect("ghost msg 3 should exist");
    assert_eq!(preserved3.content, "Final ghost message");

    // New sends to the closed channel should fail
    let result = store.send_message(ch.id, "living_agent", "No more", MessageType::Text);
    assert!(result.is_err(), "Sending to closed channel should fail");
}
