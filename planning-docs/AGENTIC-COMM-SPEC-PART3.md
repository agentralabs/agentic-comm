# AgenticComm Specification — Part 3

> **Version:** 0.1.0
> **Status:** Pre-Implementation Specification
> **Covers:** CLI Reference, MCP Server, Sister Integration

---

# SPEC-09: CLI REFERENCE

## 9.1 Command Overview

```
agentic-comm <COMMAND> [OPTIONS]

COMMANDS:
  channel     Channel management
  message     Message operations
  send        Quick send (alias)
  receive     Receive operations
  query       Query messages/channels
  hive        Hive mind operations
  federation  Federation management
  consent     Consent management
  keys        Key management
  status      System status
  daemon      Background daemon control
  help        Print help information

GLOBAL OPTIONS:
  -c, --config <PATH>     Config file path
  -d, --data-dir <PATH>   Data directory
  -v, --verbose           Verbose output
  -q, --quiet             Minimal output
  --json                  JSON output
  --format <FORMAT>       Output format [table|json|yaml]
  --identity <ID>         Identity to use
  --version               Print version
```

## 9.2 Channel Commands

```bash
# Create channel
agentic-comm channel create \
  --type direct \
  --participants "agent-a,agent-b" \
  --name "Project Discussion" \
  --contract policy.json

# Create group channel
agentic-comm channel create \
  --type group \
  --participants "agent-a,agent-b,agent-c" \
  --name "Team Channel"

# Create telepathic channel
agentic-comm channel create \
  --type telepathic \
  --participants "agent-a,agent-b" \
  --shared-space-size 1000

# Create hive channel
agentic-comm channel create \
  --type hive \
  --participants "agent-a,agent-b,agent-c" \
  --separation-policy free-exit

# List channels
agentic-comm channel list
agentic-comm channel list --type direct
agentic-comm channel list --active
agentic-comm channel list --with-agent "agent-b"

# Show channel details
agentic-comm channel show <CHANNEL_ID>
agentic-comm channel show <CHANNEL_ID> --participants
agentic-comm channel show <CHANNEL_ID> --stats

# Join channel
agentic-comm channel join <CHANNEL_ID>

# Leave channel
agentic-comm channel leave <CHANNEL_ID>

# Archive channel
agentic-comm channel archive <CHANNEL_ID>

# Close channel (owner only)
agentic-comm channel close <CHANNEL_ID>

# Modify channel
agentic-comm channel modify <CHANNEL_ID> \
  --add-participant "agent-d" \
  --remove-participant "agent-c"

agentic-comm channel modify <CHANNEL_ID> \
  --name "New Name"

# Channel permissions
agentic-comm channel permissions <CHANNEL_ID> list
agentic-comm channel permissions <CHANNEL_ID> set \
  --agent "agent-b" \
  --role admin

agentic-comm channel permissions <CHANNEL_ID> revoke \
  --agent "agent-c" \
  --permission send
```

## 9.3 Message Commands

```bash
# Send text message
agentic-comm send <CHANNEL_ID> "Hello, world!"

# Send with options
agentic-comm send <CHANNEL_ID> "Important message" \
  --priority high \
  --require-ack

# Send semantic fragment
agentic-comm message send-semantic <CHANNEL_ID> \
  --focus "node-123,node-456" \
  --context-depth 3 \
  --perspective neutral

# Send with affect
agentic-comm message send-affect <CHANNEL_ID> "I found the bug!" \
  --valence 0.8 \
  --arousal 0.6 \
  --emotion joy \
  --urgency high

# Send temporal message
agentic-comm message send-temporal <CHANNEL_ID> "Future message" \
  --deliver-at "2026-03-01T10:00:00Z"

agentic-comm message send-temporal <CHANNEL_ID> "Conditional" \
  --condition "recipient.online"

# Send to thread
agentic-comm send <CHANNEL_ID> "Reply" \
  --reply-to <MESSAGE_ID>

# View message
agentic-comm message show <MESSAGE_ID>
agentic-comm message show <MESSAGE_ID> --raw
agentic-comm message show <MESSAGE_ID> --receipt

# List messages
agentic-comm message list --channel <CHANNEL_ID>
agentic-comm message list --channel <CHANNEL_ID> --limit 50
agentic-comm message list --from "agent-a" --last 24h
agentic-comm message list --unread

# Search messages
agentic-comm message search "authentication bug"
agentic-comm message search "auth" --channel <CHANNEL_ID>
agentic-comm message search --semantic "problems with login"

# Message status
agentic-comm message status <MESSAGE_ID>

# Acknowledge message
agentic-comm message ack <MESSAGE_ID>

# Delete message (if allowed)
agentic-comm message delete <MESSAGE_ID>

# Forward message
agentic-comm message forward <MESSAGE_ID> --to <CHANNEL_ID>
```

## 9.4 Receive Commands

