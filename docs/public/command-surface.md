---
status: draft
---

# Command Surface

Install commands are documented in [Installation](installation.md).

## Binaries

- `acomm` (CLI engine)
- `agentic-comm-mcp` (MCP server)

## `acomm` Top-level

```bash
acomm init
acomm send
acomm receive
acomm ack
acomm publish
acomm broadcast
acomm search
acomm history
acomm thread
acomm stats
acomm compact
acomm channel create
acomm channel list
acomm channel info
acomm channel join
acomm channel leave
acomm channel config
acomm channel pause
acomm channel resume
acomm channel drain
acomm channel close
acomm channel delete
acomm subscribe
acomm unsubscribe
acomm export
acomm import
acomm completions
```

## `agentic-comm-mcp` Commands

```bash
agentic-comm-mcp serve
agentic-comm-mcp validate
agentic-comm-mcp info
agentic-comm-mcp export
agentic-comm-mcp compact
agentic-comm-mcp stats
```

## MCP Tools

All tools exposed by the `agentic-comm-mcp` MCP server.

### send_message

Send a message through a communication channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | ID of the target channel |
| `sender` | string | yes | -- | Participant ID of the sender |
| `message_type` | string | yes | -- | Message type: text, command, query, response, broadcast, notification, acknowledgment, error |
| `content` | string | yes | -- | Message body (max 1 MB) |
| `topic` | string | no | null | Topic string for pub/sub routing |
| `correlation_id` | string | no | null | UUID linking related messages |
| `priority` | string | no | "normal" | Priority: critical, high, normal, low, background |
| `ttl` | integer | no | null | Time-to-live in seconds |
| `metadata` | object | no | null | Key-value metadata |

**Returns:**
```json
{
  "id": 42,
  "message_type": "command",
  "sender": "agent-planner",
  "channel_id": 1,
  "content": "Deploy auth-service to staging",
  "correlation_id": "a1b2c3d4-e5f6-4789-abcd-ef0123456789",
  "priority": "normal",
  "status": "sent",
  "created_at": 1740667812
}
```

**Errors:**
- Channel not found (channel_id does not exist)
- Channel closed (channel is in Closed or Draining state)
- Not a participant (sender is not in the channel)
- Permission denied (sender is an Observer)
- Validation failed (content empty or too large)
- Store capacity exceeded

**Example request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "send_message",
    "arguments": {
      "channel_id": 1,
      "sender": "agent-planner",
      "message_type": "command",
      "content": "Deploy auth-service to staging environment",
      "priority": "high"
    }
  }
}
```

---

### receive_messages

Receive messages from a channel for a specific participant.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel to receive from |
| `recipient` | string | yes | -- | Participant ID receiving messages |
| `message_types` | array[string] | no | null | Filter by message types |
| `after` | integer | no | null | Only messages after this Unix timestamp |
| `before` | integer | no | null | Only messages before this Unix timestamp |
| `sender` | string | no | null | Filter by sender |
| `topic_pattern` | string | no | null | Filter by topic (supports wildcards) |
| `limit` | integer | no | 100 | Maximum messages to return |
| `offset` | integer | no | 0 | Pagination offset |

**Returns:**
```json
{
  "messages": [
    {
      "id": 42,
      "message_type": "command",
      "sender": "agent-planner",
      "content": "Deploy auth-service to staging",
      "created_at": 1740667812,
      "status": "delivered"
    }
  ],
  "total_count": 1,
  "has_more": false
}
```

**Errors:**
- Channel not found
- Not a participant (recipient is not in the channel)

---

### create_channel

Create a new communication channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `name` | string | yes | -- | Channel name (max 128 chars) |
| `channel_type` | string | yes | -- | Type: direct, group, broadcast, pubsub |
| `owner` | string | yes | -- | Owner participant ID |
| `description` | string | no | null | Human-readable description |
| `tags` | array[string] | no | [] | Categorization tags |

**Returns:**
```json
{
  "id": 1,
  "name": "backend-team",
  "channel_type": "group",
  "owner": "agent-lead",
  "state": "active",
  "participants": [{"id": "agent-lead", "role": "owner"}],
  "message_count": 0,
  "created_at": 1740667800
}
```

**Errors:**
- Validation failed (name invalid or too long)
- Duplicate channel (name already exists)
- Store capacity exceeded

---

### list_channels

List all channels in the store.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_type` | string | no | null | Filter by type: direct, group, broadcast, pubsub |
| `state` | string | no | null | Filter by state: active, paused, draining, closed |

