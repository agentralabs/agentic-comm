//! Conservation tests for AgenticComm.
//!
//! Exercises cache and metrics modules to verify token conservation
//! properties hold across the foundation layer.
//!
//! Note: Comm uses a domain-specific CommQueryEngine rather than
//! the standard ExtractionIntent/TokenBudget pattern.

use std::time::Duration;

use agentic_comm::cache::LruCache;
use agentic_comm::metrics::tokens::{Layer, TokenMetrics};

// ---------------------------------------------------------------------------
// Test 1: Cache hit is cheaper than miss
// ---------------------------------------------------------------------------

#[test]
fn test_cache_hit_cheaper() {
    let mut cache: LruCache<String, String> = LruCache::new(100, Duration::from_secs(300));

    // First access: miss
    assert!(cache.get(&"key1".to_string()).is_none());

    // Insert
    cache.insert("key1".to_string(), "value1".to_string());

    // Second access: hit (0 token cost)
    assert!(cache.get(&"key1".to_string()).is_some());

    // Verify metrics
    assert!(cache.metrics().hits() >= 1);
    assert!(cache.metrics().misses() >= 1);
    assert!(cache.metrics().hit_rate() > 0.0);
}

// ---------------------------------------------------------------------------
// Test 2: Query engine provides indexed access
// ---------------------------------------------------------------------------

#[test]
fn test_query_engine_creation() {
    use agentic_comm::CommQueryEngine;

    let engine = CommQueryEngine::new();
    // A new engine should have no messages or channels
    assert_eq!(engine.message_count(), 0);
    assert_eq!(engine.channel_count(), 0);
}

// ---------------------------------------------------------------------------
// Test 3: Token metrics recording
// ---------------------------------------------------------------------------

#[test]
fn test_metrics_recording() {
    let metrics = TokenMetrics::new();
    assert_eq!(metrics.total_tokens(), 0);
    assert_eq!(metrics.total_savings(), 0);

    // Record usage at different layers
    metrics.record(Layer::Full, 100, 100);
    assert_eq!(metrics.total_tokens(), 100);

    metrics.record(Layer::Cache, 0, 400);
    assert_eq!(metrics.total_tokens(), 100);
    assert_eq!(metrics.total_savings(), 400);
}

// ---------------------------------------------------------------------------
// Test 4: Conservation score
// ---------------------------------------------------------------------------

#[test]
fn test_conservation_score() {
    let metrics = TokenMetrics::new();

    // Full retrieval: 100 tokens used, potential was 100
    metrics.record(Layer::Full, 100, 100);

    // Cache hit: 0 tokens used, potential was 400
    metrics.record(Layer::Cache, 0, 400);

    // total_tokens = 100, cache_savings = 400, total_savings = 400
    // conservation = 400 / (100 + 400) = 0.8
    let score = metrics.conservation_score();
    assert!(
        score > 0.7 && score < 0.9,
        "Conservation score should be ~0.8, got {}",
        score
    );
}

// ---------------------------------------------------------------------------
// Test 5: Cache invalidation
// ---------------------------------------------------------------------------

#[test]
fn test_cache_invalidation() {
    let mut cache: LruCache<String, i32> = LruCache::new(100, Duration::from_secs(300));

    cache.insert("key".to_string(), 42);
    assert!(cache.contains(&"key".to_string()));

    // Invalidate the entry
    assert!(cache.invalidate(&"key".to_string()));
    assert!(!cache.contains(&"key".to_string()));

    // Double invalidation returns false
    assert!(!cache.invalidate(&"key".to_string()));
}
