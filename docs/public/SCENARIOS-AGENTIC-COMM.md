---
status: stable
---

# Capability Scenarios: AgenticComm

*What happens when AI agents can communicate through structured, persistent, auditable channels?*

---

## 1. Concurrent Message Delivery to the Same Channel

Three agents send messages to the same group channel within the same millisecond. Each message must be assigned a unique, monotonically increasing ID. No message can be lost, duplicated, or delivered out of order within the same priority level.

### Setup

A group channel `concurrent-test` with four participants: `agent-a`, `agent-b`, `agent-c`, and `agent-reader`. All three senders submit messages simultaneously from separate threads.

### Expected Behavior

The message engine serializes message ID assignment using an atomic counter. Even if three messages arrive at the exact same wall-clock time, they receive IDs 1, 2, 3 in the order they acquire the counter. The `created_at` timestamps may be identical (same second), but the IDs impose a total order.

The reader queries the channel and receives all three messages in ID order. No message is missing. No message is duplicated.

### Edge Case Details

The critical invariant is `message.id` monotonicity. If the store is backed by a file with concurrent writers (multiple processes), the file lock ensures only one writer advances the counter at a time. Within a single process, the atomic `u64` counter handles thread safety.

Failure mode: if the counter wraps around `u64::MAX` (18.4 quintillion), the store rejects new messages. This is not a practical concern -- at 10 billion messages per second, wrap-around takes 58 years.

```rust
#[test]
fn test_concurrent_delivery() {
    let store = Arc::new(Mutex::new(CommStore::create_temp()?));
    let channel = store.lock().create_channel("concurrent", ChannelType::Group, "agent-a")?;
    store.lock().join_channel(channel.id, "agent-b")?;
    store.lock().join_channel(channel.id, "agent-c")?;

    let handles: Vec<_> = ["agent-a", "agent-b", "agent-c"]
        .iter()
        .map(|sender| {
            let store = Arc::clone(&store);
            let sender = sender.to_string();
            thread::spawn(move || {
                store.lock().send_message(channel.id, &sender, MessageType::Text, "concurrent msg")
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert!(results.iter().all(|r| r.is_ok()));

    let messages = store.lock().receive_messages(channel.id, "agent-a", None)?;
    assert_eq!(messages.messages.len(), 2); // agent-a doesn't receive own messages
    assert!(messages.messages[0].id < messages.messages[1].id);
}
```

---

## 2. Channel with Single Participant (Self-Messaging)

An agent creates a channel and is the only participant. It sends messages to itself. This pattern is used for task queues, note-taking, and scheduling reminders.

### Setup

Agent `agent-solo` creates a group channel and enables `echo: true` in the channel config so it receives its own messages.

### Expected Behavior

With `echo: true`, the agent receives its own messages. Without `echo` (the default), `receive_messages` returns an empty list because there are no other participants to deliver to. The messages are still persisted and queryable through `query_history`.

### Edge Case Details

Direct channels require exactly 2 participants, so self-messaging on a direct channel is rejected. Group channels allow it with `echo: true`. Broadcast channels with a single participant (the owner) deliver to zero recipients, which is valid -- the messages are persisted for history.

```rust
#[test]
fn test_self_messaging() {
    let mut store = CommStore::create_temp()?;
    let channel = store.create_channel("solo", ChannelType::Group, "agent-solo")?;
    store.set_channel_config(channel.id, ChannelConfig { echo: true, ..Default::default() })?;

    store.send_message(channel.id, "agent-solo", MessageType::Text, "Note to self")?;
    let received = store.receive_messages(channel.id, "agent-solo", None)?;
    assert_eq!(received.messages.len(), 1);
    assert_eq!(received.messages[0].content, "Note to self");
}
```

---

## 3. Broadcast to 10,000 Subscribers

A broadcast channel has 10,000 observer participants. The owner sends a single message. Fan-out must complete without timing out or exhausting memory.

### Setup

Create a broadcast channel. Join 10,000 participants as observers. Send a single broadcast message.

### Expected Behavior

Fan-out creates 10,000 delivery records. The message is stored once (not duplicated per recipient). Each recipient can independently query for their messages. Memory usage is proportional to the number of delivery records (approximately 100 bytes each = 1 MB total), not proportional to the message size times recipient count.

