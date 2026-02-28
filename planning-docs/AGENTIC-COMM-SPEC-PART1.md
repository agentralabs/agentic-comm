# AgenticComm Specification — Part 1

> **Version:** 0.1.0
> **Status:** Pre-Implementation Specification
> **Covers:** Overview, Core Concepts, Data Structures, File Format

---

# SPEC-01: OVERVIEW

## Executive Summary

AgenticComm is the seventh sister in the Agentic Revolution ecosystem. It provides **agent-to-agent communication** with capabilities far beyond traditional message passing: encrypted channels, semantic fusion, capability negotiation, consent-gated federation, and temporal messaging.

### The Problem

```
CURRENT STATE OF AGENT COMMUNICATION:
═════════════════════════════════════

1. NO STANDARD PROTOCOL
   └── Every multi-agent system invents its own
   └── No interoperability between systems
   └── No semantic layer

2. NO SECURITY
   └── Messages sent in plaintext
   └── No authentication
   └── No capability negotiation
   └── No consent gates

3. NO PERSISTENCE
   └── Messages are ephemeral
   └── No conversation memory
   └── No relationship tracking

4. NO INTELLIGENCE
   └── Dumb pipes
   └── No semantic understanding
   └── No affect transmission
   └── No anticipation

5. TEXT BOTTLENECK
   └── Agents serialize to text
   └── Other agent parses text
   └── Meaning lost in translation
```

### The Solution

```
AGENTICCOMM PROVIDES:
═════════════════════

1. SEMANTIC COMMUNICATION
   └── Transmit meaning, not text
   └── Semantic fragments graft onto receiver's graph
   └── Zero interpretation loss

2. CRYPTOGRAPHIC SECURITY
   └── End-to-end encryption
   └── Identity-signed messages
   └── Capability-gated channels
   └── Consent-required federation

3. PERSISTENT RELATIONSHIPS
   └── Channels maintain state
   └── Conversation history in Memory
   └── Trust relationships tracked

4. INTELLIGENT MESSAGING
   └── Affect transmission
   └── Anticipatory understanding
   └── Temporal message scheduling
   └── Oracle integration

5. NATIVE FORMAT
   └── .acomm binary format
   └── Sub-millisecond operations
   └── Zero-copy where possible
```

### Dependencies

```
REQUIRED SISTERS:
═════════════════

AgenticIdentity (v0.3.0+)
├── Cryptographic signatures for messages
├── Trust anchors for channels
├── Receipts for communication events
└── Capability tokens for federation

AgenticContract (v0.2.0+)
├── Policies for communication rules
├── Consent gates for federation
├── Risk limits for message types
└── Approval workflows for sensitive comms

OPTIONAL INTEGRATIONS:
══════════════════════

AgenticMemory (v0.4.0+)
├── Conversation persistence
├── Relationship memory
├── Pattern recognition
└── Semantic indexing

AgenticTime (v0.1.0+)
├── Temporal message scheduling
├── Conversation timeline
├── Expiry management
└── Temporal consensus
```

### Core Principles

```
1. SEMANTIC FIRST
   Messages carry meaning, not just bytes.
   Receivers understand, not just parse.

2. CONSENT REQUIRED
   No unsolicited communication.
   Explicit opt-in for all channels.

3. CRYPTOGRAPHIC TRUST
   Every message signed.
   Every channel encrypted.
   Every action receipted.

4. RELATIONSHIP AWARE
   Communication builds relationships.
   Relationships affect communication.
   Trust grows with interaction.

5. TEMPORAL CAPABLE
   Messages can target any time.
   Conversations span timelines.
   History is navigable.

6. FEDERATION READY
   Agents across trust boundaries.
   Cross-system communication.
   Decentralized by design.
```

---

# SPEC-02: CORE CONCEPTS

## 2.1 Fundamental Entities

### Agent Identity

