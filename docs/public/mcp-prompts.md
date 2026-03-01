---
status: stable
---

# MCP Prompts Reference

MCP prompts are pre-built interaction templates that guide AI assistants through common communication workflows. Unlike tools (which perform actions) and resources (which expose data), prompts structure the assistant's reasoning by providing context, instructions, and expected output formats for multi-step communication tasks.

## Server Capabilities

The AgenticComm MCP server advertises prompt support:

```json
{
  "capabilities": {
    "prompts": {
      "listChanged": true
    }
  }
}
```

The `listChanged: true` capability means the server can notify clients when the list of available prompts changes (e.g., when a new prompt is added via a server update).

## Available Prompts

### comm-status

**Status:** Available (v0.1.0)

Summarize the current communication state across all channels, subscriptions, and recent activity.

**Name:** `comm-status`

**Description:** Generate a comprehensive summary of the communication system's current state, including active channels, recent messages, subscription topology, and any operational issues.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `include_messages` | boolean | no | `true` | Whether to include recent message summaries |
| `message_limit` | integer | no | `10` | Number of recent messages per channel to include |
| `include_subscriptions` | boolean | no | `true` | Whether to include subscription information |

**Prompt Template:**

```
You are analyzing the communication state of an AgenticComm system.

Store statistics:
{store_stats}

Active channels:
{channel_list}

{if include_messages}
Recent messages per channel (last {message_limit}):
{recent_messages}
{/if}

{if include_subscriptions}
Active subscriptions:
{subscriptions}
{/if}

Please provide:
1. A summary of the communication topology (who is talking to whom, through which channels)
2. Activity levels per channel (active, idle, stale)
3. Any potential issues:
   - Channels with no participants
   - Channels with no recent messages (possible abandoned channels)
   - Unacknowledged command messages (possible dropped tasks)
   - Dead letter queue status
4. Recommendations for communication hygiene (cleanup suggestions)
```

**Expected Output:** A structured analysis of the communication system state with actionable recommendations.

**Example Use Case:** At the start of a new session, an AI assistant calls `comm-status` to understand what communication infrastructure exists, which agents are active, and whether there are any pending issues from previous sessions.

---

### channel-health

**Status:** Available (v0.1.0)

Analyze the health of a specific communication channel.

**Name:** `channel-health`

**Description:** Perform a health assessment of a communication channel, including delivery rates, acknowledgment patterns, participant activity, and potential issues.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | ID of the channel to analyze |
| `time_window` | string | no | `"24h"` | Time window for analysis (e.g., `"1h"`, `"24h"`, `"7d"`) |

**Prompt Template:**

```
You are performing a health check on a communication channel.

Channel information:
{channel_info}

Message history for the last {time_window}:
{message_history}

Participant activity:
{participant_activity}

Dead letters (if any):
{dead_letters}

Please analyze:
1. **Delivery health:** What percentage of messages were delivered? Are there
   delivery failures?
2. **Acknowledgment rate:** For command and query messages, what percentage
   received timely acknowledgments?
3. **Participant balance:** Are all participants active, or are some silent?
   Is communication balanced or dominated by one sender?
4. **Message type distribution:** What types of messages flow through this
   channel? Is the distribution healthy for the channel type?
5. **Latency patterns:** Are there gaps in communication? Long periods of
   silence followed by bursts?
6. **Configuration review:** Is the channel configuration appropriate?
   - Is the max_participants limit reasonable?
   - Is the TTL appropriate for the message types?
   - Should encryption be enabled?
7. **Recommendations:** Specific actions to improve channel health.
```

**Expected Output:** A channel health report with metrics, identified issues, and improvement recommendations.

**Example Use Case:** A team lead agent periodically runs `channel-health` on coordination channels to ensure communication patterns are healthy and identify agents that may be overwhelmed or unresponsive.

---

### thread-summary

**Status:** Available (v0.1.0)

Summarize a message thread by following the correlation chain.

**Name:** `thread-summary`

**Description:** Reconstruct and summarize a complete message thread, tracing the conversation from initial query or command through all responses, acknowledgments, and follow-ups.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `message_id` | integer | yes | -- | ID of any message in the thread |
| `include_context` | boolean | no | `true` | Whether to include communication_log context entries |

**Prompt Template:**

```
You are summarizing a message thread from the communication history.

Starting message:
{initial_message}

Full thread (all related messages):
{thread_messages}

{if include_context}
Communication context (intent and observations logged during the thread):
{context_entries}
{/if}

Please provide:
1. **Thread overview:** What was the purpose of this conversation?
2. **Participants:** Who was involved and what role did each play?
3. **Timeline:** Key events in chronological order with timestamps.
4. **Outcome:** Was the thread resolved? If it was a command, was it
   acknowledged and completed? If it was a query, was it answered?
5. **Open items:** Any unresolved questions, unacknowledged commands,
   or pending follow-ups.
6. **Key decisions:** Any decisions made during the thread.
```

**Expected Output:** A structured summary of the conversation thread with timeline, participants, outcomes, and open items.

**Example Use Case:** An agent joining a conversation mid-thread calls `thread-summary` to quickly understand the context, decisions made so far, and what actions are still pending.

---

### comm-topology

