---
status: stable
---

# FFI Reference

The `agentic-comm-ffi` crate exposes a C-compatible API for embedding AgenticComm in non-Rust runtimes. It compiles to a shared library (`libagentic_comm_ffi.so` on Linux, `libagentic_comm_ffi.dylib` on macOS, `agentic_comm_ffi.dll` on Windows) that can be loaded by any language with C FFI support.

## Building the Shared Library

```bash
cd agentic-comm
cargo build --release -p agentic-comm-ffi

# Output location:
# target/release/libagentic_comm_ffi.so      (Linux)
# target/release/libagentic_comm_ffi.dylib    (macOS)
# target/release/agentic_comm_ffi.dll         (Windows)
```

For a C-compatible header, generate one with `cbindgen` or use the signatures documented below.

## Memory Ownership Model

The FFI layer follows a strict ownership convention:

1. **Create/free pairs.** Every function that allocates memory has a corresponding free function. The caller is responsible for calling the free function when the resource is no longer needed.

2. **Opaque pointers.** The `CommStore` is exposed as an opaque `*mut CommStore` pointer. The caller must not dereference, cast, or inspect this pointer -- it must only be passed to other `acomm_*` functions.

3. **String ownership.** Functions that return `*mut c_char` return heap-allocated strings owned by the caller. The caller must free them with `acomm_string_free()`. Functions that accept `*const c_char` borrow the string -- the caller retains ownership and must keep the string alive for the duration of the call.

4. **Null safety.** All functions check for null pointers and return a safe error value (0, null, or false) without crashing. Passing null is always defined behavior.

## Exported Functions

### acomm_version

Return the library version as a null-terminated C string.

```c
const char* acomm_version(void);
```

**Returns:** A pointer to a static string containing the version (e.g., `"0.1.0"`). This pointer is valid for the entire program lifetime and must not be freed.

**Safety:** This function is always safe to call. The returned pointer points to static memory embedded in the binary.

**Example (C):**
```c
#include <stdio.h>

extern const char* acomm_version(void);

int main(void) {
    printf("AgenticComm version: %s\n", acomm_version());
    return 0;
}
```

---

### acomm_store_create

Create a new empty CommStore.

```c
CommStore* acomm_store_create(void);
```

**Returns:** A heap-allocated pointer to a new empty `CommStore`. The caller must free it with `acomm_store_free()` when done.

**Safety:** This function is always safe to call. The returned pointer is valid until freed.

**Example (C):**
```c
extern CommStore* acomm_store_create(void);
extern void acomm_store_free(CommStore* store);

CommStore* store = acomm_store_create();
// ... use the store ...
acomm_store_free(store);
```

---

### acomm_store_free

Free a CommStore previously created by `acomm_store_create` or `acomm_load`.

```c
void acomm_store_free(CommStore* store);
```

**Parameters:**
- `store` -- Pointer to the store to free. May be null (no-op).

**Safety:**
- The pointer must have been returned by `acomm_store_create()` or `acomm_load()`.
- The pointer must not have been freed already (double-free is undefined behavior).
- After this call, the pointer is invalid and must not be used.

---

### acomm_create_channel

Create a new communication channel in the store.

```c
uint64_t acomm_create_channel(
    CommStore* store,
    const char* name,
    uint32_t channel_type
);
```

**Parameters:**
- `store` -- Valid pointer to a `CommStore` (from `acomm_store_create` or `acomm_load`).
- `name` -- Null-terminated UTF-8 string for the channel name. Must follow naming rules: 1-128 characters, alphanumeric with hyphens and underscores.
- `channel_type` -- Integer specifying the channel type:

| Value | Channel Type |
|-------|-------------|
| 0 | Direct |
| 1 | Group |
| 2 | Broadcast |
| 3 | PubSub |

**Returns:** The channel ID (a positive `u64`), or `0` on error.

**Error conditions (returns 0):**
- `store` is null.
- `name` is null.
- `name` is not valid UTF-8.
- `name` fails validation (empty, too long, invalid characters).
- `channel_type` is not 0, 1, 2, or 3.

**Safety:**
- `store` must be a valid pointer from `acomm_store_create` or `acomm_load`.
- `name` must be a valid null-terminated UTF-8 string.
- `name` must remain valid for the duration of the call (the function copies the string).

**Example (C):**
```c
uint64_t ch_id = acomm_create_channel(store, "backend-team", 1);
if (ch_id == 0) {
    fprintf(stderr, "Failed to create channel\n");
}
```

---

### acomm_send_message

Send a text message to a channel.

```c
uint64_t acomm_send_message(
    CommStore* store,
    uint64_t channel_id,
    const char* sender,
    const char* content
);
```