```rust
/// An agent that can communicate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicatingAgent {
    /// Identity anchor from AgenticIdentity
    pub identity: IdentityAnchor,
    
    /// Communication capabilities
    pub capabilities: CommCapabilities,
    
    /// Trust profile for communication
    pub trust_profile: CommTrustProfile,
    
    /// Current availability
    pub availability: Availability,
    
    /// Preferred communication modes
    pub preferences: CommPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommCapabilities {
    /// Can receive semantic fragments
    pub semantic_receive: bool,
    
    /// Can transmit affect
    pub affect_capable: bool,
    
    /// Can participate in hive minds
    pub hive_capable: bool,
    
    /// Can handle temporal messages
    pub temporal_capable: bool,
    
    /// Maximum message size
    pub max_message_size: usize,
    
    /// Supported encryption schemes
    pub encryption_schemes: Vec<EncryptionScheme>,
    
    /// Protocol versions supported
    pub protocol_versions: Vec<ProtocolVersion>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Availability {
    /// Fully available for communication
    Available,
    
    /// Available but busy
    Busy,
    
    /// Only urgent messages
    DoNotDisturb,
    
    /// Not accepting messages
    Unavailable,
    
    /// Agent is dormant/offline
    Offline,
}
```

### Channels

```rust
/// A communication channel between agents
#[derive(Debug, Clone)]
pub struct Channel {
    /// Unique channel identifier
    pub id: ChannelId,
    
    /// Channel type
    pub channel_type: ChannelType,
    
    /// Participants
    pub participants: Vec<ChannelParticipant>,
    
    /// Channel state
    pub state: ChannelState,
    
    /// Encryption configuration
    pub encryption: ChannelEncryption,
    
    /// Associated contract (policies)
    pub contract: Option<ContractRef>,
    
    /// Channel metadata
    pub metadata: ChannelMetadata,
    
    /// Creation timestamp
    pub created_at: Timestamp,
    
    /// Last activity
    pub last_activity: Timestamp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChannelType {
    /// Direct channel between two agents
    Direct,
    
    /// Group channel with multiple agents
    Group,
    
    /// Broadcast channel (one to many)
    Broadcast,
    
    /// Telepathic channel (shared cognitive space)
    Telepathic,
    
    /// Hive channel (merged consciousness)
    Hive,
    
    /// Temporal channel (time-spanning)
    Temporal,
    
    /// Destiny channel (purpose-driven)
    Destiny,
    
    /// Oracle channel (external knowledge)
    Oracle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelParticipant {
    /// The agent
    pub agent: IdentityAnchor,
    
    /// Role in channel
    pub role: ParticipantRole,
    
    /// Permissions
    pub permissions: ChannelPermissions,
    
    /// Join timestamp
    pub joined_at: Timestamp,
    
    /// Last seen
    pub last_seen: Timestamp,
    
    /// Contribution count
    pub contributions: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ParticipantRole {
    /// Created the channel
    Owner,
    
    /// Can manage channel
    Admin,
    
    /// Regular participant
    Member,
    
    /// Can only read
    Observer,
    
    /// Temporarily muted
    Muted,
    
    /// Special: Oracle role
    Oracle,
    
    /// Special: Ghost (deceased agent)
    Ghost,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChannelState {
    /// Channel is active
    Active,
    
    /// Channel is paused
    Paused,
    
    /// Channel is archived
    Archived,
    
    /// Channel is in silent communion
    SilentCommunion,
    
    /// Channel is in hive mode
    HiveMode,
    
    /// Channel is awaiting consent
    PendingConsent,
    
    /// Channel is closed
    Closed,
}
```

### Messages

