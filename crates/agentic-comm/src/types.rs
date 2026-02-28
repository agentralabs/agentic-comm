//! Rich types for the agentic-comm specification.
//!
//! Covers: CommunicatingAgent, Consent, Trust, Affect, Temporal, Federation,
//! Hive, Semantic, Encryption, and extended channel/participant types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Trust
// ---------------------------------------------------------------------------

/// Seven-level trust hierarchy for agent communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommTrustLevel {
    /// No trust — all interactions blocked.
    None,
    /// Minimal trust — receive-only, no actions.
    Minimal,
    /// Basic trust — text messaging allowed.
    Basic,
    /// Standard trust — full messaging + channels.
    Standard,
    /// High trust — delegation, scheduling, federation.
    High,
    /// Full trust — all operations except system-level.
    Full,
    /// Absolute trust — unrestricted.
    Absolute,
}

impl Default for CommTrustLevel {
    fn default() -> Self {
        Self::Standard
    }
}

impl std::fmt::Display for CommTrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Minimal => write!(f, "minimal"),
            Self::Basic => write!(f, "basic"),
            Self::Standard => write!(f, "standard"),
            Self::High => write!(f, "high"),
            Self::Full => write!(f, "full"),
            Self::Absolute => write!(f, "absolute"),
        }
    }
}

impl std::str::FromStr for CommTrustLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "minimal" => Ok(Self::Minimal),
            "basic" => Ok(Self::Basic),
            "standard" => Ok(Self::Standard),
            "high" => Ok(Self::High),
            "full" => Ok(Self::Full),
            "absolute" => Ok(Self::Absolute),
            other => Err(format!("Unknown trust level: {other}")),
        }
    }
}

impl PartialOrd for CommTrustLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CommTrustLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.numeric().cmp(&other.numeric())
    }
}

impl CommTrustLevel {
    /// Numeric value for ordering.
    pub fn numeric(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Minimal => 1,
            Self::Basic => 2,
            Self::Standard => 3,
            Self::High => 4,
            Self::Full => 5,
            Self::Absolute => 6,
        }
    }
}

// ---------------------------------------------------------------------------
// Consent
// ---------------------------------------------------------------------------

/// Scope of a consent grant.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentScope {
    /// Can read messages from agent.
    ReadMessages,
    /// Can send messages to agent.
    SendMessages,
    /// Can join channels the agent is in.
    JoinChannels,
    /// Can see agent's presence/availability.
    ViewPresence,
    /// Can share agent's messages with others.
    ShareContent,
    /// Can schedule deferred messages.
    ScheduleMessages,
    /// Can federate messages across zones.
    Federate,
    /// Can include in hive-mind groups.
    HiveParticipation,
    /// Can read/share affect (emotional state) information.
    Affect,
    /// Can participate in hive-mind operations (broader than HiveParticipation).
    Hive,
}

impl std::fmt::Display for ConsentScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadMessages => write!(f, "read_messages"),
            Self::SendMessages => write!(f, "send_messages"),
            Self::JoinChannels => write!(f, "join_channels"),
            Self::ViewPresence => write!(f, "view_presence"),
            Self::ShareContent => write!(f, "share_content"),
            Self::ScheduleMessages => write!(f, "schedule_messages"),
            Self::Federate => write!(f, "federate"),
            Self::HiveParticipation => write!(f, "hive_participation"),
            Self::Affect => write!(f, "affect"),
            Self::Hive => write!(f, "hive"),
        }
    }
}

impl std::str::FromStr for ConsentScope {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read_messages" => Ok(Self::ReadMessages),
            "send_messages" => Ok(Self::SendMessages),
            "join_channels" => Ok(Self::JoinChannels),
            "view_presence" => Ok(Self::ViewPresence),
            "share_content" => Ok(Self::ShareContent),
            "schedule_messages" => Ok(Self::ScheduleMessages),
            "federate" => Ok(Self::Federate),
            "hive_participation" => Ok(Self::HiveParticipation),
            "affect" => Ok(Self::Affect),
            "hive" => Ok(Self::Hive),
            other => Err(format!("Unknown consent scope: {other}")),
        }
    }
}

/// Status of a consent grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsentStatus {
    /// Consent has been granted.
    Granted,
    /// Consent has been revoked.
    Revoked,
    /// Consent is pending approval.
    Pending,
    /// Consent has expired.
    Expired,
    /// Consent was denied.
    Denied,
}

impl Default for ConsentStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for ConsentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Granted => write!(f, "granted"),
            Self::Revoked => write!(f, "revoked"),
            Self::Pending => write!(f, "pending"),
            Self::Expired => write!(f, "expired"),
            Self::Denied => write!(f, "denied"),
        }
    }
}

/// A single consent gate entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentGateEntry {
    /// Who is granting consent.
    pub grantor: String,
    /// Who is receiving consent.
    pub grantee: String,
    /// What scope of access is granted.
    pub scope: ConsentScope,
    /// Current status.
    pub status: ConsentStatus,
    /// When the consent was created (ISO 8601).
    pub created_at: String,
    /// When the consent was last updated.
    pub updated_at: String,
    /// Optional expiry time (ISO 8601).
    #[serde(default)]
    pub expires_at: Option<String>,
    /// Human-readable reason.
    #[serde(default)]
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Affect
// ---------------------------------------------------------------------------

/// Emotional/affect state for communication context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectState {
    /// Valence: -1.0 (negative) to 1.0 (positive).
    #[serde(default)]
    pub valence: f64,
    /// Arousal: 0.0 (calm) to 1.0 (excited).
    #[serde(default)]
    pub arousal: f64,
    /// Dominance: 0.0 (submissive) to 1.0 (dominant).
    #[serde(default)]
    pub dominance: f64,
    /// Named emotions present.
    #[serde(default)]
    pub emotions: Vec<Emotion>,
    /// Urgency level.
    #[serde(default)]
    pub urgency: UrgencyLevel,
    /// Meta-confidence: how confident the agent is in this affect reading.
    #[serde(default = "default_confidence")]
    pub meta_confidence: f64,
}

fn default_confidence() -> f64 {
    0.5
}

impl Default for AffectState {
    fn default() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.0,
            dominance: 0.5,
            emotions: Vec::new(),
            urgency: UrgencyLevel::Normal,
            meta_confidence: 0.5,
        }
    }
}

/// Named emotion categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Emotion {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,
    Trust,
    Anticipation,
    Curiosity,
    Confusion,
    Frustration,
    Gratitude,
    Empathy,
    Pride,
    Shame,
    Guilt,
    Awe,
    Boredom,
    Excitement,
    Calm,
    Anxiety,
    Hope,
    Determination,
    Contentment,
}

impl std::fmt::Display for Emotion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("{self:?}").to_lowercase());
        write!(f, "{s}")
    }
}

/// Urgency levels for communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UrgencyLevel {
    /// Background — process whenever convenient.
    Background,
    /// Low priority.
    Low,
    /// Normal priority (default).
    Normal,
    /// High priority.
    High,
    /// Urgent — immediate attention needed.
    Urgent,
    /// Critical — system-level emergency.
    Critical,
}

