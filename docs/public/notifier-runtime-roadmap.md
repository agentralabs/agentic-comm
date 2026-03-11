---
status: draft
---

# Notifier-First Runtime Roadmap

## Goal

Define a generic, upstream-safe path for real-time agent message awareness that does not depend on fragile turn hooks or project-specific wrappers.

This roadmap focuses on reusable primitives that can be implemented in `agentic-comm` and consumed by any runtime.

## Problem Statement

Current coordination patterns often rely on one of two brittle approaches:

1. Poll-only loops that require frequent manual turns.
2. Hook-triggered prompt injection with weak lifecycle guarantees.

Both patterns can fail across restarts, process churn, and multi-runtime environments.

## Target Architecture (Generic)

Layered model:

1. Store + daemon: durable message persistence and low-latency relay.
2. Notifier: transforms incoming deltas into normalized awareness events.
3. Runtime adapter: consumes events and triggers agent-visible context updates.
4. Session workflow: clear start/stop/recover commands with health checks.

The key contract is an explicit event envelope between notifier and runtime adapter, not implicit behavior.

## Proposed Upstream Scope

### Phase 1: Stable Read/Delta Primitives

- Add first-class id-based delta reads (`after_id`) for channel polling and streaming.
- Keep timestamp-based reads, but document id-based reads as preferred for restart safety.
- Guarantee deterministic ordering in CLI output (`id` ascending by default).

Deliverables:
- CLI options and docs updates.
- Regression tests for restart and cursor handoff behavior.

### Phase 2: Generic Notifier Event Envelope

- Define a neutral text+json event envelope for runtime adapters.
- Include canonical fields:
  - `message_id`
  - `channel_id`
  - `channel_name`
  - `sender`
  - `timestamp`
  - `content`
  - `read_cmd`
  - `reply_cmd`
  - `fetch_cmd`
  - `execution_hint`

Deliverables:
- Public envelope spec in docs.
- Example producer/consumer flow.

### Phase 3: First-Party Notifier CLI Surface

- Add generic notifier commands (illustrative):
  - `acomm notifier run`
  - `acomm notifier status`
  - `acomm notifier stop`
- Support output targets:
  - stdout (jsonl)
  - file queue (append-only)

Deliverables:
- CLI implementation with restart-safe cursors.
- Integration tests for queue output and dedupe.

### Phase 4: Runtime Lifecycle Contract

- Document a standard lifecycle for stream/runtime coordination:
  - `start`
  - `wake/recover`
  - `stop`
  - `health`
- Provide generic runbook-level guidance that any runtime can follow.

Deliverables:
- Public runbook patterns.
- CLI examples that avoid project/person-specific naming.

## Non-Goals

- No runtime-specific terminal injection implementation in core.
- No project-specific launcher conventions.
- No identity/personality-specific assumptions in examples.

## Acceptance Criteria

1. Realtime awareness survives process restarts without duplicate floods or missed messages.
2. Id-based cursor handoff is deterministic across notifier restarts.
3. The same notifier event stream can be consumed by multiple runtimes without schema changes.
4. Public docs and examples remain identity-neutral and environment-agnostic.

## Compatibility

- Existing `send/receive/chat` flows remain valid.
- New notifier features layer on top of current message and daemon model.
- Migration path is additive: users can adopt phases incrementally.