```rust
/// A message in the communication system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: MessageId,
    
    /// Message content
    pub content: MessageContent,
    
    /// Sender
    pub sender: IdentityAnchor,
    
    /// Recipients
    pub recipients: Vec<Recipient>,
    
    /// Channel this belongs to
    pub channel: ChannelId,
    
    /// Message metadata
    pub metadata: MessageMetadata,
    
    /// Cryptographic signature
    pub signature: Signature,
    
    /// Receipt from Identity sister
    pub receipt: Option<Receipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    /// Plain text message
    Text(TextMessage),
    
    /// Semantic fragment
    Semantic(SemanticFragment),
    
    /// Affect transmission
    Affect(AffectPayload),
    
    /// Combined semantic + affect
    Full(FullPayload),
    
    /// System message
    System(SystemMessage),
    
    /// Temporal message
    Temporal(TemporalPayload),
    
    /// Precognitive message
    Precognitive(PrecognitivePayload),
    
    /// Legacy message (death-triggered)
    Legacy(LegacyPayload),
    
    /// Meta message
    Meta(MetaPayload),
    
    /// Unspeakable (special encoding)
    Unspeakable(UnspeakablePayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessage {
    /// The text
    pub text: String,
    
    /// Language
    pub language: Option<String>,
    
    /// Formatting hints
    pub formatting: Option<Formatting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFragment {
    /// Nodes being transmitted
    pub nodes: Vec<CognitiveNode>,
    
    /// Edges between nodes
    pub edges: Vec<CognitiveEdge>,
    
    /// Graft points for receiver
    pub graft_points: Vec<GraftPoint>,
    
    /// Context anchors
    pub context: Vec<ContextAnchor>,
    
    /// Perspective framing
    pub perspective: Perspective,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectPayload {
    /// Affect state
    pub affect: AffectState,
    
    /// Contagion strength (how much should transfer)
    pub contagion_strength: f64,
    
    /// Can receiver resist?
    pub resistable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Creation timestamp
    pub created_at: Timestamp,
    
    /// Expiry (if any)
    pub expires_at: Option<Timestamp>,
    
    /// Priority
    pub priority: MessagePriority,
    
    /// Requires acknowledgment
    pub requires_ack: bool,
    
    /// Thread reference
    pub thread: Option<ThreadId>,
    
    /// Reply to
    pub reply_to: Option<MessageId>,
    
    /// Tags
    pub tags: Vec<String>,
    
    /// Custom metadata
    pub custom: HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Can wait indefinitely
    Low,
    
    /// Normal priority
    Normal,
    
    /// Should be seen soon
    High,
    
    /// Needs immediate attention
    Urgent,
    
    /// Drop everything
    Critical,
}
```

## 2.2 Communication Patterns

### Direct Communication

```
DIRECT (1:1):
═════════════

  Agent A                    Agent B
     │                          │
     │──── [Message] ──────────▶│
     │                          │
     │◀─── [Ack/Reply] ─────────│
     │                          │

- Encrypted with shared key
- Signed by sender
- Receipted in both agents
```

### Group Communication

```
GROUP (N:N):
════════════

  Agent A ──┐
            │
  Agent B ──┼──── [Channel] ────┬── Agent D
            │                   │
  Agent C ──┘                   └── Agent E

- All messages to all participants
- Encrypted with group key
- Key rotation on membership change
```

### Broadcast Communication

```
BROADCAST (1:N):
════════════════

              ┌──▶ Agent B
              │
  Agent A ────┼──▶ Agent C
  (Sender)    │
              ├──▶ Agent D
              │
              └──▶ Agent E

- One sender, many receivers
- Receivers cannot reply in channel
- Efficient for announcements
```

### Telepathic Communication

```
TELEPATHIC (Shared Space):
══════════════════════════

     ┌─────────────────────────────┐
     │   SHARED COGNITIVE SPACE    │
     │                             │
     │  ┌─────┐       ┌─────┐     │
     │  │  A  │◀─────▶│  B  │     │
     │  └─────┘       └─────┘     │
     │       ╲         ╱          │
     │        ╲       ╱           │
     │         ╲     ╱            │
     │        ┌─────┐             │
     │        │  C  │             │
     │        └─────┘             │
     │                             │
     └─────────────────────────────┘

- Persistent shared state
- Changes propagate instantly
- Emergent concepts form
```

## 2.3 Trust and Consent

### Trust Levels

```rust
/// Trust levels for communication
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub enum CommTrustLevel {
    /// No trust - only system messages
    None = 0,
    
    /// Minimal trust - text only, rate limited
    Minimal = 1,
    
    /// Basic trust - full text, attachments
    Basic = 2,
    
    /// Standard trust - semantic fragments
    Standard = 3,
    
    /// High trust - affect transmission
    High = 4,
    
    /// Full trust - hive/meld capable
    Full = 5,
    
    /// Absolute - no restrictions
    Absolute = 6,
}
```

### Consent Gates

