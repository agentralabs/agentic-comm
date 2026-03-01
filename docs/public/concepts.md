---
status: stable
---

# Core Concepts

AgenticComm models agent communication as **typed messages** flowing through **configured channels** between **identified participants**, with full lifecycle tracking and persistent history. This document explains the foundational concepts.

## Why Structured Communication?

Most multi-agent systems communicate through unstructured text passed between functions. Agent A returns a string, Agent B receives it, and the interpretation is implicit. There is no distinction between a command and a notification. There is no record of what was said, when, or by whom. There is no mechanism for acknowledgment, retry, or escalation.

Structured communication solves this by making every aspect of the interaction explicit: who sent it, who should receive it, what type of message it is, what channel it flows through, what delivery guarantees apply, and what the expected response pattern is. This is not overhead -- it is the minimum infrastructure required for reliable multi-agent coordination.

## Message Types

Every message has one of eight types. The type determines the expected behavior of sender and receiver.

### Text

A general-purpose message carrying information. Text messages have no expected response pattern -- they are informational. Use text messages for status updates, log entries, commentary, and unstructured communication.

```
Type: Text
Content: "Build completed successfully. 847 tests passed, 0 failed."
```

### Command

An instruction for the receiver to perform an action. Command messages expect an Acknowledgment in return. The sender can specify a timeout after which unacknowledged commands are retried or escalated.

```
Type: Command
Content: "Deploy service-payment to staging environment"
Expects: Acknowledgment
Timeout: 30s
```

### Query

A question that expects a Response message in return. Queries carry a correlation ID that links the response back to the original question. Unanswered queries within the timeout period trigger escalation.

```
Type: Query
Content: "What is the current CPU utilization of the production cluster?"
Expects: Response
Correlation: "q-2026-0227-001"
Timeout: 10s
```

### Response

An answer to a Query. Response messages carry the correlation ID of the query they answer. A response without a matching query correlation is treated as an orphan and logged as a warning.

```
Type: Response
Content: "CPU utilization is 67% across 8 nodes, peak 82% on node-3"
Correlation: "q-2026-0227-001"
```

### Broadcast

A one-to-many announcement. Broadcast messages are delivered to all participants in the channel (or all subscribers to the topic in pub/sub). No acknowledgment is expected. Broadcasts are used for announcements, configuration changes, and system-wide notifications.

```
Type: Broadcast
Content: "Maintenance window scheduled: 2026-03-01 02:00-04:00 UTC"
```

### Notification

A lightweight alert that something has happened. Notifications are similar to broadcasts but carry a severity level (info, warning, error, critical) and an optional action URL. They are designed for monitoring and alerting workflows.

```
Type: Notification
Content: "Memory usage exceeded 90% threshold on agent-analyzer"
Severity: warning
```

### Acknowledgment

Confirms receipt or completion of a Command. Acknowledgments carry the correlation ID of the command they acknowledge and a status (received, in_progress, completed, failed).

```
Type: Acknowledgment
Content: "Deployment initiated, ETA 45 seconds"
Correlation: "cmd-deploy-001"
Status: in_progress
```

### Error

Reports a failure condition. Error messages carry an error code, a human-readable description, and optionally a machine-readable detail payload. They can be sent in response to any message type.

```
Type: Error
Content: "Deployment failed: insufficient permissions for staging environment"
Code: "DEPLOY_PERMISSION_DENIED"
Correlation: "cmd-deploy-001"
```

## Channels

Channels are named, configured containers for communication. Every message flows through exactly one channel. Channels define who can participate, what delivery semantics apply, and how messages are routed.

### Direct Channel

A private channel between exactly two participants. Messages sent on a direct channel are visible only to the two participants. Direct channels are created implicitly when one agent sends a message to another, or explicitly by either participant.

```
Channel: direct/agent-planner/agent-executor
Type: Direct
Participants: [agent-planner, agent-executor]
```

**Use cases:** One-on-one coordination, task delegation, private feedback.

### Group Channel

A channel with multiple participants. All participants can send and receive messages. Group channels must be created explicitly and participants join by invitation or request. The channel creator is the initial administrator.

```
Channel: group/backend-team
Type: Group
Participants: [agent-api, agent-database, agent-cache, agent-auth]
Permissions: all members can read and write
```