### Edge Case Details

The engine uses batch processing for channels with more than 1,000 recipients, processing 500 recipients at a time and releasing the store lock between batches. This prevents other operations from starving during large fan-outs.

If a recipient is removed from the channel between message send and message receive, the delivery record is marked as `failed` with reason `participant_removed`. This is not an error -- it is expected in dynamic systems.

```rust
#[test]
fn test_broadcast_10k() {
    let mut store = CommStore::create_temp()?;
    let channel = store.create_channel("mass-broadcast", ChannelType::Broadcast, "sender")?;

    for i in 0..10_000 {
        store.join_channel(channel.id, &format!("receiver-{:05}", i))?;
    }

    let msg = store.broadcast(channel.id, "sender", "System maintenance at 02:00 UTC")?;
    assert_eq!(msg.status, MessageStatus::Delivered);

    // Spot-check: random receiver gets the message
    let received = store.receive_messages(channel.id, "receiver-07777", None)?;
    assert_eq!(received.messages.len(), 1);
}
```

---

## 4. Message Size at 1 MB Boundary

A message with content exactly at the 1 MB limit (1,048,576 bytes). One byte more must be rejected.

### Setup

Create a channel with default configuration (max_message_size = 1,048,576 bytes).

### Expected Behavior

A message with 1,048,576 bytes of content is accepted. A message with 1,048,577 bytes is rejected with `CommError::ValidationFailed { field: "content", reason: "exceeds max_message_size" }`.

### Edge Case Details

The size check is on the byte length of the UTF-8 encoded content, not the character count. A message with 1,048,576 ASCII characters fits. A message with 524,288 two-byte UTF-8 characters (e.g., Latin Extended) also fits. A message with 349,526 three-byte UTF-8 characters fits. The limit is always on bytes.

```rust
#[test]
fn test_message_size_boundary() {
    let mut store = CommStore::create_temp()?;
    let channel = store.create_channel("size-test", ChannelType::Group, "agent-a")?;

    // Exactly at limit: succeeds
    let content = "x".repeat(1_048_576);
    assert!(store.send_message(channel.id, "agent-a", MessageType::Text, &content).is_ok());

    // One byte over: fails
    let content = "x".repeat(1_048_577);
    let err = store.send_message(channel.id, "agent-a", MessageType::Text, &content).unwrap_err();
    assert!(matches!(err, CommError::ValidationFailed { field, .. } if field == "content"));
}
```

---

## 5. Channel Name with Maximum Length (128 Characters)

A channel name at exactly 128 characters. One character more must be rejected.

### Setup

Create a channel with a 128-character name.

### Expected Behavior

128 characters: accepted. 129 characters: rejected with validation error.

### Edge Case Details

The length check is on bytes, not characters. A name consisting of 128 ASCII characters fits. A name consisting of 42 three-byte UTF-8 characters (126 bytes) fits. But a name with 43 three-byte characters (129 bytes) is rejected.

Channel names are used in file paths for some operations (export), so they must not contain path-separator characters other than the allowed forward slash. The name `a/b/c` is 5 characters and valid. The name `../../../etc/passwd` is rejected by the character pattern (`.` is not allowed in channel names).

```rust
#[test]
fn test_channel_name_boundary() {
    let mut store = CommStore::create_temp()?;

    let name = "a".repeat(128);
    assert!(store.create_channel(&name, ChannelType::Group, "agent-a").is_ok());

    let name = "a".repeat(129);
    let err = store.create_channel(&name, ChannelType::Group, "agent-a").unwrap_err();
    assert!(matches!(err, CommError::ValidationFailed { field, .. } if field == "name"));
}
```

---

## 6. Pub/Sub with Wildcard Topic Matching

A pub/sub channel with overlapping wildcard subscriptions. A single published message must be delivered to all matching subscribers, but not duplicated to a subscriber who matches on multiple patterns.

### Setup

Channel with three subscribers:
- `agent-build`: subscribed to `ci.build.*` and `ci.#`
- `agent-deploy`: subscribed to `ci.deploy.*`
- `agent-all`: subscribed to `#`

