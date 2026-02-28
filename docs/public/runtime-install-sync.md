---
status: stable
---

# Runtime, Install Output, and Sync Contract

This page defines expected runtime behavior across installer output, CLI behavior, and web documentation for AgenticComm.

## Installer profiles

- `desktop`: installs binaries and merges detected desktop MCP config.
- `terminal`: installs binaries without desktop-specific UX assumptions.
- `server`: installs binaries without desktop config writes.

## Completion output contract

Installer must print:

1. Installed binary summary.
2. MCP restart instruction.
3. Server auth + artifact sync guidance when relevant.
4. Optional feedback instruction.

Expected completion marker:

```text
Install complete: AgenticComm (<profile>)
```

## Universal MCP config

```json
{
  "mcpServers": {
    "agentic-comm": {
      "command": "$HOME/.local/bin/agentic-comm-mcp",
      "args": ["serve"]
    }
  }
}
```

## Auto-session lifecycle (runtime-sync)

The MCP server automatically manages session state:

1. **Session start**: When the MCP client sends the `initialized` notification, the server calls `mark_session_started()` to begin tracking operations, set up temporal chaining, and prepare the communication log.
2. **Runtime sync**: During the session, every tool call is recorded in the operation log with timestamps and related entity IDs. Communication context entries are accumulated for the 20-Year Clock audit trail.
3. **Session end**: When the client disconnects (EOF on stdin or explicit `shutdown`), `auto_save_on_stop()` persists the store to disk, flushes accumulated logs, and marks the session inactive.

The store file is resolved at startup through the following chain:
1. Explicit CLI argument (`--file`).
2. `ACOMM_STORE` environment variable.
3. `.acomm/store.acomm` in the current working directory.
4. `~/.store.acomm` fallback.

## Workspace auto-detection behavior

- Installer writes `agentic-comm-mcp` launcher as MCP entrypoint.
- Launcher resolves communication store in the order above.
- If no file exists yet, launcher routes to per-workspace default path so first run creates and keeps project communication isolated.
