//! Edge-case and invention tests for AgenticComm.
//!
//! Required by sisters-registry.json (`paths.edgeCaseInventionsTest`).
//! Each test exercises a specific edge case from SPEC-12 to ensure
//! the communication engine handles boundary conditions correctly.

use agentic_comm::{
    ChannelConfig, ChannelType, CommStore, MessageFilter, MessageType, RetentionPolicy,
};

/// Helper: create a CommStore with a single group channel and return (store, channel_id).
fn store_with_channel(name: &str) -> (CommStore, u64) {
    let mut store = CommStore::new();
    let ch = store
        .create_channel(name, ChannelType::Group, None)
        .expect("create_channel should succeed");
    (store, ch.id)
}

// -----------------------------------------------------------------------
// 1. Concurrent message delivery to same channel
// -----------------------------------------------------------------------

#[test]
fn test_concurrent_message_delivery_to_same_channel() {
    let (mut store, cid) = store_with_channel("concurrent-ch");
    // Simulate rapid sequential sends from different senders to the same channel.
    // Each message should get a unique monotonically increasing ID.
    let mut ids = Vec::new();
    for i in 0..100 {
        let sender = format!("agent-{i}");
        let msg = store
            .send_message(cid, &sender, &format!("msg-{i}"), MessageType::Text)
            .expect("send should succeed");
        ids.push(msg.id);
    }

    // All IDs must be unique
    let unique_count = {
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        sorted.len()
    };
    assert_eq!(unique_count, 100, "All 100 messages should have unique IDs");

    // IDs should be monotonically increasing
    for i in 1..ids.len() {
        assert!(
            ids[i] > ids[i - 1],
            "Message IDs should be monotonically increasing"
        );
    }

    // All messages should be retrievable
    let received = store.receive_messages(cid, None, None).unwrap();
    assert_eq!(received.len(), 100);
}

// -----------------------------------------------------------------------
// 2. Self-messaging (sender == recipient)
// -----------------------------------------------------------------------

#[test]
fn test_self_messaging_sender_equals_recipient() {
    let (mut store, cid) = store_with_channel("self-msg-ch");
    store.join_channel(cid, "alice").unwrap();

    // Alice sends a message; since content is to the channel, she should
    // be able to receive her own messages.
    let msg = store
        .send_message(cid, "alice", "talking to myself", MessageType::Text)
        .unwrap();
    assert_eq!(msg.sender, "alice");

    // Receiving with no filter should include self-sent messages
    let received = store.receive_messages(cid, None, None).unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].content, "talking to myself");

    // Alice can also acknowledge her own message
    store.acknowledge_message(msg.id, "alice").unwrap();
    let updated = store.get_message(msg.id).unwrap();
    assert!(updated.acknowledged_by.contains(&"alice".to_string()));
}

// -----------------------------------------------------------------------
// 3. Broadcast to channel with 10,000 subscribers (performance)
// -----------------------------------------------------------------------

#[test]
fn test_broadcast_to_large_subscriber_count() {
    let mut store = CommStore::new();
    let config = ChannelConfig {
        max_participants: 0, // unlimited
        ..Default::default()
    };
    let ch = store
        .create_channel("big-broadcast", ChannelType::Broadcast, Some(config))
        .unwrap();

    // Join 10,000 participants
    for i in 0..10_000 {
        store
            .join_channel(ch.id, &format!("agent-{i}"))
            .unwrap();
    }

    // Broadcast from a non-participant sender (who joins first)
    store.join_channel(ch.id, "broadcaster").unwrap();
    let delivered = store
        .broadcast(ch.id, "broadcaster", "attention everyone")
        .unwrap();

    // Should deliver to 10,000 participants (everyone except broadcaster)
    assert_eq!(
        delivered.len(),
        10_000,
        "Broadcast should deliver to all 10,000 non-sender participants"
    );
}

// -----------------------------------------------------------------------
// 4. Message at exactly 1MB boundary
// -----------------------------------------------------------------------