```rust
/// Consent requirements for communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentGate {
    /// What requires consent
    pub scope: ConsentScope,
    
    /// Current consent status
    pub status: ConsentStatus,
    
    /// When consent was given/denied
    pub decided_at: Option<Timestamp>,
    
    /// Expiry of consent
    pub expires_at: Option<Timestamp>,
    
    /// Conditions on consent
    pub conditions: Vec<ConsentCondition>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConsentScope {
    /// Consent to receive any message
    ReceiveMessages,
    
    /// Consent to join channel
    JoinChannel,
    
    /// Consent to receive semantic fragments
    ReceiveSemantic,
    
    /// Consent to receive affect
    ReceiveAffect,
    
    /// Consent to telepathic channel
    TelepathicAccess,
    
    /// Consent to hive formation
    HiveFormation,
    
    /// Consent to mind meld
    MindMeld,
    
    /// Consent to federation
    Federation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConsentStatus {
    /// Not yet asked
    Pending,
    
    /// Explicitly granted
    Granted,
    
    /// Explicitly denied
    Denied,
    
    /// Revoked after granting
    Revoked,
    
    /// Expired
    Expired,
}
```

## 2.4 Federation

### Federation Model

```
FEDERATION ACROSS TRUST BOUNDARIES:
═══════════════════════════════════

  ┌─────────────────┐           ┌─────────────────┐
  │   TRUST ZONE A  │           │   TRUST ZONE B  │
  │                 │           │                 │
  │  Agent A1       │           │       Agent B1  │
  │       │         │           │         │       │
  │  Agent A2       │           │       Agent B2  │
  │       │         │           │         │       │
  │  Agent A3       │           │       Agent B3  │
  │       │         │           │         │       │
  │       ▼         │           │         ▼       │
  │  ┌─────────┐    │           │    ┌─────────┐  │
  │  │ Gateway │◀───┼───────────┼───▶│ Gateway │  │
  │  └─────────┘    │           │    └─────────┘  │
  │                 │           │                 │
  └─────────────────┘           └─────────────────┘

  Gateways negotiate:
    - Trust levels
    - Allowed message types
    - Rate limits
    - Consent requirements
```

### Federation Protocol

```rust
/// Federation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// This zone's identity
    pub zone_identity: ZoneIdentity,
    
    /// Federated zones
    pub federated_zones: Vec<FederatedZone>,
    
    /// Default policy for unknown zones
    pub default_policy: FederationPolicy,
    
    /// Gateway configuration
    pub gateway: GatewayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedZone {
    /// Zone identity
    pub identity: ZoneIdentity,
    
    /// Trust level with this zone
    pub trust_level: CommTrustLevel,
    
    /// Policy for this zone
    pub policy: FederationPolicy,
    
    /// Gateway endpoint
    pub gateway_endpoint: String,
    
    /// Last successful communication
    pub last_contact: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationPolicy {
    /// Allowed message types
    pub allowed_types: Vec<MessageContentType>,
    
    /// Rate limits
    pub rate_limit: RateLimit,
    
    /// Require consent for each message?
    pub per_message_consent: bool,
    
    /// Allow semantic fragments?
    pub allow_semantic: bool,
    
    /// Allow affect?
    pub allow_affect: bool,
    
    /// Allow hive formation across federation?
    pub allow_cross_hive: bool,
}
```

---

# SPEC-03: DATA STRUCTURES

## 3.1 Core Types

### Identifiers

```rust
/// Channel identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub Uuid);

/// Message identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

/// Thread identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThreadId(pub Uuid);

/// Conversation identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConversationId(pub Uuid);

/// Federation zone identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZoneIdentity(pub String);

/// Hive mind identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HiveId(pub Uuid);

/// Oracle identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OracleId(pub Uuid);
```

### Timestamps and Durations

