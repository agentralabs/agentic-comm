# agentic-comm

Core library for AgenticComm — portable binary communication for AI agents.

Channel-based messaging with the `.acomm` binary format. Zero dependencies beyond std.

## Usage

```rust
use agentic_comm::CommStore;

let store = CommStore::new();
// Create channels, send messages, manage subscriptions
```

## Features

- Channel-based message routing
- Monotonic message ordering
- Binary `.acomm` format with LZ4 compression
- Subscription management
- Full-text search across messages

## License

MIT — see [LICENSE](LICENSE)