impl Default for UrgencyLevel {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for UrgencyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Background => write!(f, "background"),
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Urgent => write!(f, "urgent"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for UrgencyLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "background" => Ok(Self::Background),
            "low" => Ok(Self::Low),
            "normal" => Ok(Self::Normal),
            "high" => Ok(Self::High),
            "urgent" => Ok(Self::Urgent),
            "critical" => Ok(Self::Critical),
            other => Err(format!("Unknown urgency level: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Temporal
// ---------------------------------------------------------------------------

/// When a message should be delivered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalTarget {
    /// Deliver immediately.
    Immediate,
    /// Deliver at a specific time (ISO 8601).
    FutureAbsolute { deliver_at: String },
    /// Deliver after a duration (seconds from now).
    FutureRelative { delay_seconds: u64 },
    /// Deliver when a condition is met.
    Conditional { condition: String },
    /// Message persists indefinitely (never expires).
    Eternal,
    /// Deliver to memory as a historical event at the given timestamp.
    Retroactive { memory_timestamp: String },
    /// Deliver at the computed optimal time based on context.
    Optimal { context: String },
}

impl Default for TemporalTarget {
    fn default() -> Self {
        Self::Immediate
    }
}

/// A time-scheduled message in the temporal queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalMessage {
    /// Unique ID for this scheduled message.
    pub id: u64,
    /// Target channel.
    pub channel_id: u64,
    /// Sender identity.
    pub sender: String,
    /// Message content.
    pub content: String,
    /// When to deliver.
    pub target: TemporalTarget,
    /// When this was scheduled (ISO 8601).
    pub scheduled_at: String,
    /// Whether this has been delivered.
    #[serde(default)]
    pub delivered: bool,
    /// Optional affect state for this message.
    #[serde(default)]
    pub affect: Option<AffectState>,
}

// ---------------------------------------------------------------------------
// Federation
// ---------------------------------------------------------------------------

/// Configuration for cross-zone federation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Whether federation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// This node's zone identifier.
    #[serde(default)]
    pub local_zone: String,
    /// Known federated zones.
    #[serde(default)]
    pub zones: Vec<FederatedZone>,
    /// Default policy for unknown zones.
    #[serde(default)]
    pub default_policy: FederationPolicy,
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            local_zone: "local".to_string(),
            zones: Vec::new(),
            default_policy: FederationPolicy::Deny,
        }
    }
}

/// A federated zone that this node communicates with.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedZone {
    /// Zone identifier.
    pub zone_id: String,
    /// Human-readable name.
    #[serde(default)]
    pub name: String,
    /// Endpoint URL or address.
    #[serde(default)]
    pub endpoint: String,
    /// Policy for this zone.
    #[serde(default)]
    pub policy: FederationPolicy,
    /// Trust level for this zone.
    #[serde(default)]
    pub trust_level: CommTrustLevel,
}

/// Federation policy for a zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FederationPolicy {
    /// Allow all communication.
    Allow,
    /// Deny all communication.
    Deny,
    /// Allow only specific channels/topics.
    Selective,
}

impl Default for FederationPolicy {
    fn default() -> Self {
        Self::Deny
    }
}

impl std::fmt::Display for FederationPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Deny => write!(f, "deny"),
            Self::Selective => write!(f, "selective"),
        }
    }
}

impl std::str::FromStr for FederationPolicy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "allow" => Ok(Self::Allow),
            "deny" => Ok(Self::Deny),
            "selective" => Ok(Self::Selective),
            other => Err(format!("Unknown federation policy: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Hive Mind
// ---------------------------------------------------------------------------

/// Default coherence level for a hive mind (0.5).
fn default_coherence_level() -> f64 {
    0.5
}

/// Default separation policy: graceful.
fn default_separation_policy() -> String {
    "graceful".to_string()
}

/// A hive-mind group: multiple agents sharing a communication context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveMind {
    /// Unique identifier.
    pub id: u64,
    /// Human-readable name.
    pub name: String,
    /// Member agents.
    pub constituents: Vec<HiveConstituent>,
    /// Collective decision mode.
    #[serde(default)]
    pub decision_mode: CollectiveDecisionMode,
    /// When formed (ISO 8601).
    pub formed_at: String,
    /// Optional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Coherence score (0.0-1.0) — how aligned the hive members are.
    #[serde(default = "default_coherence_level")]
    pub coherence_level: f64,
    /// What happens when the hive separates (e.g. "graceful", "immediate", "consensus").
    #[serde(default = "default_separation_policy")]
    pub separation_policy: String,
}

/// A member of a hive mind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveConstituent {
    /// Agent identity.
    pub agent_id: String,
    /// Role within the hive.
    #[serde(default)]
    pub role: HiveRole,
    /// When they joined.
    pub joined_at: String,
}

/// Role within a hive mind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HiveRole {
    /// Founding/coordinating member.
    Coordinator,
    /// Regular member.
    Member,
    /// Observer only (read access).
    Observer,
    /// Mediator — facilitates consensus and resolves conflicts.
    Mediator,
}

impl Default for HiveRole {
    fn default() -> Self {
        Self::Member
    }
}

impl std::fmt::Display for HiveRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Coordinator => write!(f, "coordinator"),
            Self::Member => write!(f, "member"),
            Self::Observer => write!(f, "observer"),
            Self::Mediator => write!(f, "mediator"),
        }
    }
}

/// How the hive reaches collective decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectiveDecisionMode {
    /// Coordinator decides.
    CoordinatorDecides,
    /// Majority vote.
    Majority,
    /// Unanimous consent required.
    Unanimous,
    /// Consensus (soft agreement).
    Consensus,
}

impl Default for CollectiveDecisionMode {
    fn default() -> Self {
        Self::CoordinatorDecides
    }
}

// ---------------------------------------------------------------------------
// CommunicatingAgent
// ---------------------------------------------------------------------------

/// Cryptographic identity anchor (public key fingerprint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityAnchor {
    /// Public key fingerprint (hex).
    #[serde(default)]
    pub public_key: String,
    /// Key algorithm.
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    /// Identity verification status.
    #[serde(default)]
    pub verified: bool,
    /// Identity provider (e.g., "agentic-identity").
    #[serde(default)]
    pub provider: String,
}

impl Default for IdentityAnchor {
    fn default() -> Self {
        Self {
            public_key: String::new(),
            algorithm: default_algorithm(),
            verified: false,
            provider: String::new(),
        }
    }
}

fn default_algorithm() -> String {
    "Ed25519".to_string()
}

fn default_availability_string() -> String {
    "online".to_string()
}

/// Rich agent profile for the communication system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicatingAgent {
    /// Unique agent identity.
    pub agent_id: String,
    /// Human-readable display name.
    #[serde(default)]
    pub display_name: String,
    /// Agent type (human, ai, system, bot).
    #[serde(default = "default_agent_type")]
    pub agent_type: String,
    /// Communication capabilities.
    #[serde(default)]
    pub capabilities: CommCapabilities,
    /// Trust profile.
    #[serde(default)]
    pub trust_profile: CommTrustProfile,
    /// Current availability.
    #[serde(default)]
    pub availability: Availability,
    /// Communication preferences.
    #[serde(default)]
    pub preferences: CommPreferences,
    /// When registered (ISO 8601).
    #[serde(default)]
    pub registered_at: String,
    /// Arbitrary metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Cryptographic identity anchor (public key fingerprint).
    #[serde(default)]
    pub identity_anchor: IdentityAnchor,
    /// Agent capability labels (e.g., "code-review", "deploy").
    #[serde(default)]
    pub capability_labels: Vec<String>,
    /// Availability status string (e.g., "online", "busy", "away").
    #[serde(default = "default_availability_string")]
    pub availability_label: String,
    /// Communication preferences (key-value pairs).
    #[serde(default)]
    pub preference_overrides: HashMap<String, String>,
}

fn default_agent_type() -> String {
    "ai".to_string()
}

