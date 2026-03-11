---
status: stable
---

# AgenticComm Overview

AgenticComm provides structured communication infrastructure for AI agent systems. It replaces ad-hoc messaging (raw HTTP calls, unstructured logs, piped stdout) with purpose-built channels, typed messages, pub/sub routing, and a persistent communication history stored in portable `.acomm` files.

## The Problem

Agent communication today is broken in five fundamental ways.

**1. No structure.** When Agent A needs to talk to Agent B, it sends a blob of text over whatever transport is available -- an HTTP POST body, a file on disk, a line written to stdout. There is no schema, no message type, no acknowledgment protocol. The receiving agent must guess the intent of every incoming payload. Was it a command? A response? An error? A broadcast? The sender has no way to know if the message was received, understood, or acted upon.

**2. No history.** Agent conversations vanish after they happen. There is no persistent record of what was communicated, when, by whom, or in what order. When a multi-agent system produces an unexpected outcome, debugging requires reconstructing the communication flow from scattered application logs, often across multiple processes, machines, and time zones. The conversation itself -- the actual sequence of messages that led to the decision -- is gone.

**3. No routing.** Every agent-to-agent connection is point-to-point. There are no channels, no topics, no broadcast mechanisms. If a coordination agent needs to notify five worker agents about a plan change, it must maintain five separate connections and send five separate messages. If a new agent joins the system, every existing agent that needs to communicate with it must be individually reconfigured.

**4. No contracts.** There is no way to express communication expectations. "This channel guarantees delivery within 5 seconds." "Messages on this topic must be acknowledged." "Responses to queries must arrive within 30 seconds or trigger escalation." Without enforceable communication contracts, multi-agent systems fail silently and unpredictably.

**5. No portability.** Communication patterns are locked into specific runtimes. A conversation that happened in one agent framework cannot be replayed, analyzed, or continued in another. Communication history is either ephemeral or stored in framework-specific formats that resist inspection.

AgenticComm solves all five problems with a single coherent system.

## Architecture

AgenticComm is built as a Rust core library with MCP (Model Context Protocol) and CLI interfaces. The architecture has four layers:

```
+-----------------------------------------------------------+
|                     MCP Interface                         |
|  send_message  receive_messages  create_channel  publish  |
|  subscribe     broadcast         query_history   ...      |
+-----------------------------------------------------------+
|                     Message Engine                        |
|  Routing  |  Delivery  |  Acknowledgment  |  Dead Letter  |
+-----------------------------------------------------------+
|                     Channel Manager                       |
|  Direct  |  Group  |  Broadcast  |  PubSub  |  Config     |
+-----------------------------------------------------------+
|                     CommStore (.acomm)                     |
|  Binary format  |  Indexes  |  Compression  |  Integrity  |
+-----------------------------------------------------------+
```

**CommStore** is the persistence layer. It manages `.acomm` files -- binary containers that hold channels, messages, subscriptions, and indexes. The format supports up to 10 million messages per file, uses bincode serialization with flate2 compression, and includes SHA-256 integrity checksums.

**Channel Manager** handles the lifecycle of communication channels. Four channel types serve different patterns: Direct (1:1), Group (N:N), Broadcast (1:N, sender-only writes), and PubSub (topic-based routing with wildcard matching). Channels are created, configured, joined, left, and archived through a unified interface.

**Message Engine** is the core processing pipeline. It validates, routes, persists, and delivers messages. It handles fan-out for broadcast channels, topic matching for pub/sub, acknowledgment tracking, retry logic, and dead letter processing for undeliverable messages. The engine operates entirely in-process (no network server required) while supporting future networked operation.

**MCP Interface** exposes all capabilities as MCP tools. AI coding assistants like Claude Code interact with AgenticComm entirely through MCP tool calls -- sending messages, creating channels, subscribing to topics, and querying history without writing any code.

## Key Features

### Typed Messages

Every message has an explicit type: Text, Command, Query, Response, Broadcast, Notification, Acknowledgment, or Error. This eliminates the "what does this message mean?" problem. A Query message expects a Response. A Command message expects an Acknowledgment. A Broadcast is fire-and-forget. The type is part of the message schema, not something the receiver infers.

