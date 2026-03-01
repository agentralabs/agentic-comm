---
status: stable
---

# Configuration Reference

AgenticComm is configured through environment variables, a TOML configuration file, CLI flags, and MCP server arguments. This document covers every configuration option, the resolution precedence, and per-project isolation setup.

## Configuration Precedence

When multiple configuration sources define the same value, the highest-priority source wins:

```
Priority 1 (highest): CLI flags / MCP server arguments
Priority 2:           Environment variables
Priority 3:           Configuration file (~/.config/agentic-comm/config.toml)
Priority 4 (lowest):  Built-in defaults
```

## Environment Variables

### Store Path

| Variable | Default | Description |
|----------|---------|-------------|
| `ACOMM_STORE` | (none) | Path to the `.acomm` store file. When set, both the CLI and MCP server use this path as the default store location. Overridden by explicit `--file` (CLI) or `--store` (MCP) flags. |

When `ACOMM_STORE` is not set and no explicit path is provided, the path resolution chain is:

```
1. .acomm/store.acomm    (current working directory)
2. ~/.store.acomm         (home directory fallback)
```

The local `.acomm/store.acomm` path enables per-project isolation: if a `.acomm/` directory exists in the current working directory, the store file inside it is used automatically.

### Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `ACOMM_LOG_LEVEL` | `warn` | Controls log verbosity for both the CLI and MCP server. Valid values: `trace`, `debug`, `info`, `warn`, `error`. |

Log levels control the amount of diagnostic output written to stderr:

| Level | What is logged |
|-------|---------------|
| `trace` | Every function call, parameter value, and internal state change. Extremely verbose. Use only for debugging store corruption or protocol issues. |
| `debug` | Tool dispatch decisions, path resolution, store load/save operations, message routing decisions. Useful for diagnosing operational issues. |
| `info` | Session start/stop, channel creation/deletion, significant operations. Suitable for monitoring normal operation. |
| `warn` | Recoverable errors (failed store load with fallback to empty store, stale lock files). Default level. |
| `error` | Unrecoverable errors (corrupt store file, I/O failures). Always logged regardless of setting. |

### Defaults

| Variable | Default | Description |
|----------|---------|-------------|
| `ACOMM_DEFAULT_CHANNEL` | (none) | Default channel name for CLI send/receive commands when `--channel` is not specified. |
| `ACOMM_SENDER` | (none) | Default sender participant ID for CLI commands when `--sender` is not specified. The CLI binary uses `cli-user` as a hardcoded fallback if this variable is not set. |

### Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `AGENTIC_TOKEN` | (none) | Authentication token for the REST bridge and webhook endpoints (future feature). |
| `ACOMM_FLUSH_MODE` | `batch` | Controls when the store is written to disk. Values: `immediate` (after every operation), `batch` (after accumulating a batch), `manual` (only on explicit save). |
| `ACOMM_BATCH_SIZE` | `100` | Number of operations to accumulate before auto-flushing in batch mode. |
| `ACOMM_BATCH_DELAY_MS` | `5000` | Maximum milliseconds to wait before auto-flushing in batch mode, even if the batch size has not been reached. |
| `ACOMM_MAX_MESSAGE_SIZE` | `1048576` | Global maximum message content size in bytes (default: 1 MB). Messages exceeding this size are rejected with a validation error. |

## Configuration File

AgenticComm reads an optional configuration file from `~/.config/agentic-comm/config.toml`. This file provides a persistent, shareable way to configure defaults that apply across all CLI invocations and MCP sessions.

### Full Configuration File Reference

```toml
# ~/.config/agentic-comm/config.toml

# -----------------------------------------------------------------------
# Store settings
# -----------------------------------------------------------------------
[store]
# Default path for the .acomm store file.
# Overridden by ACOMM_STORE env var and --file/--store flags.
default_path = "~/Documents/agentic/comm.acomm"

# When to write the store to disk.
# Values: "immediate", "batch", "manual"
flush_mode = "batch"

# Number of operations before auto-flush (batch mode only).
batch_size = 100

# Max delay in milliseconds before auto-flush (batch mode only).
batch_delay_ms = 5000

# -----------------------------------------------------------------------
# Default identities
# -----------------------------------------------------------------------
[defaults]
# Default sender participant ID for CLI commands.
sender = "claude-code"

# Default channel name for CLI commands.
channel = "main"

# -----------------------------------------------------------------------
# Logging
# -----------------------------------------------------------------------
[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Optional log file path. When set, logs are written to this file
# in addition to stderr.
# file = "/var/log/agentic-comm.log"

# -----------------------------------------------------------------------
# Identity integration
# -----------------------------------------------------------------------
[identity]
# AgenticIdentity name for message signing. When set, messages sent
# through the CLI and MCP server are signed with this identity.
# Requires agentic-identity to be installed and configured.
# name = "default"

# -----------------------------------------------------------------------
# Resource limits
# -----------------------------------------------------------------------
[limits]
# Maximum message content size in bytes.
max_message_size = 1048576

# Maximum number of channels per store.
max_channels = 100000

# Maximum number of messages per store.
max_messages = 10000000

# Maximum number of subscriptions per store.
max_subscriptions = 1000000
```