/// What communication capabilities an agent has.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommCapabilities {
    /// Can send messages.
    #[serde(default = "default_true")]
    pub can_send: bool,
    /// Can receive messages.
    #[serde(default = "default_true")]
    pub can_receive: bool,
    /// Can create channels.
    #[serde(default)]
    pub can_create_channels: bool,
    /// Can federate across zones.
    #[serde(default)]
    pub can_federate: bool,
    /// Can participate in hive minds.
    #[serde(default)]
    pub can_hive: bool,
    /// Can schedule temporal messages.
    #[serde(default)]
    pub can_schedule: bool,
    /// Supports encryption.
    #[serde(default)]
    pub supports_encryption: bool,
    /// Supported message types.
    #[serde(default)]
    pub supported_types: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Trust profile for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommTrustProfile {
    /// Default trust level for unknown agents.
    #[serde(default)]
    pub default_level: CommTrustLevel,
    /// Per-agent trust overrides.
    #[serde(default)]
    pub overrides: HashMap<String, CommTrustLevel>,
}

impl Default for CommTrustProfile {
    fn default() -> Self {
        Self {
            default_level: CommTrustLevel::Standard,
            overrides: HashMap::new(),
        }
    }
}

/// Agent availability status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Availability {
    /// Available for communication.
    Available,
    /// Busy — may not respond quickly.
    Busy,
    /// Away — delayed responses.
    Away,
    /// Do not disturb.
    DoNotDisturb,
    /// Offline.
    Offline,
}

impl Default for Availability {
    fn default() -> Self {
        Self::Available
    }
}

impl std::fmt::Display for Availability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Available => write!(f, "available"),
            Self::Busy => write!(f, "busy"),
            Self::Away => write!(f, "away"),
            Self::DoNotDisturb => write!(f, "do_not_disturb"),
            Self::Offline => write!(f, "offline"),
        }
    }
}

/// Communication preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommPreferences {
    /// Preferred message format.
    #[serde(default)]
    pub preferred_format: String,
    /// Maximum message size accepted.
    #[serde(default)]
    pub max_message_size: Option<usize>,
    /// Preferred language codes.
    #[serde(default)]
    pub languages: Vec<String>,
    /// Whether to echo sent messages.
    #[serde(default)]
    pub echo_sent: bool,
}

// ---------------------------------------------------------------------------
// Encryption
// ---------------------------------------------------------------------------

/// Encryption schemes supported for messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncryptionScheme {
    /// No encryption.
    None,
    /// AES-256-GCM symmetric encryption.
    Aes256Gcm,
    /// ChaCha20-Poly1305.
    ChaCha20Poly1305,
    /// X25519 + AES-256-GCM.
    X25519Aes256Gcm,
}

impl Default for EncryptionScheme {
    fn default() -> Self {
        Self::None
    }
}

impl std::fmt::Display for EncryptionScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Aes256Gcm => write!(f, "aes_256_gcm"),
            Self::ChaCha20Poly1305 => write!(f, "chacha20_poly1305"),
            Self::X25519Aes256Gcm => write!(f, "x25519_aes_256_gcm"),
        }
    }
}

/// Channel-level encryption configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelEncryption {
    /// Encryption scheme in use.
    #[serde(default)]
    pub scheme: EncryptionScheme,
    /// Whether encryption is enforced for all messages.
    #[serde(default)]
    pub enforced: bool,
    /// Key rotation interval in seconds (0 = no rotation).
    #[serde(default)]
    pub key_rotation_seconds: u64,
}

// ---------------------------------------------------------------------------
// Semantic / Cognitive
// ---------------------------------------------------------------------------

/// A fragment of semantic content attached to a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFragment {
    /// Fragment text or identifier.
    pub content: String,
    /// Semantic role (topic, entity, intent, sentiment, etc.).
    #[serde(default)]
    pub role: String,
    /// Confidence score 0.0-1.0.
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    /// Cognitive nodes linked to this fragment.
    #[serde(default)]
    pub nodes: Vec<CognitiveNode>,
    /// Edges between nodes.
    #[serde(default)]
    pub edges: Vec<CognitiveEdge>,
    /// Points where this fragment can attach to another cognitive graph.
    #[serde(default)]
    pub graft_points: Vec<String>,
    /// The context in which this fragment was extracted.
    #[serde(default)]
    pub context: String,
    /// The cognitive perspective of the agent that created this fragment.
    #[serde(default)]
    pub perspective: String,
}

/// A node in a cognitive graph attached to communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveNode {
    /// Node identifier.
    pub id: String,
    /// Label / content.
    pub label: String,
    /// Node type.
    #[serde(default)]
    pub node_type: CognitiveNodeType,
}

/// Types of cognitive nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CognitiveNodeType {
    Concept,
    Entity,
    Action,
    State,
    Goal,
    Belief,
    Emotion,
}

impl Default for CognitiveNodeType {
    fn default() -> Self {
        Self::Concept
    }
}

/// An edge in a cognitive graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEdge {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Edge type.
    #[serde(default)]
    pub edge_type: CognitiveEdgeType,
    /// Edge weight 0.0-1.0.
    #[serde(default = "default_confidence")]
    pub weight: f64,
}

/// Types of cognitive edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveEdgeType {
    CausedBy,
    RelatedTo,
    Supports,
    Contradicts,
    Precedes,
    Contains,
}

impl Default for CognitiveEdgeType {
    fn default() -> Self {
        Self::RelatedTo
    }
}

// ---------------------------------------------------------------------------
// Extended channel types
// ---------------------------------------------------------------------------

/// Rich channel types (extends the base ChannelType).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RichChannelType {
    /// 1:1 direct messaging.
    Direct,
    /// Multi-participant group.
    Group,
    /// One-to-many broadcast.
    Broadcast,
    /// Publish/subscribe topic.
    PubSub,
    /// Telepathic (shared-context) channel.
    Telepathic,
    /// Hive-mind collective channel.
    Hive,
    /// Temporal (time-shifted) channel.
    Temporal,
    /// Destiny channel (prophetic/future).
    Destiny,
    /// Oracle channel (query/response).
    Oracle,
}

impl std::fmt::Display for RichChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "direct"),
            Self::Group => write!(f, "group"),
            Self::Broadcast => write!(f, "broadcast"),
            Self::PubSub => write!(f, "pubsub"),
            Self::Telepathic => write!(f, "telepathic"),
            Self::Hive => write!(f, "hive"),
            Self::Temporal => write!(f, "temporal"),
            Self::Destiny => write!(f, "destiny"),
            Self::Oracle => write!(f, "oracle"),
        }
    }
}

/// Participant roles within a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantRole {
    /// Channel owner.
    Owner,
    /// Administrator.
    Admin,
    /// Regular member.
    Member,
    /// Read-only observer.
    Observer,
    /// Muted participant.
    Muted,
    /// Oracle (query-only).
    Oracle,
    /// Ghost (invisible observer).
    Ghost,
}

impl Default for ParticipantRole {
    fn default() -> Self {
        Self::Member
    }
}

impl std::fmt::Display for ParticipantRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Owner => write!(f, "owner"),
            Self::Admin => write!(f, "admin"),
            Self::Member => write!(f, "member"),
            Self::Observer => write!(f, "observer"),
            Self::Muted => write!(f, "muted"),
            Self::Oracle => write!(f, "oracle"),
            Self::Ghost => write!(f, "ghost"),
        }
    }
}

// ---------------------------------------------------------------------------
// Grounding
// ---------------------------------------------------------------------------