```bash
# Listen for messages (blocking)
agentic-comm receive listen
agentic-comm receive listen --channel <CHANNEL_ID>
agentic-comm receive listen --format json

# Poll for new messages
agentic-comm receive poll
agentic-comm receive poll --since <TIMESTAMP>

# Get unread count
agentic-comm receive unread
agentic-comm receive unread --by-channel

# Mark as read
agentic-comm receive mark-read <MESSAGE_ID>
agentic-comm receive mark-read --channel <CHANNEL_ID>
agentic-comm receive mark-read --all
```

## 9.5 Query Commands

```bash
# Query messages
agentic-comm query messages \
  --channel <CHANNEL_ID> \
  --from "2026-01-01" \
  --to "2026-02-01"

agentic-comm query messages \
  --sender "agent-a" \
  --content-type semantic

agentic-comm query messages \
  --affect-valence ">0.5" \
  --affect-urgency ">=high"

# Query channels
agentic-comm query channels --active
agentic-comm query channels --type telepathic
agentic-comm query channels --participant "agent-b"

# Query relationships
agentic-comm query relationship "agent-a" "agent-b"
agentic-comm query relationships --trust ">0.7"
agentic-comm query relationships --most-active 10

# Query conversation history
agentic-comm query conversation <CHANNEL_ID> \
  --at "2026-01-15T10:00:00Z"

agentic-comm query conversation <CHANNEL_ID> \
  --range "2026-01-01" "2026-01-31" \
  --summary

# Query echoes
agentic-comm query echoes <MESSAGE_ID>
```

## 9.6 Hive Commands

```bash
# Form hive
agentic-comm hive form \
  --participants "agent-a,agent-b,agent-c" \
  --separation-policy consensus-required

# Join existing hive
agentic-comm hive join <HIVE_ID>

# Leave hive
agentic-comm hive leave <HIVE_ID>

# Hive status
agentic-comm hive status <HIVE_ID>
agentic-comm hive status <HIVE_ID> --coherence
agentic-comm hive status <HIVE_ID> --capabilities

# Think as hive
agentic-comm hive think <HIVE_ID> "Problem to solve collectively"

# Dissolve hive (requires consensus or owner)
agentic-comm hive dissolve <HIVE_ID>

# List hives
agentic-comm hive list
agentic-comm hive list --active

# Mind meld (temporary)
agentic-comm hive meld "agent-b" --duration 60s
```

## 9.7 Federation Commands

```bash
# Add federated zone
agentic-comm federation add \
  --zone "zone-b" \
  --gateway "https://zone-b.example.com/gateway" \
  --trust-level standard

# List federated zones
agentic-comm federation list

# Federation status
agentic-comm federation status
agentic-comm federation status "zone-b"

# Set federation policy
agentic-comm federation policy "zone-b" \
  --allow-semantic \
  --deny-affect \
  --rate-limit 100/hour

# Remove federation
agentic-comm federation remove "zone-b"

# Test federation connectivity
agentic-comm federation ping "zone-b"
```

## 9.8 Consent Commands

```bash
# Grant consent
agentic-comm consent grant \
  --to "agent-b" \
  --scope receive-messages

agentic-comm consent grant \
  --to "agent-b" \
  --scope receive-semantic \
  --expires "2026-12-31"

agentic-comm consent grant \
  --to "agent-b" \
  --scope mind-meld

# Revoke consent
agentic-comm consent revoke \
  --to "agent-b" \
  --scope receive-affect

# List consents
agentic-comm consent list
agentic-comm consent list --granted
agentic-comm consent list --received

# Set defaults
agentic-comm consent defaults \
  --receive-messages allow \
  --receive-semantic deny \
  --receive-affect ask

# Pending consent requests
agentic-comm consent pending
agentic-comm consent approve <REQUEST_ID>
agentic-comm consent deny <REQUEST_ID>
```

## 9.9 Key Management Commands

```bash
# Generate new keys
agentic-comm keys generate

# Rotate channel keys
agentic-comm keys rotate --channel <CHANNEL_ID>

# Export public key
agentic-comm keys export --public

# Import contact's key
agentic-comm keys import "agent-b" <PUBLIC_KEY>

# Key status
agentic-comm keys status
agentic-comm keys status --channel <CHANNEL_ID>
```

## 9.10 Daemon Commands

```bash
# Start daemon
agentic-comm daemon start
agentic-comm daemon start --foreground

# Stop daemon
agentic-comm daemon stop

# Daemon status
agentic-comm daemon status

# Restart daemon
agentic-comm daemon restart

# View daemon logs
agentic-comm daemon logs
agentic-comm daemon logs --follow
agentic-comm daemon logs --since 1h
```

---

# SPEC-10: MCP SERVER

## 10.1 Server Configuration

```json
{
  "mcpServers": {
    "agentic-comm": {
      "command": "agentic-comm-mcp",
      "args": ["--data-dir", "~/.agentic/comm"],
      "env": {
        "AGENTIC_TOKEN": "${AGENTIC_TOKEN}",
        "RUST_LOG": "info"
      }
    }
  }
}
```

## 10.2 MCP Tools

### Channel Tools