**Parameters:**
- `store` -- Valid pointer to a `CommStore`.
- `channel_id` -- ID of the target channel (must exist in the store).
- `sender` -- Null-terminated UTF-8 string identifying the sender.
- `content` -- Null-terminated UTF-8 string containing the message body (1 byte to 1 MB).

**Returns:** The message ID (a positive `u64`), or `0` on error.

**Error conditions (returns 0):**
- `store`, `sender`, or `content` is null.
- `sender` or `content` is not valid UTF-8.
- `sender` is empty.
- `content` is empty or exceeds 1 MB.
- `channel_id` does not exist in the store.

**Notes:**
- The message type is always `Text`. To send other message types (Command, Query, etc.), use the MCP or CLI interface.
- The message is signed with a SHA-256 hash of the content.
- The message is persisted in memory. To save to disk, call `acomm_save()`.

**Safety:**
- `store` must be a valid pointer from `acomm_store_create` or `acomm_load`.
- `sender` and `content` must be valid null-terminated UTF-8 strings.
- All string parameters must remain valid for the duration of the call.

**Example (C):**
```c
uint64_t msg_id = acomm_send_message(store, ch_id, "agent-planner", "Deploy to staging");
if (msg_id == 0) {
    fprintf(stderr, "Failed to send message\n");
}
```

---

### acomm_receive_messages

Receive all messages from a channel as a JSON string.

```c
char* acomm_receive_messages(
    CommStore* store,
    uint64_t channel_id
);
```

**Parameters:**
- `store` -- Valid pointer to a `CommStore`.
- `channel_id` -- ID of the channel to read from.

**Returns:** A heap-allocated null-terminated JSON string containing an array of message objects, or `NULL` on error. The caller must free the returned string with `acomm_string_free()`.

**Error conditions (returns NULL):**
- `store` is null.
- `channel_id` does not exist in the store.
- JSON serialization fails (should not happen with valid data).

**Return format (JSON):**
```json
[
  {
    "id": 1,
    "channel_id": 1,
    "sender": "agent-planner",
    "recipient": null,
    "content": "Deploy to staging",
    "message_type": "Text",
    "timestamp": "2026-02-28T10:30:00Z",
    "metadata": {},
    "signature": "a1b2c3d4...",
    "acknowledged_by": []
  }
]
```

**Safety:**
- `store` must be a valid pointer.
- The returned string must be freed with `acomm_string_free()`.

**Example (C):**
```c
char* json = acomm_receive_messages(store, ch_id);
if (json != NULL) {
    printf("Messages: %s\n", json);
    acomm_string_free(json);
}
```

---

### acomm_list_channels

List all channels as a JSON string.

```c
char* acomm_list_channels(CommStore* store);
```

**Parameters:**
- `store` -- Valid pointer to a `CommStore`.

**Returns:** A heap-allocated null-terminated JSON string containing an array of channel objects, or `NULL` on error. The caller must free the returned string with `acomm_string_free()`.

**Return format (JSON):**
```json
[
  {
    "id": 1,
    "name": "backend-team",
    "channel_type": "Group",
    "created_at": "2026-02-28T10:00:00Z",
    "participants": [],
    "config": {
      "max_participants": 0,
      "ttl_seconds": 0,
      "persistence": true,
      "encryption_required": false
    }
  }
]
```

**Safety:**
- `store` must be a valid pointer.
- The returned string must be freed with `acomm_string_free()`.

---

### acomm_save

Save the store to a `.acomm` file.

```c
bool acomm_save(CommStore* store, const char* path);
```

**Parameters:**
- `store` -- Valid pointer to a `CommStore`.
- `path` -- Null-terminated UTF-8 string specifying the file path.

**Returns:** `true` on success, `false` on error.

**Error conditions (returns false):**
- `store` or `path` is null.
- `path` is not valid UTF-8.
- File I/O error (permission denied, disk full, invalid path).
- Serialization error (should not happen with valid data).

**Safety:**
- `store` must be a valid pointer.
- `path` must be a valid null-terminated UTF-8 string.

**Example (C):**
```c
if (!acomm_save(store, "project.acomm")) {
    fprintf(stderr, "Failed to save store\n");
}
```

---

### acomm_load

Load a store from a `.acomm` file.

```c
CommStore* acomm_load(const char* path);
```

**Parameters:**
- `path` -- Null-terminated UTF-8 string specifying the file path.

**Returns:** A heap-allocated pointer to the loaded `CommStore`, or `NULL` on error. The caller must free it with `acomm_store_free()`.

**Error conditions (returns NULL):**
- `path` is null.
- `path` is not valid UTF-8.
- File does not exist.
- File is not a valid `.acomm` file (bad magic bytes, unsupported version).
- Deserialization error (corrupt data).