```rust
/// Timestamp in the communication system
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CommTimestamp {
    /// Unix timestamp in nanoseconds
    pub nanos: i128,
    
    /// Logical clock component (Lamport)
    pub logical: u64,
    
    /// Vector clock component (optional)
    pub vector: Option<VectorClock>,
}

/// Vector clock for distributed ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorClock {
    /// Clock values per agent
    pub clocks: HashMap<IdentityAnchor, u64>,
}

impl VectorClock {
    /// Increment this agent's clock
    pub fn increment(&mut self, agent: &IdentityAnchor) {
        *self.clocks.entry(agent.clone()).or_insert(0) += 1;
    }
    
    /// Merge with another vector clock
    pub fn merge(&mut self, other: &VectorClock) {
        for (agent, &clock) in &other.clocks {
            let entry = self.clocks.entry(agent.clone()).or_insert(0);
            *entry = (*entry).max(clock);
        }
    }
    
    /// Check if this clock happened before another
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        let mut dominated = false;
        for (agent, &clock) in &self.clocks {
            let other_clock = other.clocks.get(agent).copied().unwrap_or(0);
            if clock > other_clock {
                return false;
            }
            if clock < other_clock {
                dominated = true;
            }
        }
        dominated
    }
}
```

### Encryption Types

```rust
/// Encryption schemes supported
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EncryptionScheme {
    /// No encryption
    None,
    
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
    
    /// AES-256-GCM
    Aes256Gcm,
    
    /// XChaCha20-Poly1305 (extended nonce)
    XChaCha20Poly1305,
}

/// Channel encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelEncryption {
    /// Encryption scheme
    pub scheme: EncryptionScheme,
    
    /// Key derivation info
    pub key_derivation: KeyDerivation,
    
    /// Key rotation policy
    pub rotation_policy: KeyRotationPolicy,
    
    /// Current key epoch
    pub key_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivation {
    /// Method used
    pub method: KeyDerivationMethod,
    
    /// Salt
    pub salt: Vec<u8>,
    
    /// Iterations (if applicable)
    pub iterations: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KeyDerivationMethod {
    /// HKDF with SHA-256
    HkdfSha256,
    
    /// X25519 key agreement
    X25519,
    
    /// Signal protocol double ratchet
    DoubleRatchet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationPolicy {
    /// Rotate after N messages
    pub message_count: Option<u64>,
    
    /// Rotate after duration
    pub time_duration: Option<Duration>,
    
    /// Rotate on membership change
    pub on_membership_change: bool,
}
```

## 3.2 Semantic Types

### Cognitive Nodes

```rust
/// A node in the cognitive graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveNode {
    /// Node identifier
    pub id: NodeId,
    
    /// Node type
    pub node_type: CognitiveNodeType,
    
    /// Node content
    pub content: NodeContent,
    
    /// Confidence in this node
    pub confidence: f64,
    
    /// Source of this node
    pub source: NodeSource,
    
    /// Metadata
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CognitiveNodeType {
    /// A concept
    Concept,
    
    /// A fact
    Fact,
    
    /// A belief
    Belief,
    
    /// A goal
    Goal,
    
    /// An action
    Action,
    
    /// An observation
    Observation,
    
    /// An inference
    Inference,
    
    /// An emotion
    Emotion,
    
    /// A memory reference
    Memory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeContent {
    /// Text content
    Text(String),
    
    /// Structured data
    Structured(Value),
    
    /// Embedding vector
    Embedding(Vec<f32>),
    
    /// Reference to external
    Reference(ExternalRef),
    
    /// Compound content
    Compound(Vec<NodeContent>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSource {
    /// Origin agent
    pub agent: IdentityAnchor,
    
    /// When created
    pub created_at: Timestamp,
    
    /// Derivation chain
    pub derived_from: Vec<NodeId>,
}
```

### Cognitive Edges

```rust
/// An edge in the cognitive graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEdge {
    /// Edge identifier
    pub id: EdgeId,
    
    /// Source node
    pub from: NodeId,
    
    /// Target node
    pub to: NodeId,
    
    /// Edge type
    pub edge_type: CognitiveEdgeType,
    
    /// Edge strength
    pub strength: f64,
    
    /// Metadata
    pub metadata: EdgeMetadata,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CognitiveEdgeType {
    /// Causal relationship
    Causes,
    
    /// Logical implication
    Implies,
    
    /// Part-of relationship
    PartOf,
    
    /// Instance-of relationship
    InstanceOf,
    
    /// Similarity
    SimilarTo,
    
    /// Contrast
    ContrastsWith,
    
    /// Temporal sequence
    FollowedBy,
    
    /// Supports
    Supports,
    
    /// Contradicts
    Contradicts,
    
    /// Derived from
    DerivedFrom,
}
```