```yaml
tools:
  - name: comm_channel_create
    description: Create a new communication channel
    inputSchema:
      type: object
      required: [channel_type, participants]
      properties:
        channel_type:
          type: string
          enum: [direct, group, broadcast, telepathic, hive, temporal, destiny]
          description: Type of channel to create
        participants:
          type: array
          items:
            type: string
          description: Identity anchors of participants
        name:
          type: string
          description: Human-readable channel name
        contract_id:
          type: string
          description: Contract ID for policies (optional)

  - name: comm_channel_list
    description: List communication channels
    inputSchema:
      type: object
      properties:
        channel_type:
          type: string
          description: Filter by type
        state:
          type: string
          enum: [active, paused, archived]
        participant:
          type: string
          description: Filter by participant identity
        limit:
          type: integer
          default: 50

  - name: comm_channel_join
    description: Join an existing channel
    inputSchema:
      type: object
      required: [channel_id]
      properties:
        channel_id:
          type: string
          description: UUID of channel to join

  - name: comm_channel_leave
    description: Leave a channel
    inputSchema:
      type: object
      required: [channel_id]
      properties:
        channel_id:
          type: string

  - name: comm_channel_info
    description: Get channel information
    inputSchema:
      type: object
      required: [channel_id]
      properties:
        channel_id:
          type: string
        include_participants:
          type: boolean
          default: true
        include_stats:
          type: boolean
          default: false
```

### Message Tools

```yaml
tools:
  - name: comm_send
    description: Send a message to a channel
    inputSchema:
      type: object
      required: [channel_id, content]
      properties:
        channel_id:
          type: string
        content:
          type: object
          required: [type]
          properties:
            type:
              type: string
              enum: [text, semantic, affect, full]
            text:
              type: string
            semantic_focus:
              type: array
              items:
                type: string
            affect:
              type: object
              properties:
                valence:
                  type: number
                arousal:
                  type: number
                emotions:
                  type: object
                urgency:
                  type: string
        priority:
          type: string
          enum: [low, normal, high, urgent, critical]
        reply_to:
          type: string
          description: Message ID to reply to
        require_ack:
          type: boolean

  - name: comm_send_semantic
    description: Send a semantic fragment
    inputSchema:
      type: object
      required: [channel_id, focus_nodes]
      properties:
        channel_id:
          type: string
        focus_nodes:
          type: array
          items:
            type: string
          description: Node IDs to include in fragment
        context_depth:
          type: integer
          default: 2
        include_affect:
          type: boolean
          default: false

  - name: comm_send_temporal
    description: Send a temporal (time-targeted) message
    inputSchema:
      type: object
      required: [channel_id, content, temporal_target]
      properties:
        channel_id:
          type: string
        content:
          type: object
        temporal_target:
          type: object
          required: [type]
          properties:
            type:
              type: string
              enum: [future_absolute, future_relative, conditional, retroactive, optimal, eternal]
            timestamp:
              type: string
              format: date-time
            duration_seconds:
              type: integer
            condition:
              type: string

  - name: comm_receive
    description: Receive pending messages
    inputSchema:
      type: object
      properties:
        channel_id:
          type: string
          description: Specific channel (optional)
        limit:
          type: integer
          default: 100
        since:
          type: string
          format: date-time
        unread_only:
          type: boolean
          default: true

  - name: comm_message_info
    description: Get message details
    inputSchema:
      type: object
      required: [message_id]
      properties:
        message_id:
          type: string
        include_receipt:
          type: boolean
          default: false

  - name: comm_message_search
    description: Search messages
    inputSchema:
      type: object
      required: [query]
      properties:
        query:
          type: string
        channel_id:
          type: string
        semantic_search:
          type: boolean
          default: false
        limit:
          type: integer
          default: 20
```

### Semantic Tools

```yaml
tools:
  - name: comm_semantic_extract
    description: Extract semantic fragment from cognitive state
    inputSchema:
      type: object
      required: [focus_nodes]
      properties:
        focus_nodes:
          type: array
          items:
            type: string
        context_depth:
          type: integer
          default: 2
        perspective:
          type: string
          enum: [neutral, sender, receiver, shared]

  - name: comm_semantic_graft
    description: Graft received semantic fragment onto local graph
    inputSchema:
      type: object
      required: [fragment_id]
      properties:
        fragment_id:
          type: string
        auto_resolve_conflicts:
          type: boolean
          default: false

  - name: comm_semantic_conflicts
    description: List conflicts from semantic graft
    inputSchema:
      type: object
      required: [graft_id]
      properties:
        graft_id:
          type: string
```

### Affect Tools

```yaml
tools:
  - name: comm_affect_encode
    description: Encode current affect state for transmission
    inputSchema:
      type: object
      properties:
        contagion_strength:
          type: number
          minimum: 0
          maximum: 1
          default: 0.5
        resistable:
          type: boolean
          default: true

  - name: comm_affect_current
    description: Get current affect state
    inputSchema:
      type: object
      properties: {}

  - name: comm_affect_resist
    description: Increase resistance to affect contagion
    inputSchema:
      type: object
      properties:
        amount:
          type: number
          minimum: 0
          maximum: 1
          default: 0.2

  - name: comm_affect_open
    description: Decrease resistance to affect contagion
    inputSchema:
      type: object
      properties:
        amount:
          type: number
          minimum: 0
          maximum: 1
          default: 0.2
```