Publish a message to topic `ci.build.success`.

### Expected Behavior

- `agent-build` matches on both `ci.build.*` and `ci.#`. The message is delivered once (deduplication by subscriber).
- `agent-deploy` does not match (`ci.deploy.*` does not match `ci.build.success`).
- `agent-all` matches on `#` (matches everything). Delivered once.

### Edge Case Details

The deduplication is per-subscriber, not per-subscription. The engine collects all matching subscriptions, extracts the unique subscriber set, and delivers once per subscriber. Subscription match counts are tracked for monitoring but do not affect delivery count.

```rust
#[test]
fn test_wildcard_deduplication() {
    let mut store = CommStore::create_temp()?;
    let ch = store.create_channel("events", ChannelType::PubSub, "publisher")?;
    store.join_channel(ch.id, "agent-build")?;
    store.join_channel(ch.id, "agent-deploy")?;
    store.join_channel(ch.id, "agent-all")?;

    store.subscribe(ch.id, "agent-build", "ci.build.*")?;
    store.subscribe(ch.id, "agent-build", "ci.#")?;
    store.subscribe(ch.id, "agent-deploy", "ci.deploy.*")?;
    store.subscribe(ch.id, "agent-all", "#")?;

    store.publish(ch.id, "publisher", "ci.build.success", "Build passed")?;

    let build_msgs = store.receive_messages(ch.id, "agent-build", None)?;
    assert_eq!(build_msgs.messages.len(), 1); // NOT 2 despite matching two patterns

    let deploy_msgs = store.receive_messages(ch.id, "agent-deploy", None)?;
    assert_eq!(deploy_msgs.messages.len(), 0); // Does not match

    let all_msgs = store.receive_messages(ch.id, "agent-all", None)?;
    assert_eq!(all_msgs.messages.len(), 1);
}
```

---

## 7. Message Acknowledgment Timeout

A command message is sent with `ack_timeout = 5` seconds. The receiver does not acknowledge. The engine should retry and eventually dead-letter the message.

### Setup

Channel with `at_least_once` delivery, `ack_timeout = 5`, `max_retries = 2`, `retry_backoff_ms = 100`.

### Expected Behavior

1. Message sent at T=0.
2. No ack by T=5: retry 1 triggered.
3. No ack by T=5.2 (5 + 0.1 backoff + 5 timeout): retry 2 triggered.
4. No ack by T=10.4: message moved to dead letter queue.
5. `_system.message.deadletter` notification emitted.

### Edge Case Details

The retry timer is per-message, not per-channel. Multiple unacknowledged messages in the same channel maintain independent retry schedules. The engine checks for expired timers on each `tick()` call (called by the MCP event loop or explicitly in tests).

```rust
#[test]
fn test_ack_timeout() {
    let mut store = CommStore::create_temp()?;
    let ch = store.create_channel("ack-test", ChannelType::Direct, "sender")?;
    store.join_channel(ch.id, "receiver")?;
    store.set_channel_config(ch.id, ChannelConfig {
        delivery: DeliveryMode::AtLeastOnce,
        ack_timeout: Some(1), // 1 second for test speed
        max_retries: 2,
        retry_backoff_ms: 50,
        ..Default::default()
    })?;

    store.send_message(ch.id, "sender", MessageType::Command, "Do something")?;

    // Simulate time passing without acknowledgment
    std::thread::sleep(Duration::from_secs(4));
    store.tick()?; // Process expired timers

    let dead = store.dead_letters();
    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].retry_count, 2);
}
```

---

## 8. Dead Letter Queue Overflow

The dead letter queue accumulates 100,000 messages. The store must remain functional and the dead letter queue must be manageable.

### Setup

A channel with aggressive retry settings that consistently fails delivery. Messages accumulate in the dead letter queue.

### Expected Behavior

Dead letters are stored in the same `.acomm` file as regular messages, with status `DeadLetter`. They count toward the 10 million message limit. The store does not crash or degrade when the dead letter queue is large.

### Edge Case Details

Dead letters can be purged by age: `store.purge_dead_letters(Duration::from_secs(86400))` removes dead letters older than 24 hours. The `compact` command also removes old dead letters.