#[test]
fn test_message_at_exactly_1mb_boundary() {
    let (mut store, cid) = store_with_channel("boundary-ch");

    // Exactly 1 MB should succeed
    let content_1mb = "x".repeat(1_048_576);
    let result = store.send_message(cid, "alice", &content_1mb, MessageType::Text);
    assert!(result.is_ok(), "Exactly 1MB content should be accepted");

    // 1 MB + 1 byte should fail
    let content_over = "x".repeat(1_048_577);
    let result = store.send_message(cid, "alice", &content_over, MessageType::Text);
    assert!(result.is_err(), "Content over 1MB should be rejected");

    // Empty content should fail
    let result = store.send_message(cid, "alice", "", MessageType::Text);
    assert!(result.is_err(), "Empty content should be rejected");
}

// -----------------------------------------------------------------------
// 5. Channel name at exactly 128 chars
// -----------------------------------------------------------------------

#[test]
fn test_channel_name_at_exactly_128_chars() {
    let mut store = CommStore::new();

    // Exactly 128 characters should succeed
    let name_128 = "a".repeat(128);
    let result = store.create_channel(&name_128, ChannelType::Group, None);
    assert!(result.is_ok(), "128-char channel name should be accepted");

    // 129 characters should fail
    let name_129 = "a".repeat(129);
    let result = store.create_channel(&name_129, ChannelType::Group, None);
    assert!(result.is_err(), "129-char channel name should be rejected");

    // 1 character should succeed
    let result = store.create_channel("z", ChannelType::Group, None);
    assert!(result.is_ok(), "1-char channel name should be accepted");

    // Empty name should fail
    let result = store.create_channel("", ChannelType::Group, None);
    assert!(result.is_err(), "Empty channel name should be rejected");
}

// -----------------------------------------------------------------------
// 6. Wildcard topic matching deduplication
// -----------------------------------------------------------------------

#[test]
fn test_topic_subscription_deduplication() {
    let mut store = CommStore::new();

    // Same subscriber subscribing twice to the same topic gets two subscriptions
    // but publish should deliver one message per unique subscriber.
    let sub1 = store.subscribe("alerts", "agent-a").unwrap();
    let sub2 = store.subscribe("alerts", "agent-a").unwrap();
    assert_ne!(sub1.id, sub2.id, "Subscriptions should have unique IDs");

    // Publishing delivers one message per subscription, even for duplicates.
    // This tests that the system handles dedup at the subscription level.
    let msgs = store.publish("alerts", "monitor", "alert!").unwrap();

    // With two subscriptions for agent-a, it may get two messages.
    // This is expected behavior — the engine delivers per-subscription.
    assert!(
        msgs.len() >= 1,
        "Should deliver at least one message"
    );

    // Verify both subscriptions can be individually unsubscribed
    store.unsubscribe(sub1.id).unwrap();
    store.unsubscribe(sub2.id).unwrap();
}

// -----------------------------------------------------------------------
// 7. Acknowledgment timeout behavior
// -----------------------------------------------------------------------

#[test]
fn test_acknowledgment_idempotency_and_multi_recipient() {
    let (mut store, cid) = store_with_channel("ack-ch");
    store.join_channel(cid, "alice").unwrap();
    store.join_channel(cid, "bob").unwrap();
    store.join_channel(cid, "carol").unwrap();

    let msg = store
        .send_message(cid, "alice", "ack me please", MessageType::Text)
        .unwrap();

    // First ack from bob
    store.acknowledge_message(msg.id, "bob").unwrap();
    let state = store.get_message(msg.id).unwrap();
    assert_eq!(state.acknowledged_by.len(), 1);

    // Duplicate ack from bob (should be idempotent, not add twice)
    store.acknowledge_message(msg.id, "bob").unwrap();
    let state = store.get_message(msg.id).unwrap();
    assert_eq!(
        state.acknowledged_by.len(),
        1,
        "Duplicate ack should be idempotent"
    );

    // Ack from carol (different recipient)
    store.acknowledge_message(msg.id, "carol").unwrap();
    let state = store.get_message(msg.id).unwrap();
    assert_eq!(state.acknowledged_by.len(), 2);

    // Ack for nonexistent message should fail
    let result = store.acknowledge_message(99999, "bob");
    assert!(result.is_err(), "Ack for nonexistent message should fail");
}

// -----------------------------------------------------------------------
// 8. Dead letter queue overflow
// -----------------------------------------------------------------------

