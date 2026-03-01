# agentic-comm-cli

Command-line interface for AgenticComm — the `acomm` binary.

## Install

```bash
cargo install agentic-comm-cli
```

## Usage

```bash
acomm create my-store.acomm
acomm channel my-store.acomm "tasks"
acomm send my-store.acomm "tasks" "Build the frontend"
acomm receive my-store.acomm "tasks"
acomm search my-store.acomm "frontend"
acomm info my-store.acomm
```

All commands support `--json` for programmatic output.

## License

MIT — see [LICENSE](LICENSE)
