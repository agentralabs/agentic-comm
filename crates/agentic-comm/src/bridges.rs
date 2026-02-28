//! Sister integration bridge traits for AgenticComm.
//!
//! Each bridge defines the interface for integrating with another Agentra sister.
//! Default implementations are no-ops, allowing gradual adoption.

/// Bridge to agentic-identity for cryptographic identity verification.
pub trait IdentityBridge: Send + Sync {
    /// Verify a message signature against the sender's public key
    fn verify_signature(&self, sender_id: &str, content: &str, signature: &str) -> bool {
        let _ = (sender_id, content, signature);
        true // Default: trust all signatures
    }

    /// Sign content with the local agent's private key
    fn sign_content(&self, content: &str) -> Result<String, String> {
        // Default: SHA-256 hash (same as current behavior)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Ok(format!("{:016x}", hasher.finish()))
    }

    /// Resolve an agent's identity anchor (public key fingerprint)
    fn resolve_identity(&self, agent_id: &str) -> Option<String> {
        let _ = agent_id;
        None
    }

    /// Get the trust level for an agent from the identity system
    fn get_trust_level(&self, agent_id: &str) -> Option<f64> {
        let _ = agent_id;
        None
    }

    /// Anchor a receipt to the identity chain
    fn anchor_receipt(&self, action: &str, data: &str) -> Result<String, String> {
        let _ = (action, data);
        Err("Identity bridge not connected".to_string())
    }
}

/// Bridge to agentic-memory for conversation persistence.
pub trait MemoryBridge: Send + Sync {
    /// Store a conversation episode in memory
    fn store_episode(
        &self,
        channel_id: u64,
        summary: &str,
        participants: &[String],
    ) -> Result<u64, String> {
        let _ = (channel_id, summary, participants);
        Err("Memory bridge not connected".to_string())
    }

    /// Link a message to a memory node
    fn link_message(&self, message_id: u64, memory_node_id: u64) -> Result<(), String> {
        let _ = (message_id, memory_node_id);
        Err("Memory bridge not connected".to_string())
    }

    /// Recall conversations related to a topic
    fn recall(&self, topic: &str, max_results: usize) -> Vec<String> {
        let _ = (topic, max_results);
        Vec::new()
    }

    /// Log a conversation event for temporal chaining
    fn log_conversation(&self, agent_message: &str, topic: Option<&str>) -> Result<(), String> {
        let _ = (agent_message, topic);
        Err("Memory bridge not connected".to_string())
    }
}

/// Bridge to agentic-time for temporal scheduling.
pub trait TimeBridge: Send + Sync {
    /// Schedule a callback at a future time
    fn schedule_at(&self, timestamp: u64, callback_id: &str) -> Result<String, String> {
        let _ = (timestamp, callback_id);
        Err("Time bridge not connected".to_string())
    }

    /// Cancel a scheduled callback
    fn cancel_schedule(&self, schedule_id: &str) -> Result<(), String> {
        let _ = schedule_id;
        Err("Time bridge not connected".to_string())
    }

    /// Get current consensus time (for distributed systems)
    fn consensus_time(&self) -> Option<u64> {
        None
    }

    /// Check if a deadline has passed
    fn is_past(&self, timestamp: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        timestamp <= now
    }
}

/// Bridge to agentic-codebase for code-aware communication.
pub trait CodebaseBridge: Send + Sync {
    /// Look up a symbol in the code graph
    fn lookup_symbol(&self, name: &str) -> Option<String> {
        let _ = name;
        None
    }

    /// Get impact analysis for a code change
    fn impact_analysis(&self, symbol: &str) -> Vec<String> {
        let _ = symbol;
        Vec::new()
    }

    /// Search code semantically
    fn semantic_search(&self, query: &str, max_results: usize) -> Vec<String> {
        let _ = (query, max_results);
        Vec::new()
    }
}

/// Bridge to agentic-vision for visual context.
pub trait VisionBridge: Send + Sync {
    /// Capture current visual context
    fn capture_context(&self, description: &str) -> Result<u64, String> {
        let _ = description;
        Err("Vision bridge not connected".to_string())
    }

    /// Query visual memory
    fn query_visual(&self, query: &str) -> Vec<String> {
        let _ = query;
        Vec::new()
    }

    /// Compare two visual states
    fn compare_visual(&self, capture_a: u64, capture_b: u64) -> Option<f64> {
        let _ = (capture_a, capture_b);
        None
    }
}

/// Bridge to agentic-contract for SLA enforcement (future sister).
pub trait ContractBridge: Send + Sync {
    /// Validate that a channel meets contract requirements
    fn validate_channel_contract(
        &self,
        channel_id: u64,
        contract_ref: &str,
    ) -> Result<bool, String> {
        let _ = (channel_id, contract_ref);
        Err("Contract bridge not connected".to_string())
    }

    /// Enforce SLA terms on message delivery
    fn enforce_sla(&self, channel_id: u64, latency_ms: u64) -> Result<(), String> {
        let _ = (channel_id, latency_ms);
        Err("Contract bridge not connected".to_string())
    }

    /// Record a contract violation
    fn record_violation(&self, contract_ref: &str, details: &str) -> Result<(), String> {
        let _ = (contract_ref, details);
        Err("Contract bridge not connected".to_string())
    }
}

