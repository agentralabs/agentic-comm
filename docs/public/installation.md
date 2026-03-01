---
status: stable
---

# Installation

This guide covers installing AgenticComm for all three usage profiles (desktop, terminal, server), the complete CLI command reference, environment variables, configuration files, and shell completions.

## Installation Profiles

### Desktop Profile (Claude Code / AI Coding Assistants)

Install the MCP server for use with Claude Code or other MCP-compatible assistants.

```bash
# Install the MCP server binary
cargo install agentic-comm-mcp

# Install the CLI for direct interaction
cargo install agentic-comm-cli
```

Configure Claude Code to use the MCP server by adding to `~/.claude/mcp.json`:

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

### Terminal Profile (CLI-first usage)

Install the CLI binary for terminal-based communication management.

```bash
cargo install agentic-comm-cli
```

This provides the `acomm` binary with all subcommands.

### Server Profile (multi-process / networked)

Install the MCP server with REST bridge support for multi-process or networked deployments.

```bash
cargo install agentic-comm-mcp --features rest,webhook

# Run with REST API enabled
agentic-comm-mcp serve --rest-port 9090 --webhook-port 8080
```

### From Source

```bash
git clone https://github.com/agentralabs/agentic-comm
cd agentic-comm

# Build all binaries
cargo build --release

# Install locally
cargo install --path crates/agentic-comm-cli
cargo install --path crates/agentic-comm-mcp
```

### Pre-built Binaries

Download pre-built binaries from the GitHub releases page:

| Platform | Architecture | Binary |
|----------|-------------|--------|
| macOS | ARM64 (Apple Silicon) | `acomm-aarch64-apple-darwin.tar.gz` |
| macOS | x86-64 (Intel) | `acomm-x86_64-apple-darwin.tar.gz` |
| Linux | x86-64 | `acomm-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 | `acomm-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | x86-64 | `acomm-x86_64-pc-windows-msvc.zip` |

Extract and place the binaries in your `$PATH`.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ACOMM_STORE` | `./comm.acomm` | Default path for the `.acomm` store file. Used when no path is provided to CLI commands. |
| `ACOMM_DEFAULT_CHANNEL` | (none) | Default channel name for send/receive commands when `--channel` is not specified. |
| `ACOMM_SENDER` | (none) | Default sender participant ID when `--sender` is not specified. |
| `AGENTIC_TOKEN` | (none) | Authentication token for the REST bridge and webhook endpoints. |
| `ACOMM_LOG_LEVEL` | `warn` | Logging level: `trace`, `debug`, `info`, `warn`, `error`. |
| `ACOMM_FLUSH_MODE` | `batch` | Flush mode: `immediate`, `batch`, `manual`. |
| `ACOMM_BATCH_SIZE` | `100` | Number of messages to accumulate before auto-flushing (batch mode). |
| `ACOMM_BATCH_DELAY_MS` | `5000` | Maximum milliseconds to wait before auto-flushing (batch mode). |
| `ACOMM_MAX_MESSAGE_SIZE` | `1048576` | Global maximum message content size in bytes. |

## Configuration File

AgenticComm reads configuration from `~/.config/agentic-comm/config.toml` if it exists. CLI flags and environment variables override config file values.

```toml
# ~/.config/agentic-comm/config.toml

[store]
default_path = "~/Documents/agentic/comm.acomm"
flush_mode = "batch"
batch_size = 100
batch_delay_ms = 5000

[defaults]
sender = "claude-code"
channel = "main"

[logging]
level = "info"
# file = "/var/log/agentic-comm.log"  # Optional log file

[identity]
# name = "default"  # AgenticIdentity name for message signing

[limits]
max_message_size = 1048576
max_channels = 100000
max_messages = 10000000
```

## CLI Command Reference

### `acomm` -- Top-level

```
Usage: acomm <COMMAND> [OPTIONS] [STORE_PATH]

Commands:
  init         Create a new .acomm store
  send         Send a message
  receive      Receive messages
  ack          Acknowledge a command message
  publish      Publish to a pub/sub topic
  broadcast    Broadcast to all channel participants
  search       Search messages by content
  history      Query communication history
  thread       View a message thread by correlation ID
  stats        Show store statistics
  compact      Remove old archived messages and dead letters
  channel      Channel management subcommands
  subscribe    Subscribe to a pub/sub topic
  unsubscribe  Remove a subscription
  export       Export messages to JSON
  import       Import messages from JSON
  completions  Generate shell completions

Options:
  -v, --verbose    Enable verbose output
  --log-level      Set log level (trace, debug, info, warn, error)
  -h, --help       Print help
  -V, --version    Print version