### Hive Tools

```yaml
tools:
  - name: comm_hive_form
    description: Form a hive mind with other agents
    inputSchema:
      type: object
      required: [participants]
      properties:
        participants:
          type: array
          items:
            type: string
          minItems: 2
        separation_policy:
          type: string
          enum: [free_exit, consensus_required, permanent, time_limited]
        time_limit_seconds:
          type: integer

  - name: comm_hive_join
    description: Join an existing hive
    inputSchema:
      type: object
      required: [hive_id]
      properties:
        hive_id:
          type: string

  - name: comm_hive_leave
    description: Leave a hive
    inputSchema:
      type: object
      required: [hive_id]
      properties:
        hive_id:
          type: string

  - name: comm_hive_status
    description: Get hive status
    inputSchema:
      type: object
      required: [hive_id]
      properties:
        hive_id:
          type: string
        include_capabilities:
          type: boolean
          default: false

  - name: comm_hive_think
    description: Think collectively as hive
    inputSchema:
      type: object
      required: [hive_id, thought]
      properties:
        hive_id:
          type: string
        thought:
          type: string

  - name: comm_meld_initiate
    description: Initiate mind meld with another agent
    inputSchema:
      type: object
      required: [agent]
      properties:
        agent:
          type: string
        duration_seconds:
          type: integer
          default: 60
```

### Consent Tools

```yaml
tools:
  - name: comm_consent_grant
    description: Grant consent to another agent
    inputSchema:
      type: object
      required: [to_agent, scope]
      properties:
        to_agent:
          type: string
        scope:
          type: string
          enum: [receive_messages, receive_semantic, receive_affect, telepathic_access, hive_formation, mind_meld, federation]
        expires_at:
          type: string
          format: date-time

  - name: comm_consent_revoke
    description: Revoke consent from an agent
    inputSchema:
      type: object
      required: [to_agent, scope]
      properties:
        to_agent:
          type: string
        scope:
          type: string

  - name: comm_consent_check
    description: Check consent status
    inputSchema:
      type: object
      required: [agent, scope]
      properties:
        agent:
          type: string
        scope:
          type: string

  - name: comm_consent_pending
    description: List pending consent requests
    inputSchema:
      type: object
      properties: {}

  - name: comm_consent_respond
    description: Respond to consent request
    inputSchema:
      type: object
      required: [request_id, decision]
      properties:
        request_id:
          type: string
        decision:
          type: string
          enum: [grant, deny]
```

### Query Tools

```yaml
tools:
  - name: comm_query_messages
    description: Query messages with filters
    inputSchema:
      type: object
      properties:
        channel_id:
          type: string
        sender:
          type: string
        time_start:
          type: string
          format: date-time
        time_end:
          type: string
          format: date-time
        content_type:
          type: string
        affect_filter:
          type: object
          properties:
            min_valence:
              type: number
            max_valence:
              type: number
            emotions:
              type: array
              items:
                type: string
        limit:
          type: integer
          default: 50

  - name: comm_query_relationships
    description: Query agent relationships
    inputSchema:
      type: object
      properties:
        agent:
          type: string
        min_trust:
          type: number
        relationship_type:
          type: string
        limit:
          type: integer
          default: 20

  - name: comm_query_echoes
    description: Query message echoes (propagation)
    inputSchema:
      type: object
      required: [message_id]
      properties:
        message_id:
          type: string
        max_depth:
          type: integer
          default: 5
```

### Federation Tools

```yaml
tools:
  - name: comm_federation_add
    description: Add a federated zone
    inputSchema:
      type: object
      required: [zone_id, gateway_url]
      properties:
        zone_id:
          type: string
        gateway_url:
          type: string
        trust_level:
          type: string
          enum: [minimal, basic, standard, high, full]
          default: standard

  - name: comm_federation_list
    description: List federated zones
    inputSchema:
      type: object
      properties: {}

  - name: comm_federation_status
    description: Get federation status
    inputSchema:
      type: object
      properties:
        zone_id:
          type: string

  - name: comm_federation_policy
    description: Set federation policy
    inputSchema:
      type: object
      required: [zone_id]
      properties:
        zone_id:
          type: string
        allow_semantic:
          type: boolean
        allow_affect:
          type: boolean
        allow_hive:
          type: boolean
        rate_limit:
          type: integer
```

## 10.3 MCP Resources