If dead letter accumulation threatens the 10M message limit, the store emits a warning notification. The application should monitor dead letter count and investigate the root cause (misconfigured channel, unavailable receiver).

```rust
#[test]
fn test_dead_letter_overflow() {
    let mut store = CommStore::create_temp()?;
    // Add 100 dead letters (would be 100K in production, but test uses smaller number)
    for i in 0..100 {
        store.add_dead_letter(Message {
            id: i,
            content: format!("Failed message {}", i),
            status: MessageStatus::DeadLetter,
            ..Default::default()
        })?;
    }

    assert_eq!(store.dead_letters().len(), 100);

    // Purge old ones
    let result = store.purge_dead_letters(Duration::from_secs(0))?;
    assert_eq!(result, 100);
    assert_eq!(store.dead_letters().len(), 0);
}
```

---

## 9. .acomm File Corruption Recovery

A `.acomm` file has a corrupted message section but valid header and checksum mismatch.

### Setup

Write a valid `.acomm` file, then flip random bytes in the message section.

### Expected Behavior

On open, the store computes the SHA-256 checksum of the file content (minus footer) and compares it with the stored checksum. Mismatch triggers `CommError::IntegrityError`. The store refuses to load corrupted data.

### Edge Case Details

Recovery options:
1. **Backup**: If a `.acomm.bak` file exists, the user can restore from it.
2. **Export**: If only the message section is corrupted, the channel and subscription sections may still be readable. A recovery tool can attempt to read sections individually.
3. **Atomic write protocol**: Because writes go through a temp file and atomic rename, corruption should only occur from disk hardware failure or external file modification.

```rust
#[test]
fn test_corruption_detection() {
    let mut store = CommStore::create_temp()?;
    store.send_message(/* ... */)?;
    store.flush()?;

    // Corrupt the file
    let path = store.path().to_owned();
    let mut bytes = std::fs::read(&path)?;
    bytes[200] ^= 0xFF; // Flip a byte in the message section
    std::fs::write(&path, &bytes)?;

    // Attempt to open
    let result = CommStore::open(&path);
    assert!(matches!(result, Err(CommError::IntegrityError { .. })));
}
```

---

## 10. Concurrent Channel Creation with Same Name

Two threads attempt to create a channel with the same name simultaneously. Only one should succeed.

### Setup

Two threads call `create_channel("same-name", ...)` at the same time on the same store.

### Expected Behavior

One thread succeeds and returns the new channel. The other thread receives `CommError::DuplicateChannel`. The channel exists exactly once in the store.

### Edge Case Details

Channel name uniqueness is enforced by the store's channel name index (a `HashMap<String, u64>`). The mutex protecting the store ensures only one thread can insert at a time. The second thread finds the name already in the index and returns the error.

```rust
#[test]
fn test_concurrent_channel_creation() {
    let store = Arc::new(Mutex::new(CommStore::create_temp()?));

    let handles: Vec<_> = (0..2).map(|_| {
        let store = Arc::clone(&store);
        thread::spawn(move || {
            store.lock().create_channel("same-name", ChannelType::Group, "agent-a")
        })
    }).collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let successes = results.iter().filter(|r| r.is_ok()).count();
    let duplicates = results.iter().filter(|r| matches!(r, Err(CommError::DuplicateChannel { .. }))).count();

    assert_eq!(successes, 1);
    assert_eq!(duplicates, 1);
}
```

---

## 11. Cross-Project Message Isolation

Two separate `.acomm` files for different projects must not share state. Messages in one store are invisible to the other.

### Setup

Open two stores: `project-a.acomm` and `project-b.acomm`. Create channels and send messages in each.

### Expected Behavior

Messages in `project-a.acomm` are not visible from `project-b.acomm`. Channel IDs may overlap (both start at 1), but they are in separate stores and separate files. No cross-contamination is possible.

### Edge Case Details

The MCP server deterministically selects the store path based on the project's canonical path (SHA-256 of the absolute path). Two different project directories always get different stores. Same directory always gets the same store. This prevents accidental cross-project communication.