**Use cases:** Team coordination, multi-agent planning, collaborative problem-solving.

### Broadcast Channel

A one-to-many channel with a single sender (the owner) and multiple receivers. Only the owner can send messages. Receivers can join and leave but cannot send. Broadcast channels are used for announcements and one-directional information flow.

```
Channel: broadcast/system-alerts
Type: Broadcast
Owner: agent-monitor
Receivers: [agent-api, agent-database, agent-cache, agent-auth, agent-planner]
```

**Use cases:** System announcements, configuration pushes, monitoring alerts.

### PubSub Channel

A topic-based channel where messages are routed to subscribers based on topic matching. Publishers send messages with a topic string. Subscribers express interest in topics using patterns (exact match, wildcard, or prefix). A single pub/sub channel can carry many topics simultaneously.

```
Channel: pubsub/events
Type: PubSub
Topics: build.*, deploy.*, test.*
Subscribers:
  agent-ci: [build.*, test.*]
  agent-deploy: [deploy.*]
  agent-monitor: [*.failure, *.error]
```

**Use cases:** Event-driven architectures, loosely coupled agent coordination, monitoring.

## Topic Hierarchies

Topics in pub/sub channels follow a dot-separated hierarchy. Three matching modes are supported:

### Exact Match

The topic pattern must exactly match the message topic.

```
Pattern: build.frontend.complete
Matches: build.frontend.complete
Does not match: build.backend.complete, build.frontend.start
```

### Wildcard Match (*)

A single `*` matches exactly one segment in the hierarchy.

```
Pattern: build.*.complete
Matches: build.frontend.complete, build.backend.complete
Does not match: build.frontend.test.complete (too many segments)
```

### Multi-level Wildcard (#)

A `#` at the end of a pattern matches zero or more remaining segments.

```
Pattern: build.frontend.#
Matches: build.frontend, build.frontend.complete, build.frontend.test.unit
Does not match: build.backend.complete
```

### Reserved Topics

Topics starting with `_system.` are reserved for internal use. The following system topics are predefined:

| Topic | Description |
|-------|-------------|
| `_system.channel.created` | Emitted when a new channel is created |
| `_system.channel.deleted` | Emitted when a channel is deleted |
| `_system.participant.joined` | Emitted when a participant joins a channel |
| `_system.participant.left` | Emitted when a participant leaves a channel |
| `_system.message.deadletter` | Emitted when a message enters the dead letter queue |

## Participants and Identity

Every message has a sender. Senders are identified by a participant ID -- a string that uniquely identifies the agent within the communication system. When AgenticIdentity is available, participant IDs are derived from cryptographic identities and messages are signed for authenticity verification.

### Participant ID Format

Participant IDs follow the format: `agent-<name>` or any alphanumeric string with hyphens and underscores. Maximum length is 128 characters.

```
Valid: agent-planner, build-worker-03, human-operator, claude-code-session-1
Invalid: (empty), agent planner (spaces), ../escape (path traversal)
```

### Participant Roles

Within a channel, participants have one of three roles:

| Role | Can Send | Can Receive | Can Configure |
|------|----------|-------------|---------------|
| Owner | Yes | Yes | Yes |
| Member | Yes | Yes | No |
| Observer | No | Yes | No |

The channel creator is the initial owner. Ownership can be transferred. Broadcast channels restrict sending to the owner only.

## Message Lifecycle

Every message passes through a defined lifecycle with five states:

```
Created  -->  Sent  -->  Delivered  -->  Acknowledged  -->  Archived
                |                          |
                v                          v
            Failed (retry)           Not Acknowledged
                |                          |
                v                          v
           Dead Letter               Escalated
```

### Created

The message has been constructed with all required fields (type, content, sender, channel) but has not entered the routing pipeline. Validation has not yet occurred.

### Sent

The message has passed validation, been assigned a unique ID and timestamp, and entered the routing pipeline. The message is now persisted in the CommStore.

### Delivered

The message has been routed to its intended recipient(s). For direct and group channels, this means the message is available for the recipient to read. For pub/sub, this means the message has been matched against subscriptions and placed in subscriber queues.

### Acknowledged

The recipient has explicitly acknowledged the message (for message types that require acknowledgment). The acknowledgment is recorded with a timestamp and status.

### Archived

