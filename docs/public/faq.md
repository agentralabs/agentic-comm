---
status: draft
---

# Security Hardening and FAQ

This document covers AgenticComm's security hardening measures and answers frequently asked questions.

## Security Hardening

### Strict MCP Input Validation

Every MCP tool validates all inputs before processing. There are no silent fallbacks, no default substitutions for invalid data, and no lenient parsing modes.

**Principle:** If an input is invalid, return an error immediately. Never guess what the user meant.

| Validation | Rule | Error on Violation |
|-----------|------|-------------------|
| Channel name | Non-empty, max 128 bytes, pattern `[a-zA-Z0-9_-]+(/[a-zA-Z0-9_-]+)*` | `isError: true` with field and reason |
| Participant ID | Non-empty, max 128 bytes, pattern `[a-zA-Z0-9_-]+` | `isError: true` with field and reason |
| Message content | Non-empty, max 1 MB, valid UTF-8 | `isError: true` with size details |
| Topic string | Non-empty, max 256 bytes, valid dot-separated hierarchy | `isError: true` with pattern explanation |
| Correlation ID | Valid UUID v4 format | `isError: true` with format requirement |
| Priority | Integer 0-4 | `isError: true` with valid range |
| Channel type | One of: direct, group, broadcast, pubsub | `isError: true` with valid values |
| Message type | One of: text, command, query, response, broadcast, notification, acknowledgment, error | `isError: true` with valid values |

MCP tool execution errors return `isError: true` in the tool result content. Protocol-level errors (unknown tool, malformed request) use JSON-RPC error codes:

| Error | Code | Description |
|-------|------|-------------|
| Unknown tool | -32803 | `TOOL_NOT_FOUND`: The requested tool does not exist |
| Invalid params | -32602 | `INVALID_PARAMS`: Parameters do not match the tool schema |
| Internal error | -32603 | `INTERNAL_ERROR`: Unexpected server-side failure |

The `-32803` code for unknown tools follows the MCP Quality Standard. AgenticComm never uses `-32601` (method not found) or `-32602` (invalid params) for tool-not-found errors.

### Deterministic Per-Project Identity

The MCP server computes a deterministic store path based on the project directory's canonical path. This ensures:

1. The same project always uses the same `.acomm` file.
2. Different projects never share a `.acomm` file.
3. Symlinks, relative paths, and redundant slashes are resolved before hashing.

```
store_path = ~/.local/share/agentic-comm/<sha256(canonical_path)>/comm.acomm
```

The SHA-256 hash of the absolute path produces a 64-character hex string used as the directory name. This prevents path injection (directory names are always hex strings) and ensures uniqueness.

### Zero Cross-Project Contamination

Cross-project contamination is prevented at three levels:

1. **File isolation**: Each project gets its own `.acomm` file in a hashed directory.
2. **Channel namespace**: Channel names are local to the store. Channel "main" in project A is completely independent from channel "main" in project B.
3. **Participant isolation**: Participant IDs are local to the store. "agent-planner" in project A is a different participant from "agent-planner" in project B.

There is no cross-store query mechanism. To compare communication across projects, the user must explicitly export from both stores and analyze the exports externally.

### Concurrent Startup Locking

When the MCP server starts, it acquires an advisory lock on the store directory:

1. Create `<store_dir>/acomm.startup.lock` if it does not exist.
2. Attempt to acquire an exclusive file lock (flock).
3. If the lock is held by another process:
   a. Read the lock file to check the holding PID.
   b. If the PID is not running (stale lock): wait 5 seconds, then break the lock.
   c. If the PID is running: wait up to 30 seconds for the lock to be released.
   d. If still held after 30 seconds: fail with a clear error message.
4. Write the current PID, timestamp, and hostname to the lock file.
5. Proceed with initialization.
6. Release the lock after initialization completes.

The lock file format is human-readable:

```
PID: 42351
STARTED: 1740667800
HOSTNAME: dev-machine.local
```

### Stale Lock Recovery

A lock is considered stale when:
- The PID recorded in the lock file is not running (checked via `kill(pid, 0)` on Unix, `OpenProcess` on Windows).
- The lock file was last modified more than 60 seconds ago AND the PID is not running.

Stale locks are removed automatically. A warning is logged with the stale lock details.

### Merge-Only MCP Config Updates

When the MCP server modifies the Claude Code MCP configuration (`mcp.json`), it uses merge semantics:

1. Read the existing configuration.
2. Add or update the `agentic-comm` entry.
3. Preserve all other entries (other MCP servers).
4. Write the merged configuration.

The server never overwrites the entire config file. This prevents accidental removal of other MCP server configurations. If the config file does not exist, it is created with only the `agentic-comm` entry.

### Auth Gating via AGENTIC_TOKEN

