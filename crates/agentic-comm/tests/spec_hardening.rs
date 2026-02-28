//! Spec Hardening Tests
//!
//! Additional tests covering rate limiting, Lamport clock ordering, audit
//! completeness, signature verification failures, and CommTimestamp merging.

use agentic_comm::*;

// ---------------------------------------------------------------------------
// Helper: create a store with a single group channel and two participants.
// ---------------------------------------------------------------------------
fn setup_store() -> (CommStore, u64) {
    let mut store = CommStore::new();
    let ch = store
        .create_channel("hardening-test", ChannelType::Group, None)
        .expect("channel creation should succeed");
    store.join_channel(ch.id, "alice").unwrap();
    store.join_channel(ch.id, "bob").unwrap();
    (store, ch.id)
}

// ===========================================================================
// 1. Rate limiting — send messages exceeding the rate limit, verify rejection
// ===========================================================================

#[test]
fn rate_limit_exceeds_threshold() {
    let (mut store, ch_id) = setup_store();
    // Set a very low rate limit for deterministic testing
    store.rate_limit_config.messages_per_minute = 2;

    // First two messages should succeed
    assert!(
        store
            .send_message(ch_id, "alice", "msg-1", MessageType::Text)
            .is_ok(),
        "First message within limit should succeed"
    );
    assert!(
        store
            .send_message(ch_id, "alice", "msg-2", MessageType::Text)
            .is_ok(),
        "Second message within limit should succeed"
    );

    // Third message should be rejected
    let result = store.send_message(ch_id, "alice", "msg-3", MessageType::Text);
    assert!(result.is_err(), "Third message should exceed rate limit");
    match result.unwrap_err() {
        CommError::RateLimitExceeded { limit } => {
            assert!(
                limit.contains("alice"),
                "Error should reference the sender, got: {}",
                limit
            );
        }
        other => panic!("Expected RateLimitExceeded, got: {:?}", other),
    }
}

#[test]
fn rate_limit_per_sender_isolation() {
    let (mut store, ch_id) = setup_store();
    store.rate_limit_config.messages_per_minute = 1;

    // Alice hits her limit
    store
        .send_message(ch_id, "alice", "alice-msg", MessageType::Text)
        .unwrap();
    assert!(
        store
            .send_message(ch_id, "alice", "alice-msg-2", MessageType::Text)
            .is_err(),
        "Alice should be rate-limited"
    );

    // Bob should still be able to send (independent rate tracker)
    assert!(
        store
            .send_message(ch_id, "bob", "bob-msg", MessageType::Text)
            .is_ok(),
        "Bob should have an independent rate limit"
    );
}

// ===========================================================================
// 2. Lamport clock ordering — send multiple messages, verify Lamport
//    counters increment monotonically.
// ===========================================================================

#[test]
fn lamport_clock_increments_on_send() {
    let (mut store, ch_id) = setup_store();

    let msg1 = store
        .send_message(ch_id, "alice", "first", MessageType::Text)
        .unwrap();
    let msg2 = store
        .send_message(ch_id, "alice", "second", MessageType::Text)
        .unwrap();
    let msg3 = store
        .send_message(ch_id, "bob", "third", MessageType::Text)
        .unwrap();

    // Lamport clocks should be monotonically increasing across the store
    assert!(
        msg2.comm_timestamp.lamport >= msg1.comm_timestamp.lamport,
        "Second message Lamport should be >= first: {} vs {}",
        msg2.comm_timestamp.lamport,
        msg1.comm_timestamp.lamport
    );
    assert!(
        msg3.comm_timestamp.lamport >= msg2.comm_timestamp.lamport,
        "Third message Lamport should be >= second: {} vs {}",
        msg3.comm_timestamp.lamport,
        msg2.comm_timestamp.lamport
    );
}

#[test]
fn lamport_clock_ordering_via_query() {
    let (mut store, ch_id) = setup_store();

    for i in 0..5 {
        store
            .send_message(
                ch_id,
                if i % 2 == 0 { "alice" } else { "bob" },
                &format!("msg-{}", i),
                MessageType::Text,
            )
            .unwrap();
    }

    let filter = MessageFilter {
        limit: Some(10),
        ..Default::default()
    };
    let history = store.query_history(ch_id, &filter);
    assert_eq!(history.len(), 5);

    // Verify Lamport clocks are non-decreasing in timestamp order
    for window in history.windows(2) {
        assert!(
            window[1].comm_timestamp.lamport >= window[0].comm_timestamp.lamport,
            "Messages should have non-decreasing Lamport clocks: {} vs {}",
            window[0].comm_timestamp.lamport,
            window[1].comm_timestamp.lamport
        );
    }
}

// ===========================================================================
// 3. Audit completeness — perform operations, verify each produces an audit
//    entry.
// ===========================================================================

