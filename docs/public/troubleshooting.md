---
status: stable
---

# Troubleshooting

This guide covers common problems, error messages, diagnostic procedures, and solutions for AgenticComm. Problems are organized by category: store and file issues, channel and messaging errors, MCP server problems, CLI issues, and FFI integration problems.

## Store and File Issues

### Store file not found

**Symptom:** The CLI or MCP server fails to load the store file.

```
Warning: could not load /path/to/store.acomm: No such file or directory (os error 2)
```

**Cause:** The `.acomm` file does not exist at the resolved path.

**Solution:**

1. Check which path is being resolved:
   ```bash
   # Check env var
   echo $ACOMM_STORE

   # Check for local .acomm directory
   ls -la .acomm/store.acomm

   # Check home directory fallback
   ls -la ~/.store.acomm
   ```

2. Create the store explicitly:
   ```bash
   # CLI: the add command auto-creates
   acomm add project.acomm text "initial message" --channel default

   # Or create a channel (which creates the store on save)
   acomm channel create my-channel --file project.acomm
   ```

3. For MCP, verify the `--store` argument in your MCP configuration:
   ```json
   {
     "mcpServers": {
       "agentic-comm": {
         "command": "agentic-comm-mcp",
         "args": ["--store", "/absolute/path/to/store.acomm"]
       }
     }
   }
   ```

**Note:** Both the CLI and MCP server gracefully handle a missing store by creating a new empty `CommStore` in memory. The warning is informational. The store is created on disk when the first mutating operation triggers a save.

---

### Invalid .acomm file (corrupt or wrong format)

**Symptom:** Loading fails with a format error.

```
Error reading project.acomm: Invalid .acomm file: Bad header: ...
```

or:

```
Error reading project.acomm: Invalid .acomm file: Invalid magic bytes — not an .acomm file
```

**Cause:** The file is corrupt, was written by an incompatible version, or is not an `.acomm` file at all.

**Diagnosis:**

```bash
# Check the first 8 bytes (should be ACOMM001)
xxd -l 8 project.acomm

# Check file size (should be > 0)
ls -la project.acomm

# Check if the file is gzip-compressed (first two bytes: 1f 8b)
xxd -l 2 project.acomm
```

**Solutions:**

1. **If the file is corrupt:** Restore from a backup if available. The atomic write protocol creates a temporary file (`*.acomm.tmp`) during writes. If a crash occurred during write, the original file should be intact. Check for `.acomm.bak` backup files.

2. **If the file was created by a newer version:** Upgrade AgenticComm:
   ```bash
   cargo install agentic-comm-cli --force
   cargo install agentic-comm-mcp --force
   ```

3. **If the file is not an .acomm file:** Verify you are pointing to the correct file. A common mistake is pointing to a directory instead of a file.

4. **If the file is empty (0 bytes):** The file was likely created but never written to. Delete it and let AgenticComm create a new one:
   ```bash
   rm project.acomm
   ```

---

### Permission denied when saving

**Symptom:** Mutating operations succeed in memory but the store cannot be saved to disk.

```
Warning: failed to save store: IO error: Permission denied (os error 13)
```

**Cause:** The process does not have write permission to the store file or its parent directory.

**Solution:**

1. Check file permissions:
   ```bash
   ls -la /path/to/store.acomm
   ls -la /path/to/  # parent directory
   ```

2. Fix permissions:
   ```bash
   chmod 644 /path/to/store.acomm
   chmod 755 /path/to/  # parent directory must be writable
   ```

3. Check if the file is owned by a different user:
   ```bash
   stat /path/to/store.acomm
   ```

4. For MCP servers launched by Claude Code, ensure the binary runs as the same user that owns the store file.

---

### Disk full during save

**Symptom:** Save fails with an I/O error.

```
Warning: failed to save store: IO error: No space left on device (os error 28)
```

**Solution:**

1. Free disk space.
2. Move the store to a volume with more space:
   ```bash
   mv project.acomm /volume/with/space/
   export ACOMM_STORE=/volume/with/space/project.acomm
   ```