When the `AGENTIC_TOKEN` environment variable is set, all REST bridge and webhook endpoints require authentication:

- Requests must include `Authorization: Bearer <token>` header.
- Requests without the header receive HTTP 401 Unauthorized.
- Requests with an incorrect token receive HTTP 403 Forbidden.
- Token comparison uses constant-time comparison to prevent timing attacks.

MCP transport (stdio) does not require token authentication because it operates through a trusted local pipe, not a network socket.

### Message Encryption at Rest

When the `ENCRYPTED` flag is set in the file header, message content is encrypted using AES-256-GCM before compression:

1. A key is derived from the `AGENTIC_TOKEN` using HKDF-SHA256 with a salt stored in the file header.
2. Each message content is encrypted with a unique nonce (12 bytes, monotonically increasing).
3. The ciphertext and authentication tag are stored in place of the plaintext content.
4. Message metadata (sender, channel, timestamp) is NOT encrypted -- only the content.

Encryption is opt-in. Unencrypted stores are the default. To enable encryption:

```bash
acomm init --encrypted project.acomm
# Or set in config:
# [security]
# encrypt_at_rest = true
```

### Channel Access Control

Channels enforce access control through participant membership:

- Only participants can send or receive messages on a channel.
- The `Observer` role restricts sending (can only read).
- The `Owner` role is required to modify channel configuration.
- Participant IDs can be linked to AgenticIdentity IDs for cryptographic authentication.

When AgenticIdentity integration is enabled, participant actions are verified against the identity's trust grants. A participant without the appropriate trust grant cannot join a channel, even if invited.

### Input Sanitization

All string inputs are sanitized:

1. **No null bytes**: Strings containing `\0` are rejected.
2. **No control characters**: ASCII characters 0x00-0x1F (except `\n`, `\r`, `\t`) are rejected in channel names and participant IDs. Message content allows all valid UTF-8.
3. **No path traversal**: Channel names and participant IDs containing `..` are rejected.
4. **UTF-8 validation**: All strings must be valid UTF-8. Invalid byte sequences are rejected.

---

## Frequently Asked Questions

### General

**What is AgenticComm?**

AgenticComm is a communication infrastructure library for AI agent systems. It provides structured messaging (typed messages), organized communication (channels), event-driven routing (pub/sub), and persistent history (`.acomm` files) for multi-agent coordination.

**How is it different from a message queue like RabbitMQ or Kafka?**

Message queues are infrastructure servers that require deployment, configuration, and maintenance. AgenticComm is a library -- you add it to your project and it works. The `.acomm` file is the entire communication substrate. No broker, no server, no operational overhead. Additionally, AgenticComm is agent-aware: it integrates with AgenticIdentity for authenticated messaging, AgenticContract for SLA enforcement, and AgenticMemory for cognitive graph integration.

**Do I need a running server?**

No. AgenticComm operates entirely in-process using the `.acomm` file as the persistence layer. The MCP server (`agentic-comm-mcp`) is a lightweight process that wraps the library for use with Claude Code and other MCP clients -- it is not a message broker.

**Can multiple agents in different processes communicate?**

Yes, through the shared `.acomm` file. Each process opens the file, writes messages (with file locking for safety), flushes, and other processes reload to pick up new messages. For higher-frequency multi-process communication, run the MCP server as a mediator.

**What happens if my agent crashes mid-conversation?**

Messages that were flushed to disk before the crash are preserved. Messages that were only in memory (not yet flushed) are lost. Use immediate flush mode or reduce the batch flush interval for critical channels.

### Channels and Messages

**How many channels can I create?**

Up to 100,000 channels per `.acomm` file.

**How many messages can a channel hold?**

The per-file limit is 10 million messages across all channels. Individual channels have no independent limit. If you need more than 10 million messages, use retention policies to archive old messages or start a new `.acomm` file.

**What is the maximum message size?**

1 MB (1,048,576 bytes) by default. This can be reduced (but not increased) per-channel through `ChannelConfig.max_message_size`. The 1 MB limit is a hard ceiling.

**Can I send binary data in messages?**

Message content is UTF-8 text. To send binary data, encode it as base64 and store the encoding in the message content. For large binary payloads, store the data externally and send a reference (file path, URL) in the message.

**What happens to messages in a deleted channel?**

When a channel is deleted (force-delete), all messages belonging to that channel are removed from the store. Messages that were exported or indexed to AgenticMemory before deletion are preserved in those external systems.

**Can I rename a channel?**

Not directly. Channel names are part of the uniqueness constraint. To rename, create a new channel with the desired name, move messages (export/import), and delete the old channel.

### Pub/Sub

**Can a subscriber receive messages from before they subscribed?**