### Configuration File Location

The configuration file is searched in these locations (first found wins):

| Priority | Path | Description |
|----------|------|-------------|
| 1 | `$ACOMM_CONFIG` | Explicit config path (if this env var is set) |
| 2 | `~/.config/agentic-comm/config.toml` | XDG-compliant config directory |
| 3 | `~/.agentic-comm.toml` | Home directory fallback |

If no configuration file is found, all values fall back to their built-in defaults. Missing keys in a partial configuration file are filled with defaults. The configuration file is never required.

### Creating the Configuration File

```bash
# Create the config directory
mkdir -p ~/.config/agentic-comm

# Create a minimal config
cat > ~/.config/agentic-comm/config.toml << 'EOF'
[store]
default_path = "~/Documents/agentic/comm.acomm"

[defaults]
sender = "my-agent"

[logging]
level = "info"
EOF
```

## Store Path Resolution

The store path is the most important configuration value. It determines where communication data is read from and written to.

### CLI Resolution

```
1. --file <PATH>                       (explicit flag)
2. ACOMM_STORE environment variable    (env var)
3. .acomm/store.acomm                  (local directory, if exists)
4. ~/.store.acomm                      (home directory fallback)
```

### MCP Server Resolution

The MCP server uses the `resolve_comm_path()` function from the config loader module:

```
1. --store <PATH>                      (server argument)
2. ACOMM_STORE environment variable    (env var)
3. .acomm/store.acomm                  (local directory, if exists)
4. ~/.store.acomm                      (home directory fallback)
```

### Path Expansion

Tilde (`~`) in paths is expanded to the user's home directory. Relative paths are resolved relative to the current working directory.

## Per-Project Isolation

Per-project isolation enables multiple projects to maintain independent communication histories without interfering with each other. This is the recommended setup for teams working on multiple codebases.

### Setup

Create a `.acomm/` directory in the project root:

```bash
cd /path/to/my-project
mkdir -p .acomm
```

The CLI and MCP server will automatically detect this directory and use `.acomm/store.acomm` as the store path, without any explicit configuration.

### Gitignore

Add the store file to `.gitignore` to prevent committing binary data:

```
# .gitignore
*.acomm
*.acomm.lock
*.acomm.tmp
*.acomm.bak
```

The `.acomm/` directory itself can be version-controlled (it serves as a marker for per-project isolation), but the store files inside it should not be.

### Multi-Project Example

```
~/projects/
+-- frontend/
|   +-- .acomm/
|   |   +-- store.acomm     <-- frontend communication history
|   +-- src/
|
+-- backend/
|   +-- .acomm/
|   |   +-- store.acomm     <-- backend communication history
|   +-- src/
|
+-- infra/
    +-- .acomm/
    |   +-- store.acomm     <-- infra communication history
    +-- terraform/
```

Each project has its own `.acomm/store.acomm`. When you `cd` into a project directory and run `acomm` commands, they operate on that project's store automatically.

## MCP Server Configuration

### Claude Code Integration

Add AgenticComm to Claude Code by editing the MCP configuration file:

**Global configuration** (`~/.claude/mcp.json`):

```json
{
  "mcpServers": {
    "agentic-comm": {
      "command": "agentic-comm-mcp",
      "args": []
    }
  }
}
```

**Per-project configuration** (`.mcp.json` in project root):

```json
{
  "mcpServers": {
    "agentic-comm": {
      "command": "agentic-comm-mcp",
      "args": ["--store", ".acomm/store.acomm"]
    }
  }
}
```

### MCP Server Arguments

```
agentic-comm-mcp [OPTIONS]

Options:
  --store <PATH>        Store file path [default: resolved via standard chain]
  --log-level <LEVEL>   Log level [default: warn]
  --mode <MODE>         Session mode: full, minimal [default: full]
```

