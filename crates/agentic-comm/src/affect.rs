//! AffectProcessor — emotional contagion model for agent communication.
//!
//! Implements the PAD (Pleasure-Arousal-Dominance) affect model with
//! contagion dynamics, resistance factors, and temporal decay.

use crate::types::AffectState;
use std::collections::HashMap;

/// Configuration for the affect contagion model.
#[derive(Debug, Clone)]
pub struct ContagionConfig {
    /// Base contagion strength (0.0-1.0), how strongly affect transfers
    pub base_strength: f64,
    /// Decay rate per second (affect returns to baseline over time)
    pub decay_rate: f64,
    /// Minimum change threshold (below this, affect changes are ignored)
    pub min_threshold: f64,
    /// Maximum affect value (clamp to this)
    pub max_value: f64,
}

impl Default for ContagionConfig {
    fn default() -> Self {
        Self {
            base_strength: 0.3,
            decay_rate: 0.01,
            min_threshold: 0.05,
            max_value: 1.0,
        }
    }
}

/// Tracks affect state and contagion dynamics for agents.
#[derive(Debug)]
pub struct AffectProcessor {
    /// Current affect state per agent
    states: HashMap<String, AgentAffect>,
    /// Contagion configuration
    config: ContagionConfig,
}

/// Per-agent affect tracking.
#[derive(Debug, Clone)]
pub struct AgentAffect {
    /// Current affect state (valence, arousal, dominance)
    pub state: AffectState,
    /// Resistance to contagion (0.0 = fully susceptible, 1.0 = immune)
    pub resistance: f64,
    /// Baseline state (what the agent returns to over time)
    pub baseline: AffectState,
    /// Last update timestamp (seconds since epoch)
    pub last_updated: u64,
    /// History of affect changes
    pub history: Vec<AffectChange>,
}

/// Record of an affect change event.
#[derive(Debug, Clone)]
pub struct AffectChange {
    /// Source agent that caused the change
    pub source: String,
    /// Affect delta applied
    pub delta_valence: f64,
    pub delta_arousal: f64,
    pub delta_dominance: f64,
    /// Effective contagion strength after resistance
    pub effective_strength: f64,
    /// Timestamp
    pub timestamp: u64,
}

impl AffectProcessor {
    pub fn new() -> Self {
        Self::with_config(ContagionConfig::default())
    }

    pub fn with_config(config: ContagionConfig) -> Self {
        Self {
            states: HashMap::new(),
            config,
        }
    }

    /// Register an agent with initial affect state and resistance.
    pub fn register_agent(
        &mut self,
        agent_id: &str,
        initial_state: AffectState,
        resistance: f64,
    ) {
        self.states.insert(
            agent_id.to_string(),
            AgentAffect {
                state: initial_state.clone(),
                resistance: resistance.clamp(0.0, 1.0),
                baseline: initial_state,
                last_updated: Self::now(),
                history: Vec::new(),
            },
        );
    }

    /// Get current affect state for an agent.
    pub fn get_state(&self, agent_id: &str) -> Option<&AffectState> {
        self.states.get(agent_id).map(|a| &a.state)
    }

    /// Get full agent affect info.
    pub fn get_agent_affect(&self, agent_id: &str) -> Option<&AgentAffect> {
        self.states.get(agent_id)
    }

