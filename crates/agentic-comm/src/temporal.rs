//! TemporalScheduler — advanced temporal message scheduling.
//!
//! Manages message delivery based on temporal targets including
//! conditional delivery, optimal timing, and retroactive messaging.

use crate::types::*;
use std::collections::{BTreeMap, HashMap};

/// A pending temporal message.
#[derive(Debug, Clone)]
pub struct PendingMessage {
    pub id: u64,
    pub channel_id: u64,
    pub sender: String,
    pub content: String,
    pub target: TemporalTarget,
    pub created_at: u64,
    pub attempts: u32,
    pub max_attempts: u32,
    pub status: PendingStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PendingStatus {
    Waiting,
    ReadyToDeliver,
    Delivered,
    Failed(String),
    Cancelled,
    Expired,
}

/// Result of checking which messages are ready.
#[derive(Debug)]
pub struct DeliveryCheck {
    pub ready: Vec<u64>,
    pub waiting: usize,
    pub expired: usize,
    pub failed: usize,
}

/// Advanced temporal scheduler.
pub struct TemporalScheduler {
    /// All pending messages by ID.
    messages: HashMap<u64, PendingMessage>,
    /// Time-ordered index (timestamp -> message IDs).
    time_index: BTreeMap<u64, Vec<u64>>,
    /// Next message ID.
    next_id: u64,
    /// Condition evaluator registry (condition_name -> evaluator fn).
    conditions: HashMap<String, Box<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl std::fmt::Debug for TemporalScheduler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemporalScheduler")
            .field("messages", &self.messages)
            .field("time_index", &self.time_index)
            .field("next_id", &self.next_id)
            .field("conditions", &format!("<{} registered>", self.conditions.len()))
            .finish()
    }
}