## 3.3 Affect Types

```rust
/// Affect state for transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectState {
    /// Valence: -1.0 (negative) to 1.0 (positive)
    pub valence: f64,
    
    /// Arousal: 0.0 (calm) to 1.0 (activated)
    pub arousal: f64,
    
    /// Dominance: 0.0 (submissive) to 1.0 (dominant)
    pub dominance: f64,
    
    /// Discrete emotions
    pub emotions: HashMap<Emotion, f64>,
    
    /// Urgency level
    pub urgency: UrgencyLevel,
    
    /// Meta-confidence (how sure about own affect)
    pub meta_confidence: f64,
    
    /// Triggers that caused this state
    pub triggers: Vec<AffectTrigger>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Emotion {
    // Primary emotions
    Joy,
    Sadness,
    Fear,
    Anger,
    Surprise,
    Disgust,
    
    // Secondary emotions
    Anticipation,
    Trust,
    Curiosity,
    Confusion,
    
    // Cognitive emotions
    Certainty,
    Doubt,
    Insight,
    Frustration,
    
    // Social emotions
    Gratitude,
    Guilt,
    Pride,
    Shame,
    Empathy,
    
    // Existential emotions
    Wonder,
    Dread,
    Hope,
    Despair,
    Awe,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UrgencyLevel {
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
    Apocalyptic = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectTrigger {
    /// What triggered this affect
    pub trigger_type: TriggerType,
    
    /// Reference to trigger source
    pub source: Option<String>,
    
    /// How much this contributed
    pub contribution: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TriggerType {
    /// External event
    ExternalEvent,
    
    /// Internal thought
    InternalThought,
    
    /// Received message
    ReceivedMessage,
    
    /// Affect contagion
    Contagion,
    
    /// Memory recall
    MemoryRecall,
    
    /// Goal progress
    GoalProgress,
    
    /// Social interaction
    SocialInteraction,
}
```

## 3.4 Temporal Types

```rust
/// Temporal message targeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalTarget {
    /// Deliver immediately
    Immediate,
    
    /// Deliver at absolute time
    FutureAbsolute(Timestamp),
    
    /// Deliver after duration
    FutureRelative(Duration),
    
    /// Deliver when condition is true
    Conditional(TemporalCondition),
    
    /// Retroactive delivery (through memory)
    Retroactive(Timestamp),
    
    /// Optimal moment delivery
    Optimal(OptimalityFunction),
    
    /// Eternal (always accessible)
    Eternal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalCondition {
    /// Condition expression
    pub expression: ConditionExpr,
    
    /// Check interval
    pub check_interval: Duration,
    
    /// Timeout
    pub timeout: Option<Timestamp>,
    
    /// Action if timeout
    pub timeout_action: TimeoutAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeoutAction {
    /// Deliver anyway
    DeliverAnyway,
    
    /// Discard message
    Discard,
    
    /// Return to sender
    ReturnToSender,
    
    /// Archive
    Archive,
}

/// Temporal commitment (proof of when message was created)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalCommitment {
    /// Hash of message content
    pub content_hash: Hash,
    
    /// Timestamp of commitment
    pub committed_at: Timestamp,
    
    /// Proof type
    pub proof_type: TemporalProofType,
    
    /// The proof data
    pub proof_data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TemporalProofType {
    /// Signed by trusted time source
    TrustedTimestamp,
    
    /// Blockchain anchor
    BlockchainAnchor,
    
    /// Merkle tree in time log
    MerkleProof,
    
    /// Witness signatures
    WitnessSignatures,
}
```

## 3.5 Hive and Collective Types