3. Compact the store to reduce size:
   ```bash
   acomm info project.acomm  # check current size
   ```

---

### Lock file prevents access

**Symptom:** The MCP server or CLI cannot write to the store because another process holds the lock.

**Diagnosis:**

```bash
# Check for lock file
ls -la /path/to/store.acomm.lock

# Read lock file contents
cat /path/to/store.acomm.lock
```

The lock file contains the PID, timestamp, and hostname of the holding process.

**Solution:**

1. Check if the holding process is still running:
   ```bash
   ps -p <PID_FROM_LOCK_FILE>
   ```

2. If the process is not running (stale lock), remove the lock file:
   ```bash
   rm /path/to/store.acomm.lock
   ```

3. If the process is running, wait for it to finish or terminate it.

**Note:** Stale locks are automatically recovered after 60 seconds by both the CLI and MCP server.

## Channel and Messaging Errors

### Channel not found

**Symptom:** Operations fail with a channel not found error.

```
Error: Channel not found: 42
```

**Cause:** The specified channel ID does not exist in the store.

**Diagnosis:**

```bash
# List all channels to find the correct ID
acomm channel list --file project.acomm
```

**Common mistakes:**
- Using a channel name instead of a channel ID. The `send` and `receive` commands expect integer channel IDs, not names.
- The channel was created in a different store file. Check that `ACOMM_STORE` or `--file` points to the correct store.
- The channel was created in a previous session but the store was not saved (if using manual flush mode).

---

### Message content validation errors

**Symptom:** Sending a message fails with a content validation error.

```
Error: Invalid message content: Message content cannot be empty
```

or:

```
Error: Invalid message content: Message content exceeds maximum size of 1048576 bytes
```

**Solutions:**

1. **Empty content:** Provide at least 1 byte of content. All messages must have non-empty bodies.

2. **Content too large:** The default maximum is 1 MB (1,048,576 bytes). Split large payloads into multiple messages or increase the limit via configuration:
   ```bash
   export ACOMM_MAX_MESSAGE_SIZE=10485760  # 10 MB
   ```

---

### Channel full (max participants reached)

**Symptom:** Joining a channel fails.

```
Error: Channel 1 has reached maximum participants
```

**Solution:**

1. Check the current channel configuration:
   ```bash
   acomm channel info 1 --file project.acomm
   ```

2. Increase the participant limit:
   ```json
   {
     "method": "tools/call",
     "params": {
       "name": "set_channel_config",
       "arguments": {
         "channel_id": 1,
         "max_participants": 50
       }
     }
   }
   ```

3. Set to unlimited:
   ```json
   { "max_participants": 0 }
   ```

---

### Duplicate participant

**Symptom:** Joining a channel fails because the participant is already a member.

```
Error: Participant 'agent-worker' is already in channel 1
```

**Solution:** This is expected behavior. The `join_channel` operation is not idempotent -- calling it twice with the same participant produces an error. Check the channel's participant list before joining:

```json
{
  "method": "tools/call",
  "params": {
    "name": "get_channel_info",
    "arguments": { "channel_id": 1 }
  }
}
```

---

### Participant not in channel

**Symptom:** Leaving a channel fails.

```
Error: Participant 'ghost-agent' is not in channel 1
```

**Cause:** The participant name does not match any current participant in the channel.

**Diagnosis:** Check the exact participant names (they are case-sensitive):

```bash
acomm channel info 1 --file project.acomm | jq '.participants'
```

---

### Invalid channel name

**Symptom:** Creating a channel fails with a name validation error.

```
Error: Invalid channel name: Channel name must contain only alphanumeric characters, hyphens, or underscores
```

**Rules:**
- 1-128 characters
- Only alphanumeric characters (`a-z`, `A-Z`, `0-9`), hyphens (`-`), and underscores (`_`)
- No spaces, dots, slashes, or other special characters

**Valid examples:** `backend-team`, `agent_worker_01`, `BuildChannel`

**Invalid examples:** `backend team` (spaces), `agent.worker` (dots), `build/frontend` (slashes), `` (empty)

