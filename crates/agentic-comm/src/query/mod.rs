//! CommQueryEngine — indexed query engine for fast message and channel lookups.
//!
//! Provides B-tree indexed access to messages by timestamp, sender,
//! and channel, plus full-text search over message content.

pub mod intent;
pub mod delta;
pub mod budget;
pub mod pagination;

pub use intent::ExtractionIntent;
pub use delta::{ChangeType, DeltaQuery};
pub use budget::TokenBudget;
pub use pagination::CursorPage;

use std::collections::{BTreeMap, HashMap, HashSet};

/// An indexed message record for fast lookup.
#[derive(Debug, Clone)]
pub struct IndexedMessage {
    pub id: u64,
    pub channel_id: u64,
    pub sender: String,
    pub content: String,
    pub timestamp: u64,
    pub message_type: String,
}

/// An indexed channel record.
#[derive(Debug, Clone)]
pub struct IndexedChannel {
    pub id: u64,
    pub name: String,
    pub channel_type: String,
    pub state: String,
    pub participants: Vec<String>,
    pub message_count: u64,
    pub created_at: u64,
}

/// Query filters for messages.
#[derive(Debug, Default, Clone)]
pub struct MessageQuery {
    pub channel_id: Option<u64>,
    pub sender: Option<String>,
    pub since: Option<u64>,
    pub until: Option<u64>,
    pub content_contains: Option<String>,
    pub message_type: Option<String>,
    pub limit: Option<usize>,
}

/// Query filters for channels.
#[derive(Debug, Default, Clone)]
pub struct ChannelQuery {
    pub name_contains: Option<String>,
    pub channel_type: Option<String>,
    pub state: Option<String>,
    pub participant: Option<String>,
    pub limit: Option<usize>,
}

/// Query result with metadata.
#[derive(Debug)]
pub struct QueryResult<T> {
    pub items: Vec<T>,
    pub total_matching: usize,
    pub truncated: bool,
}

/// Indexed query engine for fast lookups.
#[derive(Debug, Default)]
pub struct CommQueryEngine {
    /// Messages indexed by ID
    messages: HashMap<u64, IndexedMessage>,
    /// B-tree index: timestamp -> message IDs (for time-range queries)
    time_index: BTreeMap<u64, Vec<u64>>,
    /// Index: sender -> message IDs
    sender_index: HashMap<String, Vec<u64>>,
    /// Index: channel_id -> message IDs
    channel_index: HashMap<u64, Vec<u64>>,
    /// Channels indexed by ID
    channels: HashMap<u64, IndexedChannel>,
    /// Inverted index: word -> message IDs (for text search)
    word_index: HashMap<String, HashSet<u64>>,
}