### Channel Architecture

Channels organize communication into named, configurable contexts. A Direct channel connects two participants. A Group channel supports multiple participants with configurable read/write permissions. A Broadcast channel allows one sender and many receivers. A PubSub channel routes messages by topic hierarchy with wildcard matching (`agent.*.status`, `build.frontend.#`).

### Persistent History

Every message is persisted to the `.acomm` file with full metadata: sender, recipient(s), timestamp, message type, channel, topic, correlation ID, and delivery status. History is queryable by any combination of these fields. The complete communication record of a multi-agent session survives process restarts, context window resets, and agent replacements.

### Delivery Guarantees

Channels are configured with delivery semantics: at-most-once (fire-and-forget), at-least-once (with acknowledgment and retry), or exactly-once (with deduplication). Unacknowledged messages are retried with exponential backoff. Messages that exhaust retries are moved to a dead letter queue for inspection.

### Sister Integration

AgenticComm integrates deeply with the other Agentra sisters:

- **AgenticIdentity**: Messages are signed with the sender's cryptographic identity. Recipients verify signatures to ensure authenticity. Trust levels influence routing decisions.
- **AgenticContract**: Channels can be backed by enforceable contracts. SLA violations (missed delivery deadlines, unacknowledged messages) are detected and reported.
- **AgenticMemory**: Communication history is linked to the agent's cognitive graph. Messages that led to decisions are traversable. Conversation context persists across sessions.
- **AgenticVision**: Visual content (screenshots, diagrams) can be attached to messages and shared through channels.
- **AgenticTime**: Messages can be scheduled for future delivery. Time-windowed channels activate and deactivate on schedule. Deadline-based routing escalates messages that approach SLA boundaries.
- **AgenticCodebase**: Code snippets, diffs, and PR notifications flow through dedicated channels with syntax-aware formatting.

## Comparison with Existing Approaches

| Approach | Structured Types | Persistent History | Topic Routing | Agent-Aware | Portable |
|----------|------------------|--------------------|---------------|-------------|----------|
| Raw HTTP/REST | No | No (logs only) | No | No | No |
| gRPC | Yes (protobuf) | No | No | No | Partial |
| Message Queues (RabbitMQ, Kafka) | Partial | Yes | Yes | No | No |
| Redis Pub/Sub | No | No | Yes | No | No |
| Agent Frameworks (LangChain, CrewAI) | Partial | Partial | No | Partial | No |
| **AgenticComm** | **Yes** | **Yes (.acomm)** | **Yes** | **Yes** | **Yes** |

Message queues are the closest existing solution, but they are infrastructure-heavy (require running servers), not agent-aware (no concept of identity, trust, or cognitive events), and not portable (data lives in the broker, not in a file you can move). AgenticComm is a library, not a server. The `.acomm` file is the communication substrate -- you can copy it, version-control it, analyze it offline, or share it between agent systems.

## Artifacts

- Primary artifact: `.acomm`
- Cross-sister workflows can pair `.acomm` with `.amem` (memory), `.aid` (identity), and `.acon` (contracts)

## Start Here

- [Installation](installation.md)
- [Quickstart](quickstart.md)
- [Core Concepts](concepts.md)
- [Command Surface](command-surface.md)
- [Integration Guide](integration-guide.md)
- [Notifier-First Runtime Roadmap](notifier-runtime-roadmap.md)
- [Experience With vs Without](experience-with-vs-without.md)

## Works With

- **AgenticIdentity** -- sign and verify messages with cryptographic agent identities for authenticated communication.
- **AgenticContract** -- enforce SLAs on channels with delivery guarantees, response time limits, and violation reporting.
- **AgenticMemory** -- link communication history to cognitive graphs for traceable reasoning across conversations.
- **AgenticVision** -- attach and share visual captures (screenshots, diagrams) through communication channels.
- **AgenticTime** -- schedule messages for future delivery and manage time-windowed channel activation.
- **AgenticCodebase** -- route code-related notifications (PR reviews, build results) through dedicated channels.