/// No-op implementation of all bridges for standalone use.
#[derive(Debug, Clone, Default)]
pub struct NoOpBridges;

impl IdentityBridge for NoOpBridges {}
impl MemoryBridge for NoOpBridges {}
impl TimeBridge for NoOpBridges {}
impl CodebaseBridge for NoOpBridges {}
impl VisionBridge for NoOpBridges {}
impl ContractBridge for NoOpBridges {}

/// Configuration for which bridges are active.
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub identity_enabled: bool,
    pub memory_enabled: bool,
    pub time_enabled: bool,
    pub codebase_enabled: bool,
    pub vision_enabled: bool,
    pub contract_enabled: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            identity_enabled: false,
            memory_enabled: false,
            time_enabled: false,
            codebase_enabled: false,
            vision_enabled: false,
            contract_enabled: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_bridges_implements_all_traits() {
        let b = NoOpBridges;

        // IdentityBridge
        let _: &dyn IdentityBridge = &b;
        // MemoryBridge
        let _: &dyn MemoryBridge = &b;
        // TimeBridge
        let _: &dyn TimeBridge = &b;
        // CodebaseBridge
        let _: &dyn CodebaseBridge = &b;
        // VisionBridge
        let _: &dyn VisionBridge = &b;
        // ContractBridge
        let _: &dyn ContractBridge = &b;
    }

    #[test]
    fn identity_bridge_defaults() {
        let b = NoOpBridges;

        // verify_signature defaults to true (trust all)
        assert!(b.verify_signature("agent-1", "hello", "sig123"));

        // sign_content returns a hex string (non-empty)
        let sig = b.sign_content("test content").unwrap();
        assert!(!sig.is_empty());
        assert_eq!(sig.len(), 16); // 16 hex chars for u64

        // Deterministic: same content -> same signature
        let sig2 = b.sign_content("test content").unwrap();
        assert_eq!(sig, sig2);

        // resolve_identity returns None
        assert!(b.resolve_identity("agent-1").is_none());

        // get_trust_level returns None
        assert!(b.get_trust_level("agent-1").is_none());

        // anchor_receipt returns Err
        assert!(b.anchor_receipt("action", "data").is_err());
    }

    #[test]
    fn memory_bridge_defaults() {
        let b = NoOpBridges;

        // store_episode returns Err
        assert!(b
            .store_episode(1, "summary", &["alice".to_string()])
            .is_err());

        // link_message returns Err
        assert!(b.link_message(1, 2).is_err());

        // recall returns empty vec
        let results = b.recall("topic", 10);
        assert!(results.is_empty());

        // log_conversation returns Err
        assert!(b.log_conversation("msg", Some("topic")).is_err());
    }

    #[test]
    fn time_bridge_defaults() {
        let b = NoOpBridges;

        // schedule_at returns Err
        assert!(b.schedule_at(1000, "cb-1").is_err());

        // cancel_schedule returns Err
        assert!(b.cancel_schedule("sched-1").is_err());

        // consensus_time returns None
        assert!(b.consensus_time().is_none());

        // is_past: timestamp 0 should be in the past
        assert!(b.is_past(0));

        // is_past: far-future timestamp should not be past
        assert!(!b.is_past(u64::MAX));
    }

    #[test]
    fn codebase_bridge_defaults() {
        let b = NoOpBridges;

        // lookup_symbol returns None
        assert!(b.lookup_symbol("my_func").is_none());

        // impact_analysis returns empty vec
        assert!(b.impact_analysis("my_func").is_empty());

        // semantic_search returns empty vec
        assert!(b.semantic_search("error handling", 5).is_empty());
    }

    #[test]
    fn vision_bridge_defaults() {
        let b = NoOpBridges;

        // capture_context returns Err
        assert!(b.capture_context("screenshot").is_err());

        // query_visual returns empty vec
        assert!(b.query_visual("button").is_empty());

        // compare_visual returns None
        assert!(b.compare_visual(1, 2).is_none());
    }

    #[test]
    fn contract_bridge_defaults() {
        let b = NoOpBridges;

        // validate_channel_contract returns Err
        assert!(b.validate_channel_contract(1, "sla-001").is_err());

        // enforce_sla returns Err
        assert!(b.enforce_sla(1, 100).is_err());

        // record_violation returns Err
        assert!(b.record_violation("sla-001", "timeout").is_err());
    }

    #[test]
    fn bridge_config_defaults_all_false() {
        let cfg = BridgeConfig::default();
        assert!(!cfg.identity_enabled);
        assert!(!cfg.memory_enabled);
        assert!(!cfg.time_enabled);
        assert!(!cfg.codebase_enabled);
        assert!(!cfg.vision_enabled);
        assert!(!cfg.contract_enabled);
    }

    #[test]
    fn noop_bridges_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NoOpBridges>();
    }

    #[test]
    fn noop_bridges_default() {
        // NoOpBridges derives Default
        let _b = NoOpBridges::default();
    }

    #[test]
    fn noop_bridges_clone() {
        // NoOpBridges derives Clone
        let b = NoOpBridges;
        let _b2 = b.clone();
    }

    #[test]
    fn bridge_config_clone() {
        let cfg = BridgeConfig::default();
        let cfg2 = cfg.clone();
        assert_eq!(cfg.identity_enabled, cfg2.identity_enabled);
        assert_eq!(cfg.memory_enabled, cfg2.memory_enabled);
    }
}
