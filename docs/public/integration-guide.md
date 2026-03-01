---
status: stable
---

# Integration Guide

This guide covers how AgenticComm integrates with the other Agentra sisters, how to embed it in applications, and how to bridge communication to external systems.

## Sister Integration

AgenticComm is designed to work alongside the other six Agentra sisters. Each integration adds a distinct capability to the communication layer.

### AgenticIdentity Integration

AgenticIdentity provides cryptographic agent identities. When integrated with AgenticComm, every message can be signed by the sender and verified by the receiver.

#### Message Signing

When a sender has an AgenticIdentity, messages are automatically signed with the sender's Ed25519 private key. The signature covers the message content, sender ID, timestamp, and channel ID -- preventing tampering and replay attacks.

```rust
use agentic_comm::CommStore;
use agentic_identity::Identity;

let identity = Identity::load("default")?;
let mut store = CommStore::open("project.acomm")?;

// Configure the store to use this identity for signing
store.set_identity(identity)?;

// Messages are now automatically signed
let msg = store.send_message(channel_id, "agent-planner", MessageType::Command, "Deploy to staging")?;
assert!(msg.signature.is_some());
```

#### Sender Verification

When receiving a message, the recipient can verify the sender's identity:

```rust
let msg = store.get_message(message_id)?;
let verified = store.verify_sender(&msg)?;
match verified {
    SenderVerification::Verified { identity_id } => {
        println!("Message verified from identity {}", identity_id);
    },
    SenderVerification::Unverified => {
        println!("Message has no signature or signature is invalid");
    },
    SenderVerification::IdentityNotFound => {
        println!("Sender identity not in local trust store");
    },
}
```

#### Trust-Based Routing

Channels can require a minimum trust level for participation. When a message arrives from a sender whose trust level is below the channel's threshold, the message is held for review rather than delivered.

```rust
let config = ChannelConfig {
    min_trust_level: Some(0.7),  // Require 70% trust score
    ..Default::default()
};
store.set_channel_config(channel_id, config)?;
```

Trust levels are computed by AgenticIdentity's dynamic trust system. They decay over time without reinforcement and increase with successful interactions.

### AgenticContract Integration

AgenticContract provides enforceable agreements between agents. When integrated with AgenticComm, channels can be backed by contracts that define communication SLAs.

#### Contract-Backed Channels

A contract-backed channel has enforceable delivery guarantees:

```rust
use agentic_comm::CommStore;
use agentic_contract::Contract;

let mut store = CommStore::open("project.acomm")?;

// Create a channel with a contract
let channel = store.create_channel_with_contract(
    "critical-ops",
    ChannelType::Direct,
    "agent-controller",
    ContractTerms {
        max_delivery_latency_ms: 5000,     // Messages must be delivered within 5s
        ack_required: true,                 // All commands must be acknowledged
        ack_timeout_ms: 30000,             // Acknowledgment within 30s
        max_message_size: 524288,           // 512 KB max
        uptime_guarantee: 0.999,            // 99.9% availability
    },
)?;
```

#### SLA Violation Detection

When a contract-backed channel violates its SLA, AgenticComm generates a violation event that can be consumed by AgenticContract for enforcement:

```rust
// Query contract violations for a channel
let violations = store.get_contract_violations(channel_id)?;
for v in &violations {
    println!(
        "Violation: {} at {} (message {})",
        v.violation_type, v.timestamp, v.message_id
    );
}
```

Violation types:
- `DeliveryLatencyExceeded` -- Message took longer than the contracted max delivery time.
- `AcknowledgmentTimeout` -- A required acknowledgment was not received in time.
- `MessageSizeExceeded` -- A message exceeded the contracted max size.
- `ChannelUnavailable` -- The channel was unavailable when a message was sent.

### AgenticMemory Integration

AgenticMemory stores an agent's cognitive graph. When integrated with AgenticComm, communication history is linked to the agent's reasoning trail.

#### Memory-Backed Message Search

Messages can be indexed into AgenticMemory's vector store for semantic search:

```rust
use agentic_comm::CommStore;
use agentic_memory::Brain;

let store = CommStore::open("project.acomm")?;
let brain = Brain::open("agent.amem")?;

// Index channel messages into memory
store.index_to_memory(&brain, channel_id)?;

// Now semantic search finds communication context
let results = brain.search("database connection pool", top_k: 10)?;
// Results include both memory nodes and indexed messages
```

#### Auto-Capture Integration

When AgenticMemory's auto-capture is active, significant communication events are automatically recorded as cognitive events:

- Commands received become `Decision` nodes (the agent decided to act on the command).
- Query responses become `Fact` nodes (the agent received information).
- Error messages become `Correction` nodes (something went wrong).
- Broadcast announcements become `Fact` nodes (the agent was informed).