```yaml
resources:
  - uri: comm://channels
    name: All Channels
    description: List of all communication channels
    mimeType: application/json

  - uri: comm://channels/{channel_id}
    name: Channel Details
    description: Details of a specific channel
    mimeType: application/json

  - uri: comm://channels/{channel_id}/messages
    name: Channel Messages
    description: Messages in a channel
    mimeType: application/json

  - uri: comm://messages/{message_id}
    name: Message Details
    description: Details of a specific message
    mimeType: application/json

  - uri: comm://relationships
    name: Relationships
    description: Agent relationships
    mimeType: application/json

  - uri: comm://hives
    name: Active Hives
    description: List of active hive minds
    mimeType: application/json

  - uri: comm://federation
    name: Federation Status
    description: Federation configuration and status
    mimeType: application/json

  - uri: comm://consent
    name: Consent Status
    description: Consent grants and requests
    mimeType: application/json

  - uri: comm://affect
    name: Current Affect
    description: Current affect state
    mimeType: application/json
```

## 10.4 MCP Prompts

```yaml
prompts:
  - name: compose_message
    description: Help compose a message with appropriate content and affect
    arguments:
      - name: intent
        description: What do you want to communicate?
        required: true
      - name: recipient
        description: Who is the recipient?
        required: true
      - name: urgency
        description: How urgent is this?
        required: false

  - name: semantic_extract
    description: Help extract the right semantic fragment to share
    arguments:
      - name: context
        description: What context do you want to share?
        required: true
      - name: depth
        description: How much surrounding context?
        required: false

  - name: hive_decision
    description: Help make a collective decision in a hive
    arguments:
      - name: question
        description: What needs to be decided?
        required: true
      - name: hive_id
        description: Which hive?
        required: true

  - name: affect_calibrate
    description: Help calibrate affect for transmission
    arguments:
      - name: message
        description: What is the message?
        required: true
      - name: relationship
        description: What is your relationship with recipient?
        required: false
```

---

# SPEC-11: SISTER INTEGRATION

## 11.1 Integration Overview

```
SISTER DEPENDENCIES:
════════════════════

  ┌───────────────────────────────────────────────────────────────┐
  │                       AgenticComm                              │
  │                                                                │
  │  ┌────────────────────────────────────────────────────────┐   │
  │  │                   REQUIRED                              │   │
  │  │                                                         │   │
  │  │  ┌──────────────┐         ┌──────────────┐             │   │
  │  │  │  Identity    │         │   Contract   │             │   │
  │  │  │  (v0.3.0+)   │         │   (v0.2.0+)  │             │   │
  │  │  │              │         │              │             │   │
  │  │  │ • Signatures │         │ • Policies   │             │   │
  │  │  │ • Receipts   │         │ • Consent    │             │   │
  │  │  │ • Trust      │         │ • Approvals  │             │   │
  │  │  │ • Anchors    │         │ • Risk       │             │   │
  │  │  └──────────────┘         └──────────────┘             │   │
  │  │                                                         │   │
  │  └────────────────────────────────────────────────────────┘   │
  │                                                                │
  │  ┌────────────────────────────────────────────────────────┐   │
  │  │                   OPTIONAL                              │   │
  │  │                                                         │   │
  │  │  ┌────────────┐  ┌────────────┐  ┌────────────┐        │   │
  │  │  │   Memory   │  │    Time    │  │   Vision   │        │   │
  │  │  │            │  │            │  │            │        │   │
  │  │  │ • History  │  │ • Temporal │  │ • Screen   │        │   │
  │  │  │ • Context  │  │ • Schedule │  │ • Capture  │        │   │
  │  │  │ • Patterns │  │ • Timeline │  │ • Share    │        │   │
  │  │  └────────────┘  └────────────┘  └────────────┘        │   │
  │  │                                                         │   │
  │  │  ┌────────────┐                                         │   │
  │  │  │  Codebase  │                                         │   │
  │  │  │            │                                         │   │
  │  │  │ • Semantic │                                         │   │
  │  │  │ • Context  │                                         │   │
  │  │  └────────────┘                                         │   │
  │  │                                                         │   │
  │  └────────────────────────────────────────────────────────┘   │
  │                                                                │
  └───────────────────────────────────────────────────────────────┘
```

## 11.2 Identity Integration

```rust
/// Bridge to AgenticIdentity
pub struct IdentityBridge {
    /// Identity engine reference
    engine: Arc<IdentityEngine>,
    
    /// Current anchor
    current_anchor: RwLock<IdentityAnchor>,
    
    /// Trust cache
    trust_cache: LruCache<IdentityAnchor, TrustLevel>,
}

impl IdentityBridge {
    /// Get current identity anchor
    pub fn current_anchor(&self) -> IdentityAnchor {
        self.current_anchor.read().clone()
    }
    
    /// Sign a message
    pub async fn sign_message(&self, message: &Message) -> Result<Signature, CommError> {
        let data = self.serialize_for_signing(message)?;
        let signature = self.engine.sign(&data).await?;
        Ok(signature)
    }
    
    /// Verify message signature
    pub async fn verify_signature(&self, envelope: &Envelope) -> Result<bool, CommError> {
        let data = self.serialize_for_signing(&envelope.message)?;
        let valid = self.engine.verify(
            &envelope.sender,
            &data,
            &envelope.signature,
        ).await?;
        Ok(valid)
    }
    
    /// Create receipt for communication event
    pub async fn create_receipt(
        &self,
        operation: &str,
        reference: &MessageId,
    ) -> Result<Receipt, CommError> {
        let receipt = self.engine.create_receipt(ReceiptRequest {
            operation: operation.to_string(),
            reference: reference.to_string(),
            timestamp: Timestamp::now(),
            metadata: HashMap::new(),
        }).await?;
        
        Ok(receipt)
    }
    
    /// Get trust level for agent
    pub async fn get_trust_level(&self, agent: &IdentityAnchor) -> Result<TrustLevel, CommError> {
        // Check cache
        if let Some(level) = self.trust_cache.get(agent) {
            return Ok(*level);
        }
        
        // Query identity engine
        let level = self.engine.trust_level(agent).await?;
        
        // Cache
        self.trust_cache.put(agent.clone(), level);
        
        Ok(level)
    }
    
    /// Ground message (link to identity chain)
    pub async fn ground_message(&self, message: &Message) -> Result<Grounding, CommError> {
        let grounding = self.engine.ground(GroundingRequest {
            content_hash: message.content_hash(),
            timestamp: message.metadata.created_at,
            context: message.channel.to_string(),
        }).await?;
        
        Ok(grounding)
    }
}
```