    /// Set resistance level for an agent.
    pub fn set_resistance(&mut self, agent_id: &str, resistance: f64) -> bool {
        if let Some(agent) = self.states.get_mut(agent_id) {
            agent.resistance = resistance.clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }

    /// Apply affect contagion from sender to receiver.
    ///
    /// The effective change is modulated by:
    /// - contagion_strength (from the message)
    /// - receiver's resistance factor
    /// - base_strength from config
    ///
    /// Returns the effective strength applied (0.0 if blocked by resistance).
    pub fn apply_contagion(
        &mut self,
        sender_id: &str,
        receiver_id: &str,
        incoming_affect: &AffectState,
        contagion_strength: f64,
    ) -> f64 {
        // Get receiver's current state
        let receiver = match self.states.get(receiver_id) {
            Some(r) => r.clone(),
            None => return 0.0,
        };

        // Calculate effective strength: contagion * config_base * (1 - resistance)
        let effective =
            contagion_strength * self.config.base_strength * (1.0 - receiver.resistance);

        if effective < self.config.min_threshold {
            return 0.0;
        }

        // Calculate deltas
        let delta_v = (incoming_affect.valence - receiver.state.valence) * effective;
        let delta_a = (incoming_affect.arousal - receiver.state.arousal) * effective;
        let delta_d = (incoming_affect.dominance - receiver.state.dominance) * effective;

        // Apply to receiver
        if let Some(r) = self.states.get_mut(receiver_id) {
            r.state.valence = (r.state.valence + delta_v)
                .clamp(-self.config.max_value, self.config.max_value);
            r.state.arousal =
                (r.state.arousal + delta_a).clamp(0.0, self.config.max_value);
            r.state.dominance = (r.state.dominance + delta_d)
                .clamp(-self.config.max_value, self.config.max_value);
            r.last_updated = Self::now();
            r.history.push(AffectChange {
                source: sender_id.to_string(),
                delta_valence: delta_v,
                delta_arousal: delta_a,
                delta_dominance: delta_d,
                effective_strength: effective,
                timestamp: Self::now(),
            });
        }

        effective
    }

    /// Apply temporal decay — move all agents' states toward their baselines.
    /// Call this periodically (e.g., every second).
    pub fn apply_decay(&mut self, elapsed_seconds: f64) {
        let decay_factor = (-self.config.decay_rate * elapsed_seconds).exp();

        for agent in self.states.values_mut() {
            agent.state.valence = agent.baseline.valence
                + (agent.state.valence - agent.baseline.valence) * decay_factor;
            agent.state.arousal = agent.baseline.arousal
                + (agent.state.arousal - agent.baseline.arousal) * decay_factor;
            agent.state.dominance = agent.baseline.dominance
                + (agent.state.dominance - agent.baseline.dominance) * decay_factor;
        }
    }

    /// Get the affect distance between two agents (Euclidean in PAD space).
    pub fn affect_distance(&self, agent_a: &str, agent_b: &str) -> Option<f64> {
        let a = self.states.get(agent_a)?;
        let b = self.states.get(agent_b)?;

        let dv = a.state.valence - b.state.valence;
        let da = a.state.arousal - b.state.arousal;
        let dd = a.state.dominance - b.state.dominance;

        Some((dv * dv + da * da + dd * dd).sqrt())
    }

    /// Get number of registered agents.
    pub fn agent_count(&self) -> usize {
        self.states.len()
    }

    /// Get affect change history for an agent.
    pub fn get_history(&self, agent_id: &str) -> Option<&[AffectChange]> {
        self.states.get(agent_id).map(|a| a.history.as_slice())
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl Default for AffectProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AffectState, UrgencyLevel};

    /// Helper to build an AffectState with just PAD values.
    fn pad(valence: f64, arousal: f64, dominance: f64) -> AffectState {
        AffectState {
            valence,
            arousal,
            dominance,
            emotions: Vec::new(),
            urgency: UrgencyLevel::Normal,
            meta_confidence: 0.5,
        }
    }

    #[test]
    fn test_register_and_get_state() {
        let mut proc = AffectProcessor::new();
        proc.register_agent("alice", pad(0.5, 0.3, 0.6), 0.0);

        assert_eq!(proc.agent_count(), 1);

        let state = proc.get_state("alice").expect("alice should be registered");
        assert!((state.valence - 0.5).abs() < f64::EPSILON);
        assert!((state.arousal - 0.3).abs() < f64::EPSILON);
        assert!((state.dominance - 0.6).abs() < f64::EPSILON);

        // Unknown agent returns None
        assert!(proc.get_state("bob").is_none());
    }

    #[test]
    fn test_contagion_applies_with_no_resistance() {
        let mut proc = AffectProcessor::with_config(ContagionConfig {
            base_strength: 1.0,
            decay_rate: 0.0,
            min_threshold: 0.0,
            max_value: 1.0,
        });

        // Receiver starts at neutral (0,0,0.5), resistance = 0
        proc.register_agent("sender", pad(0.8, 0.9, 0.7), 0.0);
        proc.register_agent("receiver", pad(0.0, 0.0, 0.5), 0.0);

        // Full-strength contagion
        let effective = proc.apply_contagion(
            "sender",
            "receiver",
            &pad(0.8, 0.9, 0.7),
            1.0,
        );

        // effective = 1.0 * 1.0 * (1.0 - 0.0) = 1.0
        assert!((effective - 1.0).abs() < f64::EPSILON);

        let state = proc.get_state("receiver").unwrap();
        // delta_v = (0.8 - 0.0) * 1.0 = 0.8, new valence = 0.0 + 0.8 = 0.8
        assert!((state.valence - 0.8).abs() < 1e-10);
        // delta_a = (0.9 - 0.0) * 1.0 = 0.9, new arousal = 0.0 + 0.9 = 0.9
        assert!((state.arousal - 0.9).abs() < 1e-10);
        // delta_d = (0.7 - 0.5) * 1.0 = 0.2, new dominance = 0.5 + 0.2 = 0.7
        assert!((state.dominance - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_contagion_blocked_by_high_resistance() {
        let mut proc = AffectProcessor::with_config(ContagionConfig {
            base_strength: 1.0,
            decay_rate: 0.0,
            min_threshold: 0.0,
            max_value: 1.0,
        });

        proc.register_agent("sender", pad(0.8, 0.9, 0.7), 0.0);
        proc.register_agent("receiver", pad(0.0, 0.0, 0.5), 1.0); // fully resistant

        let effective = proc.apply_contagion(
            "sender",
            "receiver",
            &pad(0.8, 0.9, 0.7),
            1.0,
        );

        // effective = 1.0 * 1.0 * (1.0 - 1.0) = 0.0
        assert!((effective - 0.0).abs() < f64::EPSILON);

        // Receiver should be unchanged
        let state = proc.get_state("receiver").unwrap();
        assert!((state.valence - 0.0).abs() < f64::EPSILON);
        assert!((state.arousal - 0.0).abs() < f64::EPSILON);
        assert!((state.dominance - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_contagion_partial_resistance() {
        let mut proc = AffectProcessor::with_config(ContagionConfig {
            base_strength: 1.0,
            decay_rate: 0.0,
            min_threshold: 0.0,
            max_value: 1.0,
        });

        proc.register_agent("sender", pad(0.8, 0.8, 0.8), 0.0);
        proc.register_agent("receiver", pad(0.0, 0.0, 0.0), 0.5); // 50% resistant

        let effective = proc.apply_contagion(
            "sender",
            "receiver",
            &pad(0.8, 0.8, 0.8),
            1.0,
        );

        // effective = 1.0 * 1.0 * (1.0 - 0.5) = 0.5
        assert!((effective - 0.5).abs() < f64::EPSILON);

        let state = proc.get_state("receiver").unwrap();
        // delta_v = (0.8 - 0.0) * 0.5 = 0.4, new valence = 0.0 + 0.4 = 0.4
        assert!((state.valence - 0.4).abs() < 1e-10);
        // delta_a = (0.8 - 0.0) * 0.5 = 0.4, new arousal = 0.0 + 0.4 = 0.4
        assert!((state.arousal - 0.4).abs() < 1e-10);
        // delta_d = (0.8 - 0.0) * 0.5 = 0.4, new dominance = 0.0 + 0.4 = 0.4
        assert!((state.dominance - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_decay_moves_toward_baseline() {
        let mut proc = AffectProcessor::with_config(ContagionConfig {
            base_strength: 1.0,
            decay_rate: 0.1, // 10% per second
            min_threshold: 0.0,
            max_value: 1.0,
        });

        // Baseline is (0, 0, 0.5) — register then perturb
        proc.register_agent("agent", pad(0.0, 0.0, 0.5), 0.0);

        // Manually push agent's state away from baseline via contagion
        proc.apply_contagion(
            "external",
            "agent",
            &pad(1.0, 1.0, 1.0),
            1.0,
        );

        let before = proc.get_state("agent").unwrap().clone();
        assert!(before.valence > 0.5); // was pushed up

        // Simulate 10 seconds of decay
        proc.apply_decay(10.0);

        let after = proc.get_state("agent").unwrap();
        // After decay, valence should be closer to baseline (0.0)
        assert!(after.valence.abs() < before.valence.abs());
        // Arousal should be closer to baseline (0.0)
        assert!(after.arousal < before.arousal);
        // Dominance should be closer to baseline (0.5)
        assert!((after.dominance - 0.5).abs() < (before.dominance - 0.5).abs());
    }

    #[test]
    fn test_affect_distance() {
        let mut proc = AffectProcessor::new();
        proc.register_agent("a", pad(0.0, 0.0, 0.0), 0.0);
        proc.register_agent("b", pad(1.0, 0.0, 0.0), 0.0);

        let dist = proc.affect_distance("a", "b").unwrap();
        // Distance should be 1.0 (only valence differs by 1.0)
        assert!((dist - 1.0).abs() < 1e-10);

        // Distance to self should be 0
        let self_dist = proc.affect_distance("a", "a").unwrap();
        assert!((self_dist - 0.0).abs() < 1e-10);

        // Unknown agent returns None
        assert!(proc.affect_distance("a", "unknown").is_none());
    }

    #[test]
    fn test_contagion_history_recorded() {
        let mut proc = AffectProcessor::with_config(ContagionConfig {
            base_strength: 1.0,
            decay_rate: 0.0,
            min_threshold: 0.0,
            max_value: 1.0,
        });

        proc.register_agent("sender", pad(0.5, 0.5, 0.5), 0.0);
        proc.register_agent("receiver", pad(0.0, 0.0, 0.0), 0.0);

        // Apply contagion twice
        proc.apply_contagion("sender", "receiver", &pad(0.5, 0.5, 0.5), 1.0);
        proc.apply_contagion("sender", "receiver", &pad(0.8, 0.8, 0.8), 0.5);

        let history = proc.get_history("receiver").expect("receiver should exist");
        assert_eq!(history.len(), 2);

        assert_eq!(history[0].source, "sender");
        assert!((history[0].effective_strength - 1.0).abs() < f64::EPSILON);

        assert_eq!(history[1].source, "sender");
        assert!((history[1].effective_strength - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_set_resistance() {
        let mut proc = AffectProcessor::new();
        proc.register_agent("agent", pad(0.0, 0.0, 0.5), 0.2);

        // Verify initial resistance
        let affect = proc.get_agent_affect("agent").unwrap();
        assert!((affect.resistance - 0.2).abs() < f64::EPSILON);

        // Change resistance
        assert!(proc.set_resistance("agent", 0.8));
        let affect = proc.get_agent_affect("agent").unwrap();
        assert!((affect.resistance - 0.8).abs() < f64::EPSILON);

        // Resistance is clamped to [0.0, 1.0]
        proc.set_resistance("agent", 1.5);
        let affect = proc.get_agent_affect("agent").unwrap();
        assert!((affect.resistance - 1.0).abs() < f64::EPSILON);

        proc.set_resistance("agent", -0.3);
        let affect = proc.get_agent_affect("agent").unwrap();
        assert!((affect.resistance - 0.0).abs() < f64::EPSILON);

        // Unknown agent returns false
        assert!(!proc.set_resistance("unknown", 0.5));
    }
}