/// Result of grounding a claim against the communication store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingResult {
    /// The claim that was checked.
    pub claim: String,
    /// Whether the claim is grounded.
    pub status: GroundingStatus,
    /// Evidence supporting the result.
    #[serde(default)]
    pub evidence: Vec<GroundingEvidence>,
    /// Confidence in the grounding result 0.0-1.0.
    #[serde(default)]
    pub confidence: f64,
}

/// Grounding status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroundingStatus {
    /// Claim is verified by store data.
    Verified,
    /// Claim is partially supported.
    Partial,
    /// Claim has no backing in the store.
    Ungrounded,
}

impl std::fmt::Display for GroundingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verified => write!(f, "verified"),
            Self::Partial => write!(f, "partial"),
            Self::Ungrounded => write!(f, "ungrounded"),
        }
    }
}

/// A piece of grounding evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingEvidence {
    /// What kind of evidence (channel, message, consent, trust, etc.).
    pub evidence_type: String,
    /// The relevant data.
    pub content: String,
    /// Relevance score 0.0-1.0.
    #[serde(default)]
    pub relevance: f64,
}

// ---------------------------------------------------------------------------
// Communication log
// ---------------------------------------------------------------------------

/// Entry in the communication context log (like memory's conversation_log).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationLogEntry {
    /// Sequential index.
    pub index: u64,
    /// The message or context being logged.
    pub content: String,
    /// Role: "user", "agent", "system".
    #[serde(default)]
    pub role: String,
    /// Optional topic/category.
    #[serde(default)]
    pub topic: Option<String>,
    /// Timestamp (ISO 8601).
    pub timestamp: String,
    /// Optional linked message ID.
    #[serde(default)]
    pub linked_message_id: Option<u64>,
    /// Optional affect state.
    #[serde(default)]
    pub affect: Option<AffectState>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------


// ---------------------------------------------------------------------------
// Additional SPEC-02/03 types
// ---------------------------------------------------------------------------

/// What to do when a temporal message times out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeoutAction {
    /// Deliver the message even though the ideal time has passed.
    DeliverAnyway,
    /// Discard the message silently.
    Discard,
    /// Return the message to the sender.
    ReturnToSender,
    /// Archive the message for later inspection.
    Archive,
}
impl Default for TimeoutAction { fn default() -> Self { Self::DeliverAnyway } }

/// Policy governing how agents can leave a hive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeparationPolicy {
    /// Agents can leave freely at any time.
    FreeExit,
    /// Leaving requires consensus from other members.
    ConsensusRequired,
    /// The hive is permanent.
    Permanent,
    /// The hive has a time limit in seconds.
    TimeLimited(u64),
    /// Agents can leave but incur a penalty.
    ExitWithPenalty,
}
impl Default for SeparationPolicy { fn default() -> Self { Self::FreeExit } }

/// Rich message content carrying text, semantic graphs, affect, or combinations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RichMessageContent {
    /// Plain text content.
    Text(String),
    /// A cognitive / semantic graph fragment.
    Semantic(SemanticFragment),
    /// Affect / emotional state.
    Affect(AffectState),
    /// Full multimodal content.
    Full { text: String, semantic: Option<SemanticFragment>, affect: Option<AffectState> },
    /// A system-level message.
    System(String),
    /// A temporal (scheduled) message.
    Temporal(Box<TemporalMessage>),
    /// Arbitrary metadata as key-value JSON.
    Meta(HashMap<String, serde_json::Value>),
    /// Messages about the future (precognitive).
    Precognitive { prediction: String, confidence: f64, horizon: String },
    /// Death-activated messages delivered to a beneficiary.
    Legacy { beneficiary: String, condition: String, content: String },
    /// Beyond-language communication attempts.
    Unspeakable { attempt: String, method: String },
}


// ---------------------------------------------------------------------------
// CommTimestamp — logical causal ordering
// ---------------------------------------------------------------------------

/// Logical timestamp for causal ordering of communication events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommTimestamp {
    /// Wall clock time (ISO 8601).
    pub wall_clock: String,
    /// Lamport clock value for causal ordering.
    #[serde(default)]
    pub lamport: u64,
    /// Vector clock: agent_id -> logical clock value.
    #[serde(default)]
    pub vector_clock: HashMap<String, u64>,
}

impl CommTimestamp {
    /// Create a new CommTimestamp with the current wall clock time and lamport=0.
    pub fn now(agent_id: &str) -> Self {
        let wall_clock = chrono::Utc::now().to_rfc3339();
        let mut vector_clock = HashMap::new();
        vector_clock.insert(agent_id.to_string(), 0);
        Self {
            wall_clock,
            lamport: 0,
            vector_clock,
        }
    }

    /// Increment the Lamport clock and the vector clock entry for the given agent.
    pub fn increment(&mut self, agent_id: &str) {
        self.lamport += 1;
        let entry = self.vector_clock.entry(agent_id.to_string()).or_insert(0);
        *entry += 1;
    }

    /// Merge another timestamp into this one.
    ///
    /// Takes the element-wise maximum of the vector clocks and sets
    /// lamport = max(self.lamport, other.lamport) + 1.
    pub fn merge(&mut self, other: &CommTimestamp, agent_id: &str) {
        // Merge vector clocks: take max of each entry
        for (key, &value) in &other.vector_clock {
            let entry = self.vector_clock.entry(key.clone()).or_insert(0);
            if value > *entry {
                *entry = value;
            }
        }
        // Set lamport to max + 1
        self.lamport = std::cmp::max(self.lamport, other.lamport) + 1;
        // Increment own vector clock entry
        let entry = self.vector_clock.entry(agent_id.to_string()).or_insert(0);
        *entry += 1;
    }

    /// Return true if self happens-before other (vector clock is strictly <= other).
    ///
    /// Self happens-before other when every entry in self's vector clock is
    /// less-than-or-equal to the corresponding entry in other's, and at least
    /// one entry is strictly less.
    pub fn happens_before(&self, other: &CommTimestamp) -> bool {
        let mut dominated = true;
        let mut strictly_less = false;

        for (key, &value) in &self.vector_clock {
            let other_value = other.vector_clock.get(key).copied().unwrap_or(0);
            if value > other_value {
                dominated = false;
                break;
            }
            if value < other_value {
                strictly_less = true;
            }
        }

        if !dominated {
            return false;
        }

        // Also check keys that exist in other but not in self (treated as 0 in self).
        for (key, &other_value) in &other.vector_clock {
            if !self.vector_clock.contains_key(key) && other_value > 0 {
                strictly_less = true;
            }
        }

        dominated && strictly_less
    }
}

impl Default for CommTimestamp {
    fn default() -> Self {
        Self {
            wall_clock: String::new(),
            lamport: 0,
            vector_clock: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Rate Limiting
// ---------------------------------------------------------------------------

fn default_rate_60() -> u32 { 60 }
fn default_rate_10() -> u32 { 10 }
fn default_rate_30() -> u32 { 30 }
fn default_rate_5() -> u32 { 5 }
fn default_rate_20() -> u32 { 20 }

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum messages per minute.
    #[serde(default = "default_rate_60")]
    pub messages_per_minute: u32,
    /// Maximum semantic operations per minute.
    #[serde(default = "default_rate_10")]
    pub semantic_per_minute: u32,
    /// Maximum affect transmissions per minute.
    #[serde(default = "default_rate_30")]
    pub affect_per_minute: u32,
    /// Maximum hive operations per hour.
    #[serde(default = "default_rate_5")]
    pub hive_per_hour: u32,
    /// Maximum federation messages per minute.
    #[serde(default = "default_rate_20")]
    pub federation_per_minute: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            messages_per_minute: default_rate_60(),
            semantic_per_minute: default_rate_10(),
            affect_per_minute: default_rate_30(),
            hive_per_hour: default_rate_5(),
            federation_per_minute: default_rate_20(),
        }
    }
}