**Safety:**
- `path` must be a valid null-terminated UTF-8 string.
- The returned pointer must be freed with `acomm_store_free()`.

**Example (C):**
```c
CommStore* store = acomm_load("project.acomm");
if (store == NULL) {
    fprintf(stderr, "Failed to load store\n");
}
// ... use store ...
acomm_store_free(store);
```

---

### acomm_string_free

Free a string previously returned by an `acomm_*` function.

```c
void acomm_string_free(char* s);
```

**Parameters:**
- `s` -- Pointer to a string returned by `acomm_receive_messages()` or `acomm_list_channels()`. May be null (no-op).

**Safety:**
- The pointer must have been returned by an `acomm_*` function that returns `*mut c_char`.
- The pointer must not have been freed already.
- After this call, the pointer is invalid and must not be used.

## Language Binding Examples

### Python (ctypes)

```python
import ctypes
import json

# Load the shared library
lib = ctypes.CDLL("./target/release/libagentic_comm_ffi.dylib")

# Define function signatures
lib.acomm_version.restype = ctypes.c_char_p
lib.acomm_version.argtypes = []

lib.acomm_store_create.restype = ctypes.c_void_p
lib.acomm_store_create.argtypes = []

lib.acomm_store_free.restype = None
lib.acomm_store_free.argtypes = [ctypes.c_void_p]

lib.acomm_create_channel.restype = ctypes.c_uint64
lib.acomm_create_channel.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_uint32]

lib.acomm_send_message.restype = ctypes.c_uint64
lib.acomm_send_message.argtypes = [ctypes.c_void_p, ctypes.c_uint64, ctypes.c_char_p, ctypes.c_char_p]

lib.acomm_receive_messages.restype = ctypes.c_char_p
lib.acomm_receive_messages.argtypes = [ctypes.c_void_p, ctypes.c_uint64]

lib.acomm_list_channels.restype = ctypes.c_char_p
lib.acomm_list_channels.argtypes = [ctypes.c_void_p]

lib.acomm_save.restype = ctypes.c_bool
lib.acomm_save.argtypes = [ctypes.c_void_p, ctypes.c_char_p]

lib.acomm_load.restype = ctypes.c_void_p
lib.acomm_load.argtypes = [ctypes.c_char_p]

lib.acomm_string_free.restype = None
lib.acomm_string_free.argtypes = [ctypes.c_char_p]

# Usage
print(f"Version: {lib.acomm_version().decode()}")

store = lib.acomm_store_create()
ch_id = lib.acomm_create_channel(store, b"my-channel", 1)
print(f"Channel ID: {ch_id}")

msg_id = lib.acomm_send_message(store, ch_id, b"python-agent", b"Hello from Python")
print(f"Message ID: {msg_id}")

messages_json = lib.acomm_receive_messages(store, ch_id)
messages = json.loads(messages_json.decode())
print(f"Messages: {json.dumps(messages, indent=2)}")

lib.acomm_save(store, b"python-test.acomm")
lib.acomm_store_free(store)
```

### Node.js (ffi-napi)

```javascript
const ffi = require("ffi-napi");
const ref = require("ref-napi");

const lib = ffi.Library("./target/release/libagentic_comm_ffi", {
  acomm_version:          ["string", []],
  acomm_store_create:     ["pointer", []],
  acomm_store_free:       ["void", ["pointer"]],
  acomm_create_channel:   ["uint64", ["pointer", "string", "uint32"]],
  acomm_send_message:     ["uint64", ["pointer", "uint64", "string", "string"]],
  acomm_receive_messages: ["string", ["pointer", "uint64"]],
  acomm_list_channels:    ["string", ["pointer"]],
  acomm_save:             ["bool", ["pointer", "string"]],
  acomm_load:             ["pointer", ["string"]],
  acomm_string_free:      ["void", ["pointer"]],
});

console.log(`Version: ${lib.acomm_version()}`);

const store = lib.acomm_store_create();
const chId = lib.acomm_create_channel(store, "node-channel", 1);
console.log(`Channel ID: ${chId}`);

const msgId = lib.acomm_send_message(store, chId, "node-agent", "Hello from Node.js");
console.log(`Message ID: ${msgId}`);

const messagesJson = lib.acomm_receive_messages(store, chId);
const messages = JSON.parse(messagesJson);
console.log(`Messages:`, messages);

lib.acomm_save(store, "node-test.acomm");
lib.acomm_store_free(store);
```

### Swift

