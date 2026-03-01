# AgenticComm Python SDK

Python SDK for AgenticComm -- portable binary communication for AI agents. Channel-based messaging, zero dependencies.

## Install

```bash
pip install agentic-comm
```

## Quick Start

```python
from agentic_comm import CommStore

store = CommStore("my_agents.acomm")
print(store.info())
```

## Core Operations

```python
from agentic_comm import CommStore, Channel, Message

store = CommStore("my_agents.acomm")

# Channel management
store.create_channel("task-queue", description="Work items")
channels = store.list_channels()

# Send messages
store.send("task-queue", "Build the login page")
store.send("task-queue", "Review PR #42")

# Receive messages
messages = store.receive("task-queue")
for msg in messages:
    print(f"[{msg.timestamp}] {msg.content}")

# Search
results = store.search("login")

# Broadcast
store.broadcast("System update: v1.2.0 deployed")
```

## Subscriptions

```python
# Subscribe an agent to a channel
store.subscribe("task-queue", "worker-agent-1")

# Check subscriptions
subs = store.subscriptions("task-queue")

# Poll all subscribed channels
new_messages = store.poll("worker-agent-1")
```

## Test Coverage

Tests across import validation, model verification, and CLI bridge integration.

## Requirements

- Python >= 3.10
- `acomm` binary (Rust core engine) -- install via `cargo install agentic-comm`

## Documentation

- [API Reference](../docs/public/api-reference.md)
- [Integration Guide](../docs/public/integration-guide.md)
- [Benchmarks](../docs/public/benchmarks.md)
- [Full README](../README.md)

## License

MIT
