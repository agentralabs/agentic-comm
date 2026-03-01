---
status: stable
---

# System Architecture

AgenticComm is a four-crate Rust workspace that provides structured communication infrastructure for AI agent systems. This document describes the internal architecture: crate layout, data flow, storage format, session management, and per-project isolation.

## Workspace Layout

The workspace contains four crates, each with a single responsibility:

```
agentic-comm/
+-- Cargo.toml                  (workspace root)
+-- crates/
    +-- agentic-comm/           (core library)
    |   +-- src/lib.rs
    +-- agentic-comm-cli/       (CLI binary: acomm)
    |   +-- src/main.rs
    +-- agentic-comm-mcp/       (MCP server binary)
    |   +-- src/
    |       +-- main.rs
    |       +-- tools/
    |       |   +-- registry.rs
    |       |   +-- communication_log.rs
    |       +-- session/
    |       |   +-- manager.rs
    |       +-- config/
    |           +-- loader.rs
    +-- agentic-comm-ffi/       (C FFI shared library)
        +-- src/lib.rs
```

### Crate Dependency Graph

```
+-------------------+
|  agentic-comm-mcp |--+
+-------------------+  |
                       |    +----------------+
+-------------------+  +--->|  agentic-comm  |
|  agentic-comm-cli |------>|  (core library)|
+-------------------+  +--->|                |
                       |    +----------------+
+-------------------+  |
|  agentic-comm-ffi |--+
+-------------------+
```

All three consumer crates (CLI, MCP, FFI) depend on the core `agentic-comm` crate. They never depend on each other. This strict dependency tree ensures that the core library is the single source of truth for all communication logic, and any of the three interfaces can be used independently.

## Crate Responsibilities

### agentic-comm (core library)

The core crate owns all communication logic and data structures. It has zero knowledge of transport protocols, CLI parsing, or MCP framing. Its public API is a set of Rust structs and methods that any consumer can call.

**Owns:**
- `CommStore` -- the main in-memory store holding channels, messages, and subscriptions
- `Channel`, `ChannelType`, `ChannelConfig` -- channel model and configuration
- `Message`, `MessageType`, `MessageFilter` -- message model, types, and query filters
- `Subscription` -- pub/sub subscription model
- `AcommHeader` -- binary file format header
- Validation logic (channel names, message content, sender identity)
- SHA-256 content signatures
- Persistence (save/load `.acomm` files via bincode + flate2)
- Query engine (history, full-text search)
- Pub/sub routing (topic matching, fan-out delivery)
- Broadcast delivery (per-participant message cloning)

**Does not own:**
- Session management (MCP crate)
- CLI argument parsing (CLI crate)
- MCP protocol framing (MCP crate)
- C ABI surface (FFI crate)

### agentic-comm-cli (CLI binary)

The CLI crate provides the `acomm` binary for terminal-based interaction. It parses command-line arguments with `clap`, resolves the store path, loads or creates a `CommStore`, calls core library methods, prints JSON output, and saves the store back to disk.

**Owns:**
- Argument parsing and validation
- Store path resolution (CLI flag, `ACOMM_STORE` env var, `.acomm/store.acomm`, `~/.store.acomm`)
- Human-readable output formatting (JSON pretty-print)
- Exit code management (0 = success, 1 = error)
- The `add` subcommand for hook-compatible message insertion

**Key design decisions:**
- Every mutating command loads the store, performs the operation, saves, and exits. There is no long-running process.
- Output is always valid JSON, even for errors (written to stderr).
- The `add` subcommand auto-creates channels by name, enabling hook integration without pre-setup.

### agentic-comm-mcp (MCP server)

The MCP crate implements the Model Context Protocol server. It runs as a long-lived process communicating over stdio (JSON-RPC), maintains a session with operation tracking, and dispatches tool calls to core library methods.

**Owns:**
- MCP protocol handling (initialize, tools/list, tools/call, shutdown)
- `SessionManager` -- wraps `CommStore` with session state, operation logging, and temporal chaining
- `ToolRegistry` -- maps tool names to handler functions (17 tools)
- `CommunicationLogEntry` -- the 20-Year Clock context capture tool
- `OperationRecord` -- per-tool-call audit trail
- Config resolution for MCP context