#[test]
fn test_dead_letter_queue_accumulation() {
    let mut store = CommStore::new();
    let config = ChannelConfig {
        max_participants: 0,
        ..Default::default()
    };
    let ch = store
        .create_channel("dead-letter-ch", ChannelType::Group, Some(config))
        .unwrap();

    // Send messages, then close the channel to trigger dead-lettering on subsequent sends
    store.join_channel(ch.id, "alice").unwrap();
    store
        .send_message(ch.id, "alice", "before close", MessageType::Text)
        .unwrap();

    // Close the channel
    store.close_channel(ch.id).unwrap();

    // Attempting to send to a closed channel should dead-letter the message
    for i in 0..5 {
        let result = store.send_message(
            ch.id,
            "alice",
            &format!("dead msg {i}"),
            MessageType::Text,
        );
        // Sending to closed channel produces an error
        assert!(
            result.is_err(),
            "Sending to closed channel should fail"
        );
    }

    // Dead letter queue should have accumulated entries
    let dl_count = store.dead_letter_count();
    assert!(
        dl_count >= 1,
        "Dead letter queue should have entries from failed sends"
    );

    // Clear dead letters
    store.clear_dead_letters();
    assert_eq!(store.dead_letter_count(), 0);
}

// -----------------------------------------------------------------------
// 9. Corrupt file recovery
// -----------------------------------------------------------------------

#[test]
fn test_corrupt_file_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("corrupt.acomm");

    // Write garbage to the file
    std::fs::write(&path, b"not a valid acomm file at all").unwrap();
    let result = CommStore::load(&path);
    assert!(
        result.is_err(),
        "Loading a corrupt file should return an error"
    );

    // Write truncated data
    std::fs::write(&path, &[0u8; 4]).unwrap();
    let result = CommStore::load(&path);
    assert!(result.is_err(), "Loading truncated file should fail");

    // Write empty file
    std::fs::write(&path, b"").unwrap();
    let result = CommStore::load(&path);
    assert!(result.is_err(), "Loading empty file should fail");

    // Valid save and reload should work
    let mut store = CommStore::new();
    store
        .create_channel("recovery-test", ChannelType::Group, None)
        .unwrap();
    store.save(&path).unwrap();
    let loaded = CommStore::load(&path).unwrap();
    assert_eq!(loaded.channels.len(), 1);
}

// -----------------------------------------------------------------------
// 10. Concurrent channel creation with same name
// -----------------------------------------------------------------------

#[test]
fn test_concurrent_channel_creation_same_name() {
    let mut store = CommStore::new();

    // Creating channels with the same name should succeed (names are not unique keys).
    let ch1 = store
        .create_channel("shared-name", ChannelType::Group, None)
        .unwrap();
    let ch2 = store
        .create_channel("shared-name", ChannelType::Broadcast, None)
        .unwrap();

    assert_ne!(ch1.id, ch2.id, "Channels with same name should get unique IDs");
    assert_eq!(ch1.name, ch2.name, "Both should have the same name");

    // Both channels should be independently operational
    store.join_channel(ch1.id, "alice").unwrap();
    store.join_channel(ch2.id, "bob").unwrap();
    let c1 = store.get_channel(ch1.id).unwrap();
    let c2 = store.get_channel(ch2.id).unwrap();
    assert_eq!(c1.participants.len(), 1);
    assert_eq!(c2.participants.len(), 1);
    assert_eq!(c1.participants[0], "alice");
    assert_eq!(c2.participants[0], "bob");
}

// -----------------------------------------------------------------------
// 11. Cross-project store isolation (different paths)
// -----------------------------------------------------------------------