The `communication_log` MCP tool parallels AgenticMemory's `conversation_log` tool, creating a temporal chain of communication events linked to the memory graph.

#### Conversation Persistence

When a channel's communication history is linked to memory, the conversation survives context window resets:

```rust
// Session 1: Agent has a conversation
store.send_message(channel_id, "agent-a", MessageType::Text,
    "We decided to use PostgreSQL for the auth service")?;

// Session 2: Agent resumes from memory
let brain = Brain::open("agent.amem")?;
let context = brain.session_resume()?;
// Context includes the communication history from session 1
```

### AgenticVision Integration

AgenticVision captures and manages visual content. When integrated with AgenticComm, visual artifacts can be shared through channels.

#### Screenshot Sharing

Agents can share screenshots and visual captures through communication channels:

```rust
use agentic_comm::CommStore;
use agentic_vision::VisionStore;

let mut store = CommStore::open("project.acomm")?;
let vision = VisionStore::open("captures.avis")?;

// Capture a screenshot and share it
let capture = vision.capture_screenshot()?;
store.send_message_with_options(
    channel_id,
    "agent-monitor",
    MessageType::Notification,
    "Dashboard showing CPU spike on node-3",
    SendOptions {
        metadata: Some(MessageMetadata::from([
            ("vision_capture_id", MetadataValue::Integer(capture.id as i64)),
            ("capture_type", MetadataValue::String("screenshot".into())),
        ])),
        ..Default::default()
    },
)?;
```

#### Visual Diff Channels

A dedicated channel pattern for visual regression testing:

```rust
// Create a channel for visual diffs
let channel = store.create_channel("visual-diffs", ChannelType::PubSub, "agent-qa")?;

// Subscribe the design review agent
store.subscribe(channel.id, "agent-design-review", "diff.*.regression")?;

// QA agent publishes visual regressions
store.publish(
    channel.id,
    "agent-qa",
    "diff.homepage.regression",
    "Visual regression detected on homepage: header alignment shifted 3px",
)?;
```

### AgenticTime Integration

AgenticTime manages temporal operations. When integrated with AgenticComm, messages can be scheduled and channels can operate on time windows.

#### Scheduled Messages

Messages can be scheduled for future delivery:

```rust
use agentic_comm::CommStore;
use agentic_time::TimeManager;

let mut store = CommStore::open("project.acomm")?;
let time = TimeManager::new()?;

// Schedule a reminder for tomorrow at 9 AM
store.schedule_message(
    channel_id,
    "agent-scheduler",
    MessageType::Notification,
    "Daily standup reminder: share progress updates",
    time.parse("2026-03-01T09:00:00Z")?,
)?;
```

#### Time-Windowed Channels

Channels can be configured to activate and deactivate on a schedule:

```rust
let config = ChannelConfig {
    active_window: Some(TimeWindow {
        start: time.parse("09:00")?,
        end: time.parse("17:00")?,
        timezone: "America/New_York".into(),
        weekdays_only: true,
    }),
    ..Default::default()
};
store.set_channel_config(channel_id, config)?;
```

Messages sent outside the active window are queued and delivered when the window opens.

#### Deadline-Based Routing

Messages approaching their TTL deadline can be automatically escalated:

```rust
let config = ChannelConfig {
    escalation_threshold: Some(0.8), // Escalate when 80% of TTL has elapsed
    escalation_channel: Some(escalation_channel_id),
    ..Default::default()
};
```

When a message's remaining TTL drops below the threshold, a copy is forwarded to the escalation channel with a `Notification` type and `Critical` priority.

### AgenticCodebase Integration

AgenticCodebase analyzes code structure. When integrated with AgenticComm, code-related events flow through dedicated channels.

#### PR Notification Channels

```rust
// Create a pub/sub channel for code events
let channel = store.create_channel("code-events", ChannelType::PubSub, "agent-ci")?;

// Subscribe to PR events
store.subscribe(channel.id, "agent-reviewer", "pr.*.opened")?;
store.subscribe(channel.id, "agent-reviewer", "pr.*.updated")?;
store.subscribe(channel.id, "agent-deploy", "pr.*.merged")?;

// CI agent publishes PR events
store.publish(
    channel.id,
    "agent-ci",
    "pr.auth-service.opened",
    "PR #142: Add rate limiting to auth endpoints\nFiles changed: 5\nAdditions: 230\nDeletions: 12",
)?;
```

#### Code Snippet Sharing

Messages can carry code snippets with language metadata for syntax-aware display:

