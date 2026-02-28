# Contributing to AgenticComm

Thank you for your interest in contributing to AgenticComm! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/agentic-comm.git`
3. Create a feature branch: `git checkout -b my-feature`
4. Make your changes
5. Run the tests (see below)
6. Commit and push
7. Open a pull request

## Development Setup

This is a Cargo workspace monorepo. All Rust crates are under `crates/`.

### Rust Workspace

```bash
# Build everything (core + CLI + MCP + FFI)
cargo build

# Run all tests
cargo test --workspace

# Core library only
cargo test -p agentic-comm
cargo bench -p agentic-comm

# CLI tool only
cargo test -p agentic-comm-cli

# MCP server only
cargo test -p agentic-comm-mcp

# FFI bindings only
cargo test -p agentic-comm-ffi

# Run the CLI
cargo run -p agentic-comm-cli -- channel list

# Run the MCP server
cargo run -p agentic-comm-mcp
```

### Python SDK

```bash
cd python/
python3 -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
pytest tests/ -v
```

## Ways to Contribute

### Report Bugs

File an issue with:
- Steps to reproduce
- Expected behavior
- Actual behavior
- System info (OS, Python version, Rust version)

### Add an MCP Tool

1. Add tool definition in `crates/agentic-comm-mcp/src/tools.rs`
2. Add dispatch case in `dispatch()`
3. Implement handler function
4. Add tests
5. Update README

### Write Examples

1. Add a new example in `examples/`
2. Ensure it runs without errors
3. Add a docstring explaining what it demonstrates

### Improve Documentation

All docs are in `docs/`. Fix typos, add examples, clarify explanations.

## Code Guidelines

- **Rust**: Follow standard Rust conventions. Run `cargo clippy` and `cargo fmt`. All code must be clippy-clean with `-D warnings`.
- **Python**: Follow PEP 8. Use type hints. Run `mypy` for type checking.
- **Tests**: Every feature needs tests. We maintain comprehensive tests across the stack.
- **Documentation**: Update docs when changing public APIs.

## MCP Quality Standard

All MCP tools must comply with the Agentra MCP Quality Standard:

- **Tool descriptions**: verb-first imperative, no trailing periods
- **Error handling**: tool execution errors use `isError: true`; protocol errors use JSON-RPC error codes
- **Unknown tool**: error code `-32803` (TOOL_NOT_FOUND)
- **Input validation**: no silent fallback for invalid parameters

## Commit Messages

Use conventional commit prefixes:
- `feat:` new feature or capability
- `fix:` bug fix
- `chore:` maintenance, dependencies, config
- `docs:` documentation changes
- `test:` test additions or fixes
- `refactor:` code restructuring without behavior change

## Pull Request Guidelines

- Keep PRs focused
- Include tests for new functionality
- Update documentation if needed
- Ensure all tests pass before submitting
- Ensure `cargo clippy` and `cargo fmt` pass
- Write a clear PR description

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
