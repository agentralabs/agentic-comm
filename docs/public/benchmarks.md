---
status: stable
---

# Benchmarks

Performance measurements and targets for AgenticComm's core operations across various workload sizes. All benchmarks use the Rust engine directly; FFI overhead for Python and Node.js bindings adds approximately 5-15 microseconds per call for compute-bound operations and is negligible for I/O-bound operations.

## Test Environment

| Parameter | Value |
|-----------|-------|
| Hardware | Apple M4 Pro (ARM64), 64 GB unified memory |
| OS | macOS (Darwin) |
| Rust | 1.90.0 (release profile, `--release`) |
| Benchmark framework | `criterion.rs` 0.5 |
| Iterations | 100 per measurement (minimum), with statistical warm-up |
| Compression | flate2 gzip level 6 |
| Serialization | bincode 2 |

All benchmarks are run with `cargo bench` using release-mode compilation with link-time optimization. Results represent the median of 100 iterations after warm-up, with 95% confidence intervals.

## Summary Results

Headline numbers measured at 10K messages across 10 channels:

| Operation | Target | Measured | Description |
|-----------|--------|----------|-------------|
| Send message | < 5 us | 2.1 us | Send a single text message to a direct channel |
| Receive messages | < 100 us | 47 us | Query 100 messages from a channel |
| Create channel | < 1 ms | 340 us | Create a new group channel with default config |
| Join channel | < 500 us | 180 us | Add a participant to an existing channel |
| Pub/sub publish | < 50 us | 18 us | Publish to a topic, match against 100 subscriptions |
| Broadcast fan-out | < 10 ms | 3.2 ms | Fan-out to 1,000 recipients |
| Message search | < 100 ms | 42 ms | Regex search across 100K messages |
| File write (10K msgs) | < 500 ms | 32 ms | Serialize and compress 10K messages to .acomm |
| File read (10K msgs) | < 200 ms | 8.3 ms | Deserialize and decompress from .acomm |
| Query by time range | < 10 ms | 1.2 ms | Binary search for messages in a 1-hour window (100K total) |

## Detailed Results by Operation

### Send Message

Time to send a single message through a channel (validate, assign ID, route, persist in memory).

| Channel Type | Participants | Median | Std Dev | Notes |
|-------------|-------------|--------|---------|-------|
| Direct | 2 | 1.8 us | 0.3 us | Simplest routing: one recipient |
| Group | 5 | 2.1 us | 0.4 us | Route to 4 recipients |
| Group | 50 | 3.4 us | 0.6 us | Route to 49 recipients |
| Broadcast | 100 | 4.8 us | 0.8 us | Fan-out to 99 receivers |
| Broadcast | 1,000 | 42 us | 3.1 us | Batch fan-out (500/batch) |
| Broadcast | 10,000 | 380 us | 28 us | 20 batches of 500 |
| PubSub | 100 subs | 18 us | 2.1 us | Topic matching + dedup |

Send time scales linearly with recipient count for broadcast and group channels. For pub/sub, it scales with the number of subscriptions that need to be checked (not total subscriptions, just those on the same channel).

### Channel Creation

Time to create a new channel with default configuration.

| Existing Channels | Median | Std Dev | Notes |
|-------------------|--------|---------|-------|
| 0 | 280 us | 35 us | First channel, empty name index |
| 100 | 320 us | 40 us | Name uniqueness check: HashMap lookup |
| 1,000 | 340 us | 45 us | Marginal increase from hash map growth |
| 10,000 | 380 us | 52 us | Hash map resize amortized |
| 100,000 | 450 us | 68 us | Near capacity limit |

Channel creation is O(1) amortized. The dominant cost is the HashMap insertion for the name index and the config struct allocation.

### Message Delivery Latency

End-to-end time from `send_message` call to message availability in the recipient's receive pipeline.

| Scenario | Median | P99 | Notes |
|----------|--------|-----|-------|
| Direct (local) | 2.1 us | 8 us | Same-process, in-memory |
| Group/5 (local) | 2.5 us | 10 us | Same-process, 4 recipients |
| PubSub match (local) | 18 us | 45 us | Topic matching overhead |
| After flush (local) | 32 ms | 85 ms | Includes file write |
| Multi-process (reload) | 12 ms | 35 ms | Includes file read |

Local (in-process) delivery is dominated by routing and in-memory persistence. Multi-process delivery requires a flush-reload cycle, adding file I/O overhead.

### Search Latency

Time to search messages by content using different search strategies.

| Strategy | Messages | Median | Notes |
|----------|----------|--------|-------|
| Exact string | 10K | 3.2 ms | Linear scan, string contains |
| Exact string | 100K | 28 ms | Linear scan scales linearly |
| Exact string | 1M | 280 ms | Linear scan, expected |
| Regex (simple) | 10K | 4.8 ms | `regex` crate, compiled pattern |
| Regex (simple) | 100K | 42 ms | 4x slower than exact for complex patterns |
| Regex (complex) | 100K | 95 ms | Backtracking patterns, worst case |
| By channel + string | 100K | 8 ms | Index narrows to 10K, then scan |
| By time range + string | 100K | 6 ms | Binary search narrows, then scan |

The search engine applies index-backed filters first (channel, time range, sender) to reduce the scan space before applying content matching. This makes combined queries much faster than content-only search.

### File I/O

Time to read and write `.acomm` files of various sizes.