```rust
store.send_message_with_options(
    channel_id,
    "agent-reviewer",
    MessageType::Text,
    "Found potential SQL injection in auth/login.rs line 47:\n```rust\nlet query = format!(\"SELECT * FROM users WHERE email = '{}'\", email);\n```",
    SendOptions {
        metadata: Some(MessageMetadata::from([
            ("code_language", MetadataValue::String("rust".into())),
            ("file_path", MetadataValue::String("auth/login.rs".into())),
            ("line_number", MetadataValue::Integer(47)),
        ])),
        ..Default::default()
    },
)?;
```

## Embedding in Applications

### Rust Application

Add `agentic-comm` to your `Cargo.toml`:

```toml
[dependencies]
agentic-comm = "0.1"
```

```rust
use agentic_comm::{CommStore, ChannelType, MessageType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = CommStore::open("app.acomm")?;

    // Create channels for your agent system
    let control = store.create_channel("control", ChannelType::Broadcast, "orchestrator")?;
    let feedback = store.create_channel("feedback", ChannelType::Group, "orchestrator")?;

    // Add worker agents
    for i in 0..5 {
        let name = format!("worker-{}", i);
        store.join_channel(control.id, &name)?;
        store.join_channel(feedback.id, &name)?;
    }

    // Broadcast a task
    store.broadcast(control.id, "orchestrator", "Process batch 2026-0227")?;

    // Workers report back through the feedback channel
    store.send_message(feedback.id, "worker-0", MessageType::Text, "Batch processed: 1000 records")?;

    store.flush()?;
    Ok(())
}
```

### Python Application

```python
from agentic_comm import CommStore, ChannelType, MessageType

store = CommStore.open("app.acomm")

# Create a direct channel
channel = store.create_channel("planner-executor", ChannelType.DIRECT, "planner")
store.join_channel(channel.id, "executor")

# Send and receive
store.send_message(channel.id, "planner", MessageType.COMMAND, "Analyze dataset")
messages = store.receive_messages(channel.id, "executor")
print(f"Received: {messages[0].content}")

store.flush()
```

### Node.js Application

```javascript
const { CommStore, ChannelType, MessageType } = require('@agenticamem/comm');

const store = CommStore.open('app.acomm');

const channel = store.createChannel('events', ChannelType.PubSub, 'publisher');
store.subscribe(channel.id, 'consumer', 'order.*');

store.publish(channel.id, 'publisher', 'order.created', JSON.stringify({
  orderId: '12345',
  total: 99.99,
}));

const messages = store.receiveMessages(channel.id, 'consumer');
console.log(`Received ${messages.length} messages`);

store.flush();
```

## Webhook Integration

AgenticComm can bridge messages to and from HTTP webhooks.

### Outbound Webhooks

Configure a channel to forward messages to an HTTP endpoint:

```rust
let config = ChannelConfig {
    webhook: Some(WebhookConfig {
        url: "https://api.example.com/agent-events".into(),
        method: "POST".into(),
        headers: vec![("Authorization".into(), "Bearer token123".into())],
        batch_size: 10,      // Send up to 10 messages per request
        batch_delay_ms: 1000, // Wait up to 1s to accumulate a batch
    }),
    ..Default::default()
};
store.set_channel_config(channel_id, config)?;
```

The webhook payload is a JSON array of message objects.

### Inbound Webhooks

The MCP server can receive messages from HTTP webhooks when running in server mode:

```bash
agentic-comm-mcp serve --webhook-port 8080
```

POST to `/webhook/<channel_name>`:
```json
{
  "sender": "external-service",
  "type": "notification",
  "content": "Deployment complete",
  "topic": "deploy.production.success"
}
```

## REST Bridge

For systems that cannot use MCP, AgenticComm provides a REST API bridge:

```bash
agentic-comm-mcp serve --rest-port 9090
```

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/channels` | List all channels |
| POST | `/channels` | Create a channel |
| GET | `/channels/:id` | Get channel info |
| POST | `/channels/:id/messages` | Send a message |
| GET | `/channels/:id/messages` | Receive messages (with query params for filtering) |
| POST | `/channels/:id/subscribe` | Subscribe to a topic |
| GET | `/history` | Query communication history |

### Authentication

The REST bridge requires the `AGENTIC_TOKEN` environment variable to be set. All requests must include the `Authorization: Bearer <token>` header.

## Multi-Process Communication

When multiple agent processes need to communicate through the same `.acomm` file:

1. Each process opens the store with `CommStore::open()`.
2. Write operations acquire the file lock automatically.
3. After writing, the process calls `flush()` to persist changes.
4. Other processes call `reload()` to pick up changes from disk.

```rust
// Process A: write
let mut store_a = CommStore::open("shared.acomm")?;
store_a.send_message(channel_id, "agent-a", MessageType::Text, "Hello from A")?;
store_a.flush()?;

// Process B: read
let mut store_b = CommStore::open("shared.acomm")?;
store_b.reload()?;  // Pick up Process A's changes
let messages = store_b.receive_messages(channel_id, "agent-b", None)?;
```

For high-frequency multi-process communication, consider using the MCP server as a mediator. Multiple MCP clients can connect to the same server, which serializes access to the store.