#[test]
fn audit_entry_on_channel_creation() {
    let mut store = CommStore::new();
    let initial_audit_len = store.audit_log.len();

    store
        .create_channel("audit-ch", ChannelType::Group, None)
        .unwrap();

    assert!(
        store.audit_log.len() > initial_audit_len,
        "Creating a channel should produce an audit entry"
    );
    let last = store.audit_log.last().unwrap();
    assert_eq!(last.event_type, AuditEventType::ChannelCreated);
}

#[test]
fn audit_entry_on_message_send() {
    let (mut store, ch_id) = setup_store();
    let before = store.audit_log.len();

    store
        .send_message(ch_id, "alice", "audit test", MessageType::Text)
        .unwrap();

    assert!(
        store.audit_log.len() > before,
        "Sending a message should produce an audit entry"
    );
    // Find the MessageSent audit entry
    let sent_entries: Vec<_> = store
        .audit_log
        .iter()
        .filter(|e| matches!(e.event_type, AuditEventType::MessageSent))
        .collect();
    assert!(
        !sent_entries.is_empty(),
        "Should have at least one MessageSent audit entry"
    );
}

#[test]
fn audit_entry_on_consent_and_trust() {
    let mut store = CommStore::new();
    let before_consent = store.audit_log.len();

    // Grant consent
    store
        .grant_consent("alice", "bob", ConsentScope::SendMessages, None, None)
        .unwrap();
    assert!(
        store.audit_log.len() > before_consent,
        "Consent grant should produce an audit entry"
    );

    let before_trust = store.audit_log.len();
    store.set_trust_level("alice", CommTrustLevel::High);
    assert!(
        store.audit_log.len() > before_trust,
        "Trust level change should produce an audit entry"
    );
}

#[test]
fn audit_completeness_multiple_operations() {
    let mut store = CommStore::new();

    // Perform a sequence of diverse operations
    let ch = store
        .create_channel("audit-complete", ChannelType::Group, None)
        .unwrap();
    store.join_channel(ch.id, "alice").unwrap();
    store.join_channel(ch.id, "bob").unwrap();
    store
        .send_message(ch.id, "alice", "hello", MessageType::Text)
        .unwrap();
    store
        .grant_consent("alice", "bob", ConsentScope::SendMessages, None, None)
        .unwrap();
    store.set_trust_level("bob", CommTrustLevel::Standard);

    // We expect at least one audit entry per auditable operation:
    // channel_created, message_sent, consent_granted, trust_updated
    let event_types: Vec<_> = store
        .audit_log
        .iter()
        .map(|e| format!("{:?}", e.event_type))
        .collect();

    assert!(
        event_types.iter().any(|t| t.contains("ChannelCreated")),
        "Missing ChannelCreated audit entry; got: {:?}",
        event_types
    );
    assert!(
        event_types.iter().any(|t| t.contains("MessageSent")),
        "Missing MessageSent audit entry; got: {:?}",
        event_types
    );
    assert!(
        event_types.iter().any(|t| t.contains("ConsentGranted")),
        "Missing ConsentGranted audit entry; got: {:?}",
        event_types
    );
    assert!(
        event_types.iter().any(|t| t.contains("TrustUpdated")),
        "Missing TrustUpdated audit entry; got: {:?}",
        event_types
    );
}

// ===========================================================================
// 4. Signature verification failure — tamper with a message, verify detection
// ===========================================================================

#[test]
fn signature_verification_detects_tamper() {
    let (mut store, ch_id) = setup_store();

    let msg = store
        .send_message(ch_id, "alice", "original content", MessageType::Text)
        .unwrap();

    // The message should have a valid signature initially
    assert!(
        store.verify_message_signature(msg.id),
        "Freshly sent message should have a valid signature"
    );

    // Tamper with the message content directly
    if let Some(stored_msg) = store.messages.get_mut(&msg.id) {
        stored_msg.content = "TAMPERED content".to_string();
    }

    // Now verification should fail
    assert!(
        !store.verify_message_signature(msg.id),
        "Tampered message should fail signature verification"
    );

    // Check that a SignatureWarning audit entry was created
    let warnings: Vec<_> = store
        .audit_log
        .iter()
        .filter(|e| matches!(e.event_type, AuditEventType::SignatureWarning))
        .collect();
    assert!(
        !warnings.is_empty(),
        "Signature mismatch should produce a SignatureWarning audit entry"
    );
}

#[test]
fn signature_verification_passes_for_untampered() {
    let (mut store, ch_id) = setup_store();

    let msg = store
        .send_message(ch_id, "alice", "safe content", MessageType::Text)
        .unwrap();

    // Verify twice — should pass both times
    assert!(store.verify_message_signature(msg.id));
    assert!(store.verify_message_signature(msg.id));

    // No SignatureWarning should exist
    let warnings: Vec<_> = store
        .audit_log
        .iter()
        .filter(|e| matches!(e.event_type, AuditEventType::SignatureWarning))
        .collect();
    assert!(
        warnings.is_empty(),
        "Untampered message should not produce SignatureWarning"
    );
}

