# Installation Guide

## Quick Install (one-liner)

```bash
curl -fsSL https://agentralabs.tech/install/comm | bash
```

Downloads a pre-built `agentic-comm-mcp` binary, installs to `~/.local/bin/`, and merges the MCP server config into Claude Desktop and Claude Code. Store defaults to `~/.comms.acomm`. Requires `curl` and `jq`.

### Install by environment

```bash
# Desktop MCP clients (auto-merge Claude configs)
curl -fsSL https://agentralabs.tech/install/comm/desktop | bash

# Terminal-only (no desktop config writes)
curl -fsSL https://agentralabs.tech/install/comm/terminal | bash

# Remote/server host (no desktop config writes)
curl -fsSL https://agentralabs.tech/install/comm/server | bash
```

### Server auth and artifact sync

Cloud/server runtime cannot read files from your laptop directly.

```bash
export AGENTIC_TOKEN="$(openssl rand -hex 32)"
```

All MCP clients must send `Authorization: Bearer <same-token>`.
If `.acomm` artifacts were created elsewhere, sync them to the server first.

---

## 1. Python SDK (recommended for most users)

The Python SDK gives you the `CommStore` class and message handling. Requires **Python 3.10+**.

```bash
pip install agentic-comm
```

### Verify

```python
from agentic_comm import CommStore

store = CommStore("test.acomm")
print(store.info())
```

> **Note:** The Python SDK requires the `acomm` binary (Rust core engine). Install it via Step 2 below, or build from source via Step 3.

---

## 2. Rust CLI

The `acomm` binary is the core engine. Use it standalone or as the backend for the Python SDK. Requires **Rust 1.70+**.

```bash
cargo install agentic-comm
```

This installs the `acomm` command-line tool.

### Verify

```bash
acomm --help
acomm create test.acomm
acomm channel test.acomm "general"
acomm send test.acomm "general" "Hello world"
acomm info test.acomm
```

### Available commands

| Command | Description |
|:---|:---|
| `acomm create` | Create a new empty `.acomm` file |
| `acomm channel` | Create a communication channel |
| `acomm send` | Send a message to a channel |
| `acomm receive` | Receive messages from a channel |
| `acomm peek` | Preview messages without consuming |
| `acomm subscribe` | Subscribe an agent to a channel |
| `acomm unsubscribe` | Remove a subscription |
| `acomm search` | Search messages by content |
| `acomm history` | View channel message history |
| `acomm broadcast` | Send to all channels |
| `acomm poll` | Poll for new messages |
| `acomm info` | Display store information |
| `acomm health` | Check store health status |
| `acomm export` | Export store as JSON |
| `acomm import` | Import from JSON |

All commands support `--json` output for programmatic consumption.

---

## 3. MCP Server (for Claude Desktop, VS Code, Cursor, Windsurf)

The MCP server exposes a comm store as 17 tools, resources, and prompts to any MCP-compatible LLM client.

```bash
cargo install agentic-comm-mcp
```

### Configure Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "agentic-comm": {
      "command": "agentic-comm-mcp",
      "args": ["serve"]
    }
  }
}
```

> Zero-config: defaults to `~/.comms.acomm`. Override with `"args": ["--store", "/path/to/store.acomm", "serve"]`.

### Configure VS Code / Cursor

Add to `.vscode/settings.json`:

```json
{
  "mcp.servers": {
    "agentic-comm": {
      "command": "agentic-comm-mcp",
      "args": ["serve"]
    }
  }
}
```

### Verify

Once connected, the LLM gains access to tools like `comm_send`, `comm_receive`, `comm_channel_create`, and more. Test by asking the LLM:

> "Create a channel called 'tasks' and send a message."

The LLM should call `comm_channel_create` and `comm_send` and confirm.

See the [MCP server README](crates/agentic-comm-mcp/README.md) for the full tool/resource/prompt reference.

---

## Build from Source

```bash
git clone https://github.com/agentralabs/agentic-comm.git
cd agentic-comm

# Build entire workspace (core library + MCP server)
cargo build --release

# Install core CLI
cargo install --path crates/agentic-comm

# Install MCP server
cargo install --path crates/agentic-comm-mcp

# Install Python SDK (development mode)
cd python
pip install -e ".[dev]"
```

### Run tests

```bash
# All workspace tests
cargo test --workspace

# Core library only
cargo test -p agentic-comm

# MCP server only
cargo test -p agentic-comm-mcp

# Python SDK tests
cd python && pytest tests/ -v
```

---

## Package Registry Links

| Package | Registry | Install |
|:---|:---|:---|
| **agentic-comm** | [crates.io](https://crates.io/crates/agentic-comm) | `cargo install agentic-comm` |
| **agentic-comm-mcp** | [crates.io](https://crates.io/crates/agentic-comm-mcp) | `cargo install agentic-comm-mcp` |
| **agentic-comm** | [PyPI](https://pypi.org/project/agentic-comm/) | `pip install agentic-comm` |

---

## Requirements

| Component | Minimum version |
|:---|:---|
| Python | 3.10+ |
| Rust | 1.70+ (only for building from source or `cargo install`) |
| OS | macOS, Linux, Windows |

---

## Troubleshooting

### `pip: command not found`

Use `pip3` instead, or the full path to your Python:

```bash
python3 -m pip install agentic-comm
```

### `acomm: command not found` after `cargo install`

Make sure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add this line to your `~/.zshrc` or `~/.bashrc` to make it permanent.

### Python SDK says "acomm binary not found"

Install the Rust core engine first:

```bash
cargo install agentic-comm
```

Or build from source:

```bash
git clone https://github.com/agentralabs/agentic-comm.git
cd agentic-comm
cargo build --release
cp target/release/acomm /usr/local/bin/
```