**Key design decisions:**
- The session manager persists the store to disk after every mutating tool call, preventing data loss on unexpected termination.
- Operation records capture every tool invocation with timestamp and related entity ID, enabling session replay and debugging.
- The `communication_log` tool captures intent and observation metadata that links communication actions to their reasoning context (the 20-Year Clock pattern shared across all Agentra sisters).

### agentic-comm-ffi (C FFI library)

The FFI crate exposes a minimal C-compatible API for embedding AgenticComm in non-Rust runtimes. It compiles to a shared library (`libagentic_comm_ffi.so` / `.dylib` / `.dll`) that can be loaded by Python, Node.js, Swift, or any language with C FFI support.

**Owns:**
- C ABI function signatures (`extern "C"`)
- Pointer-based memory management (create/free pairs)
- Null-terminated string conversions (`CStr` / `CString`)
- JSON serialization for complex return values
- Error signaling via null pointers and zero return values

**Key design decisions:**
- Every function is `unsafe` at the Rust level but safe at the C level when used correctly. Safety invariants are documented on each function.
- Complex return values (lists of channels, message arrays) are returned as heap-allocated JSON strings that the caller must free with `acomm_string_free`.
- The API uses opaque pointers (`*mut CommStore`) rather than handles, keeping the FFI layer as thin as possible.

## Data Flow Architecture

### Message Pipeline

When a message is sent (via any interface), it flows through this pipeline:

```
+--------+     +----------+     +--------+     +---------+     +-------+
| Sender |---->| Validate |---->| Route  |---->| Persist |---->| Index |
+--------+     +----------+     +--------+     +---------+     +-------+
                   |                |
                   v                v
              Reject with      Fan-out for
              CommError        broadcast/pubsub
```

**1. Validate.** The sender string, content bytes, and message type are validated against the rules:
  - Sender must be non-empty.
  - Content must be 1 byte to 1 MB (`MAX_CONTENT_SIZE`).
  - The target channel must exist in the store.

**2. Route.** Based on channel type:
  - **Direct / Group**: A single message is created, addressed to the channel. All participants can read it.
  - **Broadcast**: One message per recipient is created (excluding the sender). Each copy has the recipient set explicitly.
  - **PubSub**: The topic is matched against all active subscriptions. One message per matching subscriber is created.

**3. Persist.** Each message is assigned a monotonically increasing ID, timestamped with UTC, signed with SHA-256, and inserted into the `messages` HashMap.

**4. Index.** The channel-to-message mapping is updated. For pub/sub messages, the topic index is updated. The sender index is updated.

### Query Pipeline

Queries flow through a filter chain:

```
+----------+     +----------+     +------+     +--------+
| All msgs |---->| Filter   |---->| Sort |---->| Limit  |
| in store |     | by field |     | by   |     | and    |
+----------+     +----------+     | time |     | return |
                                  +------+     +--------+
```

The `query_history` method applies filters (channel, sender, message type, time range) as a single pass over the message HashMap, then sorts by timestamp, then truncates to the requested limit. The `search_messages` method performs case-insensitive substring matching on message content.

## .acomm Binary Format

The `.acomm` file is the persistence format for all communication data. It uses bincode serialization with flate2 (gzip) compression.

### On-Disk Structure

```
+------------------------------------------+
|  ACOMM001 (8 bytes magic)                |
|  Header fields (version, counts)         |
+------------------------------------------+
|  Serialized CommStore (bincode + gzip)   |
|  - channels HashMap<u64, Channel>        |
|  - messages HashMap<u64, Message>        |
|  - subscriptions HashMap<u64, Sub>       |
|  - next_channel_id, next_message_id,     |
|    next_subscription_id                  |
+------------------------------------------+
```

The file is written atomically: the entire store is serialized, compressed, and written in a single operation. On load, the magic bytes are verified (`ACOMM001`), the version is checked, and the store is deserialized from the remaining bytes.

### Format Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `ACOMM_MAGIC` | `b"ACOMM001"` | 8-byte file identifier |
| `ACOMM_VERSION` | `1` | Current format version |
| `MAX_CONTENT_SIZE` | `1,048,576` | 1 MB maximum message content |

### Compression

The bincode-serialized store is compressed with flate2's default compression level. Typical compression ratios:

| Message count | Uncompressed | Compressed | Ratio |
|---------------|-------------|------------|-------|
| 100 | ~48 KB | ~18 KB | 2.7:1 |
| 5,000 | ~2.1 MB | ~680 KB | 3.1:1 |
| 50,000 | ~19 MB | ~5.8 MB | 3.3:1 |
| 500,000 | ~180 MB | ~52 MB | 3.5:1 |