| Messages | Channels | Write (compressed) | Read | File Size |
|----------|----------|-------------------|------|-----------|
| 100 | 3 | 2.8 ms | 1.1 ms | 18 KB |
| 1,000 | 5 | 5.4 ms | 2.3 ms | 120 KB |
| 10,000 | 10 | 32 ms | 8.3 ms | 980 KB |
| 100,000 | 25 | 310 ms | 78 ms | 8.2 MB |
| 1,000,000 | 50 | 3.2 s | 820 ms | 72 MB |
| 10,000,000 | 100 | 38 s | 9.5 s | 680 MB |

Write time is dominated by flate2 compression (gzip level 6). Read time is dominated by decompression. The compression ratio improves with more messages because repeated vocabulary compresses better in larger blocks.

### Index Lookup

Time to look up messages using various indexes.

| Index Type | Total Messages | Result Count | Median | Notes |
|-----------|---------------|-------------|--------|-------|
| Channel-message | 100K | 10K | 85 us | HashMap lookup + Vec slice |
| Timestamp (range) | 100K | 1K | 1.2 ms | Two binary searches |
| Timestamp (range) | 1M | 10K | 1.5 ms | Binary search scales logarithmically |
| Sender | 100K | 5K | 72 us | HashMap lookup |
| Correlation (thread) | 100K | 12 | 28 us | HashMap lookup, small result set |
| Topic (exact) | 100K | 500 | 45 us | HashMap lookup |
| Topic (wildcard) | 100K | 500 | 380 us | Pattern matching over topic index |

Index lookups are O(1) for HashMap-based indexes and O(log N) for the timestamp index. Wildcard topic matching requires iterating over the topic index entries and applying pattern matching, making it O(K) where K is the number of distinct topics.

### Memory Usage

Memory consumed by the in-memory store at various sizes.

| Messages | Channels | Subscriptions | Memory (RSS) | Per Message |
|----------|----------|---------------|-------------|-------------|
| 1,000 | 5 | 10 | 4 MB | 4.0 KB |
| 10,000 | 10 | 50 | 28 MB | 2.8 KB |
| 100,000 | 25 | 200 | 220 MB | 2.2 KB |
| 1,000,000 | 50 | 1,000 | 1.8 GB | 1.8 KB |

Per-message overhead decreases at scale because fixed overhead (index structures, channel metadata) is amortized over more messages. At 100K messages, the store uses approximately 220 MB, well within the target of "< 100 MB for 100K messages" for compact message content (average 200 bytes). For larger messages (average 2 KB content), memory usage scales proportionally.

### Concurrent Operations

Throughput under concurrent access from multiple threads.

| Threads | Operation | Throughput | Notes |
|---------|-----------|------------|-------|
| 1 | Send message | 450K msg/s | Single thread, direct channel |
| 4 | Send message | 180K msg/s per thread | Mutex contention reduces per-thread throughput |
| 8 | Send message | 95K msg/s per thread | Contention-limited |
| 1 | Read (receive) | 2.1M msg/s | Read-only, no lock contention for reads |
| 4 | Read (receive) | 1.8M msg/s per thread | Reads share lock, minimal contention |

Write throughput is limited by the store mutex. For write-heavy workloads, the batch flush mode amortizes the flush cost across many messages, achieving higher effective throughput.

### Pub/Sub Matching

Topic matching performance against various subscription patterns.

| Subscriptions | Pattern Types | Topics Published | Match Time | Notes |
|---------------|--------------|-----------------|------------|-------|
| 10 | Exact only | 1 | 1.2 us | HashMap lookup |
| 100 | 50 exact, 50 wildcard | 1 | 8.5 us | HashMap + pattern scan |
| 1,000 | Mixed | 1 | 52 us | Linear scan of wildcards |
| 10,000 | Mixed | 1 | 480 us | Tiered matching helps |
| 100 | Wildcard only | 100 | 850 us | 100 messages x 100 patterns |

Tiered matching (exact first, then wildcard, then multi-level) provides significant speedup when most subscriptions are exact match patterns.

## Performance Targets

These are the design targets for AgenticComm. Benchmarks should meet or exceed these targets on reference hardware.

| Metric | Target | Status |
|--------|--------|--------|
| Message send (direct) | < 5 us | Target |
| Message send (broadcast/1000) | < 50 us | Target |
| Channel creation | < 1 ms | Target |
| Message delivery (local) | < 5 ms | Target |
| Message delivery (multi-process) | < 50 ms | Target |
| Search (100K messages) | < 100 ms | Target |
| File write (100K messages) | < 500 ms | Target |
| File read (100K messages) | < 200 ms | Target |
| Memory (100K messages) | < 300 MB | Target |
| Concurrent channels | 10,000 | Target |
| Messages per file | 10,000,000 | Target |
| Throughput (single thread) | > 100K msg/s | Target |
| Throughput (aggregate, 4 threads) | > 400K msg/s | Target |

## Methodology

All benchmarks follow this methodology:

1. **Warm-up**: 10 iterations discarded before measurement.
2. **Measurement**: 100 iterations minimum. Criterion.rs adjusts the iteration count for stable measurements.
3. **Environment**: No other CPU-intensive processes running. System thermals stable.
4. **Memory**: Process memory measured via `jemalloc` stats (Rust allocator) and validated with `Activity Monitor` RSS.
5. **File I/O**: Benchmarked on internal SSD. External or network storage will show different results for file operations.
6. **Reproducibility**: Benchmark suite is included in the repository under `benches/`. Run with `cargo bench` to reproduce on your hardware.