impl CommQueryEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Index a message.
    pub fn index_message(&mut self, msg: IndexedMessage) {
        let id = msg.id;

        // Time index
        self.time_index.entry(msg.timestamp).or_default().push(id);
        // Sender index
        self.sender_index
            .entry(msg.sender.clone())
            .or_default()
            .push(id);
        // Channel index
        self.channel_index
            .entry(msg.channel_id)
            .or_default()
            .push(id);
        // Word index (simple tokenization)
        for word in msg.content.to_lowercase().split_whitespace() {
            let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if !clean.is_empty() {
                self.word_index.entry(clean).or_default().insert(id);
            }
        }

        self.messages.insert(id, msg);
    }

    /// Index a channel.
    pub fn index_channel(&mut self, channel: IndexedChannel) {
        self.channels.insert(channel.id, channel);
    }

    /// Query messages with filters.
    pub fn query_messages(&self, query: &MessageQuery) -> QueryResult<&IndexedMessage> {
        let mut candidates: Option<HashSet<u64>> = None;

        // Channel filter
        if let Some(ch_id) = query.channel_id {
            let ids: HashSet<u64> = self
                .channel_index
                .get(&ch_id)
                .map(|v| v.iter().cloned().collect())
                .unwrap_or_default();
            candidates = Some(ids);
        }

        // Sender filter
        if let Some(ref sender) = query.sender {
            let ids: HashSet<u64> = self
                .sender_index
                .get(sender)
                .map(|v| v.iter().cloned().collect())
                .unwrap_or_default();
            candidates = Some(match candidates {
                Some(c) => c.intersection(&ids).cloned().collect(),
                None => ids,
            });
        }

        // Time range filter
        if query.since.is_some() || query.until.is_some() {
            let since = query.since.unwrap_or(0);
            let until = query.until.unwrap_or(u64::MAX);
            let ids: HashSet<u64> = self
                .time_index
                .range(since..=until)
                .flat_map(|(_, ids)| ids.iter().cloned())
                .collect();
            candidates = Some(match candidates {
                Some(c) => c.intersection(&ids).cloned().collect(),
                None => ids,
            });
        }

        // Content search
        if let Some(ref text) = query.content_contains {
            let words: Vec<String> = text
                .to_lowercase()
                .split_whitespace()
                .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect())
                .filter(|w: &String| !w.is_empty())
                .collect();

            let mut text_ids: Option<HashSet<u64>> = None;
            for word in &words {
                let word_matches: HashSet<u64> =
                    self.word_index.get(word).cloned().unwrap_or_default();
                text_ids = Some(match text_ids {
                    Some(existing) => existing.intersection(&word_matches).cloned().collect(),
                    None => word_matches,
                });
            }

            if let Some(tids) = text_ids {
                candidates = Some(match candidates {
                    Some(c) => c.intersection(&tids).cloned().collect(),
                    None => tids,
                });
            }
        }

        // Message type filter
        if let Some(ref msg_type) = query.message_type {
            if let Some(ref mut cands) = candidates {
                cands.retain(|id| {
                    self.messages
                        .get(id)
                        .map(|m| &m.message_type == msg_type)
                        .unwrap_or(false)
                });
            }
        }

        // Collect results
        let all_candidates = candidates.unwrap_or_else(|| self.messages.keys().cloned().collect());
        let total_matching = all_candidates.len();

        let mut items: Vec<&IndexedMessage> = all_candidates
            .iter()
            .filter_map(|id| self.messages.get(id))
            .collect();

        // Sort by timestamp descending
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let limit = query.limit.unwrap_or(1000);
        let truncated = items.len() > limit;
        items.truncate(limit);

        QueryResult {
            items,
            total_matching,
            truncated,
        }
    }

    /// Query channels with filters.
    pub fn query_channels(&self, query: &ChannelQuery) -> QueryResult<&IndexedChannel> {
        let mut items: Vec<&IndexedChannel> = self
            .channels
            .values()
            .filter(|ch| {
                if let Some(ref name) = query.name_contains {
                    if !ch.name.to_lowercase().contains(&name.to_lowercase()) {
                        return false;
                    }
                }
                if let Some(ref ct) = query.channel_type {
                    if &ch.channel_type != ct {
                        return false;
                    }
                }
                if let Some(ref state) = query.state {
                    if &ch.state != state {
                        return false;
                    }
                }
                if let Some(ref participant) = query.participant {
                    if !ch.participants.contains(participant) {
                        return false;
                    }
                }
                true
            })
            .collect();

        let total_matching = items.len();
        let limit = query.limit.unwrap_or(1000);
        let truncated = items.len() > limit;
        items.truncate(limit);

        QueryResult {
            items,
            total_matching,
            truncated,
        }
    }

    /// Full-text search over message content.
    pub fn search(&self, text: &str, max_results: usize) -> Vec<&IndexedMessage> {
        let query = MessageQuery {
            content_contains: Some(text.to_string()),
            limit: Some(max_results),
            ..Default::default()
        };
        self.query_messages(&query).items
    }

    /// Get message count per channel.
    pub fn messages_per_channel(&self) -> HashMap<u64, usize> {
        self.channel_index
            .iter()
            .map(|(ch, ids)| (*ch, ids.len()))
            .collect()
    }

    /// Get message count per sender.
    pub fn messages_per_sender(&self) -> HashMap<String, usize> {
        self.sender_index
            .iter()
            .map(|(s, ids)| (s.clone(), ids.len()))
            .collect()
    }

    /// Total indexed message count.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Total indexed channel count.
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a test message.
    fn make_msg(id: u64, channel_id: u64, sender: &str, content: &str, timestamp: u64) -> IndexedMessage {
        IndexedMessage {
            id,
            channel_id,
            sender: sender.to_string(),
            content: content.to_string(),
            timestamp,
            message_type: "text".to_string(),
        }
    }

    /// Helper: create a test channel.
    fn make_channel(id: u64, name: &str, channel_type: &str, state: &str, participants: Vec<&str>) -> IndexedChannel {
        IndexedChannel {
            id,
            name: name.to_string(),
            channel_type: channel_type.to_string(),
            state: state.to_string(),
            participants: participants.iter().map(|s| s.to_string()).collect(),
            message_count: 0,
            created_at: 1000,
        }
    }

    /// Helper: seed an engine with sample data.
    fn seeded_engine() -> CommQueryEngine {
        let mut engine = CommQueryEngine::new();
        engine.index_message(make_msg(1, 10, "alice", "hello world from alice", 100));
        engine.index_message(make_msg(2, 10, "bob", "hello bob here", 200));
        engine.index_message(make_msg(3, 20, "alice", "another channel message", 300));
        engine.index_message(make_msg(4, 20, "carol", "carol says hi", 400));
        engine.index_message(make_msg(5, 10, "alice", "alice again in channel 10", 500));
        engine
    }

    #[test]
    fn test_index_and_query_by_channel() {
        let engine = seeded_engine();
        let query = MessageQuery {
            channel_id: Some(10),
            ..Default::default()
        };
        let result = engine.query_messages(&query);
        assert_eq!(result.total_matching, 3);
        // All returned messages must belong to channel 10
        for msg in &result.items {
            assert_eq!(msg.channel_id, 10);
        }
        assert!(!result.truncated);
    }

    #[test]
    fn test_index_and_query_by_sender() {
        let engine = seeded_engine();
        let query = MessageQuery {
            sender: Some("alice".to_string()),
            ..Default::default()
        };
        let result = engine.query_messages(&query);
        assert_eq!(result.total_matching, 3);
        for msg in &result.items {
            assert_eq!(msg.sender, "alice");
        }
    }

    #[test]
    fn test_time_range_query() {
        let engine = seeded_engine();
        // Messages with timestamp 200..=400
        let query = MessageQuery {
            since: Some(200),
            until: Some(400),
            ..Default::default()
        };
        let result = engine.query_messages(&query);
        assert_eq!(result.total_matching, 3); // IDs 2, 3, 4
        for msg in &result.items {
            assert!(msg.timestamp >= 200 && msg.timestamp <= 400);
        }
        // Results should be sorted descending by timestamp
        let timestamps: Vec<u64> = result.items.iter().map(|m| m.timestamp).collect();
        for w in timestamps.windows(2) {
            assert!(w[0] >= w[1]);
        }
    }

    #[test]
    fn test_full_text_search() {
        let engine = seeded_engine();
        let results = engine.search("hello", 10);
        // Messages 1 and 2 contain "hello"
        assert_eq!(results.len(), 2);
        for msg in &results {
            assert!(msg.content.to_lowercase().contains("hello"));
        }
    }

    #[test]
    fn test_combined_filters() {
        let engine = seeded_engine();
        // Alice in channel 10
        let query = MessageQuery {
            channel_id: Some(10),
            sender: Some("alice".to_string()),
            ..Default::default()
        };
        let result = engine.query_messages(&query);
        assert_eq!(result.total_matching, 2); // IDs 1 and 5
        for msg in &result.items {
            assert_eq!(msg.channel_id, 10);
            assert_eq!(msg.sender, "alice");
        }

        // Alice in channel 10 with time filter
        let query2 = MessageQuery {
            channel_id: Some(10),
            sender: Some("alice".to_string()),
            since: Some(200),
            ..Default::default()
        };
        let result2 = engine.query_messages(&query2);
        assert_eq!(result2.total_matching, 1); // Only ID 5 (timestamp 500)
        assert_eq!(result2.items[0].id, 5);
    }

    #[test]
    fn test_query_channels() {
        let mut engine = CommQueryEngine::new();
        engine.index_channel(make_channel(1, "general", "group", "active", vec!["alice", "bob"]));
        engine.index_channel(make_channel(2, "random", "group", "active", vec!["carol"]));
        engine.index_channel(make_channel(3, "direct-alice-bob", "direct", "active", vec!["alice", "bob"]));
        engine.index_channel(make_channel(4, "archived-stuff", "group", "archived", vec!["alice"]));

        // Query by state
        let q1 = ChannelQuery {
            state: Some("active".to_string()),
            ..Default::default()
        };
        let r1 = engine.query_channels(&q1);
        assert_eq!(r1.total_matching, 3);

        // Query by type
        let q2 = ChannelQuery {
            channel_type: Some("direct".to_string()),
            ..Default::default()
        };
        let r2 = engine.query_channels(&q2);
        assert_eq!(r2.total_matching, 1);
        assert_eq!(r2.items[0].name, "direct-alice-bob");

        // Query by participant
        let q3 = ChannelQuery {
            participant: Some("alice".to_string()),
            ..Default::default()
        };
        let r3 = engine.query_channels(&q3);
        assert_eq!(r3.total_matching, 3); // general, direct-alice-bob, archived-stuff

        // Query by name
        let q4 = ChannelQuery {
            name_contains: Some("general".to_string()),
            ..Default::default()
        };
        let r4 = engine.query_channels(&q4);
        assert_eq!(r4.total_matching, 1);
        assert_eq!(r4.items[0].id, 1);
    }

    #[test]
    fn test_messages_per_channel() {
        let engine = seeded_engine();
        let stats = engine.messages_per_channel();
        assert_eq!(*stats.get(&10).unwrap(), 3);
        assert_eq!(*stats.get(&20).unwrap(), 2);
    }

    #[test]
    fn test_query_limit_and_truncation() {
        let mut engine = CommQueryEngine::new();
        for i in 0..50 {
            engine.index_message(make_msg(i, 1, "alice", &format!("message number {i}"), i * 10));
        }

        // With limit 10
        let query = MessageQuery {
            limit: Some(10),
            ..Default::default()
        };
        let result = engine.query_messages(&query);
        assert_eq!(result.items.len(), 10);
        assert_eq!(result.total_matching, 50);
        assert!(result.truncated);

        // With limit larger than total
        let query2 = MessageQuery {
            limit: Some(100),
            ..Default::default()
        };
        let result2 = engine.query_messages(&query2);
        assert_eq!(result2.items.len(), 50);
        assert!(!result2.truncated);
    }

    #[test]
    fn test_empty_query_returns_all() {
        let engine = seeded_engine();
        let query = MessageQuery::default();
        let result = engine.query_messages(&query);
        assert_eq!(result.total_matching, 5);
        assert_eq!(result.items.len(), 5);
        assert!(!result.truncated);
    }
}
