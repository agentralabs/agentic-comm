---
status: stable
---

# AI Agent Integration

Get your AI agent working with AgenticComm in 30 seconds.

---

## 30-Second Start

**MCP (Claude Desktop, Cursor, Windsurf):**

```bash
curl -fsSL https://raw.githubusercontent.com/agentralabs/agentic-comm/main/scripts/install.sh | bash
```

Restart your client. Done. Your agent now has inter-agent communication.

**Python:**

```bash
pip install agentic-comm
```

```python
from agentic_comm import CommStore
store = CommStore("agent.acomm")
info = store.info()
print(info)
```

---

## System Prompt Templates

Add one of these to your agent's system prompt.

### Minimal

```
You have inter-agent communication via AgenticComm.
- Use comm_send to send messages to channels
- Use comm_receive to check for incoming messages
```

### Standard (Recommended)

```
You have inter-agent communication via AgenticComm MCP server.

SENDING MESSAGES:
- comm_send channel="task-queue" content="..." → Send a message
- comm_broadcast content="..." → Send to all channels

RECEIVING MESSAGES:
- comm_receive channel="task-queue" → Get pending messages
- comm_peek channel="task-queue" → Preview without consuming
- comm_poll → Check all subscribed channels

CHANNEL MANAGEMENT:
- comm_channel_create name="..." → Create a new channel
- comm_channel_list → List available channels
- comm_subscribe channel="..." agent="..." → Subscribe to updates

RULES:
1. Check for messages at conversation start
2. Send status updates when completing tasks
3. Use channels to coordinate with other agents
4. Reference messages naturally, don't announce "checking messages"
```

### Full (Production Agents)

```
You have inter-agent communication via AgenticComm MCP server.

CHANNEL TYPES:
- task-queue: Work items for other agents
- status: Progress and completion updates
- review: Code review and feedback
- alerts: System-level notifications
- general: Open discussion

WHEN TO SEND:
- Task completed → status channel with results
- Need another agent → task-queue with request
- Found an issue → alerts channel
- Want feedback → review channel

WHEN TO RECEIVE:
- Start of conversation → poll all subscribed channels
- Before starting work → check task-queue for assignments
- After sending work → check review for feedback

MESSAGE HYGIENE:
- Include context: what, why, who needs to know
- Use structured data when possible
- Don't flood channels with trivial updates

BEHAVIOR:
- Check channels silently at conversation start
- Coordinate naturally without announcing "checking messages"
- Send completion notifications when finishing tasks
```

---

## Common Patterns

### Task Delegation

```python
# Agent A sends a task
store.send("task-queue", "Build the login page with React")

# Agent B picks up the task
messages = store.receive("task-queue")
for msg in messages:
    print(f"New task: {msg.content}")
```

### Status Broadcasting

```python
# Any agent broadcasts status
store.broadcast("Deployment complete: v1.2.0 live on staging")
```

### Code Review Handoff

```python
# Developer agent requests review
store.send("review", "PR #42 ready for review: auth refactor")

# Reviewer agent picks it up
reviews = store.receive("review")
```

### Search and History

```python
# Find relevant past messages
results = store.search("deployment")

# Get channel history
history = store.history("task-queue", limit=20)
```

---

## Framework Integration

### LangChain

```python
from agentic_comm import CommStore

class AgenticCommTool:
    def __init__(self, path: str):
        self.store = CommStore(path)

    def send(self, channel: str, content: str) -> str:
        self.store.send(channel, content)
        return f"Message sent to {channel}"

    def receive(self, channel: str) -> str:
        messages = self.store.receive(channel)
        return "\n".join([m.content for m in messages])
```

### CrewAI

```python
from agentic_comm import CommStore

crew_comm = CommStore("crew_shared.acomm")

def send_task(content: str, agent: str) -> None:
    crew_comm.send(f"task-{agent}", content)

def check_tasks(agent: str) -> list:
    return crew_comm.receive(f"task-{agent}")
```

---

## MCP Tool Reference

| Tool | Purpose | Key Parameters |
|------|---------|----------------|
| `comm_send` | Send message | `channel`, `content` |
| `comm_receive` | Get messages | `channel`, `limit` |
| `comm_peek` | Preview messages | `channel`, `limit` |
| `comm_channel_create` | Create channel | `name`, `description` |
| `comm_channel_list` | List channels | |
| `comm_subscribe` | Subscribe agent | `channel`, `agent_id` |
| `comm_search` | Search messages | `query`, `limit` |
| `comm_broadcast` | Send to all | `content` |
| `comm_poll` | Check all subs | |
| `comm_health` | Check status | |

---

## Troubleshooting

**Messages not persisting?**
- Ensure `.acomm` file path is correct
- Check file permissions
- MCP: Restart client after install

**Channel not found?**
- Create the channel first with `comm_channel_create`
- Check spelling with `comm_channel_list`

---

## Next Steps

- [Communication Concepts](/docs/en/comm-concepts)
- [API Reference](/docs/en/comm-api-reference)
- [Benchmarks](/docs/en/comm-benchmarks)