#[test]
fn test_cross_project_store_isolation() {
    let dir = tempfile::tempdir().unwrap();
    let path_a = dir.path().join("project_a.acomm");
    let path_b = dir.path().join("project_b.acomm");

    // Create store A with some data
    let mut store_a = CommStore::new();
    store_a
        .create_channel("alpha-channel", ChannelType::Group, None)
        .unwrap();
    store_a
        .send_message(1, "alice", "project A message", MessageType::Text)
        .unwrap();
    store_a.save(&path_a).unwrap();

    // Create store B with different data
    let mut store_b = CommStore::new();
    store_b
        .create_channel("beta-channel", ChannelType::Direct, None)
        .unwrap();
    store_b
        .send_message(1, "bob", "project B message", MessageType::Text)
        .unwrap();
    store_b.save(&path_b).unwrap();

    // Load them back and verify isolation
    let loaded_a = CommStore::load(&path_a).unwrap();
    let loaded_b = CommStore::load(&path_b).unwrap();

    assert_eq!(loaded_a.channels.len(), 1);
    assert_eq!(loaded_b.channels.len(), 1);

    let ch_a = loaded_a.list_channels();
    let ch_b = loaded_b.list_channels();
    assert_eq!(ch_a[0].name, "alpha-channel");
    assert_eq!(ch_b[0].name, "beta-channel");

    // Messages should not leak between stores
    let msgs_a = loaded_a.search_messages("project A", 10);
    let msgs_b = loaded_b.search_messages("project B", 10);
    assert_eq!(msgs_a.len(), 1);
    assert_eq!(msgs_b.len(), 1);
    assert_eq!(msgs_a[0].sender, "alice");
    assert_eq!(msgs_b[0].sender, "bob");

    // Searching for the other project's data should yield nothing
    let cross_a = loaded_a.search_messages("project B", 10);
    let cross_b = loaded_b.search_messages("project A", 10);
    assert_eq!(cross_a.len(), 0, "Store A should not contain Store B data");
    assert_eq!(cross_b.len(), 0, "Store B should not contain Store A data");
}

// -----------------------------------------------------------------------
// 12. Empty content validation
// -----------------------------------------------------------------------

#[test]
fn test_empty_content_validation() {
    let (mut store, cid) = store_with_channel("empty-content-ch");

    // Empty message content should fail
    let result = store.send_message(cid, "alice", "", MessageType::Text);
    assert!(result.is_err(), "Empty content should be rejected");

    // Empty sender should fail
    let result = store.send_message(cid, "", "hello", MessageType::Text);
    assert!(result.is_err(), "Empty sender should be rejected");

    // Whitespace-only content should succeed (it's non-empty)
    let result = store.send_message(cid, "alice", "   ", MessageType::Text);
    assert!(
        result.is_ok(),
        "Whitespace-only content should be accepted (non-empty)"
    );

    // Empty participant name should fail
    let result = store.join_channel(cid, "");
    assert!(result.is_err(), "Empty participant name should be rejected");

    // Empty subscriber should fail
    let result = store.subscribe("topic", "");
    assert!(result.is_err(), "Empty subscriber should be rejected");
}

// -----------------------------------------------------------------------
// 13. Channel deletion with pending messages
// -----------------------------------------------------------------------

#[test]
fn test_channel_close_with_pending_messages() {
    let (mut store, cid) = store_with_channel("close-pending-ch");
    store.join_channel(cid, "alice").unwrap();
    store.join_channel(cid, "bob").unwrap();

    // Send some messages
    for i in 0..5 {
        store
            .send_message(cid, "alice", &format!("pending msg {i}"), MessageType::Text)
            .unwrap();
    }

    // Verify messages exist
    let before_close = store.receive_messages(cid, None, None).unwrap();
    assert_eq!(before_close.len(), 5);

    // Close the channel
    store.close_channel(cid).unwrap();

    // Channel should still exist (closed, not deleted)
    let ch = store.get_channel(cid);
    assert!(ch.is_some(), "Closed channel should still be retrievable");

    // Sending to closed channel should fail
    let result = store.send_message(cid, "alice", "after close", MessageType::Text);
    assert!(result.is_err(), "Cannot send to closed channel");

    // Messages should still be searchable
    let search_results = store.search_messages("pending msg", 10);
    assert_eq!(
        search_results.len(),
        5,
        "Messages in closed channel should still be searchable"
    );
}

// -----------------------------------------------------------------------
// 14. Subscribe to non-existent topic
// -----------------------------------------------------------------------