By default, no. Subscriptions only match messages published after the subscription is created. To receive historical messages, enable `sticky_messages: true` on the channel config. Alternatively, query the channel history directly.

**What happens if no subscribers match a published message?**

The message is persisted in the store but delivered to zero recipients. This is not an error. The message is queryable through `query_history` and will be delivered to future subscribers if `sticky_messages` is enabled.

**Can I subscribe to all topics?**

Yes, use the `#` pattern, which matches zero or more segments. `subscribe(channel, subscriber, "#")` receives every message published on the channel regardless of topic.

**Is topic matching case-sensitive?**

Yes. Topic `Build.Frontend.Complete` does not match pattern `build.frontend.complete`. Use consistent casing conventions in your topic hierarchy.

### Performance

**How fast is message delivery?**

In-process: 2-5 microseconds per message for direct channels. Multi-process (via flush/reload): 10-50 milliseconds. See [Benchmarks](benchmarks.md) for detailed measurements.

**How large can a .acomm file get?**

Maximum 2 GB. At typical compression ratios (3:1), this holds approximately 500 MB of uncompressed message data, which is enough for 5-10 million average-sized messages.

**Does search performance degrade with more messages?**

Content search (string matching, regex) is linear in the number of messages. Index-backed queries (by channel, sender, timestamp, correlation ID) are O(1) or O(log N). Combine index filters with content search for best performance.

### Integration

**Does AgenticComm work without the other Agentra sisters?**

Yes. AgenticComm is fully self-contained. Sister integrations add capabilities (signed messages, SLA enforcement, memory linking) but are not required. The core library has no dependencies on other sisters.

**Can I use AgenticComm with LangChain, CrewAI, or AutoGen?**

Yes. AgenticComm is a Rust library with Python and Node.js bindings. You can use it from any Python-based agent framework. Create channels for your agents, send messages through the library API, and query history as needed.

**Is there a cloud-hosted version?**

No. AgenticComm is a local-first library. Your communication data stays on your machine in `.acomm` files. There is no cloud service, no telemetry, no data collection.

### Troubleshooting

**"IntegrityError: checksum mismatch"**

The `.acomm` file failed checksum verification. Possible causes:
1. The file was modified by an external process (text editor, hex editor).
2. Disk corruption.
3. Incomplete write (process crashed during flush without atomic rename).

Recovery: Check for a `.acomm.bak` backup file. If available, rename it to replace the corrupted file.

**"StoreCapacity: file size would exceed 2 GB limit"**

The store is full. Options:
1. Run `acomm compact --older-than 30d` to remove old archived messages and dead letters.
2. Export important channels and start a new store.
3. Adjust retention policies to automatically archive old messages.

**"ChannelNotEmpty: channel has N pending messages"**

You tried to delete a channel that has unacknowledged messages. Options:
1. Acknowledge the pending messages.
2. Drain the channel: `acomm channel drain --name <name>`
3. Force-delete: `acomm channel delete --name <name> --force`

**MCP server does not start**

1. Check that the binary is in your PATH: `which agentic-comm-mcp`
2. Check the MCP config: `cat ~/.claude/mcp.json` and verify the `agentic-comm` entry.
3. Check for stale locks: `ls ~/.local/share/agentic-comm/*/acomm.startup.lock`
4. Check logs: set `ACOMM_LOG_LEVEL=debug` and restart.

**Messages not appearing for a participant**

1. Verify the participant is in the channel: `acomm channel info --name <channel>`
2. Check the participant's role (Observers cannot send, but can receive).
3. For pub/sub, verify the subscription pattern matches the published topic.
4. Check message TTL -- expired messages are not delivered.
5. For multi-process scenarios, ensure the reading process called `reload()`.

**High memory usage**

The in-memory store holds all messages. For large stores:
1. Use retention policies to limit active message count.
2. Use `compact` to remove old data.
3. Consider splitting into multiple stores (one per project or time period).

### Compatibility

**What Rust version is required?**

Rust 1.75 or later (for stable async traits and other features used by the MCP server).

**What operating systems are supported?**

macOS (ARM64, x86-64), Linux (x86-64, ARM64), and Windows (x86-64). The file format is platform-independent (little-endian, standard sizes). A `.acomm` file created on macOS can be read on Linux and vice versa.

**Is the .acomm format stable?**

The format is versioned (currently version 1). Breaking changes will increment the version number. Older readers can open newer files if they support the version, and unknown sections are safely skipped. We commit to maintaining backward compatibility for at least 3 major versions.

**Can I use AgenticComm from C or other languages?**

Yes, through the FFI layer (`agentic-comm-ffi`). The FFI exposes a C-compatible API that can be called from any language with C FFI support (Python via ctypes, Ruby via FFI gem, Go via cgo, etc.).