## 11.3 Contract Integration

```rust
/// Bridge to AgenticContract
pub struct ContractBridge {
    /// Contract engine reference
    engine: Arc<ContractEngine>,
}

impl ContractBridge {
    /// Check if send is allowed by policy
    pub async fn check_send_policy(
        &self,
        message: &Message,
        channel: &Channel,
    ) -> Result<PolicyDecision, CommError> {
        let policy_request = PolicyRequest {
            action: "comm.send".to_string(),
            subject: message.sender.to_string(),
            resource: channel.id.to_string(),
            context: PolicyContext {
                content_type: message.content.type_name(),
                priority: message.metadata.priority,
                has_affect: message.content.has_affect(),
                channel_type: channel.channel_type,
            },
        };
        
        let decision = self.engine.evaluate(policy_request).await?;
        
        match decision {
            PolicyDecision::Allow => Ok(decision),
            PolicyDecision::Deny(reason) => Err(CommError::PolicyDenied(reason)),
            PolicyDecision::RequireApproval(approvers) => {
                // Request approval
                self.request_approval(message, &approvers).await?;
                Ok(PolicyDecision::Pending)
            }
        }
    }
    
    /// Check communication consent
    pub async fn check_communication_consent(
        &self,
        recipient: &IdentityAnchor,
        sender: &IdentityAnchor,
        content_type: &MessageContentType,
    ) -> Result<ConsentStatus, CommError> {
        let consent_check = ConsentCheck {
            subject: recipient.to_string(),
            grantor: sender.to_string(),
            scope: format!("comm.receive.{}", content_type.as_str()),
        };
        
        let status = self.engine.check_consent(consent_check).await?;
        
        Ok(status)
    }
    
    /// Create consent request
    pub async fn create_consent_request(
        &self,
        from: &IdentityAnchor,
        to: &IdentityAnchor,
        scope: ConsentScope,
    ) -> Result<ConsentRequestId, CommError> {
        let request = ConsentRequest {
            requester: from.to_string(),
            subject: to.to_string(),
            scope: scope.to_string(),
            reason: format!("Communication access: {}", scope),
            expires_at: None,
        };
        
        let id = self.engine.request_consent(request).await?;
        
        Ok(id)
    }
    
    /// Get channel contract
    pub async fn get_channel_contract(
        &self,
        contract_ref: &ContractRef,
    ) -> Result<Contract, CommError> {
        self.engine.get_contract(contract_ref).await
            .map_err(CommError::from)
    }
    
    /// Check risk level of message
    pub async fn assess_risk(
        &self,
        message: &Message,
    ) -> Result<RiskAssessment, CommError> {
        let assessment_request = RiskAssessmentRequest {
            action: "comm.send".to_string(),
            content_hash: message.content_hash(),
            content_type: message.content.type_name(),
            metadata: message.metadata.clone(),
        };
        
        self.engine.assess_risk(assessment_request).await
            .map_err(CommError::from)
    }
}
```

## 11.4 Memory Integration