---

### Invalid message type

**Symptom:** Sending a message with an unknown type fails.

```
Invalid message type: Unknown message type: request
```

**Valid message types:** `text`, `command`, `query`, `response`, `broadcast`, `notification`, `acknowledgment` (or `ack`), `error`

**Note:** Message types are case-insensitive. `Text`, `TEXT`, and `text` are all accepted.

## MCP Server Issues

### MCP server not starting

**Symptom:** Claude Code reports that the MCP server failed to connect.

**Diagnosis:**

1. Verify the binary is installed:
   ```bash
   which agentic-comm-mcp
   agentic-comm-mcp --version
   ```

2. Test the server manually:
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}' | agentic-comm-mcp
   ```

3. Check the MCP configuration in Claude Code:
   ```bash
   cat ~/.claude/mcp.json
   ```

4. Verify the command path is absolute or in PATH:
   ```json
   {
     "mcpServers": {
       "agentic-comm": {
         "command": "/Users/username/.cargo/bin/agentic-comm-mcp",
         "args": []
       }
     }
   }
   ```

---

### Tool not found error

**Symptom:** A tool call returns a JSON-RPC error with code `-32803`.

```json
{"jsonrpc": "2.0", "error": {"code": -32803, "message": "Tool not found: unknown_tool"}, "id": 1}
```

**Cause:** The tool name in the `tools/call` request does not match any registered tool.

**Solution:** Check the tool name against the list of 17 registered tools:

`send_message`, `receive_messages`, `create_channel`, `list_channels`, `join_channel`, `leave_channel`, `get_channel_info`, `subscribe`, `unsubscribe`, `publish`, `broadcast`, `query_history`, `search_messages`, `get_message`, `acknowledge_message`, `set_channel_config`, `communication_log`

**Note:** Tool names are exact-match, case-sensitive.

---

### Invalid parameters error

**Symptom:** A tool call fails with a parameter validation error.

```json
{"jsonrpc": "2.0", "error": {"code": -32602, "message": "channel_id is required"}, "id": 1}
```

**Cause:** A required parameter is missing, has the wrong type, or has an invalid value.

**Solution:** Check the parameter specification in [MCP Tools Reference](mcp-tools.md). Common issues:

- Sending a string where an integer is expected (e.g., `"channel_id": "1"` instead of `"channel_id": 1`)
- Omitting required parameters
- Using a parameter name that does not exist for the tool

---

### MCP server crashes on large stores

**Symptom:** The MCP server exits unexpectedly when loading a large `.acomm` file.

**Cause:** The store exceeds available memory. The entire store is loaded into memory on startup.

**Solution:**

1. Check the store size:
   ```bash
   acomm info large-store.acomm
   ```

2. If the message count is very high, consider archiving old messages or creating a new store.

3. Increase available memory or move to a machine with more RAM.

## CLI Issues

### Command not found

**Symptom:** The `acomm` command is not recognized.

```
zsh: command not found: acomm
```

**Solution:**

1. Install the CLI:
   ```bash
   cargo install agentic-comm-cli
   ```

2. Verify it is in PATH:
   ```bash
   echo $PATH | tr ':' '\n' | grep cargo
   # Should include ~/.cargo/bin
   ```

3. Add cargo bin to PATH if needed:
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

---

### JSON output is empty array

**Symptom:** `acomm receive` or `acomm channel list` returns `[]`.

**Cause:** The store is empty or the query filters are too restrictive.

**Diagnosis:**

```bash
# Check store statistics
acomm info project.acomm

# Check if channels exist
acomm channel list --file project.acomm