**Status:** Available (v0.1.0)

Map the communication topology of the agent network.

**Name:** `comm-topology`

**Description:** Analyze the communication graph to understand how agents are connected, which channels bridge different groups, and where communication bottlenecks exist.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `time_window` | string | no | `"7d"` | Time window for activity analysis |

**Prompt Template:**

```
You are mapping the communication topology of an agent network.

All channels:
{channels}

All subscriptions:
{subscriptions}

Message flow data (last {time_window}):
{message_flow}

Please produce:
1. **Network map:** Which agents communicate with which other agents?
   Identify clusters (groups of agents that primarily communicate among
   themselves).
2. **Hub agents:** Which agents participate in the most channels? These
   are communication hubs -- if they fail, communication breaks down.
3. **Bridge channels:** Which channels connect otherwise separate groups?
   These are critical for cross-team coordination.
4. **Communication patterns:**
   - One-to-one pairs (frequent direct communication)
   - Broadcast patterns (one agent sending to many)
   - Pub/sub patterns (event-driven communication)
5. **Bottlenecks:** Agents or channels that handle disproportionate
   traffic. These may need load balancing or delegation.
6. **Isolation risks:** Agents that participate in very few channels.
   These may miss important communications.
7. **Redundancy:** Are there backup communication paths if a primary
   channel fails?
```

**Expected Output:** A communication topology analysis identifying hubs, bridges, bottlenecks, and risks.

**Example Use Case:** A coordination agent runs `comm-topology` to understand the shape of the agent network before planning a multi-agent task that requires reliable communication across teams.

---

### comm-debug

**Status:** Available (v0.1.0)

Diagnose communication failures and delivery issues.

**Name:** `comm-debug`

**Description:** Investigate why messages are not being delivered, why acknowledgments are missing, or why a communication pattern is failing. Combines store statistics, dead letter analysis, and operation log inspection.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `symptom` | string | yes | -- | Description of the problem (e.g., "messages to channel 3 are not being received") |
| `channel_id` | integer | no | null | Channel to focus investigation on |
| `message_id` | integer | no | null | Specific message to trace |

**Prompt Template:**

```
You are debugging a communication issue in an AgenticComm system.

Reported symptom: {symptom}

Store statistics:
{store_stats}

{if channel_id}
Channel details:
{channel_info}

Recent messages in channel:
{channel_messages}
{/if}

{if message_id}
Message details:
{message_info}
{/if}

Dead letter queue:
{dead_letters}

Session operation log:
{operation_log}

Communication context log:
{context_log}

Please diagnose:
1. **Root cause analysis:** What is likely causing the reported symptom?
2. **Evidence:** Which specific data points support the diagnosis?
3. **Verification steps:** What additional checks should be performed to
   confirm the diagnosis?
4. **Resolution:** Step-by-step instructions to fix the issue.
5. **Prevention:** What configuration changes or practices would prevent
   this issue from recurring?
```

**Expected Output:** A structured diagnostic report with root cause analysis, evidence, and resolution steps.

**Example Use Case:** An agent reports that it is not receiving responses to its query messages. The debugging prompt analyzes the channel state, participant list, message delivery records, and dead letter queue to identify whether the issue is a missing subscription, a closed channel, or a participant that has left.

## Prompt Interaction Protocol

Prompts are invoked through the MCP prompts interface:

### Listing Available Prompts

```json
{"method": "prompts/list"}
```

Response:
```json
{
  "prompts": [
    {
      "name": "comm-status",
      "description": "Summarize current communication state: channels, pending messages, recent activity"
    },
    {
      "name": "channel-health",
      "description": "Analyze a channel's health: delivery rate, acknowledgment rate, dead letters"
    },
    {
      "name": "thread-summary",
      "description": "Summarize a message thread by correlation ID"
    },
    {
      "name": "comm-topology",
      "description": "Map the communication topology of the agent network"
    },
    {
      "name": "comm-debug",
      "description": "Diagnose communication failures and delivery issues"
    }
  ]
}
```

### Getting a Prompt

```json
{
  "method": "prompts/get",
  "params": {
    "name": "channel-health",
    "arguments": {
      "channel_id": 1,
      "time_window": "24h"
    }
  }
}
```

Response:
```json
{
  "description": "Analyze a channel's health: delivery rate, acknowledgment rate, dead letters",
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "You are performing a health check on communication channel 'backend-team' (ID: 1).\n\nChannel information:\n..."
      }
    }
  ]
}
```

The server resolves the prompt template by fetching the relevant data from the store and injecting it into the template. The client receives a fully materialized prompt that can be sent to the AI model.

## Design Rationale

Prompts serve a different purpose than tools:

| Aspect | Tools | Prompts |
|--------|-------|---------|
| Purpose | Perform actions, return data | Structure reasoning, guide analysis |
| Mutating | Yes (send_message, etc.) | No (read-only) |
| Output | Raw data (JSON) | Analytical guidance (natural language) |
| Audience | Machine (parsed programmatically) | AI model (interpreted as instructions) |
| Frequency | Per-operation | Per-workflow (beginning of session, periodic health checks) |

Prompts are designed for workflows where the AI assistant needs to synthesize information from multiple sources and produce an analytical summary rather than just retrieving raw data.