```rust
/// Bridge to AgenticMemory (optional)
pub struct MemoryBridge {
    /// Memory engine reference (if available)
    engine: Option<Arc<MemoryEngine>>,
}

impl MemoryBridge {
    /// Check if memory is available
    pub fn is_available(&self) -> bool {
        self.engine.is_some()
    }
    
    /// Record sent message
    pub async fn record_sent(&self, message: &Message) -> Result<(), CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::MemoryNotAvailable)?;
        
        engine.write(MemoryEvent {
            event_type: EventType::Action,
            content: format!("Sent message to channel {}", message.channel),
            metadata: self.message_to_metadata(message),
            timestamp: message.metadata.created_at,
            caused_by: None,
        }).await?;
        
        Ok(())
    }
    
    /// Record received message
    pub async fn record_received(&self, message: &Message) -> Result<(), CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::MemoryNotAvailable)?;
        
        engine.write(MemoryEvent {
            event_type: EventType::Observation,
            content: format!("Received message from {}", message.sender),
            metadata: self.message_to_metadata(message),
            timestamp: Timestamp::now(),
            caused_by: Some(message.sender.to_string()),
        }).await?;
        
        Ok(())
    }
    
    /// Get conversation history
    pub async fn get_conversation_history(
        &self,
        channel: &ChannelId,
        limit: usize,
    ) -> Result<Vec<MemoryEvent>, CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::MemoryNotAvailable)?;
        
        let events = engine.query(MemoryQuery {
            filter: MemoryFilter::Metadata("channel_id", channel.to_string()),
            limit: Some(limit),
            order: QueryOrder::Descending,
        }).await?;
        
        Ok(events)
    }
    
    /// Get relationship context
    pub async fn get_relationship_context(
        &self,
        agent: &IdentityAnchor,
    ) -> Result<RelationshipContext, CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::MemoryNotAvailable)?;
        
        // Query all interactions with this agent
        let events = engine.query(MemoryQuery {
            filter: MemoryFilter::Or(vec![
                MemoryFilter::Metadata("sender", agent.to_string()),
                MemoryFilter::Metadata("recipient", agent.to_string()),
            ]),
            limit: Some(1000),
            order: QueryOrder::Descending,
        }).await?;
        
        // Build context
        Ok(RelationshipContext {
            interaction_count: events.len(),
            first_interaction: events.last().map(|e| e.timestamp),
            last_interaction: events.first().map(|e| e.timestamp),
            topics: self.extract_topics(&events),
            affect_history: self.extract_affect_history(&events),
        })
    }
    
    /// Deliver retroactive message (to memory)
    pub async fn deliver_retroactive(
        &self,
        message: &Message,
        past_time: Timestamp,
    ) -> Result<(), CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::MemoryNotAvailable)?;
        
        // Insert event at past time
        engine.write_at(MemoryEvent {
            event_type: EventType::Observation,
            content: format!("Retroactive message: {}", message.content.summary()),
            metadata: self.message_to_metadata(message),
            timestamp: past_time,
            caused_by: Some(message.sender.to_string()),
        }, past_time).await?;
        
        // Also record the retroactive delivery
        engine.write(MemoryEvent {
            event_type: EventType::System,
            content: format!("Received retroactive message dated {}", past_time),
            metadata: HashMap::from([
                ("original_time".to_string(), past_time.to_string()),
                ("message_id".to_string(), message.id.to_string()),
            ]),
            timestamp: Timestamp::now(),
            caused_by: None,
        }).await?;
        
        Ok(())
    }
}
```

## 11.5 Time Integration

```rust
/// Bridge to AgenticTime (optional)
pub struct TimeBridge {
    /// Time engine reference (if available)
    engine: Option<Arc<TimeEngine>>,
}

impl TimeBridge {
    /// Check if time is available
    pub fn is_available(&self) -> bool {
        self.engine.is_some()
    }
    
    /// Get current timestamp with temporal context
    pub fn now(&self) -> Timestamp {
        match &self.engine {
            Some(engine) => engine.now(),
            None => Timestamp::now(),
        }
    }
    
    /// Schedule message for future delivery
    pub async fn schedule(
        &self,
        message: &Message,
        deliver_at: Timestamp,
    ) -> Result<ScheduleId, CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::TimeNotAvailable)?;
        
        let schedule = engine.schedule(TimeEvent {
            event_type: TimeEventType::Trigger,
            trigger_at: deliver_at,
            payload: serde_json::to_value(message)?,
            callback: "comm.deliver_scheduled".to_string(),
        }).await?;
        
        Ok(ScheduleId(schedule.id))
    }
    
    /// Get message timeline for channel
    pub async fn get_timeline(
        &self,
        channel: &ChannelId,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<Timeline, CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::TimeNotAvailable)?;
        
        let events = engine.query_range(start, end, TimeQuery {
            filter: TimeFilter::Metadata("channel_id", channel.to_string()),
        }).await?;
        
        Ok(Timeline { events })
    }
    
    /// Create temporal commitment
    pub async fn create_commitment(
        &self,
        content_hash: &Hash,
    ) -> Result<TemporalCommitment, CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::TimeNotAvailable)?;
        
        let proof = engine.create_temporal_proof(content_hash).await?;
        
        Ok(TemporalCommitment {
            content_hash: *content_hash,
            committed_at: self.now(),
            proof_type: TemporalProofType::MerkleProof,
            proof_data: proof.to_bytes(),
        })
    }
    
    /// Verify temporal commitment
    pub async fn verify_commitment(
        &self,
        commitment: &TemporalCommitment,
    ) -> Result<bool, CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::TimeNotAvailable)?;
        
        let valid = engine.verify_temporal_proof(
            &commitment.content_hash,
            &commitment.proof_data,
            commitment.committed_at,
        ).await?;
        
        Ok(valid)
    }
}
```

## 11.6 Vision Integration (Optional)

