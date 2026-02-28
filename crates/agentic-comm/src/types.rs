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
        };
        let json = serde_json::to_string(&hive).unwrap();
        let parsed: HiveMind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test-hive");
        assert_eq!(parsed.constituents.len(), 1);
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
        };
        let json = serde_json::to_string(&frag).unwrap();
        let parsed: SemanticFragment = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.edges.len(), 1);
    }
}