```

### `acomm init`

Create a new `.acomm` store file.

```
Usage: acomm init [STORE_PATH]

Arguments:
  [STORE_PATH]  Path for the new store file [default: ./comm.acomm]

Options:
  --force  Overwrite if file exists
```

### `acomm send`

Send a message through a channel.

```
Usage: acomm send [OPTIONS] <CONTENT> [STORE_PATH]

Arguments:
  <CONTENT>     Message content (use - to read from stdin)
  [STORE_PATH]  Path to the .acomm store [default: $ACOMM_STORE]

Options:
  --channel <NAME>       Channel name [default: $ACOMM_DEFAULT_CHANNEL]
  --sender <ID>          Sender participant ID [default: $ACOMM_SENDER]
  --type <TYPE>          Message type: text, command, query, broadcast, notification [default: text]
  --topic <TOPIC>        Topic string (for pub/sub channels)
  --priority <LEVEL>     Priority: critical, high, normal, low, background [default: normal]
  --ttl <SECONDS>        Time-to-live in seconds
  --correlation <ID>     Correlation ID for threading
  --metadata <JSON>      JSON metadata object
```

### `acomm receive`

Receive messages from a channel.

```
Usage: acomm receive [OPTIONS] [STORE_PATH]

Options:
  --channel <NAME>   Channel name [default: $ACOMM_DEFAULT_CHANNEL]
  --as <ID>          Receive as this participant [default: $ACOMM_SENDER]
  --type <TYPE>      Filter by message type (comma-separated)
  --after <TIME>     Messages after this timestamp (ISO 8601 or Unix seconds)
  --before <TIME>    Messages before this timestamp
  --limit <N>        Maximum messages to return [default: 100]
  --offset <N>       Skip first N messages [default: 0]
  --format <FMT>     Output format: table, json, csv [default: table]
  --unread           Only show unread messages
```

### `acomm ack`

Acknowledge a command message.

```
Usage: acomm ack [OPTIONS] [CONTENT] [STORE_PATH]

Options:
  --message-id <ID>  ID of the message to acknowledge [required]
  --sender <ID>      Acknowledging participant [default: $ACOMM_SENDER]
  --status <STATUS>  Ack status: received, in_progress, completed, failed [required]
```

### `acomm publish`

Publish a message to a pub/sub topic.

```
Usage: acomm publish [OPTIONS] <CONTENT> [STORE_PATH]

Options:
  --channel <NAME>   PubSub channel name [required]
  --sender <ID>      Publisher participant ID [default: $ACOMM_SENDER]
  --topic <TOPIC>    Topic string [required]
  --priority <LEVEL> Priority [default: normal]
```

### `acomm broadcast`

Broadcast a message to all channel participants.

```
Usage: acomm broadcast [OPTIONS] <CONTENT> [STORE_PATH]

Options:
  --channel <NAME>  Channel name [required]
  --sender <ID>     Sender (must be channel owner for broadcast channels) [default: $ACOMM_SENDER]
```

### `acomm search`

Search messages by content.

```
Usage: acomm search [OPTIONS] <QUERY> [STORE_PATH]

Arguments:
  <QUERY>  Search string (prefix with "regex:" for regex matching)

Options:
  --channel <NAME>   Limit search to a specific channel
  --sender <ID>      Limit search to messages from this sender
  --type <TYPE>      Limit search to this message type
  --limit <N>        Maximum results [default: 50]
  --format <FMT>     Output format: table, json, csv [default: table]
```

### `acomm history`

Query communication history.

```
Usage: acomm history [OPTIONS] [STORE_PATH]

Options:
  --channel <NAME>   Filter by channel
  --sender <ID>      Filter by sender
  --type <TYPE>      Filter by message type
  --after <TIME>     After timestamp
  --before <TIME>    Before timestamp
  --status <STATUS>  Filter by status
  --limit <N>        Maximum results [default: 100]
  --offset <N>       Pagination offset [default: 0]
  --format <FMT>     Output format: table, json, csv [default: table]
  --include-archived Include archived messages
```

### `acomm thread`

View all messages in a thread.

```
Usage: acomm thread <CORRELATION_ID> [STORE_PATH]
```

### `acomm stats`

Show store statistics.

```
Usage: acomm stats [STORE_PATH]
```

### `acomm compact`

Remove old messages and reclaim disk space.

```
Usage: acomm compact [OPTIONS] [STORE_PATH]

Options:
  --older-than <DURATION>  Remove items older than this (e.g., "7d", "30d", "1y")
  --dry-run                Show what would be removed without removing
```

### `acomm channel` Subcommands

```
Usage: acomm channel <SUBCOMMAND> [STORE_PATH]