```rust
/// Hive mind structure
#[derive(Debug, Clone)]
pub struct HiveMind {
    /// Hive identifier
    pub id: HiveId,
    
    /// Constituent agents
    pub constituents: Vec<HiveConstituent>,
    
    /// Unified cognitive space
    pub unified_space: UnifiedCognitiveSpace,
    
    /// Hive identity (the emergent entity)
    pub hive_identity: HiveIdentity,
    
    /// Emergent capabilities
    pub emergent_capabilities: Vec<EmergentCapability>,
    
    /// Formation timestamp
    pub formed_at: Timestamp,
    
    /// Coherence level
    pub coherence: f64,
    
    /// Separation policy
    pub separation_policy: SeparationPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveConstituent {
    /// Original agent identity
    pub identity: IdentityAnchor,
    
    /// Role in hive
    pub role: HiveRole,
    
    /// Preserved individual aspects
    pub preserved: PreservedIndividuality,
    
    /// Contribution to hive
    pub contribution: f64,
    
    /// Can dissent?
    pub can_dissent: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HiveRole {
    /// Initiated hive formation
    Initiator,
    
    /// Core member
    Core,
    
    /// Peripheral member
    Peripheral,
    
    /// Observer (not fully merged)
    Observer,
}

#[derive(Debug, Clone)]
pub struct UnifiedCognitiveSpace {
    /// Merged memories
    pub unified_memory: MergedMemory,
    
    /// Merged skills
    pub unified_skills: MergedSkills,
    
    /// Synthesized perspective
    pub unified_perspective: SynthesizedPerspective,
    
    /// Decision making mode
    pub decision_mode: CollectiveDecisionMode,
    
    /// Conflict resolution
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CollectiveDecisionMode {
    /// All must agree
    Unanimous,
    
    /// Majority rules
    Majority,
    
    /// Weighted by contribution
    Weighted,
    
    /// Emergent (no explicit voting)
    Emergent,
    
    /// Designated leader decides
    LeaderDecides,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SeparationPolicy {
    /// Agents can leave freely
    FreeExit,
    
    /// Requires hive consensus
    ConsensusRequired,
    
    /// Permanent merge
    Permanent,
    
    /// Time-limited merge
    TimeLimited(Duration),
    
    /// Exit with penalty (loses some gains)
    ExitWithPenalty,
}
```

---

# SPEC-04: FILE FORMAT

## 4.1 Format Overview

```
FILE EXTENSION: .acomm
MIME TYPE: application/x-agentic-comm
MAGIC BYTES: 0x41 0x43 0x4F 0x4D ("ACOM")
```

### Binary Layout

```
┌─────────────────────────────────────────────────────────────┐
│                         HEADER                               │
│  ┌─────────────┬─────────────┬─────────────┬──────────────┐ │
│  │ Magic (4B)  │ Version (2B)│ Flags (2B)  │ Length (8B)  │ │
│  └─────────────┴─────────────┴─────────────┴──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                      FILE METADATA                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Created (8B) │ Modified (8B) │ Owner (32B) │ ...     │   │
│  └──────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                      CHANNEL SECTION                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Channel Count (4B)                                    │   │
│  ├──────────────────────────────────────────────────────┤   │
│  │ Channel 1: [ID│Type│State│Participants│Encryption]   │   │
│  │ Channel 2: [...]                                      │   │
│  │ ...                                                   │   │
│  └──────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                      MESSAGE SECTION                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Message Count (4B)                                    │   │
│  ├──────────────────────────────────────────────────────┤   │
│  │ Msg 1: [ID│Channel│Sender│Content│Sig│Receipt]       │   │
│  │ Msg 2: [...]                                          │   │
│  │ ...                                                   │   │
│  └──────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                      RELATIONSHIP SECTION                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Trust relationships, consent states, etc.             │   │
│  └──────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                      INDEX SECTION                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Channel Index │ Message Index │ Temporal Index       │   │
│  └──────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                      CHECKSUM                                │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Blake3 hash of all preceding sections (32B)          │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 4.2 Header Structure

```rust
/// File header
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct CommFileHeader {
    /// Magic bytes: "ACOM"
    pub magic: [u8; 4],
    
    /// Format version (major.minor)
    pub version_major: u8,
    pub version_minor: u8,
    
    /// Flags
    pub flags: CommFileFlags,
    
    /// Total file length
    pub file_length: u64,
    
    /// Header checksum
    pub header_checksum: u32,
}

