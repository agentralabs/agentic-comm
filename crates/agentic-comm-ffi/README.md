# agentic-comm-ffi

C FFI bindings for AgenticComm — use from Python, Node.js, Ruby, or any language with C FFI support.

## Build

```bash
cargo build --release -p agentic-comm-ffi
```

Produces `libagentic_comm_ffi.{so,dylib,dll}` in `target/release/`.

## Functions

| Function | Description |
|----------|-------------|
| `acomm_store_create` | Create a new comm store |
| `acomm_store_free` | Free a store handle |
| `acomm_channel_create` | Create a channel |
| `acomm_send` | Send a message |
| `acomm_receive` | Receive messages (JSON) |
| `acomm_search` | Search messages (JSON) |
| `acomm_last_error` | Get last error message |
| `acomm_free_string` | Free a returned string |

## Python Example

```python
import ctypes

lib = ctypes.CDLL("target/release/libagentic_comm_ffi.dylib")
# Use functions via ctypes
```

## License

MIT — see [LICENSE](LICENSE)