Subcommands:
  create   Create a new channel
  list     List all channels
  info     Show channel details
  join     Add a participant
  leave    Remove a participant
  config   Update channel configuration
  pause    Pause message delivery on a channel
  resume   Resume a paused channel
  drain    Stop accepting new messages, deliver pending
  close    Close a channel
  delete   Delete a channel and its messages
```

### `acomm channel create`

```
Usage: acomm channel create [OPTIONS] [STORE_PATH]

Options:
  --name <NAME>     Channel name [required]
  --type <TYPE>     Channel type: direct, group, broadcast, pubsub [required]
  --owner <ID>      Owner participant ID [required]
  --description <TEXT>  Channel description
  --tags <TAGS>     Comma-separated tags
```

### `acomm subscribe`

```
Usage: acomm subscribe [OPTIONS] [STORE_PATH]

Options:
  --channel <NAME>      PubSub channel name [required]
  --subscriber <ID>     Subscriber participant ID [required]
  --topic <PATTERN>     Topic pattern (supports *, #) [required]
```

### `acomm export`

```
Usage: acomm export [OPTIONS] [STORE_PATH]

Options:
  --output <PATH>    Output file (default: stdout)
  --channel <NAME>   Export only this channel
  --format <FMT>     Export format: json, csv [default: json]
```

### `acomm import`

```
Usage: acomm import [OPTIONS] <INPUT_PATH> [STORE_PATH]

Options:
  --format <FMT>     Input format: json, csv [default: json]
  --channel <NAME>   Import into this channel (create if needed)
```

### `acomm completions`

Generate shell completion scripts.

```
Usage: acomm completions <SHELL>

Arguments:
  <SHELL>  Shell: bash, zsh, fish, powershell
```

Install completions:

```bash
# Bash
acomm completions bash > ~/.local/share/bash-completion/completions/acomm

# Zsh
acomm completions zsh > ~/.zfunc/_acomm

# Fish
acomm completions fish > ~/.config/fish/completions/acomm.fish
```

## `agentic-comm-mcp` Commands

### `agentic-comm-mcp serve`

Start the MCP server.

```
Usage: agentic-comm-mcp serve [OPTIONS]

Options:
  --store <PATH>        Store file path [default: $ACOMM_STORE]
  --log-level <LEVEL>   Log level [default: warn]
  --rest-port <PORT>    Enable REST bridge on this port
  --webhook-port <PORT> Enable webhook receiver on this port
  --mode <MODE>         Session mode: full, minimal [default: full]
```

### `agentic-comm-mcp info`

Print store information.

```
Usage: agentic-comm-mcp info [STORE_PATH]
```

### `agentic-comm-mcp validate`

Validate a `.acomm` file for integrity errors.

```
Usage: agentic-comm-mcp validate <STORE_PATH>
```

### `agentic-comm-mcp export`

Export store data.

```
Usage: agentic-comm-mcp export [OPTIONS] [STORE_PATH]
```

### `agentic-comm-mcp compact`

Compact a store file.

```
Usage: agentic-comm-mcp compact [OPTIONS] [STORE_PATH]
```

### `agentic-comm-mcp stats`

Print store statistics.

```
Usage: agentic-comm-mcp stats [STORE_PATH]
```

## Upgrade Instructions

### Minor version upgrade (0.1.x -> 0.1.y)

```bash
cargo install agentic-comm-cli --force
cargo install agentic-comm-mcp --force
```

Minor versions are always backward compatible. No store migration is required.

### Major version upgrade (0.x -> 1.0)

Major versions may change the `.acomm` file format. When upgrading:

1. Export your data:
   ```bash
   acomm export --output backup.json project.acomm
   ```

2. Install the new version:
   ```bash
   cargo install agentic-comm-cli --force
   ```

3. Create a new store and import:
   ```bash
   acomm init project-v2.acomm
   acomm import backup.json project-v2.acomm
   ```

The new version will attempt to open old-format files automatically. If migration is possible, it will be performed in place with a backup created at `<path>.acomm.bak`.

## Verifying Installation

Run the built-in health check:

```bash
acomm init /tmp/test.acomm
acomm channel create --name test --type group --owner test-agent /tmp/test.acomm
acomm send --channel test --sender test-agent "Hello, AgenticComm" /tmp/test.acomm
acomm stats /tmp/test.acomm
rm /tmp/test.acomm
```

Expected output for `acomm stats`:
```
Store: /tmp/test.acomm
Channels:      1
Messages:      1
Subscriptions: 0
Dead Letters:  0
```

## npm (WASM) package

```bash
npm install @agenticamem/comm
```

## One-Line Installer



## One-Line Installer

```bash
curl -fsSL https://agentralabs.tech/install/comm | bash
```