**Returns:**
```json
{
  "channels": [
    {
      "id": 1,
      "name": "backend-team",
      "channel_type": "group",
      "state": "active",
      "participant_count": 4,
      "message_count": 127,
      "created_at": 1740667800
    }
  ]
}
```

---

### join_channel

Add a participant to an existing channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel to join |
| `participant` | string | yes | -- | Participant ID |
| `role` | string | no | "member" | Role: member, observer |

**Returns:**
```json
{
  "channel_id": 1,
  "participant": "agent-worker-03",
  "role": "member",
  "joined_at": 1740668000
}
```

**Errors:**
- Channel not found
- Channel closed
- Already a participant
- Max participants reached
- Validation failed (participant ID invalid)

---

### leave_channel

Remove a participant from a channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel to leave |
| `participant` | string | yes | -- | Participant ID |

**Returns:**
```json
{
  "channel_id": 1,
  "participant": "agent-worker-03",
  "left_at": 1740669000
}
```

**Errors:**
- Channel not found
- Not a participant
- Owner cannot leave (must transfer ownership first)

---

### get_channel_info

Get detailed information about a channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID |

**Returns:**
```json
{
  "id": 1,
  "name": "backend-team",
  "channel_type": "group",
  "owner": "agent-lead",
  "state": "active",
  "participants": [
    {"id": "agent-lead", "role": "owner", "joined_at": 1740667800},
    {"id": "agent-api", "role": "member", "joined_at": 1740667810},
    {"id": "agent-db", "role": "member", "joined_at": 1740667820}
  ],
  "config": {
    "delivery": "at_most_once",
    "max_message_size": 1048576,
    "retention": "forever",
    "echo": false
  },
  "message_count": 127,
  "description": "Backend team coordination channel",
  "tags": ["backend", "coordination"],
  "created_at": 1740667800,
  "modified_at": 1740670000
}
```

---

### subscribe