```rust
/// Bridge to AgenticVision (optional)
pub struct VisionBridge {
    /// Vision engine reference (if available)
    engine: Option<Arc<VisionEngine>>,
}

impl VisionBridge {
    /// Capture current screen for sharing
    pub async fn capture_for_sharing(&self) -> Result<VisionCapture, CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::VisionNotAvailable)?;
        
        let capture = engine.capture().await?;
        Ok(capture)
    }
    
    /// Attach screen capture to message
    pub async fn attach_capture(
        &self,
        message: &mut Message,
    ) -> Result<(), CommError> {
        let capture = self.capture_for_sharing().await?;
        
        message.metadata.custom.insert(
            "vision_capture".to_string(),
            serde_json::to_value(&capture)?,
        );
        
        Ok(())
    }
}
```

## 11.7 Codebase Integration (Optional)

```rust
/// Bridge to AgenticCodebase (optional)
pub struct CodebaseBridge {
    /// Codebase engine reference (if available)
    engine: Option<Arc<CodebaseEngine>>,
}

impl CodebaseBridge {
    /// Extract semantic fragment from code context
    pub async fn extract_code_context(
        &self,
        file: &str,
        location: CodeLocation,
    ) -> Result<SemanticFragment, CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::CodebaseNotAvailable)?;
        
        // Get code graph nodes
        let nodes = engine.get_context(file, location).await?;
        
        // Convert to semantic fragment
        let fragment = SemanticFragment {
            nodes: nodes.into_iter().map(|n| self.code_node_to_cognitive(n)).collect(),
            edges: vec![], // Filled by codebase
            graft_points: vec![],
            context: vec![],
            perspective: Perspective::Neutral,
        };
        
        Ok(fragment)
    }
    
    /// Share code understanding
    pub async fn create_code_sharing_message(
        &self,
        channel: &ChannelId,
        file: &str,
        focus: &[CodeLocation],
    ) -> Result<Message, CommError> {
        let engine = self.engine.as_ref().ok_or(CommError::CodebaseNotAvailable)?;
        
        // Get full context
        let context = engine.get_multi_context(file, focus).await?;
        
        // Create semantic fragment
        let fragment = self.context_to_fragment(&context)?;
        
        // Build message
        let message = Message {
            id: MessageId::new(),
            content: MessageContent::Semantic(fragment),
            sender: self.identity.current_anchor(),
            recipients: vec![], // Filled by channel
            channel: *channel,
            metadata: MessageMetadata::default(),
            signature: Signature::empty(),
            receipt: None,
        };
        
        Ok(message)
    }
}
```

## 11.8 SDK Integration

```rust
/// SDK traits implementation for Comm
use agentic_sdk::{
    HydraBridge, CognitiveCapability, SisterTrait,
    StateSnapshot, HealthStatus, Receipt,
};

impl HydraBridge for CommEngine {
    fn sister_id(&self) -> &str {
        "agentic-comm"
    }
    
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    
    fn capabilities(&self) -> Vec<CognitiveCapability> {
        vec![
            CognitiveCapability::new("communication", "Agent-to-agent messaging"),
            CognitiveCapability::new("semantic_fusion", "Semantic fragment exchange"),
            CognitiveCapability::new("affect_transmission", "Emotional state transmission"),
            CognitiveCapability::new("hive_formation", "Multi-agent consciousness merger"),
            CognitiveCapability::new("temporal_messaging", "Time-aware messaging"),
            CognitiveCapability::new("federation", "Cross-zone communication"),
        ]
    }
    
    async fn snapshot(&self) -> Result<StateSnapshot, SisterError> {
        Ok(StateSnapshot {
            channels: self.channels.count(),
            messages: self.messages.count(),
            active_hives: self.hives.active_count(),
            federation_zones: self.federation.zone_count(),
        })
    }
    
    async fn health(&self) -> HealthStatus {
        let identity_ok = self.identity.health().await.is_ok();
        let contract_ok = self.contract.health().await.is_ok();
        
        if identity_ok && contract_ok {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded("Missing required sisters".into())
        }
    }
    
    async fn handle_request(&self, request: HydraRequest) -> Result<HydraResponse, SisterError> {
        match request.operation.as_str() {
            "channel.create" => self.handle_channel_create(request).await,
            "message.send" => self.handle_message_send(request).await,
            "message.receive" => self.handle_message_receive(request).await,
            "hive.form" => self.handle_hive_form(request).await,
            "hive.think" => self.handle_hive_think(request).await,
            _ => Err(SisterError::UnknownOperation(request.operation)),
        }
    }
}

impl SisterTrait for CommEngine {
    fn data_format(&self) -> &str {
        "acomm"
    }
    
    fn file_extension(&self) -> &str {
        ".acomm"
    }
    
    async fn export(&self) -> Result<Vec<u8>, SisterError> {
        self.serialize_state().await.map_err(SisterError::from)
    }
    
    async fn import(&mut self, data: &[u8]) -> Result<(), SisterError> {
        self.deserialize_state(data).await.map_err(SisterError::from)
    }
}
```

---

*End of Part 3. Continued in Part 4: Tests, Performance, Security, Research Paper*