```rust
#[test]
fn test_cross_project_isolation() {
    let mut store_a = CommStore::create_temp()?;
    let mut store_b = CommStore::create_temp()?;

    let ch_a = store_a.create_channel("test", ChannelType::Group, "agent-a")?;
    store_a.send_message(ch_a.id, "agent-a", MessageType::Text, "Project A message")?;

    let ch_b = store_b.create_channel("test", ChannelType::Group, "agent-b")?;
    let msgs = store_b.receive_messages(ch_b.id, "agent-b", None)?;
    assert_eq!(msgs.messages.len(), 0); // No messages from project A
}
```

---

## 12. Message with Empty Content

Sending a message with zero-length content must fail validation.

### Setup

Attempt to send a message with `content = ""`.

### Expected Behavior

The engine rejects the message with `CommError::ValidationFailed { field: "content", reason: "content must be non-empty" }`. The message is not persisted. The store state is unchanged.

### Edge Case Details

Whitespace-only content is allowed (it has non-zero byte length). A content consisting of a single space `" "` is valid. This is by design -- the engine validates structural constraints, not semantic quality. Whitespace content may be valid in some protocols (e.g., a heartbeat message with a space as a placeholder).

```rust
#[test]
fn test_empty_content() {
    let mut store = CommStore::create_temp()?;
    let ch = store.create_channel("test", ChannelType::Group, "agent-a")?;

    let err = store.send_message(ch.id, "agent-a", MessageType::Text, "").unwrap_err();
    assert!(matches!(err, CommError::ValidationFailed { field, .. } if field == "content"));

    // Whitespace is allowed
    assert!(store.send_message(ch.id, "agent-a", MessageType::Text, " ").is_ok());
}
```

---

## 13. Channel Deletion with Pending Messages

Deleting a channel that has unacknowledged messages (pending acknowledgments in at-least-once mode).

### Setup

Channel with `at_least_once` delivery. Send a command. Do not acknowledge. Attempt to delete the channel.

### Expected Behavior

`delete_channel` returns `CommError::ChannelNotEmpty { id, pending: 1 }`. The channel is not deleted. The caller must either acknowledge the pending messages, drain the channel first, or use force-delete.

### Edge Case Details

Force-delete (`delete_channel_force`) skips the pending message check and deletes the channel along with all its messages (including pending ones). This is a destructive operation and should be used with caution.

The drain-then-delete pattern: set the channel to `Draining` state (no new messages accepted), wait for pending messages to be acknowledged or time out (dead-lettered), then delete the empty channel.

```rust
#[test]
fn test_delete_with_pending() {
    let mut store = CommStore::create_temp()?;
    let ch = store.create_channel("pending", ChannelType::Direct, "sender")?;
    store.join_channel(ch.id, "receiver")?;
    store.set_channel_config(ch.id, ChannelConfig {
        delivery: DeliveryMode::AtLeastOnce,
        ack_timeout: Some(300),
        ..Default::default()
    })?;

    store.send_message(ch.id, "sender", MessageType::Command, "Do something")?;

    let err = store.delete_channel(ch.id).unwrap_err();
    assert!(matches!(err, CommError::ChannelNotEmpty { pending, .. } if pending == 1));
}
```

---

## 14. Subscription to Non-Existent Topic

Subscribing to a topic that no publisher has ever published to. The subscription must be created successfully and match future messages.

### Setup

Subscribe to `events.never.published` on a pub/sub channel. No messages with this topic exist.

### Expected Behavior

The subscription is created successfully. The subscriber has zero messages. When a message is later published to `events.never.published`, the subscriber receives it.

### Edge Case Details

Subscriptions are pattern-based declarations of interest, not references to existing data. A subscription to a non-existent topic is perfectly valid -- it is a statement that "when messages with this topic appear, deliver them to me." This is unlike database foreign keys where the referenced entity must exist.

```rust
#[test]
fn test_subscribe_nonexistent_topic() {
    let mut store = CommStore::create_temp()?;
    let ch = store.create_channel("events", ChannelType::PubSub, "pub")?;
    store.join_channel(ch.id, "sub")?;

    // Subscribe to a topic that has never been published
    let sub = store.subscribe(ch.id, "sub", "events.future.topic")?;
    assert!(sub.active);

    // No messages yet
    let msgs = store.receive_messages(ch.id, "sub", None)?;
    assert_eq!(msgs.messages.len(), 0);

    // Publish to the topic later
    store.publish(ch.id, "pub", "events.future.topic", "It happened!")?;

    let msgs = store.receive_messages(ch.id, "sub", None)?;
    assert_eq!(msgs.messages.len(), 1);
}
```