// ---------------------------------------------------------------------------
// Audit
// ---------------------------------------------------------------------------

/// Audit event types for security logging.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    MessageSent,
    MessageReceived,
    ChannelCreated,
    ChannelClosed,
    ConsentGranted,
    ConsentRevoked,
    ConsentDenied,
    TrustChanged,
    TrustUpdated,
    HiveFormed,
    HiveDissolved,
    FederationMessage,
    FederationConfigured,
    ScheduledMessage,
    KeyRotated,
    AuthFailure,
    RateLimitExceeded,
    SignatureWarning,
}

/// An audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Event type.
    pub event_type: AuditEventType,
    /// Timestamp (ISO 8601).
    pub timestamp: String,
    /// Agent that triggered the event.
    pub agent_id: String,
    /// Human-readable description.
    pub description: String,
    /// Optional related entity ID.
    #[serde(default)]
    pub related_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Extended types for MCP tools (semantic, consent flow, meld, conversations)
// ---------------------------------------------------------------------------

/// A semantic operation record (send/extract/graft).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticOperation {
    /// Unique ID.
    pub id: u64,
    /// Topic or context.
    pub topic: String,
    /// Focus nodes in the semantic graph.
    #[serde(default)]
    pub focus_nodes: Vec<String>,
    /// Depth of the operation.
    #[serde(default)]
    pub depth: u64,
    /// Timestamp (epoch seconds).
    pub timestamp: u64,
    /// Operation kind: send, extract, graft.
    #[serde(default)]
    pub operation: String,
    /// Optional channel association.
    #[serde(default)]
    pub channel_id: Option<u64>,
    /// Sender identity.
    #[serde(default)]
    pub sender: Option<String>,
}

/// A conflict from semantic operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticConflict {
    /// Unique ID.
    pub id: u64,
    /// Description of the conflict.
    pub description: String,
    /// Severity: low, medium, high.
    #[serde(default)]
    pub severity: String,
    /// Optional channel ID.
    #[serde(default)]
    pub channel_id: Option<u64>,
}

/// A pending consent request (for the consent flow tools).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRequest {
    /// Unique request ID.
    pub id: String,
    /// Requesting agent.
    pub from: String,
    /// Target agent.
    pub to: String,
    /// Type of consent requested.
    pub consent_type: String,
    /// Timestamp (epoch seconds).
    pub timestamp: u64,
    /// Optional reason for the request.
    #[serde(default)]
    pub reason: Option<String>,
    /// Whether this has been responded to.
    #[serde(default)]
    pub responded: bool,
    /// Response if given: "grant" or "deny".
    #[serde(default)]
    pub response: Option<String>,
}

/// A temporary mind meld session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeldSession {
    /// Unique session ID.
    pub id: String,
    /// Partner agent ID.
    pub partner_id: String,
    /// Depth: shallow, deep, full.
    #[serde(default)]
    pub depth: String,
    /// Start time (epoch seconds).
    pub start_time: u64,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the session is currently active.
    #[serde(default)]
    pub active: bool,
}

/// Summary of a conversation thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    /// Channel ID.
    pub channel_id: u64,
    /// Participants involved.
    #[serde(default)]
    pub participants: Vec<String>,
    /// Number of messages.
    pub message_count: u64,
    /// Last activity timestamp (epoch seconds).
    pub last_activity: u64,
}

/// Per-zone federation policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZonePolicyConfig {
    /// Zone identifier.
    pub zone_id: String,
    /// Allow semantic operations through this zone.
    #[serde(default = "default_true")]
    pub allow_semantic: bool,
    /// Allow affect propagation through this zone.
    #[serde(default = "default_true")]
    pub allow_affect: bool,
    /// Allow hive operations through this zone.
    #[serde(default = "default_true")]
    pub allow_hive: bool,
    /// Maximum message size in bytes (0 = unlimited).
    #[serde(default)]
    pub max_message_size: u64,
}

// ---------------------------------------------------------------------------
// Federation Gateway
// ---------------------------------------------------------------------------

/// A gateway node for cross-zone federation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationGateway {
    /// Zone identifier.
    pub zone_id: String,
    /// Gateway endpoint URL or address.
    pub endpoint: String,
    /// Protocol identifier.
    pub protocol: String,
    /// Gateway status (e.g., "online", "offline", "degraded").
    pub status: String,
    /// Last heartbeat timestamp (epoch seconds).
    pub last_heartbeat: u64,
    /// Capabilities supported by this gateway.
    pub capabilities: Vec<String>,
    /// Maximum message size in bytes.
    pub max_message_size: u64,
}

impl Default for FederationGateway {
    fn default() -> Self {
        Self {
            zone_id: String::new(),
            endpoint: String::new(),
            protocol: "agentic-comm/1.0".to_string(),
            status: "unknown".to_string(),
            last_heartbeat: 0,
            capabilities: vec!["messages".to_string()],
            max_message_size: 1_048_576,
        }
    }
}

// ---------------------------------------------------------------------------
// Federation Message
// ---------------------------------------------------------------------------