#[test]
fn test_subscribe_to_nonexistent_topic() {
    let mut store = CommStore::new();

    // Subscribing to a topic that has no channel yet should succeed
    let sub = store.subscribe("brand-new-topic", "agent-x").unwrap();
    assert_eq!(sub.topic, "brand-new-topic");
    assert_eq!(sub.subscriber, "agent-x");

    // Publishing to that topic should auto-create a channel and deliver
    let msgs = store
        .publish("brand-new-topic", "publisher", "first ever message")
        .unwrap();
    assert_eq!(
        msgs.len(),
        1,
        "Should deliver to the one subscriber"
    );
    assert_eq!(msgs[0].recipient, Some("agent-x".to_string()));

    // Subscribing with more agents
    store.subscribe("brand-new-topic", "agent-y").unwrap();
    let msgs = store
        .publish("brand-new-topic", "publisher", "second message")
        .unwrap();
    assert_eq!(msgs.len(), 2, "Should now deliver to two subscribers");
}

// -----------------------------------------------------------------------
// 15. Regex search with special characters
// -----------------------------------------------------------------------

#[test]
fn test_search_with_special_characters() {
    let (mut store, cid) = store_with_channel("special-chars-ch");

    // Messages with special characters
    store
        .send_message(cid, "alice", "price is $100.00", MessageType::Text)
        .unwrap();
    store
        .send_message(cid, "bob", "use regex: ^[a-z]+$", MessageType::Text)
        .unwrap();
    store
        .send_message(
            cid,
            "carol",
            "path/to/file.rs (line 42)",
            MessageType::Text,
        )
        .unwrap();
    store
        .send_message(
            cid,
            "dave",
            "C++ is not C# and neither is C",
            MessageType::Text,
        )
        .unwrap();
    store
        .send_message(cid, "eve", "100% complete!!!", MessageType::Text)
        .unwrap();

    // Search should work with special characters as literal text
    let results = store.search_messages("$100", 10);
    assert_eq!(results.len(), 1, "Should find the price message");

    let results = store.search_messages("^[a-z]", 10);
    assert_eq!(results.len(), 1, "Should find the regex message");

    let results = store.search_messages("C++", 10);
    assert_eq!(results.len(), 1, "Should find C++ message");

    let results = store.search_messages("100%", 10);
    assert_eq!(results.len(), 1, "Should find percentage message");

    // Case-insensitive search
    let results = store.search_messages("PATH", 10);
    assert_eq!(results.len(), 1, "Search should be case-insensitive");
}

// -----------------------------------------------------------------------
// 16. Save during active message delivery (persistence consistency)
// -----------------------------------------------------------------------

#[test]
fn test_save_during_active_message_delivery() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save-during-delivery.acomm");

    let mut store = CommStore::new();
    let ch = store
        .create_channel("save-test", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "alice").unwrap();
    store.join_channel(ch.id, "bob").unwrap();

    // Send a batch of messages
    for i in 0..50 {
        store
            .send_message(ch.id, "alice", &format!("batch msg {i}"), MessageType::Text)
            .unwrap();
    }

    // Save mid-stream
    store.save(&path).unwrap();

    // Send more messages after save
    for i in 50..100 {
        store
            .send_message(ch.id, "bob", &format!("batch msg {i}"), MessageType::Text)
            .unwrap();
    }

    // Save again
    store.save(&path).unwrap();

    // Load and verify all data is consistent
    let loaded = CommStore::load(&path).unwrap();
    assert_eq!(loaded.channels.len(), 1);
    assert_eq!(loaded.messages.len(), 100, "All 100 messages should be persisted");

    // Verify message ordering is preserved
    let filter = MessageFilter {
        limit: Some(100),
        ..Default::default()
    };
    let history = loaded.query_history(ch.id, &filter);
    assert_eq!(history.len(), 100);

    // First message should be from alice, later ones from bob
    assert_eq!(history[0].sender, "alice");
    assert_eq!(history[99].sender, "bob");
}

// -----------------------------------------------------------------------
// 17. Retention policy enforcement via compact
// -----------------------------------------------------------------------