```swift
import Foundation

// Link against the dynamic library
@_silgen_name("acomm_version")
func acomm_version() -> UnsafePointer<CChar>

@_silgen_name("acomm_store_create")
func acomm_store_create() -> UnsafeMutableRawPointer

@_silgen_name("acomm_store_free")
func acomm_store_free(_ store: UnsafeMutableRawPointer?)

@_silgen_name("acomm_create_channel")
func acomm_create_channel(_ store: UnsafeMutableRawPointer,
                           _ name: UnsafePointer<CChar>,
                           _ channelType: UInt32) -> UInt64

@_silgen_name("acomm_send_message")
func acomm_send_message(_ store: UnsafeMutableRawPointer,
                         _ channelId: UInt64,
                         _ sender: UnsafePointer<CChar>,
                         _ content: UnsafePointer<CChar>) -> UInt64

@_silgen_name("acomm_save")
func acomm_save(_ store: UnsafeMutableRawPointer,
                _ path: UnsafePointer<CChar>) -> Bool

// Usage
let version = String(cString: acomm_version())
print("Version: \(version)")

let store = acomm_store_create()
let chId = acomm_create_channel(store, "swift-channel", 1)
print("Channel ID: \(chId)")

let msgId = acomm_send_message(store, chId, "swift-agent", "Hello from Swift")
print("Message ID: \(msgId)")

let saved = acomm_save(store, "swift-test.acomm")
print("Saved: \(saved)")

acomm_store_free(store)
```

### C (Complete Example)

```c
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

// Forward declarations (from the shared library)
typedef struct CommStore CommStore;

extern const char* acomm_version(void);
extern CommStore* acomm_store_create(void);
extern void acomm_store_free(CommStore* store);
extern uint64_t acomm_create_channel(CommStore* store, const char* name, uint32_t channel_type);
extern uint64_t acomm_send_message(CommStore* store, uint64_t channel_id, const char* sender, const char* content);
extern char* acomm_receive_messages(CommStore* store, uint64_t channel_id);
extern char* acomm_list_channels(CommStore* store);
extern bool acomm_save(CommStore* store, const char* path);
extern CommStore* acomm_load(const char* path);
extern void acomm_string_free(char* s);

int main(void) {
    printf("AgenticComm FFI version: %s\n", acomm_version());

    // Create a store and channel
    CommStore* store = acomm_store_create();
    uint64_t ch_id = acomm_create_channel(store, "c-channel", 1);
    if (ch_id == 0) {
        fprintf(stderr, "Failed to create channel\n");
        acomm_store_free(store);
        return 1;
    }
    printf("Channel ID: %llu\n", (unsigned long long)ch_id);

    // Send messages
    uint64_t msg1 = acomm_send_message(store, ch_id, "agent-a", "First message");
    uint64_t msg2 = acomm_send_message(store, ch_id, "agent-b", "Second message");
    printf("Message IDs: %llu, %llu\n", (unsigned long long)msg1, (unsigned long long)msg2);

    // Receive and print messages
    char* messages = acomm_receive_messages(store, ch_id);
    if (messages) {
        printf("Messages JSON:\n%s\n", messages);
        acomm_string_free(messages);
    }

    // List channels
    char* channels = acomm_list_channels(store);
    if (channels) {
        printf("Channels JSON:\n%s\n", channels);
        acomm_string_free(channels);
    }

    // Save and reload
    if (acomm_save(store, "c-test.acomm")) {
        printf("Store saved successfully\n");
    }
    acomm_store_free(store);

    // Load from file
    CommStore* loaded = acomm_load("c-test.acomm");
    if (loaded) {
        char* loaded_channels = acomm_list_channels(loaded);
        if (loaded_channels) {
            printf("Loaded channels:\n%s\n", loaded_channels);
            acomm_string_free(loaded_channels);
        }
        acomm_store_free(loaded);
    }

    return 0;
}
```

Compile with:
```bash
gcc -o test_ffi test_ffi.c -L./target/release -lagentic_comm_ffi -Wl,-rpath,./target/release
```

## Thread Safety

The FFI functions are **not thread-safe**. A `CommStore` pointer must not be shared across threads without external synchronization. If multiple threads need to access the store concurrently, the caller must protect all `acomm_*` calls with a mutex.

## Error Handling Summary

| Function | Success | Error |
|----------|---------|-------|
| `acomm_version` | `const char*` (static) | Never fails |
| `acomm_store_create` | `CommStore*` (heap) | Never fails |
| `acomm_store_free` | void | Never fails (null-safe) |
| `acomm_create_channel` | Channel ID > 0 | Returns `0` |
| `acomm_send_message` | Message ID > 0 | Returns `0` |
| `acomm_receive_messages` | JSON `char*` (heap) | Returns `NULL` |
| `acomm_list_channels` | JSON `char*` (heap) | Returns `NULL` |
| `acomm_save` | `true` | `false` |
| `acomm_load` | `CommStore*` (heap) | Returns `NULL` |
| `acomm_string_free` | void | Never fails (null-safe) |