// ===========================================================================
// 5. CommTimestamp merge — merge two timestamps, verify vector clock
//    convergence.
// ===========================================================================

#[test]
fn comm_timestamp_merge_basic() {
    let mut ts_a = CommTimestamp::now("agent_a");
    ts_a.increment("agent_a");
    ts_a.increment("agent_a");

    let mut ts_b = CommTimestamp::now("agent_b");
    ts_b.increment("agent_b");

    // Before merge: A has {agent_a: 2}, B has {agent_b: 1}
    assert_eq!(
        *ts_a.vector_clock.get("agent_a").unwrap_or(&0),
        2,
        "agent_a should have clock value 2 before merge"
    );
    assert_eq!(
        *ts_b.vector_clock.get("agent_b").unwrap_or(&0),
        1,
        "agent_b should have clock value 1 before merge"
    );

    let lamport_before = ts_a.lamport;

    // Merge B into A
    ts_a.merge(&ts_b, "agent_a");

    // After merge:
    // - agent_a's vector clock should contain both agent_a and agent_b entries
    // - The Lamport clock should be max(A.lamport, B.lamport) + 1
    assert!(
        ts_a.vector_clock.contains_key("agent_a"),
        "Merged clock should contain agent_a"
    );
    assert!(
        ts_a.vector_clock.contains_key("agent_b"),
        "Merged clock should contain agent_b"
    );

    // agent_b entry should be at least 1 (from B's clock)
    assert!(
        *ts_a.vector_clock.get("agent_b").unwrap() >= 1,
        "agent_b entry should be >= 1 after merge"
    );

    // agent_a entry should have been incremented by 1 during merge
    assert!(
        *ts_a.vector_clock.get("agent_a").unwrap() >= 3,
        "agent_a entry should be >= 3 after merge (was 2, incremented)"
    );

    // Lamport should have advanced
    assert!(
        ts_a.lamport > lamport_before,
        "Lamport clock should advance after merge"
    );
}

#[test]
fn comm_timestamp_merge_convergence() {
    // Simulate two agents that each perform independent work, then merge
    let mut ts_a = CommTimestamp::now("agent_a");
    let mut ts_b = CommTimestamp::now("agent_b");

    // Agent A does 3 operations
    for _ in 0..3 {
        ts_a.increment("agent_a");
    }
    // Agent B does 5 operations
    for _ in 0..5 {
        ts_b.increment("agent_b");
    }

    // Merge B into A
    ts_a.merge(&ts_b, "agent_a");
    // Merge A into B (B now sees A's merged state)
    ts_b.merge(&ts_a, "agent_b");

    // After bidirectional merge, both should see each other's entries
    assert!(
        ts_a.vector_clock.contains_key("agent_b"),
        "A should know about B after merge"
    );
    assert!(
        ts_b.vector_clock.contains_key("agent_a"),
        "B should know about A after merge"
    );

    // The max of each entry in both clocks should be the same
    // (convergence property of vector clocks)
    for key in ts_a.vector_clock.keys() {
        let a_val = ts_a.vector_clock.get(key).copied().unwrap_or(0);
        let b_val = ts_b.vector_clock.get(key).copied().unwrap_or(0);
        // B merged A's state, so B should have at least A's values
        assert!(
            b_val >= a_val || key == "agent_a",
            "After bidirectional merge, B's clock for {} ({}) should be >= A's ({})",
            key,
            b_val,
            a_val
        );
    }
}

#[test]
fn comm_timestamp_happens_before() {
    let mut ts_a = CommTimestamp::now("agent_a");
    ts_a.increment("agent_a");

    let mut ts_b = ts_a.clone();
    ts_b.increment("agent_a"); // Advance beyond A

    // A should happen-before B
    assert!(
        ts_a.happens_before(&ts_b),
        "ts_a should happen-before ts_b since ts_b advanced further"
    );
    // B should NOT happen-before A
    assert!(
        !ts_b.happens_before(&ts_a),
        "ts_b should NOT happen-before ts_a"
    );
}

// ===========================================================================
// Bonus: MessageFilter new field tests
// ===========================================================================

#[test]
fn message_filter_priority() {
    let (mut store, ch_id) = setup_store();

    // Send messages with different priorities
    store
        .send_message(ch_id, "alice", "normal msg", MessageType::Text)
        .unwrap();
    let high_msg = store
        .send_message_with_priority(
            ch_id,
            "bob",
            "urgent msg",
            MessageType::Text,
            MessagePriority::High,
        )
        .unwrap();

    // Filter by High priority (value = 2)
    let filter = MessageFilter {
        priority: Some(2), // High
        ..Default::default()
    };
    let results = store.query_history(ch_id, &filter);
    assert_eq!(results.len(), 1, "Should find exactly one High priority message");
    assert_eq!(results[0].id, high_msg.id);
}