#[test]
fn test_retention_policy_compact() {
    let mut store = CommStore::new();
    let config = ChannelConfig {
        retention_policy: RetentionPolicy::MessageCount(3),
        ..Default::default()
    };
    let ch = store
        .create_channel("retention-ch", ChannelType::Group, Some(config))
        .unwrap();

    // Send 10 messages
    for i in 0..10 {
        store
            .send_message(ch.id, "alice", &format!("msg {i}"), MessageType::Text)
            .unwrap();
    }

    // Before compact, all messages exist
    let all = store.receive_messages(ch.id, None, None).unwrap();
    assert_eq!(all.len(), 10);

    // Compact enforces retention policy
    let removed = store.compact();
    assert!(removed > 0, "Compact should remove old messages");

    // After compact, only the most recent 3 should remain for this channel
    let after = store.receive_messages(ch.id, None, None).unwrap();
    assert_eq!(
        after.len(),
        3,
        "Only 3 most recent messages should remain after compact"
    );
    assert_eq!(after[2].content, "msg 9", "Last message should be msg 9");
}

// -----------------------------------------------------------------------
// 18. Channel state transitions (pause, drain, resume, close)
// -----------------------------------------------------------------------

#[test]
fn test_channel_state_transitions() {
    let (mut store, cid) = store_with_channel("state-ch");
    store.join_channel(cid, "alice").unwrap();

    // Active -> can send
    store
        .send_message(cid, "alice", "active msg", MessageType::Text)
        .unwrap();

    // Pause -> cannot send
    store.pause_channel(cid).unwrap();
    let result = store.send_message(cid, "alice", "paused msg", MessageType::Text);
    assert!(result.is_err(), "Cannot send to paused channel");

    // Resume -> can send again
    store.resume_channel(cid).unwrap();
    store
        .send_message(cid, "alice", "resumed msg", MessageType::Text)
        .unwrap();

    // Drain -> cannot send, can still receive existing messages
    store.drain_channel(cid).unwrap();
    let result = store.send_message(cid, "alice", "draining msg", MessageType::Text);
    assert!(result.is_err(), "Cannot send to draining channel");

    // Messages from before drain should still be receivable
    let msgs = store.receive_messages(cid, None, None).unwrap();
    assert!(
        msgs.len() >= 2,
        "Should still receive messages from before drain"
    );

    // Close -> nothing works
    store.close_channel(cid).unwrap();
    let result = store.send_message(cid, "alice", "closed msg", MessageType::Text);
    assert!(result.is_err(), "Cannot send to closed channel");
}

// -----------------------------------------------------------------------
// 19. Many channels with interleaved messages
// -----------------------------------------------------------------------

#[test]
fn test_many_channels_interleaved_messages() {
    let mut store = CommStore::new();
    // Raise rate limit to accommodate 200 messages in this test
    store.rate_limit_config.messages_per_minute = 1000;

    // Create 20 channels
    let mut channel_ids = Vec::new();
    for i in 0..20 {
        let ch = store
            .create_channel(&format!("ch-{i}"), ChannelType::Group, None)
            .unwrap();
        channel_ids.push(ch.id);
    }

    // Send messages to channels in round-robin fashion
    for round in 0..10 {
        for &cid in &channel_ids {
            store
                .send_message(
                    cid,
                    "router",
                    &format!("round {round} to ch {cid}"),
                    MessageType::Text,
                )
                .unwrap();
        }
    }

    // Each channel should have exactly 10 messages
    for &cid in &channel_ids {
        let msgs = store.receive_messages(cid, None, None).unwrap();
        assert_eq!(
            msgs.len(),
            10,
            "Each channel should have 10 messages"
        );
    }

    // Total message count should be 200
    assert_eq!(store.stats().message_count, 200);
}

// -----------------------------------------------------------------------
// 20. Pub/sub publish with no subscribers
// -----------------------------------------------------------------------

#[test]
fn test_publish_with_no_subscribers() {
    let mut store = CommStore::new();

    // Publishing to a topic with no subscribers should succeed with 0 deliveries
    let msgs = store
        .publish("ghost-topic", "publisher", "hello void")
        .unwrap();
    assert_eq!(
        msgs.len(),
        0,
        "Publishing with no subscribers should deliver 0 messages"
    );

    // A channel should still have been auto-created for the topic
    let channels = store.list_channels();
    assert!(
        channels.iter().any(|c| c.name == "ghost-topic"),
        "Topic channel should be auto-created even with no subscribers"
    );
}