Subscribe to a topic pattern on a pub/sub channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | PubSub channel ID |
| `subscriber` | string | yes | -- | Subscriber participant ID |
| `topic_pattern` | string | yes | -- | Topic pattern (supports *, #) |

**Returns:**
```json
{
  "id": 1,
  "channel_id": 3,
  "subscriber": "agent-deploy",
  "topic_pattern": "build.*.success",
  "match_mode": "wildcard",
  "created_at": 1740668000
}
```

**Errors:**
- Channel not found
- Wrong channel type (not a PubSub channel)
- Not a participant
- Validation failed (invalid topic pattern)
- Duplicate subscription

---

### unsubscribe

Remove a pub/sub subscription.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `subscription_id` | integer | yes | -- | Subscription ID to remove |

**Returns:**
```json
{
  "subscription_id": 1,
  "removed": true
}
```

---

### publish

Publish a message to a pub/sub topic.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | PubSub channel ID |
| `sender` | string | yes | -- | Publisher participant ID |
| `topic` | string | yes | -- | Topic string (dot-separated) |
| `content` | string | yes | -- | Message content |
| `priority` | string | no | "normal" | Message priority |

**Returns:**
```json
{
  "id": 55,
  "topic": "build.frontend.success",
  "matched_subscriptions": 2,
  "status": "delivered",
  "created_at": 1740668500
}
```

**Errors:**
- Channel not found
- Wrong channel type (not PubSub)
- Not a participant
- Validation failed (topic or content invalid)

---

### broadcast

Broadcast a message to all participants in a channel.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID |
| `sender` | string | yes | -- | Sender participant ID |
| `content` | string | yes | -- | Broadcast message content |
| `priority` | string | no | "normal" | Message priority |

**Returns:**
```json
{
  "id": 60,
  "message_type": "broadcast",
  "recipients": 5,
  "status": "delivered",
  "created_at": 1740669000
}
```

---

### query_history

Query communication history with filters.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | no | null | Filter by channel |
| `sender` | string | no | null | Filter by sender |
| `message_types` | array[string] | no | null | Filter by message types |
| `after` | integer | no | null | After Unix timestamp |
| `before` | integer | no | null | Before Unix timestamp |
| `status` | array[string] | no | null | Filter by status |
| `topic_pattern` | string | no | null | Filter by topic |
| `correlation_id` | string | no | null | Filter by thread |
| `limit` | integer | no | 100 | Max results |
| `offset` | integer | no | 0 | Pagination offset |
| `sort_by` | string | no | "created_at" | Sort field |
| `sort_order` | string | no | "descending" | Sort direction |
| `include_archived` | boolean | no | false | Include archived messages |

**Returns:**
```json
{
  "messages": [...],
  "total_count": 250,
  "has_more": true
}
```

---

### search_messages

Search messages by content with optional filters.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | yes | -- | Search string (prefix with "regex:" for regex) |
| `channel_id` | integer | no | null | Limit to channel |
| `sender` | string | no | null | Limit to sender |
| `limit` | integer | no | 50 | Max results |

**Returns:**
```json
{
  "messages": [...],
  "total_count": 12,
  "has_more": false
}
```

---

### get_message

Get a specific message by ID.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `message_id` | integer | yes | -- | Message ID |

**Returns:** A single message object.

**Errors:**
- Message not found

---

### acknowledge_message

Acknowledge receipt or completion of a command message.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `message_id` | integer | yes | -- | ID of the Command message |
| `sender` | string | yes | -- | Acknowledging participant |
| `status` | string | yes | -- | Status: received, in_progress, completed, failed |
| `content` | string | no | null | Optional acknowledgment body |

**Returns:**
```json
{
  "id": 65,
  "message_type": "acknowledgment",
  "correlation_id": "a1b2c3d4-e5f6-4789-abcd-ef0123456789",
  "status": "completed",
  "created_at": 1740670000
}
```

---

### set_channel_config

Update channel configuration.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `channel_id` | integer | yes | -- | Channel ID |
| `delivery` | string | no | null | Delivery mode: at_most_once, at_least_once, exactly_once |
| `max_message_size` | integer | no | null | Max message size in bytes |
| `retention` | string | no | null | Retention: forever, or duration/count/size with value |
| `ack_timeout` | integer | no | null | Ack timeout in seconds |
| `max_retries` | integer | no | null | Max delivery retries |
| `echo` | boolean | no | null | Whether senders receive own messages |

**Returns:**
```json
{
  "channel_id": 1,
  "config": {
    "delivery": "at_least_once",
    "max_message_size": 1048576,
    "retention": "forever",
    "ack_timeout": 30,
    "max_retries": 3,
    "echo": false
  },
  "updated_at": 1740671000
}
```

---

### communication_log

Log communication context for memory integration and temporal chaining.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `user_message` | string | no | null | What the user said or asked |
| `agent_response` | string | no | null | Summary of the agent's communication action |
| `topic` | string | no | null | Topic or category |

**Returns:**
```json
{
  "logged": true,
  "chain_position": 42
}
```

## MCP Resources

The MCP server exposes read-only resources:

| URI | Description |
|-----|-------------|
| `comm://store/stats` | Store statistics (channel count, message count, etc.) |
| `comm://channels` | List of all channels |
| `comm://channels/{id}` | Channel details |
| `comm://channels/{id}/messages` | Messages in a channel (paginated) |
| `comm://dead-letters` | Dead letter queue contents |

## MCP Prompts

| Prompt | Description |
|--------|-------------|
| `comm-status` | Summarize current communication state: channels, pending messages, recent activity |
| `channel-health` | Analyze a channel's health: delivery rate, acknowledgment rate, dead letters |
| `thread-summary` | Summarize a message thread by correlation ID |

## Server Capabilities

The MCP server advertises the following capabilities:

```json
{
  "capabilities": {
    "tools": {},
    "resources": {
      "subscribe": true,
      "listChanged": true
    },
    "prompts": {
      "listChanged": true
    }
  }
}
```

## Complete MCP Tool Index

All 204 tools registered by the MCP server, grouped by module.

### Core Tools (108)

| Tool | Category |
|------|----------|
| `comm_send_message` | Messaging |
| `comm_receive_messages` | Messaging |
| `comm_create_channel` | Channels |
| `comm_list_channels` | Channels |
| `comm_join_channel` | Channels |
| `comm_leave_channel` | Channels |
| `comm_get_channel_info` | Channels |
| `comm_subscribe` | PubSub |
| `comm_unsubscribe` | PubSub |
| `comm_publish` | PubSub |
| `comm_broadcast` | Messaging |
| `comm_query_history` | Query |
| `comm_search_messages` | Query |
| `comm_get_message` | Query |
| `comm_acknowledge_message` | Messaging |
| `comm_set_channel_config` | Channels |
| `comm_communication_log` | Context |
| `comm_manage_consent` | Consent |
| `comm_check_consent` | Consent |
| `comm_set_trust_level` | Trust |
| `comm_get_trust_level` | Trust |
| `comm_schedule_message` | Temporal |
| `comm_list_scheduled` | Temporal |
| `comm_form_hive` | Hive |
| `comm_get_stats` | Stats |
| `comm_send_affect` | Affect |
| `comm_ground` | Grounding |
| `comm_evidence` | Grounding |
| `comm_suggest` | Grounding |
| `comm_list_consent_gates` | Consent |
| `comm_respond_consent` | Consent |
| `comm_list_pending_consent` | Consent |
| `comm_list_trust_levels` | Trust |
| `comm_query_relationships` | Query |
| `comm_send_reply` | Messaging |
| `comm_get_replies` | Query |
| `comm_get_thread` | Query |
| `comm_forward_message` | Messaging |
| `comm_query_echo_chain` | Query |
| `comm_query_echoes` | Query |
| `comm_get_echo_depth` | Query |
| `comm_query_conversations` | Query |
| `comm_summarize_conversation` | Query |
| `comm_send_with_priority` | Messaging |
| `comm_expire_messages` | Lifecycle |
| `comm_cancel_scheduled` | Temporal |
| `comm_deliver_pending` | Temporal |
| `comm_configure_federation` | Federation |
| `comm_add_federated_zone` | Federation |
| `comm_remove_federated_zone` | Federation |
| `comm_list_federated_zones` | Federation |
| `comm_get_federation_status` | Federation |
| `comm_set_federation_policy` | Federation |
| `comm_set_zone_policy` | Federation |
| `comm_dissolve_hive` | Hive |
| `comm_join_hive` | Hive |
| `comm_leave_hive` | Hive |
| `comm_list_hives` | Hive |
| `comm_get_hive` | Hive |
| `comm_hive_think` | Hive |
| `comm_initiate_meld` | Hive |
| `comm_get_affect_state` | Affect |
| `comm_process_affect_contagion` | Affect |
| `comm_apply_affect_decay` | Affect |
| `comm_set_affect_resistance` | Affect |
| `comm_get_affect_history` | Affect |
| `comm_send_semantic` | Semantic |
| `comm_extract_semantic` | Semantic |
| `comm_graft_semantic` | Semantic |
| `comm_list_semantic_conflicts` | Semantic |
| `comm_generate_keypair` | Crypto |
| `comm_verify_signature` | Crypto |
| `comm_generate_key` | Crypto |
| `comm_encrypt_message` | Crypto |
| `comm_decrypt_message` | Crypto |
| `comm_list_keys` | Crypto |
| `comm_get_key` | Crypto |
| `comm_rotate_key` | Crypto |
| `comm_revoke_key` | Crypto |
| `comm_export_key` | Crypto |
| `comm_get_public_key` | Crypto |
| `comm_get_audit_log` | Audit |
| `comm_list_dead_letters` | Dead Letters |
| `comm_replay_dead_letter` | Dead Letters |
| `comm_clear_dead_letters` | Dead Letters |
| `comm_close_channel` | Channels |
| `comm_pause_channel` | Channels |
| `comm_resume_channel` | Channels |
| `comm_drain_channel` | Channels |
| `comm_compact` | Maintenance |
| `comm_workspace_create` | Workspace |
| `comm_workspace_add` | Workspace |
| `comm_workspace_list` | Workspace |
| `comm_workspace_query` | Workspace |
| `comm_workspace_compare` | Workspace |
| `comm_workspace_xref` | Workspace |
| `comm_log_communication` | Context |
| `comm_get_comm_log` | Context |
| `comm_conversation_log` | Context |
| `comm_send_rich_message` | Messaging |
| `comm_get_rich_content` | Messaging |
| `comm_assign_comm_ids` | Identity |
| `comm_get_by_comm_id` | Identity |
| `comm_id` | Identity |
| `comm_session_start` | Session |
| `comm_session_end` | Session |
| `comm_session_resume` | Session |
| `session_start` | Session (alias) |
| `session_end` | Session (alias) |

### Invention: Collaboration (16)

| Tool | Category |
|------|----------|
| `comm_hive_consciousness_sync` | Hive Consciousness |
| `comm_hive_consciousness_merge` | Hive Consciousness |
| `comm_hive_consciousness_split` | Hive Consciousness |
| `comm_hive_consciousness_status` | Hive Consciousness |
| `comm_collective_intelligence_contribute` | Collective Intelligence |
| `comm_collective_intelligence_query` | Collective Intelligence |
| `comm_collective_intelligence_consensus` | Collective Intelligence |
| `comm_collective_intelligence_dissent` | Collective Intelligence |
| `comm_ancestor_trace` | Ancestor |
| `comm_ancestor_lineage` | Ancestor |
| `comm_ancestor_inherit` | Ancestor |
| `comm_ancestor_verify` | Ancestor |
| `comm_telepathy_link` | Telepathy |
| `comm_telepathy_broadcast` | Telepathy |
| `comm_telepathy_listen` | Telepathy |
| `comm_telepathy_consensus` | Telepathy |

### Invention: Semantics (16)

| Tool | Category |
|------|----------|
| `comm_semantic_graft` | Semantic Grafting |
| `comm_semantic_extract` | Semantic Grafting |
| `comm_semantic_merge` | Semantic Grafting |
| `comm_semantic_cluster` | Semantic Grafting |
| `comm_echo_chamber_detect` | Echo Chamber |
| `comm_echo_chamber_break` | Echo Chamber |
| `comm_echo_chamber_analyze` | Echo Chamber |
| `comm_echo_chamber_health` | Echo Chamber |
| `comm_ghost_create` | Ghost Conversations |
| `comm_ghost_reveal` | Ghost Conversations |
| `comm_ghost_list` | Ghost Conversations |
| `comm_ghost_dissolve` | Ghost Conversations |
| `comm_metamessage_encode` | Metamessages |
| `comm_metamessage_decode` | Metamessages |
| `comm_metamessage_inject` | Metamessages |
| `comm_metamessage_trace` | Metamessages |

### Invention: Affect (16)

| Tool | Category |
|------|----------|
| `comm_affect_contagion_simulate` | Affect Contagion |
| `comm_affect_contagion_immunize` | Affect Contagion |
| `comm_affect_contagion_outbreak` | Affect Contagion |
| `comm_affect_contagion_model` | Affect Contagion |
| `comm_affect_archaeology_dig` | Emotional Archaeology |
| `comm_affect_archaeology_timeline` | Emotional Archaeology |
| `comm_affect_archaeology_artifacts` | Emotional Archaeology |
| `comm_affect_archaeology_reconstruct` | Emotional Archaeology |
| `comm_affect_prophecy_predict` | Affect Prophecy |
| `comm_affect_prophecy_similar` | Affect Prophecy |
| `comm_affect_prophecy_track` | Affect Prophecy |
| `comm_affect_prophecy_warn` | Affect Prophecy |
| `comm_unspeakable_encode` | Unspeakable Content |
| `comm_unspeakable_decode` | Unspeakable Content |
| `comm_unspeakable_detect` | Unspeakable Content |
| `comm_unspeakable_translate` | Unspeakable Content |

### Invention: Federation (16)

| Tool | Category |
|------|----------|
| `comm_federation_gateway_create` | Federation Gateway |
| `comm_federation_gateway_connect` | Federation Gateway |
| `comm_federation_gateway_disconnect` | Federation Gateway |
| `comm_federation_gateway_status` | Federation Gateway |
| `comm_federation_route_message` | Federation Routing |
| `comm_federation_route_trace` | Federation Routing |
| `comm_federation_route_optimize` | Federation Routing |
| `comm_federation_route_policy` | Federation Routing |
| `comm_federation_zone_create` | Zone Policies |
| `comm_federation_zone_list` | Zone Policies |
| `comm_federation_zone_merge` | Zone Policies |
| `comm_federation_zone_policy` | Zone Policies |
| `comm_reality_fork` | Reality Bending |
| `comm_reality_merge` | Reality Bending |
| `comm_reality_detect` | Reality Bending |
| `comm_reality_bend` | Reality Bending |

### Invention: Temporal (16)

| Tool | Category |
|------|----------|
| `comm_precognition_predict` | Precognitive Messaging |
| `comm_precognition_prepare` | Precognitive Messaging |
| `comm_precognition_accuracy` | Precognitive Messaging |
| `comm_precognition_calibrate` | Precognitive Messaging |
| `comm_temporal_schedule` | Temporal Scheduling |
| `comm_temporal_cancel` | Temporal Scheduling |
| `comm_temporal_pending` | Temporal Scheduling |
| `comm_temporal_reschedule` | Temporal Scheduling |
| `comm_legacy_compose` | Legacy Messages |
| `comm_legacy_seal` | Legacy Messages |
| `comm_legacy_unseal` | Legacy Messages |
| `comm_legacy_list` | Legacy Messages |
| `comm_dead_letter_resurrect` | Dead Letter Resurrection |
| `comm_dead_letter_autopsy` | Dead Letter Resurrection |
| `comm_dead_letter_phoenix` | Dead Letter Resurrection |
| `comm_dead_letter_analyze` | Dead Letter Resurrection |

### Invention: Forensics (16)

| Tool | Category |
|------|----------|
| `comm_forensics_investigate` | Communication Forensics |
| `comm_forensics_evidence` | Communication Forensics |
| `comm_forensics_timeline` | Communication Forensics |
| `comm_forensics_report` | Communication Forensics |
| `comm_pattern_detect` | Pattern Detection |
| `comm_pattern_recurring` | Pattern Detection |
| `comm_pattern_anomaly` | Pattern Detection |
| `comm_pattern_predict` | Pattern Detection |
| `comm_health_status` | Health Monitoring |
| `comm_health_diagnose` | Health Monitoring |
| `comm_health_prescribe` | Health Monitoring |
| `comm_health_history` | Health Monitoring |
| `comm_oracle_query` | Oracle Predictions |
| `comm_oracle_prophecy` | Oracle Predictions |
| `comm_oracle_calibrate` | Oracle Predictions |
| `comm_oracle_verify` | Oracle Predictions |