### Integrity Verification

Every message carries an optional SHA-256 signature computed from its content bytes. On load, the `AcommHeader` magic bytes and version are verified before deserialization proceeds. Files with invalid magic bytes or unsupported versions are rejected with a descriptive `CommError::InvalidFile` error.

## Session Manager Architecture

The MCP server wraps the raw `CommStore` in a `SessionManager` that adds session-aware behavior:

```
+------------------------------------------------------------+
|                     SessionManager                          |
|                                                             |
|  +-------------+  +----------------+  +------------------+ |
|  |  CommStore   |  | OperationLog   |  | CommunicationLog | |
|  |  (channels,  |  | (tool calls    |  | (intent +        | |
|  |   messages,  |  |  with times    |  |  observations,   | |
|  |   subs)      |  |  and entity    |  |  20-Year Clock)  | |
|  |              |  |  IDs)          |  |                  | |
|  +-------------+  +----------------+  +------------------+ |
|                                                             |
|  store_path: PathBuf                                        |
|  session_start_time: Instant                                |
|  last_message_id: Option<u64>     (temporal chaining)       |
|  session_active: bool                                       |
+------------------------------------------------------------+
```

### Session Lifecycle

1. **Creation**: The `SessionManager::new()` constructor resolves the store path, loads the existing `.acomm` file (or creates an empty store), and initializes the session state.

2. **Activation**: When the MCP client sends the `initialized` notification, the session is marked active via `mark_session_started()`. The session start time is recorded.

3. **Operation**: Each tool call goes through `ToolRegistry::dispatch()`, which calls the appropriate handler. The handler calls core library methods on `session.store`, then calls `session.record_operation()` to log the tool invocation.

4. **Context Capture**: The `communication_log` tool allows agents to record the intent behind their communication actions. Each entry is timestamped and stored in the `context_log` vector.

5. **Temporal Chaining**: The `last_message_id` field tracks the most recently created or referenced message, enabling temporal linkage between operations.

6. **Shutdown**: On MCP shutdown or EOF, the store is saved to disk. The operation log and context log are available for post-session analysis.

### Operation Record Structure

Each operation record captures:

```rust
pub struct OperationRecord {
    pub tool_name: String,      // e.g., "send_message"
    pub timestamp: String,      // ISO 8601 RFC 3339
    pub related_id: Option<u64>, // message_id, channel_id, etc.
}
```

This enables reconstruction of the complete tool invocation sequence for debugging, auditing, and Hydra orchestration.

## Tool Registry Architecture

The `ToolRegistry` is a static dispatch table mapping tool names to handler functions. It provides two methods:

```
ToolRegistry::list_tools()  -> Vec<ToolDefinition>   (for tools/list)
ToolRegistry::dispatch()    -> Result<ToolCallResult> (for tools/call)
```

### Dispatch Flow

```
MCP Request (tools/call)
    |
    v
ToolRegistry::dispatch(tool_name, params, session)
    |
    +-- "send_message"       -> handle_send_message()
    +-- "receive_messages"   -> handle_receive_messages()
    +-- "create_channel"     -> handle_create_channel()
    +-- "list_channels"      -> handle_list_channels()
    +-- "join_channel"       -> handle_join_channel()
    +-- "leave_channel"      -> handle_leave_channel()
    +-- "get_channel_info"   -> handle_get_channel_info()
    +-- "subscribe"          -> handle_subscribe()
    +-- "unsubscribe"        -> handle_unsubscribe()
    +-- "publish"            -> handle_publish()
    +-- "broadcast"          -> handle_broadcast()
    +-- "query_history"      -> handle_query_history()
    +-- "search_messages"    -> handle_search_messages()
    +-- "get_message"        -> handle_get_message()
    +-- "acknowledge_message"-> handle_acknowledge_message()
    +-- "set_channel_config" -> handle_set_channel_config()
    +-- "communication_log"  -> handle_communication_log()
    +-- unknown              -> McpError::ToolNotFound
```

Each handler extracts parameters from the JSON `Value`, calls core library methods, records the operation, and returns a `ToolCallResult` (success with JSON content, or error with description).

### Error Handling

The MCP quality standard requires two error categories:

1. **Tool execution errors** (e.g., channel not found, invalid content): Return `isError: true` in the `ToolCallResult`. The JSON-RPC response is still a success -- the error is in the tool result.

2. **Protocol errors** (e.g., unknown tool name, malformed params): Return a JSON-RPC error with the appropriate error code. Unknown tools use error code `-32803` (`TOOL_NOT_FOUND`).

## Per-Project Isolation

AgenticComm supports per-project store isolation through path resolution:

```
Priority 1: --file CLI flag / explicit path in MCP config
Priority 2: ACOMM_STORE environment variable
Priority 3: .acomm/store.acomm in current working directory
Priority 4: ~/.store.acomm (global fallback)
```

### Per-Project Setup

When a `.acomm/` directory exists in the project root, AgenticComm uses the local store automatically. This enables project-specific communication histories that are independent of other projects.

```
my-project/
+-- .acomm/
|   +-- store.acomm       <-- project-local store
+-- src/
+-- Cargo.toml
```

### MCP Server Configuration

When configured in Claude Code's `mcp.json`, the MCP server can be pointed at a specific store:

```json
{
  "mcpServers": {
    "agentic-comm": {
      "command": "agentic-comm-mcp",
      "args": ["--store", "/path/to/project/.acomm/store.acomm"]
    }
  }
}
```

Without explicit configuration, the server uses the standard path resolution chain.

## Communication Log (20-Year Clock)

The `communication_log` tool implements the 20-Year Clock pattern shared across all Agentra sisters. The principle is: "Never capture WHAT without capturing WHY."

Every sister has a context-capture tool:
- AgenticMemory: `conversation_log`
- AgenticVision: `observation_log`
- AgenticCodebase: `analysis_log`
- AgenticIdentity: `action_context`
- AgenticComm: `communication_log`

### CommunicationLogEntry Structure

```rust
pub struct CommunicationLogEntry {
    pub intent: String,                  // WHY this communication is happening
    pub observation: Option<String>,     // WHAT was noticed or concluded
    pub related_message_id: Option<u64>, // Links to a specific message
    pub topic: Option<String>,           // Category (e.g., "agent-coordination")
    pub timestamp: String,               // ISO 8601 timestamp
}
```

These entries accumulate in the session's `context_log` and enable post-session analysis of communication intent -- why agents communicated, not just what they said.

## Cross-Crate Type Sharing

The core `agentic-comm` crate defines all shared types. Consumer crates import them directly:

```rust
// In CLI crate
use agentic_comm::{ChannelType, CommStore, MessageFilter, MessageType};

// In MCP crate
use agentic_comm::{ChannelConfig, ChannelType, MessageFilter, MessageType};

// In FFI crate
use agentic_comm::{ChannelType, CommStore, MessageType};
```

This ensures type consistency across all interfaces. A `ChannelType::PubSub` created via CLI is identical to one created via MCP or FFI.

## Dependencies

The workspace depends on the following external crates:

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.0 | Serialization/deserialization |
| `serde_json` | 1.0 | JSON handling |
| `tokio` | 1.35 | Async runtime (MCP server) |
| `chrono` | 0.4 | Timestamps with UTC timezone |
| `uuid` | 1 | UUID v4 generation |
| `thiserror` | 1 | Error type derivation |
| `sha2` | 0.10 | SHA-256 content signatures |
| `bincode` | 1 | Binary serialization for .acomm files |
| `flate2` | 1 | Gzip compression |
| `clap` | 4 | CLI argument parsing |

## Design Principles

1. **Core library is transport-agnostic.** The `CommStore` knows nothing about MCP, CLI flags, or C ABI. It operates on Rust types and returns Rust types.

2. **All mutation goes through CommStore methods.** Neither the MCP handlers nor the CLI subcommands manipulate internal data structures directly. They call `store.send_message()`, `store.create_channel()`, etc.

3. **Every tool call is logged.** The MCP session manager records every tool invocation, enabling session replay, debugging, and audit trails.

4. **The .acomm file is self-contained.** A single `.acomm` file contains all channels, messages, subscriptions, and indexes. It can be copied, version-controlled, or shared without external dependencies.

5. **Errors are explicit and typed.** The `CommError` enum covers every failure mode with a human-readable message. There are no panics in normal operation.

6. **FFI uses the thinnest possible layer.** The FFI crate translates between C types and Rust types, calling core library methods directly. It adds no business logic.