#[test]
fn message_filter_content_contains() {
    let (mut store, ch_id) = setup_store();

    store
        .send_message(ch_id, "alice", "Hello world", MessageType::Text)
        .unwrap();
    store
        .send_message(ch_id, "bob", "Goodbye cruel world", MessageType::Text)
        .unwrap();
    store
        .send_message(ch_id, "alice", "Something else", MessageType::Text)
        .unwrap();

    let filter = MessageFilter {
        content_contains: Some("world".to_string()),
        ..Default::default()
    };
    let results = store.query_history(ch_id, &filter);
    assert_eq!(results.len(), 2, "Should find two messages containing 'world'");
}

#[test]
fn message_filter_content_contains_case_insensitive() {
    let (mut store, ch_id) = setup_store();

    store
        .send_message(ch_id, "alice", "HELLO WORLD", MessageType::Text)
        .unwrap();

    let filter = MessageFilter {
        content_contains: Some("hello".to_string()),
        ..Default::default()
    };
    let results = store.query_history(ch_id, &filter);
    assert_eq!(
        results.len(),
        1,
        "Content filter should be case-insensitive"
    );
}

// ===========================================================================
// Bonus: ChannelType new variants
// ===========================================================================

#[test]
fn channel_type_new_variants_roundtrip() {
    let variants = vec![
        ("telepathic", ChannelType::Telepathic),
        ("hive", ChannelType::Hive),
        ("temporal", ChannelType::Temporal),
        ("destiny", ChannelType::Destiny),
        ("oracle", ChannelType::Oracle),
    ];

    for (name, expected) in &variants {
        let parsed: ChannelType = name.parse().expect(&format!("Should parse '{}'", name));
        assert_eq!(&parsed, expected);
        assert_eq!(parsed.to_string(), *name);
    }
}

#[test]
fn channel_type_new_variants_create_channel() {
    let mut store = CommStore::new();

    let ch = store
        .create_channel("oracle-ch", ChannelType::Oracle, None)
        .unwrap();
    assert_eq!(ch.channel_type, ChannelType::Oracle);

    let ch2 = store
        .create_channel("telepathic-ch", ChannelType::Telepathic, None)
        .unwrap();
    assert_eq!(ch2.channel_type, ChannelType::Telepathic);
}

// ===========================================================================
// Bonus: ground_evidence and ground_suggest tests
// ===========================================================================

#[test]
fn ground_evidence_finds_messages() {
    let (mut store, ch_id) = setup_store();
    store
        .send_message(ch_id, "alice", "the quick brown fox", MessageType::Text)
        .unwrap();
    store
        .send_message(ch_id, "bob", "lazy dog sleeps", MessageType::Text)
        .unwrap();

    let evidence = store.ground_evidence("fox");
    assert!(
        !evidence.is_empty(),
        "Should find evidence for 'fox' in messages"
    );
    assert!(
        evidence.iter().any(|e| e.content.contains("fox")),
        "Evidence should contain the matching content"
    );
}

#[test]
fn ground_evidence_finds_channels() {
    let mut store = CommStore::new();
    store
        .create_channel("alpha-channel", ChannelType::Group, None)
        .unwrap();

    let evidence = store.ground_evidence("alpha");
    assert!(
        !evidence.is_empty(),
        "Should find evidence for 'alpha' in channel names"
    );
    assert!(
        evidence
            .iter()
            .any(|e| e.evidence_type.starts_with("channel")),
        "Evidence should be of channel type"
    );
}

#[test]
fn ground_suggest_returns_suggestions() {
    let (mut store, ch_id) = setup_store();
    store
        .send_message(ch_id, "alice", "important meeting notes", MessageType::Text)
        .unwrap();

    let suggestions = store.ground_suggest("alice", 10);
    assert!(
        !suggestions.is_empty(),
        "Should return suggestions for 'alice'"
    );
    assert!(
        suggestions.iter().any(|s| s.contains("alice")),
        "Suggestions should contain alice: {:?}",
        suggestions
    );
}

#[test]
fn ground_suggest_respects_limit() {
    let mut store = CommStore::new();
    // Create many channels to test limit
    for i in 0..20 {
        store
            .create_channel(&format!("test-ch-{}", i), ChannelType::Group, None)
            .unwrap();
    }

    let suggestions = store.ground_suggest("test", 5);
    assert!(
        suggestions.len() <= 5,
        "Should respect the limit of 5, got: {}",
        suggestions.len()
    );
}