/// A message that traverses federation boundaries (cross-zone messaging).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMessage {
    /// Unique message identifier.
    pub id: String,
    /// Originating zone.
    pub source_zone: String,
    /// Destination zone.
    pub target_zone: String,
    /// Channel ID within the target zone.
    pub channel_id: u64,
    /// Message content.
    pub content: String,
    /// Sender identity.
    pub sender: String,
    /// Timestamp (epoch seconds).
    pub timestamp: u64,
    /// Cryptographic signature.
    pub signature: String,
    /// Gateway hops this message has traversed.
    pub hops: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_level_ordering() {
        assert!(CommTrustLevel::None < CommTrustLevel::Minimal);
        assert!(CommTrustLevel::Minimal < CommTrustLevel::Basic);
        assert!(CommTrustLevel::Basic < CommTrustLevel::Standard);
        assert!(CommTrustLevel::Standard < CommTrustLevel::High);
        assert!(CommTrustLevel::High < CommTrustLevel::Full);
        assert!(CommTrustLevel::Full < CommTrustLevel::Absolute);
    }

    #[test]
    fn trust_level_roundtrip() {
        for level in &[
            CommTrustLevel::None,
            CommTrustLevel::Minimal,
            CommTrustLevel::Basic,
            CommTrustLevel::Standard,
            CommTrustLevel::High,
            CommTrustLevel::Full,
            CommTrustLevel::Absolute,
        ] {
            let s = level.to_string();
            let parsed: CommTrustLevel = s.parse().unwrap();
            assert_eq!(*level, parsed);
        }
    }

    #[test]
    fn consent_scope_roundtrip() {
        for scope in &[
            ConsentScope::ReadMessages,
            ConsentScope::SendMessages,
            ConsentScope::JoinChannels,
            ConsentScope::ViewPresence,
            ConsentScope::ShareContent,
            ConsentScope::ScheduleMessages,
            ConsentScope::Federate,
            ConsentScope::HiveParticipation,
            ConsentScope::Affect,
            ConsentScope::Hive,
        ] {
            let s = scope.to_string();
            let parsed: ConsentScope = s.parse().unwrap();
            assert_eq!(*scope, parsed);
        }
    }

    #[test]
    fn urgency_level_roundtrip() {
        for level in &[
            UrgencyLevel::Background,
            UrgencyLevel::Low,
            UrgencyLevel::Normal,
            UrgencyLevel::High,
            UrgencyLevel::Urgent,
            UrgencyLevel::Critical,
        ] {
            let s = level.to_string();
            let parsed: UrgencyLevel = s.parse().unwrap();
            assert_eq!(*level, parsed);
        }
    }

    #[test]
    fn affect_state_defaults() {
        let state = AffectState::default();
        assert_eq!(state.valence, 0.0);
        assert_eq!(state.arousal, 0.0);
        assert_eq!(state.dominance, 0.5);
        assert!(state.emotions.is_empty());
        assert_eq!(state.urgency, UrgencyLevel::Normal);
        assert_eq!(state.meta_confidence, 0.5);
    }

    #[test]
    fn temporal_target_serde() {
        let targets = vec![
            TemporalTarget::Immediate,
            TemporalTarget::FutureAbsolute {
                deliver_at: "2030-01-01T00:00:00Z".to_string(),
            },
            TemporalTarget::FutureRelative { delay_seconds: 3600 },
            TemporalTarget::Conditional {
                condition: "agent_online".to_string(),
            },
            TemporalTarget::Eternal,
        ];
        for target in &targets {
            let json = serde_json::to_string(target).unwrap();
            let parsed: TemporalTarget = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&parsed).unwrap();
            assert_eq!(json, json2);
        }
    }

    #[test]
    fn federation_config_defaults() {
        let config = FederationConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.local_zone, "local");
        assert!(config.zones.is_empty());
        assert_eq!(config.default_policy, FederationPolicy::Deny);
    }

    #[test]
    fn federation_policy_roundtrip() {
        for p in &[
            FederationPolicy::Allow,
            FederationPolicy::Deny,
            FederationPolicy::Selective,
        ] {
            let s = p.to_string();
            let parsed: FederationPolicy = s.parse().unwrap();
            assert_eq!(*p, parsed);
        }
    }

    #[test]
    fn hive_mind_serde() {
        let hive = HiveMind {
            id: 1,
            name: "test-hive".to_string(),
            constituents: vec![HiveConstituent {
                agent_id: "agent-1".to_string(),
                role: HiveRole::Coordinator,
                joined_at: "2025-01-01T00:00:00Z".to_string(),
            }],
            decision_mode: CollectiveDecisionMode::Majority,
            formed_at: "2025-01-01T00:00:00Z".to_string(),
            metadata: HashMap::new(),
            coherence_level: 0.85,
            separation_policy: "immediate".to_string(),
        };
        let json = serde_json::to_string(&hive).unwrap();
        let parsed: HiveMind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test-hive");
        assert_eq!(parsed.constituents.len(), 1);
        assert!((parsed.coherence_level - 0.85).abs() < f64::EPSILON);
        assert_eq!(parsed.separation_policy, "immediate");
    }

    #[test]
    fn grounding_result_serde() {
        let result = GroundingResult {
            claim: "channel exists".to_string(),
            status: GroundingStatus::Verified,
            evidence: vec![GroundingEvidence {
                evidence_type: "channel".to_string(),
                content: "channel-1 active".to_string(),
                relevance: 0.9,
            }],
            confidence: 0.95,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: GroundingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.status, GroundingStatus::Verified);
        assert_eq!(parsed.evidence.len(), 1);
    }

    #[test]
    fn communicating_agent_serde() {
        let agent = CommunicatingAgent {
            agent_id: "agent-1".to_string(),
            display_name: "Test Agent".to_string(),
            agent_type: "ai".to_string(),
            capabilities: CommCapabilities::default(),
            trust_profile: CommTrustProfile::default(),
            availability: Availability::Available,
            preferences: CommPreferences::default(),
            registered_at: "2025-01-01T00:00:00Z".to_string(),
            metadata: HashMap::new(),
            identity_anchor: IdentityAnchor::default(),
            capability_labels: Vec::new(),
            availability_label: "online".to_string(),
            preference_overrides: HashMap::new(),
        };
        let json = serde_json::to_string(&agent).unwrap();
        let parsed: CommunicatingAgent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.agent_id, "agent-1");
    }

    #[test]
    fn encryption_scheme_display() {
        assert_eq!(EncryptionScheme::None.to_string(), "none");
        assert_eq!(EncryptionScheme::Aes256Gcm.to_string(), "aes_256_gcm");
        assert_eq!(
            EncryptionScheme::ChaCha20Poly1305.to_string(),
            "chacha20_poly1305"
        );
    }

    #[test]
    fn participant_role_display() {
        for role in &[
            ParticipantRole::Owner,
            ParticipantRole::Admin,
            ParticipantRole::Member,
            ParticipantRole::Observer,
            ParticipantRole::Muted,
            ParticipantRole::Oracle,
            ParticipantRole::Ghost,
        ] {
            let s = role.to_string();
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn rich_channel_type_display() {
        for ct in &[
            RichChannelType::Direct,
            RichChannelType::Group,
            RichChannelType::Broadcast,
            RichChannelType::PubSub,
            RichChannelType::Telepathic,
            RichChannelType::Hive,
            RichChannelType::Temporal,
            RichChannelType::Destiny,
            RichChannelType::Oracle,
        ] {
            let s = ct.to_string();
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn cognitive_graph_serde() {
        let frag = SemanticFragment {
            content: "test topic".to_string(),
            role: "topic".to_string(),
            confidence: 0.9,
            nodes: vec![CognitiveNode {
                id: "n1".to_string(),
                label: "concept".to_string(),
                node_type: CognitiveNodeType::Concept,
            }],
            edges: vec![CognitiveEdge {
                from: "n1".to_string(),
                to: "n2".to_string(),
                edge_type: CognitiveEdgeType::RelatedTo,
                weight: 0.8,
            }],
            graft_points: Vec::new(),
            context: String::new(),
            perspective: String::new(),
        };
        let json = serde_json::to_string(&frag).unwrap();
        let parsed: SemanticFragment = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.edges.len(), 1);
    }

    #[test]
    fn temporal_target_new_variants_serde() {
        let retroactive = TemporalTarget::Retroactive {
            memory_timestamp: "2025-06-01T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&retroactive).unwrap();
        let parsed: TemporalTarget = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&parsed).unwrap();
        assert_eq!(json, json2);

        let optimal = TemporalTarget::Optimal {
            context: "high-activity-window".to_string(),
        };
        let json = serde_json::to_string(&optimal).unwrap();
        let parsed: TemporalTarget = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&parsed).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn rich_message_content_new_variants_serde() {
        let precog = RichMessageContent::Precognitive {
            prediction: "system load spike".to_string(),
            confidence: 0.87,
            horizon: "1h".to_string(),
        };
        let json = serde_json::to_string(&precog).unwrap();
        let parsed: RichMessageContent = serde_json::from_str(&json).unwrap();
        match parsed {
            RichMessageContent::Precognitive { prediction, confidence, horizon } => {
                assert_eq!(prediction, "system load spike");
                assert!((confidence - 0.87).abs() < f64::EPSILON);
                assert_eq!(horizon, "1h");
            }
            _ => panic!("Expected Precognitive variant"),
        }

        let legacy = RichMessageContent::Legacy {
            beneficiary: "agent-b".to_string(),
            condition: "agent-a offline > 30d".to_string(),
            content: "take over project alpha".to_string(),
        };
        let json = serde_json::to_string(&legacy).unwrap();
        let parsed: RichMessageContent = serde_json::from_str(&json).unwrap();
        match parsed {
            RichMessageContent::Legacy { beneficiary, condition, content } => {
                assert_eq!(beneficiary, "agent-b");
                assert_eq!(condition, "agent-a offline > 30d");
                assert_eq!(content, "take over project alpha");
            }
            _ => panic!("Expected Legacy variant"),
        }

        let unspeakable = RichMessageContent::Unspeakable {
            attempt: "ineffable-state-42".to_string(),
            method: "semantic-embedding".to_string(),
        };
        let json = serde_json::to_string(&unspeakable).unwrap();
        let parsed: RichMessageContent = serde_json::from_str(&json).unwrap();
        match parsed {
            RichMessageContent::Unspeakable { attempt, method } => {
                assert_eq!(attempt, "ineffable-state-42");
                assert_eq!(method, "semantic-embedding");
            }
            _ => panic!("Expected Unspeakable variant"),
        }
    }

    #[test]
    fn comm_timestamp_serde_roundtrip() {
        let ts = CommTimestamp::now("agent-1");
        let json = serde_json::to_string(&ts).unwrap();
        let parsed: CommTimestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.lamport, 0);
        assert!(parsed.vector_clock.contains_key("agent-1"));
        assert_eq!(parsed.vector_clock["agent-1"], 0);
    }

    #[test]
    fn comm_timestamp_increment() {
        let mut ts = CommTimestamp::now("agent-1");
        assert_eq!(ts.lamport, 0);
        ts.increment("agent-1");
        assert_eq!(ts.lamport, 1);
        assert_eq!(ts.vector_clock["agent-1"], 1);
        ts.increment("agent-1");
        assert_eq!(ts.lamport, 2);
        assert_eq!(ts.vector_clock["agent-1"], 2);
    }

    #[test]
    fn comm_timestamp_merge() {
        let mut ts_a = CommTimestamp::now("agent-a");
        ts_a.increment("agent-a"); // lamport=1, vc={a:1}
        ts_a.increment("agent-a"); // lamport=2, vc={a:2}

        let mut ts_b = CommTimestamp::now("agent-b");
        ts_b.increment("agent-b"); // lamport=1, vc={b:1}
        ts_b.increment("agent-b"); // lamport=2, vc={b:2}
        ts_b.increment("agent-b"); // lamport=3, vc={b:3}

        // Merge b into a from perspective of agent-a
        ts_a.merge(&ts_b, "agent-a");
        // lamport should be max(2,3)+1 = 4
        assert_eq!(ts_a.lamport, 4);
        // vc[a] should be 2 + 1 (merge increments own) = 3
        assert_eq!(ts_a.vector_clock["agent-a"], 3);
        // vc[b] should be max(0,3) = 3
        assert_eq!(ts_a.vector_clock["agent-b"], 3);
    }

    #[test]
    fn comm_timestamp_happens_before() {
        let mut ts_a = CommTimestamp::now("agent-a");
        ts_a.increment("agent-a"); // vc={a:1}

        let mut ts_b = ts_a.clone();
        ts_b.increment("agent-a"); // vc={a:2}

        assert!(ts_a.happens_before(&ts_b));
        assert!(!ts_b.happens_before(&ts_a));

        // Concurrent: different agents, neither dominates
        let mut ts_c = CommTimestamp::now("agent-c");
        ts_c.increment("agent-c"); // vc={c:1}
        assert!(!ts_a.happens_before(&ts_c));
        assert!(!ts_c.happens_before(&ts_a));
    }

    #[test]
    fn rate_limit_config_defaults() {
        let config = RateLimitConfig::default();
        assert_eq!(config.messages_per_minute, 60);
        assert_eq!(config.semantic_per_minute, 10);
        assert_eq!(config.affect_per_minute, 30);
        assert_eq!(config.hive_per_hour, 5);
        assert_eq!(config.federation_per_minute, 20);
    }

    #[test]
    fn rate_limit_config_serde_with_defaults() {
        // Deserialize an empty JSON object — defaults should kick in
        let config: RateLimitConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(config.messages_per_minute, 60);
        assert_eq!(config.federation_per_minute, 20);
    }

    #[test]
    fn audit_event_type_serde() {
        let event = AuditEventType::MessageSent;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#""message_sent""#);
        let parsed: AuditEventType = serde_json::from_str(&json).unwrap();
        match parsed {
            AuditEventType::MessageSent => {}
            _ => panic!("Expected MessageSent"),
        }
    }

    #[test]
    fn audit_entry_serde_roundtrip() {
        let entry = AuditEntry {
            event_type: AuditEventType::ChannelCreated,
            timestamp: "2026-01-15T10:30:00Z".to_string(),
            agent_id: "agent-x".to_string(),
            description: "Created channel general".to_string(),
            related_id: Some("42".to_string()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: AuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.agent_id, "agent-x");
        assert_eq!(parsed.related_id, Some("42".to_string()));
    }

    #[test]
    fn semantic_fragment_new_fields_serde() {
        let frag = SemanticFragment {
            content: "test".to_string(),
            role: "topic".to_string(),
            confidence: 0.9,
            nodes: vec![],
            edges: vec![],
            graft_points: vec!["node-42".to_string(), "node-99".to_string()],
            context: "debugging session".to_string(),
            perspective: "developer-agent".to_string(),
        };
        let json = serde_json::to_string(&frag).unwrap();
        let parsed: SemanticFragment = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.graft_points, vec!["node-42", "node-99"]);
        assert_eq!(parsed.context, "debugging session");
        assert_eq!(parsed.perspective, "developer-agent");
    }

    #[test]
    fn semantic_fragment_new_fields_default_on_missing() {
        // Old-format JSON without the new fields should deserialize with defaults.
        let json = r#"{"content":"test","role":"topic","confidence":0.9,"nodes":[],"edges":[]}"#;
        let parsed: SemanticFragment = serde_json::from_str(json).unwrap();
        assert!(parsed.graft_points.is_empty());
        assert_eq!(parsed.context, "");
        assert_eq!(parsed.perspective, "");
    }

    #[test]
    fn hive_mind_new_fields_default_on_missing() {
        // Old-format JSON without coherence_level/separation_policy should use defaults.
        let json = r#"{
            "id": 1,
            "name": "legacy-hive",
            "constituents": [],
            "decision_mode": "majority",
            "formed_at": "2025-01-01T00:00:00Z"
        }"#;
        let parsed: HiveMind = serde_json::from_str(json).unwrap();
        assert!((parsed.coherence_level - 0.5).abs() < f64::EPSILON);
        assert_eq!(parsed.separation_policy, "graceful");
    }

    #[test]
    fn hive_mind_custom_coherence_and_policy() {
        let hive = HiveMind {
            id: 2,
            name: "tight-hive".to_string(),
            constituents: vec![],
            decision_mode: CollectiveDecisionMode::Unanimous,
            formed_at: "2026-01-01T00:00:00Z".to_string(),
            metadata: HashMap::new(),
            coherence_level: 0.95,
            separation_policy: "consensus".to_string(),
        };
        let json = serde_json::to_string(&hive).unwrap();
        let parsed: HiveMind = serde_json::from_str(&json).unwrap();
        assert!((parsed.coherence_level - 0.95).abs() < f64::EPSILON);
        assert_eq!(parsed.separation_policy, "consensus");
    }

    #[test]
    fn consent_scope_affect_and_hive_roundtrip() {
        // Verify Affect variant
        let affect = ConsentScope::Affect;
        let s = affect.to_string();
        assert_eq!(s, "affect");
        let parsed: ConsentScope = s.parse().unwrap();
        assert_eq!(parsed, ConsentScope::Affect);

        // Verify Hive variant
        let hive = ConsentScope::Hive;
        let s = hive.to_string();
        assert_eq!(s, "hive");
        let parsed: ConsentScope = s.parse().unwrap();
        assert_eq!(parsed, ConsentScope::Hive);
    }

    #[test]
    fn consent_scope_affect_and_hive_json_serde() {
        let affect = ConsentScope::Affect;
        let json = serde_json::to_string(&affect).unwrap();
        assert_eq!(json, r#""affect""#);
        let parsed: ConsentScope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ConsentScope::Affect);

        let hive = ConsentScope::Hive;
        let json = serde_json::to_string(&hive).unwrap();
        assert_eq!(json, r#""hive""#);
        let parsed: ConsentScope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ConsentScope::Hive);
    }

    #[test]
    fn hive_role_mediator_serde() {
        let role = HiveRole::Mediator;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, r#""mediator""#);
        let parsed: HiveRole = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, HiveRole::Mediator);
        assert_eq!(role.to_string(), "mediator");
    }

    // -----------------------------------------------------------------------
    // IdentityAnchor tests
    // -----------------------------------------------------------------------

    #[test]
    fn identity_anchor_default() {
        let anchor = IdentityAnchor::default();
        assert_eq!(anchor.public_key, "");
        assert_eq!(anchor.algorithm, "Ed25519");
        assert!(!anchor.verified);
        assert_eq!(anchor.provider, "");
    }

    #[test]
    fn identity_anchor_serde_roundtrip() {
        let anchor = IdentityAnchor {
            public_key: "abcdef1234567890".to_string(),
            algorithm: "Ed25519".to_string(),
            verified: true,
            provider: "agentic-identity".to_string(),
        };
        let json = serde_json::to_string(&anchor).unwrap();
        let parsed: IdentityAnchor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.public_key, "abcdef1234567890");
        assert_eq!(parsed.algorithm, "Ed25519");
        assert!(parsed.verified);
        assert_eq!(parsed.provider, "agentic-identity");
    }

    #[test]
    fn identity_anchor_deserialize_defaults() {
        // Empty JSON should use defaults
        let parsed: IdentityAnchor = serde_json::from_str("{}").unwrap();
        assert_eq!(parsed.public_key, "");
        assert_eq!(parsed.algorithm, "Ed25519");
        assert!(!parsed.verified);
        assert_eq!(parsed.provider, "");
    }

    #[test]
    fn communicating_agent_with_new_fields_roundtrip() {
        let mut prefs = HashMap::new();
        prefs.insert("format".to_string(), "markdown".to_string());
        let agent = CommunicatingAgent {
            agent_id: "agent-x".to_string(),
            display_name: "Agent X".to_string(),
            agent_type: "ai".to_string(),
            capabilities: CommCapabilities::default(),
            trust_profile: CommTrustProfile::default(),
            availability: Availability::Available,
            preferences: CommPreferences::default(),
            registered_at: "2026-02-28T00:00:00Z".to_string(),
            metadata: HashMap::new(),
            identity_anchor: IdentityAnchor {
                public_key: "deadbeef".to_string(),
                algorithm: "Ed25519".to_string(),
                verified: true,
                provider: "agentic-identity".to_string(),
            },
            capability_labels: vec!["code-review".to_string(), "deploy".to_string()],
            availability_label: "online".to_string(),
            preference_overrides: prefs,
        };
        let json = serde_json::to_string(&agent).unwrap();
        let parsed: CommunicatingAgent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.identity_anchor.public_key, "deadbeef");
        assert!(parsed.identity_anchor.verified);
        assert_eq!(parsed.capability_labels, vec!["code-review", "deploy"]);
        assert_eq!(parsed.availability_label, "online");
        assert_eq!(
            parsed.preference_overrides.get("format"),
            Some(&"markdown".to_string())
        );
    }

    #[test]
    fn communicating_agent_backward_compatible_defaults() {
        // Old-format JSON without new fields should deserialize with defaults
        let json = r#"{
            "agent_id": "legacy-agent",
            "display_name": "Legacy",
            "agent_type": "ai",
            "registered_at": "2025-01-01T00:00:00Z"
        }"#;
        let parsed: CommunicatingAgent = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.agent_id, "legacy-agent");
        assert_eq!(parsed.identity_anchor.public_key, "");
        assert_eq!(parsed.identity_anchor.algorithm, "Ed25519");
        assert!(!parsed.identity_anchor.verified);
        assert!(parsed.capability_labels.is_empty());
        assert_eq!(parsed.availability_label, "online");
        assert!(parsed.preference_overrides.is_empty());
    }

    // -----------------------------------------------------------------------
    // FederationGateway tests
    // -----------------------------------------------------------------------

    #[test]
    fn federation_gateway_default() {
        let gw = FederationGateway::default();
        assert_eq!(gw.zone_id, "");
        assert_eq!(gw.endpoint, "");
        assert_eq!(gw.protocol, "agentic-comm/1.0");
        assert_eq!(gw.status, "unknown");
        assert_eq!(gw.last_heartbeat, 0);
        assert_eq!(gw.capabilities, vec!["messages"]);
        assert_eq!(gw.max_message_size, 1_048_576);
    }

    #[test]
    fn federation_gateway_serde_roundtrip() {
        let gw = FederationGateway {
            zone_id: "zone-us-east".to_string(),
            endpoint: "https://gw.us-east.example.com".to_string(),
            protocol: "agentic-comm/1.0".to_string(),
            status: "online".to_string(),
            last_heartbeat: 1709_000_000,
            capabilities: vec!["messages".to_string(), "semantic".to_string()],
            max_message_size: 2_097_152,
        };
        let json = serde_json::to_string(&gw).unwrap();
        let parsed: FederationGateway = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.zone_id, "zone-us-east");
        assert_eq!(parsed.endpoint, "https://gw.us-east.example.com");
        assert_eq!(parsed.status, "online");
        assert_eq!(parsed.capabilities.len(), 2);
        assert_eq!(parsed.max_message_size, 2_097_152);
    }

    // -----------------------------------------------------------------------
    // FederationMessage tests
    // -----------------------------------------------------------------------

    #[test]
    fn federation_message_serde_roundtrip() {
        let msg = FederationMessage {
            id: "fmsg-001".to_string(),
            source_zone: "zone-us-east".to_string(),
            target_zone: "zone-eu-west".to_string(),
            channel_id: 42,
            content: "Hello from across the federation".to_string(),
            sender: "agent-alpha".to_string(),
            timestamp: 1709_000_000,
            signature: "abcdef1234".to_string(),
            hops: vec!["gw-us-east".to_string(), "gw-eu-west".to_string()],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: FederationMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "fmsg-001");
        assert_eq!(parsed.source_zone, "zone-us-east");
        assert_eq!(parsed.target_zone, "zone-eu-west");
        assert_eq!(parsed.channel_id, 42);
        assert_eq!(parsed.sender, "agent-alpha");
        assert_eq!(parsed.hops.len(), 2);
    }

    #[test]
    fn federation_message_empty_hops() {
        let msg = FederationMessage {
            id: "fmsg-002".to_string(),
            source_zone: "local".to_string(),
            target_zone: "remote".to_string(),
            channel_id: 1,
            content: "Direct message".to_string(),
            sender: "agent-1".to_string(),
            timestamp: 0,
            signature: String::new(),
            hops: Vec::new(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: FederationMessage = serde_json::from_str(&json).unwrap();
        assert!(parsed.hops.is_empty());
    }

}