# Try without filters
acomm receive 1 --file project.acomm
```

---

### Wrong store file being used

**Symptom:** Operations succeed but data does not appear where expected.

**Diagnosis:**

1. Check all sources of path resolution:
   ```bash
   echo "ACOMM_STORE=$ACOMM_STORE"
   ls -la .acomm/store.acomm 2>/dev/null
   ls -la ~/.store.acomm 2>/dev/null
   ```

2. Use explicit `--file` to confirm:
   ```bash
   acomm channel list --file /exact/path/to/store.acomm
   ```

3. Enable debug logging to see path resolution:
   ```bash
   ACOMM_LOG_LEVEL=debug acomm channel list 2>&1 | head -10
   ```

## FFI Issues

### Shared library not found

**Symptom:** Loading the shared library fails.

```python
OSError: dlopen(libagentic_comm_ffi.dylib, ...): image not found
```

**Solution:**

1. Build the library:
   ```bash
   cargo build --release -p agentic-comm-ffi
   ```

2. Use the full path:
   ```python
   lib = ctypes.CDLL("/path/to/target/release/libagentic_comm_ffi.dylib")
   ```

3. Or set the library search path:
   ```bash
   export LD_LIBRARY_PATH=/path/to/target/release:$LD_LIBRARY_PATH  # Linux
   export DYLD_LIBRARY_PATH=/path/to/target/release:$DYLD_LIBRARY_PATH  # macOS
   ```

---

### Function returns 0 or NULL unexpectedly

**Symptom:** FFI functions return error values (0, NULL, false) without explanation.

**Cause:** The FFI layer does not expose detailed error messages. A return value of 0 or NULL means one of the safety checks failed.

**Diagnosis checklist:**

1. **Is the store pointer valid?** Make sure `acomm_store_create()` returned a non-null pointer and the store has not been freed.

2. **Are string arguments null-terminated UTF-8?** The FFI functions expect C-style null-terminated strings. Passing binary data or non-UTF-8 strings causes silent failure.

3. **Does the channel exist?** When sending messages, the channel ID must refer to a channel that exists in the store.

4. **Is the channel type valid?** `acomm_create_channel` expects 0-3 for the channel type. Other values return 0.

**Debugging tip:** Test the same operation with the CLI to confirm it works at the core library level. If the CLI succeeds but the FFI fails, the issue is in the FFI parameter passing.

---

### Memory leak from unreleased strings

**Symptom:** Memory usage grows continuously when calling FFI functions repeatedly.

**Cause:** Strings returned by `acomm_receive_messages()` and `acomm_list_channels()` are heap-allocated and must be freed by the caller.

**Solution:** Always call `acomm_string_free()` on returned string pointers:

```python
json_str = lib.acomm_receive_messages(store, ch_id)
if json_str:
    # Process the string...
    lib.acomm_string_free(json_str)  # Must free!
```

## Diagnostic Commands

### Quick health check

```bash
# Check binary versions
acomm --version
agentic-comm-mcp --version 2>/dev/null || echo "MCP server not installed"

# Check store health
acomm info project.acomm

# List channels
acomm channel list --file project.acomm

# Test send/receive cycle
acomm channel create test-diag --file /tmp/diag.acomm
acomm send 1 "diagnostic message" --file /tmp/diag.acomm
acomm receive 1 --file /tmp/diag.acomm
rm /tmp/diag.acomm
```

### Enable verbose logging

```bash
# CLI
ACOMM_LOG_LEVEL=debug acomm send 1 "test" --file project.acomm

# MCP server
ACOMM_LOG_LEVEL=trace agentic-comm-mcp --store project.acomm
```

### Check MCP protocol manually

```bash
# Send an initialize request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | agentic-comm-mcp

# List tools
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | agentic-comm-mcp
```

## Known Limitations (v0.1.0)

1. **In-memory store:** The entire store is loaded into memory. Stores with millions of messages require proportional RAM.

2. **No concurrent access:** The FFI functions are not thread-safe. The MCP server handles one request at a time via stdio.

3. **No network transport:** The MCP server communicates via stdio only. REST and webhook bridges are planned for a future release.

4. **Text-only messages via FFI:** The `acomm_send_message` FFI function always sends `Text` type messages. Other message types require the CLI or MCP interface.

5. **No encryption at rest:** The `encryption_required` channel config flag is accepted but encryption is not yet implemented. Messages are stored in plaintext (compressed but not encrypted).

6. **No message deletion:** Individual messages cannot be deleted. The entire store must be replaced to remove specific messages.