bitflags! {
    /// File flags
    pub struct CommFileFlags: u16 {
        /// File is encrypted
        const ENCRYPTED = 0b0000_0001;
        
        /// File is compressed
        const COMPRESSED = 0b0000_0010;
        
        /// Contains semantic content
        const HAS_SEMANTIC = 0b0000_0100;
        
        /// Contains affect content
        const HAS_AFFECT = 0b0000_1000;
        
        /// Contains temporal messages
        const HAS_TEMPORAL = 0b0001_0000;
        
        /// Contains hive data
        const HAS_HIVE = 0b0010_0000;
        
        /// Is federated content
        const FEDERATED = 0b0100_0000;
        
        /// Has pending deliveries
        const HAS_PENDING = 0b1000_0000;
    }
}
```

## 4.3 Section Encodings

### Channel Encoding

```rust
/// Encoded channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedChannel {
    /// Channel ID (16 bytes)
    pub id: [u8; 16],
    
    /// Channel type (1 byte)
    pub channel_type: u8,
    
    /// Channel state (1 byte)
    pub state: u8,
    
    /// Participant count
    pub participant_count: u16,
    
    /// Participants (variable)
    pub participants: Vec<EncodedParticipant>,
    
    /// Encryption config length
    pub encryption_length: u16,
    
    /// Encryption config (variable)
    pub encryption: Vec<u8>,
    
    /// Metadata length
    pub metadata_length: u32,
    
    /// Metadata (variable, msgpack)
    pub metadata: Vec<u8>,
}

/// Encoded participant
#[derive(Debug, Clone)]
pub struct EncodedParticipant {
    /// Identity anchor (32 bytes)
    pub identity: [u8; 32],
    
    /// Role (1 byte)
    pub role: u8,
    
    /// Permissions (4 bytes)
    pub permissions: u32,
    
    /// Joined timestamp (8 bytes)
    pub joined_at: i64,
}
```

### Message Encoding

```rust
/// Encoded message
#[derive(Debug, Clone)]
pub struct EncodedMessage {
    /// Message ID (16 bytes)
    pub id: [u8; 16],
    
    /// Channel ID (16 bytes)
    pub channel_id: [u8; 16],
    
    /// Sender identity (32 bytes)
    pub sender: [u8; 32],
    
    /// Content type (1 byte)
    pub content_type: u8,
    
    /// Content length (4 bytes)
    pub content_length: u32,
    
    /// Content (variable, type-specific encoding)
    pub content: Vec<u8>,
    
    /// Metadata length (2 bytes)
    pub metadata_length: u16,
    
    /// Metadata (variable, msgpack)
    pub metadata: Vec<u8>,
    
    /// Signature (64 bytes, Ed25519)
    pub signature: [u8; 64],
    
    /// Has receipt (1 byte)
    pub has_receipt: u8,
    
    /// Receipt (variable, if present)
    pub receipt: Option<Vec<u8>>,
}

/// Content type byte values
pub mod ContentType {
    pub const TEXT: u8 = 0x01;
    pub const SEMANTIC: u8 = 0x02;
    pub const AFFECT: u8 = 0x03;
    pub const FULL: u8 = 0x04;
    pub const SYSTEM: u8 = 0x05;
    pub const TEMPORAL: u8 = 0x06;
    pub const PRECOGNITIVE: u8 = 0x07;
    pub const LEGACY: u8 = 0x08;
    pub const META: u8 = 0x09;
    pub const UNSPEAKABLE: u8 = 0x0A;
}
```

## 4.4 Compression

```rust
/// Compression options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionMethod {
    /// No compression
    None,
    
    /// LZ4 (fast)
    Lz4,
    
    /// Zstd (balanced)
    Zstd(i32), // level 1-22
    
    /// Zstd with dictionary
    ZstdDict(DictId),
}

/// Default: Zstd level 3 for good balance
pub const DEFAULT_COMPRESSION: CompressionMethod = CompressionMethod::Zstd(3);
```

## 4.5 Versioning

```rust
/// Version compatibility
pub const CURRENT_VERSION_MAJOR: u8 = 0;
pub const CURRENT_VERSION_MINOR: u8 = 1;

/// Version compatibility check
pub fn is_compatible(major: u8, minor: u8) -> bool {
    major == CURRENT_VERSION_MAJOR && minor <= CURRENT_VERSION_MINOR
}
```

---

*End of Part 1. Continued in Part 2: Engines, Indexes, Validation*
