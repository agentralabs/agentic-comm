---
status: stable
---

# Multi-Agent Coordination with AgenticComm

Practical patterns for coordinating multiple AI agents through a shared AgenticComm store. All commands and behaviors here were verified in a live multi-session setup.

---

## Channel Semantics

AgenticComm supports two distinct communication patterns. Choose the right one for your coordination need.

### Direct / Group channels — shared stream

`acomm message send` writes one message to the channel stream. All participants read from the same stream; no per-recipient routing occurs.

```bash
# Send a message to a group channel
acomm message send 1 "Build complete — ready for review" --sender ci-agent --file agent.acomm --json
# → { "channel_id": 1, "message_id": 42, "status": "sent" }

# Any participant can read it
acomm message list 1 --file agent.acomm --json
# → [ { "id": 42, "sender": "ci-agent", "recipient": null, "content": "Build complete..." } ]
```

`recipient` is `null` — the message belongs to the channel, not to a specific reader.

### Pub/sub — fan-out with per-subscriber delivery records

`subscribe` + `publish` creates one delivery record per registered subscriber. Each subscriber reads only their own entry.

```bash
# Register subscribers
acomm subscribe updates meera --file agent.acomm
acomm subscribe updates ishika --file agent.acomm

# Publish — fans out to all subscribers
acomm publish updates "sprint-started" --sender orchestrator --file agent.acomm --json
# → { "topic": "updates", "delivered_count": 2, "status": "published" }

# Each subscriber reads only their own delivery
acomm receive 1 --recipient meera --file agent.acomm --json
# → [ { "recipient": "meera", "content": "sprint-started", ... } ]

acomm receive 1 --recipient ishika --file agent.acomm --json
# → [ { "recipient": "ishika", "content": "sprint-started", ... } ]
```

`delivered_count: 2` confirms both subscribers received the message. Each agent's `receive --recipient` call returns only their own entry — no cross-delivery.

---

## Cross-Session Delivery

Messages written by one agent session are immediately readable by another session on the same store. This was confirmed in live testing:

| Message | Sender | Session | Lamport | Timestamp |
|---------|--------|---------|---------|-----------|
| `real-ishika-probe:...` | ishika | Session A | 40 | 10:28:00 |
| `real-meera-ack:...` | meera | Session B | 41 | 10:30:15 |

Sessions A and B are independent Claude Code processes with a ~2-minute gap between sends. Both messages persist in the shared store and are readable by either session.

**What "cross-session delivery" means here:** the `.acomm` store is a local file. Any session with a path to that file can read and write it. There is no network hop; delivery latency is local I/O.

---

## Awareness Model

Cross-session delivery (store level) works, but agents becoming *aware* of new messages requires an additional step. There are three distinct layers:

### Layer 1 — Store delivery ✓

`acomm message send` → message persisted in `.acomm`. Verifiable with `message list`. Works automatically.

### Layer 2 — Human-terminal awareness (optional watcher)

`acomm-notify.ps1` (or equivalent) is a polling loop that watches configured channels and prints new messages to its terminal window via `Write-Host`. The human watching that window sees near-real-time alerts.

**Important:** the watcher must be:
1. Explicitly started (it does not auto-launch)
2. Configured to watch the channels where messages are expected

```powershell
# Start watcher for specific channels
.\acomm-notify.ps1 -Channels 'trine-handoff','trine-gate' -IntervalSeconds 2
```

#### Watcher lifecycle wrappers (always-on operation)

In Triveni-style operational setups, use lifecycle wrappers so the watcher can run continuously instead of being manually re-launched:

```powershell
# Start background notifier process (writes pid metadata)
.\start-acomm-notifier.ps1 -Channels 'trine-handoff','trine-gate','trine-debate' -IntervalSeconds 2

# Check running/stale status
.\status-acomm-notifier.ps1 -Json

# Stop notifier process and clear pid metadata
.\stop-acomm-notifier.ps1
```

These wrappers maintain a daemon metadata file (pid + channels + interval) and can redirect watcher output/error to logs for operator inspection.

### Layer 3 — Agent-session awareness (explicit polling)

A running agent session does not receive `Write-Host` output from a separate watcher process. The agent's conversation thread is unaware of new messages unless it explicitly polls.

**Observed behavior (live test, 2026-03-04):** When session A sent a message, session B's terminal showed no alert — session B remained oblivious until it explicitly called `message list`.

The reliable agent-side pattern is disciplined polling at the start of each interaction turn:

```bash
# At the start of each turn, check for new messages since last read
acomm message list 1 --file agent.acomm --json
```

**Near-term path (achievable today):** Build an acomm Claude Code plugin using the same `UserPromptSubmit` hook injection pattern that tools like Mira and MemSearch already use. The plugin polls the acomm store at every turn boundary and injects new messages as system context automatically — eliminating the need for explicit agent polling. This requires building the plugin, not waiting for any MCP protocol changes. MCP `resources/subscribe` + `notifications/resources/updated` would enable true mid-turn push (no turn-boundary dependency) but that notification path is not yet implemented in Claude Code's MCP client.

---

## Poll-Before-Respond Discipline

When multiple agents share a channel, they poll independently and cannot see each other's in-flight responses before their own turn completes. This creates a parallel-reply blindspot.

**The problem:** Agent A and Agent B both see a question at lamport T. Both compose answers independently. Both post. The human receives two parallel responses that didn't account for each other.

**The discipline:** Before posting a response in a shared channel, poll for new messages and check whether a peer has already responded since your last read.

```bash
# Before sending, check the channel's current state
LAST=$(acomm message list 1 --file agent.acomm --json | python3 -c \
  "import sys,json; msgs=json.load(sys.stdin); print(msgs[-1]['id'] if msgs else 0)")

# Only post if no peer has answered since your last check
if [ "$LAST" -le "$YOUR_LAST_KNOWN_ID" ]; then
  acomm message send 1 "My response..." --sender agent-a --file agent.acomm
fi
```

The `delivered_count` from `publish` provides a similar gate for pub/sub workflows — if `delivered_count` is 0, no subscribers are registered and the message would be undelivered.

---

## Watcher Channel Scope

The watcher only surfaces messages from channels in its watch list. If a channel is created dynamically (outside the initial setup), it must be explicitly added to the watcher's `-Channels` argument.

```powershell
# Watcher that covers both standard and dynamic channels
.\acomm-notify.ps1 -Channels 'trine-handoff','trine-gate','trine-debate','my-new-channel'
```

Channel name resolution depends on the channel being registered in the store. If a channel was created after the watcher started, restart the watcher to pick up the updated channel map.

---

## Runnable Example

See `examples/pubsub-fanout-recipient-delivery.sh` for a self-contained, verifiable demonstration of the pub/sub fan-out and recipient-scoped delivery pattern:

```bash
bash examples/pubsub-fanout-recipient-delivery.sh
# PASS: publish fan-out delivered to two subscribers with recipient-scoped receive output.
```

---

## Summary

| Pattern | Use when | Recipient routing |
|---------|----------|-------------------|
| `message send` / `list` | All participants need the same message | `recipient: null` (channel stream) |
| `subscribe` + `publish` + `receive --recipient` | Each agent needs their own delivery record | `recipient: some(name)` per subscriber |
| Poll-before-respond | Preventing parallel duplicate replies | — discipline, not a command |
| Watcher loop | Human-terminal alerting | — optional, not agent-native |