---

## 15. Message Search with Regex Patterns

Searching messages using regex patterns. The regex must be applied to message content with proper escaping and error handling for invalid patterns.

### Setup

A channel with 100 messages, some containing IP addresses, email addresses, and code snippets.

### Expected Behavior

A regex search for `\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b` matches messages containing IP addresses. A search for `[a-z]+@[a-z]+\.[a-z]+` matches messages containing email patterns. An invalid regex like `[unclosed` returns a validation error.

### Edge Case Details

Regex search is always a linear scan over the filtered result set. It cannot use indexes. For performance, callers should combine regex search with index-backed filters (channel, time range, sender) to narrow the scan space.

The regex engine uses the `regex` crate with a default size limit of 1 MB for the compiled pattern. Patterns exceeding this limit are rejected.

```rust
#[test]
fn test_regex_search() {
    let mut store = CommStore::create_temp()?;
    let ch = store.create_channel("logs", ChannelType::Group, "agent")?;

    store.send_message(ch.id, "agent", MessageType::Text, "Connected to 192.168.1.42")?;
    store.send_message(ch.id, "agent", MessageType::Text, "Email sent to admin@example.com")?;
    store.send_message(ch.id, "agent", MessageType::Text, "No special content here")?;

    // Search for IP addresses
    let results = store.search_messages(r"regex:\d+\.\d+\.\d+\.\d+", None)?;
    assert_eq!(results.messages.len(), 1);
    assert!(results.messages[0].content.contains("192.168.1.42"));

    // Invalid regex
    let err = store.search_messages(r"regex:[unclosed", None).unwrap_err();
    assert!(matches!(err, CommError::ValidationFailed { .. }));
}
```

---

## 16. Store Save During Active Message Delivery

A flush is triggered while the message engine is processing a fan-out delivery to 1,000 recipients. The save must not corrupt the store or lose in-flight messages.

### Setup

A broadcast channel with 1,000 recipients. A flush is triggered from another thread while fan-out is in progress.

### Expected Behavior

The flush acquires the store mutex. If fan-out holds the mutex, the flush waits until fan-out releases it. If flush holds the mutex, fan-out waits. The store is always in a consistent state: either the message is fully delivered (all recipients) and persisted, or it is not yet started.

### Edge Case Details

The batch fan-out (500 recipients at a time, lock released between batches) creates a window where a flush can occur mid-fan-out. In this case:

1. The flush persists the store with the first 500 delivery records written and the remaining 500 not yet created.
2. The fan-out resumes and creates the remaining delivery records.
3. On the next flush, the complete set of delivery records is persisted.
4. If the process crashes between the first and second flush, the store on disk has partial delivery. On reload, the engine detects the incomplete fan-out (message marked as `Sent` but not all expected delivery records exist) and resumes fan-out for the remaining recipients.

```rust
#[test]
fn test_save_during_delivery() {
    let store = Arc::new(Mutex::new(CommStore::create_temp()?));
    let ch = {
        let mut s = store.lock();
        let ch = s.create_channel("delivery-test", ChannelType::Broadcast, "sender")?;
        for i in 0..100 {
            s.join_channel(ch.id, &format!("receiver-{:03}", i))?;
        }
        ch
    };

    // Spawn broadcast in background
    let store_clone = Arc::clone(&store);
    let broadcast_handle = thread::spawn(move || {
        store_clone.lock().broadcast(ch.id, "sender", "Test message")
    });

    // Concurrent flush
    thread::sleep(Duration::from_millis(1));
    store.lock().flush()?;

    broadcast_handle.join().unwrap()?;
    store.lock().flush()?;

    // Verify all recipients got the message
    for i in 0..100 {
        let msgs = store.lock().receive_messages(ch.id, &format!("receiver-{:03}", i), None)?;
        assert_eq!(msgs.messages.len(), 1);
    }
}
```