**Session modes:**

| Mode | Behavior |
|------|----------|
| `full` | Auto-start session on MCP `initialized` notification. Record all operations. Capture communication context. Save on shutdown. |
| `minimal` | Skip auto-session. No operation recording. Reduced memory footprint. Suitable for automated pipelines where session tracking is not needed. |

### Environment Variable Passthrough

When the MCP server is launched by Claude Code, it inherits the environment of the calling process. Environment variables set in the terminal where Claude Code runs are available to the MCP server:

```bash
# Set before launching Claude Code
export ACOMM_STORE=/path/to/project/comm.acomm
export ACOMM_LOG_LEVEL=debug

# Now start Claude Code — the MCP server inherits these
claude
```

## Flush Modes

The flush mode controls when in-memory changes are persisted to the `.acomm` file on disk.

### Immediate Mode

Every mutating operation (send_message, create_channel, etc.) triggers an immediate write to disk. This provides the strongest durability guarantee but has the highest I/O overhead.

```bash
export ACOMM_FLUSH_MODE=immediate
```

### Batch Mode (Default)

Operations accumulate in memory until either the batch size is reached or the batch delay expires, whichever comes first. This balances durability with performance.

```bash
export ACOMM_FLUSH_MODE=batch
export ACOMM_BATCH_SIZE=100
export ACOMM_BATCH_DELAY_MS=5000
```

### Manual Mode

The store is only written to disk when explicitly requested (via CLI `save` or MCP shutdown). Use this for maximum performance when durability is not critical.

```bash
export ACOMM_FLUSH_MODE=manual
```

## Resource Limits

AgenticComm enforces resource limits to prevent unbounded growth and ensure predictable performance.

| Resource | Default Limit | Environment Variable | Description |
|----------|--------------|---------------------|-------------|
| Message content | 1 MB | `ACOMM_MAX_MESSAGE_SIZE` | Maximum size of a single message body |
| Channel name | 128 chars | (not configurable) | Maximum channel name length |
| Topic name | 128 chars | (not configurable) | Maximum topic name length |
| Channels per store | 100,000 | `[limits] max_channels` | Maximum channels in one store |
| Messages per store | 10,000,000 | `[limits] max_messages` | Maximum messages in one store |
| Subscriptions per store | 1,000,000 | `[limits] max_subscriptions` | Maximum subscriptions in one store |
| File size | 2 GB | (not configurable) | Maximum `.acomm` file size |

When a limit is reached, the operation fails with a descriptive error. The store never silently drops or truncates data.

## Validation Rules

These validation rules are enforced regardless of configuration:

### Channel Names
- Must be 1-128 characters long.
- Must contain only alphanumeric characters, hyphens (`-`), and underscores (`_`).
- Spaces, dots, slashes, and other special characters are rejected.

### Message Content
- Must be non-empty (at least 1 byte).
- Must not exceed `MAX_CONTENT_SIZE` (default 1 MB).
- Must be valid UTF-8.

### Sender Identity
- Must be non-empty.
- No length limit beyond system constraints.
- Any UTF-8 string is accepted.

### Topic Names (Pub/Sub)
- Follow the same rules as channel names: 1-128 chars, alphanumeric with hyphens and underscores.
- Dot-separated hierarchies are validated at the topic matching level, not at the name validation level.

## Diagnostics

### Checking Active Configuration

To see what configuration the CLI is using:

```bash
# Check the resolved store path
ACOMM_LOG_LEVEL=debug acomm info test.acomm 2>&1 | head -5

# Check environment variables
env | grep ACOMM

# Check if local .acomm directory exists
ls -la .acomm/ 2>/dev/null || echo "No local .acomm directory"

# Check config file
cat ~/.config/agentic-comm/config.toml 2>/dev/null || echo "No config file"
```

### Common Configuration Patterns

**Development (verbose, local store):**
```bash
export ACOMM_STORE=.acomm/store.acomm
export ACOMM_LOG_LEVEL=debug
export ACOMM_FLUSH_MODE=immediate
export ACOMM_SENDER=dev-agent
```

**CI/CD (minimal, ephemeral):**
```bash
export ACOMM_STORE=/tmp/ci-comm.acomm
export ACOMM_LOG_LEVEL=error
export ACOMM_FLUSH_MODE=manual
export ACOMM_SENDER=ci-agent
```

**Production (balanced, project-local):**
```bash
# Use per-project isolation (no env vars needed)
mkdir -p .acomm
# Config file handles defaults
```