impl TemporalScheduler {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
            time_index: BTreeMap::new(),
            next_id: 1,
            conditions: HashMap::new(),
        }
    }

    /// Schedule a message for temporal delivery.
    pub fn schedule(
        &mut self,
        channel_id: u64,
        sender: &str,
        content: &str,
        target: TemporalTarget,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let now = Self::now();
        let deliver_at = match &target {
            TemporalTarget::Immediate => now,
            TemporalTarget::FutureAbsolute { deliver_at } => {
                // Parse ISO 8601 string to epoch seconds; fall back to now + 3600
                chrono::DateTime::parse_from_rfc3339(deliver_at)
                    .map(|dt| dt.timestamp() as u64)
                    .unwrap_or(now + 3600)
            }
            TemporalTarget::FutureRelative { delay_seconds } => now + delay_seconds,
            TemporalTarget::Conditional { .. } => u64::MAX, // Check condition on each tick
            TemporalTarget::Eternal => u64::MAX,
            TemporalTarget::Retroactive { .. } => now, // Deliver immediately with retroactive metadata
            TemporalTarget::Optimal { .. } => now + 60, // Default: wait 60s for optimal timing
        };

        let msg = PendingMessage {
            id,
            channel_id,
            sender: sender.to_string(),
            content: content.to_string(),
            target,
            created_at: now,
            attempts: 0,
            max_attempts: 3,
            status: PendingStatus::Waiting,
        };

        self.messages.insert(id, msg);
        self.time_index.entry(deliver_at).or_default().push(id);
        id
    }

    /// Register a condition evaluator.
    pub fn register_condition<F>(&mut self, name: &str, evaluator: F)
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.conditions.insert(name.to_string(), Box::new(evaluator));
    }

    /// Check which messages are ready for delivery.
    pub fn check_ready(&mut self) -> DeliveryCheck {
        let now = Self::now();
        let mut ready = Vec::new();
        let mut expired = 0;
        let mut failed = 0;

        // Check time-based messages
        let ready_times: Vec<u64> = self.time_index.range(..=now).map(|(k, _)| *k).collect();
        for time in ready_times {
            if let Some(ids) = self.time_index.remove(&time) {
                for id in ids {
                    if let Some(msg) = self.messages.get_mut(&id) {
                        if msg.status == PendingStatus::Waiting {
                            msg.status = PendingStatus::ReadyToDeliver;
                            ready.push(id);
                        }
                    }
                }
            }
        }

        // Check conditional messages
        let conditional_ids: Vec<u64> = self
            .messages
            .iter()
            .filter(|(_, m)| {
                matches!(&m.target, TemporalTarget::Conditional { .. })
                    && m.status == PendingStatus::Waiting
            })
            .map(|(id, _)| *id)
            .collect();

        for id in conditional_ids {
            if let Some(msg) = self.messages.get(&id) {
                if let TemporalTarget::Conditional { condition } = &msg.target {
                    // Try to evaluate condition
                    let condition_met = self
                        .conditions
                        .get(condition.as_str())
                        .map(|f| f(&msg.content))
                        .unwrap_or(false);

                    if condition_met {
                        if let Some(msg) = self.messages.get_mut(&id) {
                            msg.status = PendingStatus::ReadyToDeliver;
                            ready.push(id);
                        }
                    }
                }
            }
        }

        // Count statuses
        let waiting = self
            .messages
            .values()
            .filter(|m| m.status == PendingStatus::Waiting)
            .count();
        for msg in self.messages.values() {
            match &msg.status {
                PendingStatus::Expired => expired += 1,
                PendingStatus::Failed(_) => failed += 1,
                _ => {}
            }
        }

        DeliveryCheck {
            ready,
            waiting,
            expired,
            failed,
        }
    }

    /// Mark a message as delivered.
    pub fn mark_delivered(&mut self, id: u64) -> bool {
        if let Some(msg) = self.messages.get_mut(&id) {
            msg.status = PendingStatus::Delivered;
            true
        } else {
            false
        }
    }

    /// Cancel a pending message.
    pub fn cancel(&mut self, id: u64) -> bool {
        if let Some(msg) = self.messages.get_mut(&id) {
            if msg.status == PendingStatus::Waiting {
                msg.status = PendingStatus::Cancelled;
                return true;
            }
        }
        false
    }

    /// Get a pending message by ID.
    pub fn get(&self, id: u64) -> Option<&PendingMessage> {
        self.messages.get(&id)
    }

    /// List all pending messages by status.
    pub fn list_by_status(&self, status: &PendingStatus) -> Vec<&PendingMessage> {
        self.messages
            .values()
            .filter(|m| &m.status == status)
            .collect()
    }

    /// Get total message counts: (waiting, delivered, cancelled, failed).
    pub fn stats(&self) -> (usize, usize, usize, usize) {
        let waiting = self
            .messages
            .values()
            .filter(|m| m.status == PendingStatus::Waiting)
            .count();
        let delivered = self
            .messages
            .values()
            .filter(|m| m.status == PendingStatus::Delivered)
            .count();
        let cancelled = self
            .messages
            .values()
            .filter(|m| m.status == PendingStatus::Cancelled)
            .count();
        let failed = self
            .messages
            .values()
            .filter(|m| matches!(m.status, PendingStatus::Failed(_)))
            .count();
        (waiting, delivered, cancelled, failed)
    }

    /// Expire messages older than max_age_secs.
    pub fn expire_old(&mut self, max_age_secs: u64) -> usize {
        let now = Self::now();
        let mut expired = 0;
        for msg in self.messages.values_mut() {
            if msg.status == PendingStatus::Waiting && (now - msg.created_at) > max_age_secs {
                msg.status = PendingStatus::Expired;
                expired += 1;
            }
        }
        expired
    }

    /// Get count of all messages.
    pub fn total_count(&self) -> usize {
        self.messages.len()
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl Default for TemporalScheduler {
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

    #[test]
    fn test_schedule_immediate() {
        let mut scheduler = TemporalScheduler::new();
        let id = scheduler.schedule(1, "alice", "hello", TemporalTarget::Immediate);
        assert_eq!(id, 1);

        let check = scheduler.check_ready();
        assert!(check.ready.contains(&id), "immediate message should be ready");
        assert_eq!(check.waiting, 0);
    }

    #[test]
    fn test_schedule_future() {
        let mut scheduler = TemporalScheduler::new();
        // Schedule 1 hour in the future
        let id = scheduler.schedule(
            1,
            "alice",
            "later",
            TemporalTarget::FutureRelative {
                delay_seconds: 3600,
            },
        );
        assert_eq!(id, 1);

        let check = scheduler.check_ready();
        assert!(
            !check.ready.contains(&id),
            "future message should not be ready yet"
        );
        assert_eq!(check.waiting, 1);
    }

    #[test]
    fn test_cancel_pending() {
        let mut scheduler = TemporalScheduler::new();
        let id = scheduler.schedule(
            1,
            "alice",
            "cancel me",
            TemporalTarget::FutureRelative {
                delay_seconds: 3600,
            },
        );

        assert!(scheduler.cancel(id));
        let msg = scheduler.get(id).unwrap();
        assert_eq!(msg.status, PendingStatus::Cancelled);

        // Cancelling again should fail (not in Waiting state)
        assert!(!scheduler.cancel(id));
    }

    #[test]
    fn test_mark_delivered() {
        let mut scheduler = TemporalScheduler::new();
        let id = scheduler.schedule(1, "alice", "hello", TemporalTarget::Immediate);

        // Move to ready first
        let _check = scheduler.check_ready();

        assert!(scheduler.mark_delivered(id));
        let msg = scheduler.get(id).unwrap();
        assert_eq!(msg.status, PendingStatus::Delivered);

        // Non-existent ID returns false
        assert!(!scheduler.mark_delivered(9999));
    }

    #[test]
    fn test_conditional_delivery() {
        let mut scheduler = TemporalScheduler::new();

        // Register condition that always triggers
        scheduler.register_condition("agent_online", |_content| true);

        let id = scheduler.schedule(
            1,
            "alice",
            "when online",
            TemporalTarget::Conditional {
                condition: "agent_online".to_string(),
            },
        );

        let check = scheduler.check_ready();
        assert!(
            check.ready.contains(&id),
            "conditional message with satisfied condition should be ready"
        );

        // Now test a condition that never triggers
        let mut scheduler2 = TemporalScheduler::new();
        scheduler2.register_condition("never_true", |_content| false);

        let id2 = scheduler2.schedule(
            1,
            "bob",
            "never",
            TemporalTarget::Conditional {
                condition: "never_true".to_string(),
            },
        );

        let check2 = scheduler2.check_ready();
        assert!(
            !check2.ready.contains(&id2),
            "conditional message with unsatisfied condition should NOT be ready"
        );
        assert_eq!(check2.waiting, 1);
    }

    #[test]
    fn test_expire_old() {
        let mut scheduler = TemporalScheduler::new();

        // Schedule a future message
        let id = scheduler.schedule(
            1,
            "alice",
            "old message",
            TemporalTarget::FutureRelative {
                delay_seconds: 7200,
            },
        );

        // Manually set created_at to the past to simulate aging
        if let Some(msg) = scheduler.messages.get_mut(&id) {
            msg.created_at = TemporalScheduler::now().saturating_sub(500);
        }

        // Expire messages older than 100 seconds
        let expired = scheduler.expire_old(100);
        assert_eq!(expired, 1);

        let msg = scheduler.get(id).unwrap();
        assert_eq!(msg.status, PendingStatus::Expired);
    }

    #[test]
    fn test_stats() {
        let mut scheduler = TemporalScheduler::new();

        // 1 immediate (will become ready)
        let id1 = scheduler.schedule(1, "alice", "msg1", TemporalTarget::Immediate);
        // 1 future (stays waiting)
        let _id2 = scheduler.schedule(
            1,
            "bob",
            "msg2",
            TemporalTarget::FutureRelative {
                delay_seconds: 3600,
            },
        );
        // 1 to cancel
        let id3 = scheduler.schedule(
            1,
            "carol",
            "msg3",
            TemporalTarget::FutureRelative {
                delay_seconds: 7200,
            },
        );

        // Process ready
        let _check = scheduler.check_ready();

        // Deliver id1
        scheduler.mark_delivered(id1);
        // Cancel id3
        scheduler.cancel(id3);

        let (waiting, delivered, cancelled, failed) = scheduler.stats();
        assert_eq!(waiting, 1, "one future message still waiting");
        assert_eq!(delivered, 1, "one immediate message delivered");
        assert_eq!(cancelled, 1, "one message cancelled");
        assert_eq!(failed, 0, "no failures");
        assert_eq!(scheduler.total_count(), 3);
    }

    #[test]
    fn test_retroactive_target() {
        let mut scheduler = TemporalScheduler::new();

        // Retroactive messages should be delivered immediately
        let id = scheduler.schedule(
            1,
            "alice",
            "past event",
            TemporalTarget::Retroactive {
                memory_timestamp: "2025-01-01T00:00:00Z".to_string(),
            },
        );

        let check = scheduler.check_ready();
        assert!(
            check.ready.contains(&id),
            "retroactive message should be ready immediately"
        );

        let msg = scheduler.get(id).unwrap();
        assert_eq!(msg.status, PendingStatus::ReadyToDeliver);
        // Verify the target preserves the memory_timestamp
        if let TemporalTarget::Retroactive { memory_timestamp } = &msg.target {
            assert_eq!(memory_timestamp, "2025-01-01T00:00:00Z");
        } else {
            panic!("expected Retroactive target");
        }
    }
}