The message has exceeded its retention period and has been moved from the active message set to the archive section. Archived messages are still queryable but may be compressed more aggressively.

### Failed States

- **Failed**: The message could not be delivered (invalid channel, unknown recipient). It is retried according to the channel's retry policy.
- **Dead Letter**: The message has exhausted all retry attempts and been moved to the dead letter queue for manual inspection.
- **Escalated**: An acknowledgment was expected but not received within the timeout period. An escalation notification is generated.

## Channel Lifecycle

Channels have four states:

| State | Description |
|-------|-------------|
| Active | Channel is open for communication. Messages can be sent and received. |
| Paused | Channel exists but message delivery is suspended. Messages are queued. |
| Draining | Channel is closing. No new messages accepted. Existing messages are delivered. |
| Closed | Channel is closed. No messages can be sent or received. History is retained. |

## Communication Protocols

AgenticComm supports three communication patterns built on top of the message type system:

### Request-Response

A Query message is sent and a corresponding Response is expected. The correlation ID links the response to the query. A timeout defines how long the sender waits.

```
Sender: send Query(content, timeout=10s) -> correlation_id
Receiver: receive Query -> process -> send Response(result, correlation_id)
Sender: receive Response(correlation_id) -> matched, done
```

If the timeout expires without a response, the sender receives a timeout error and can choose to retry, escalate, or abandon.

### Fire-and-Forget

A message (Text, Broadcast, Notification) is sent without expecting a response. The sender is notified of delivery (the message reached the channel) but not of processing.

```
Sender: send Broadcast(content) -> delivery_confirmed
Receivers: receive Broadcast -> process independently
```

### Command-Acknowledge

A Command is sent and an Acknowledgment is expected. This is similar to request-response but the acknowledgment can be partial (received, in_progress, completed).

```
Sender: send Command(instruction, timeout=30s) -> correlation_id
Receiver: receive Command -> send Ack(correlation_id, status=received)
Receiver: process -> send Ack(correlation_id, status=completed)
Sender: receive Ack(completed) -> done
```

## Priority and Precedence

Messages carry a priority level that influences delivery order within a channel:

| Priority | Value | Description |
|----------|-------|-------------|
| Critical | 0 | Delivered immediately, preempts all other messages |
| High | 1 | Delivered before normal priority messages |
| Normal | 2 | Default priority, FIFO within the priority level |
| Low | 3 | Delivered after all higher-priority messages |
| Background | 4 | Delivered only when no other messages are pending |

Within the same priority level, messages are delivered in FIFO (first-in, first-out) order. Cross-priority ordering is strict: all priority-0 messages are delivered before any priority-1 message, and so on.

## Message Routing Rules

The routing engine determines where a message goes based on the channel type and message metadata:

1. **Direct channels**: Route to the other participant.
2. **Group channels**: Route to all participants except the sender.
3. **Broadcast channels**: Route to all receivers (sender is the owner).
4. **PubSub channels**: Match the message topic against all active subscriptions. Route to each matching subscriber.

### Routing Modifiers

- **Exclude sender**: By default, the sender does not receive their own message. This can be overridden with `echo: true`.
- **Sticky routing**: In group channels, messages can be marked as `sticky`, meaning they are delivered to any new participants who join the channel after the message was sent.
- **Expiry routing**: Messages with a `ttl` (time-to-live) are not delivered after the TTL expires, even if the recipient connects later.

## Correlation and Threading

Messages can be grouped into threads using correlation IDs. A thread starts with a Query or Command and includes all subsequent Responses, Acknowledgments, and follow-up messages that share the same correlation ID. Threads enable:

- Viewing a complete conversation about a single topic
- Tracking the lifecycle of a command from issuance to completion
- Measuring response times from query to response

Correlation IDs are UUIDs generated by the sender. The receiver copies the correlation ID into reply messages.

## Retention and Archival

Channels are configured with a retention policy that determines how long messages are kept in the active store:

| Policy | Behavior |
|--------|----------|
| `forever` | Messages are never archived (default) |
| `duration(N)` | Messages older than N seconds are archived |
| `count(N)` | Only the most recent N messages are kept active |
| `size(N)` | Active message storage is limited to N bytes |

Archived messages are compressed and moved to the archive section of the `.acomm` file. They remain queryable through the `query_history` tool with the `include_archived: true` flag.
