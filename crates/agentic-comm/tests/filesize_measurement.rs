//! File size measurement tests for the .acomm format.
//!
//! Measures actual file sizes at 100, 1K, and 10K messages and prints
//! bytes-per-message to help track format efficiency over time.

use agentic_comm::{ChannelType, CommStore, MessageType};

/// Create a CommStore with rate limits disabled for bulk operations.
fn unlocked_store() -> CommStore {
    let mut store = CommStore::new();
    store.rate_limit_config.messages_per_minute = u32::MAX;
    store
}

/// Build a store with `n` messages, save it, and return (file_size_bytes, n).
fn measure_file_size(n: usize) -> (u64, usize) {
    let mut store = unlocked_store();
    let ch = store
        .create_channel("measure", ChannelType::Direct, None)
        .unwrap();
    for i in 0..n {
        store
            .send_message(
                ch.id,
                &format!("agent-{}", i % 10),
                &format!("message number {} with some realistic content payload", i),
                MessageType::Text,
            )
            .unwrap();
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("measure.acomm");
    store.save(&path).unwrap();
    let metadata = std::fs::metadata(&path).unwrap();
    (metadata.len(), n)
}

#[test]
fn filesize_100_messages() {
    let (bytes, n) = measure_file_size(100);
    let bpm = bytes as f64 / n as f64;
    println!();
    println!("=== File Size: {} messages ===", n);
    println!("  Total bytes : {}", bytes);
    println!("  Bytes/msg   : {:.1}", bpm);
    println!();
    // Sanity: file should be non-trivial but not absurdly large
    assert!(bytes > 0, "File should not be empty");
    assert!(
        bpm < 2000.0,
        "Bytes-per-message ({:.1}) exceeds 2000 — likely a regression",
        bpm
    );
}

#[test]
fn filesize_1k_messages() {
    let (bytes, n) = measure_file_size(1_000);
    let bpm = bytes as f64 / n as f64;
    println!();
    println!("=== File Size: {} messages ===", n);
    println!("  Total bytes : {}", bytes);
    println!("  Bytes/msg   : {:.1}", bpm);
    println!();
    assert!(bytes > 0);
    assert!(
        bpm < 2000.0,
        "Bytes-per-message ({:.1}) exceeds 2000 — likely a regression",
        bpm
    );
}

#[test]
fn filesize_10k_messages() {
    let (bytes, n) = measure_file_size(10_000);
    let bpm = bytes as f64 / n as f64;
    println!();
    println!("=== File Size: {} messages ===", n);
    println!("  Total bytes : {}", bytes);
    println!("  Bytes/msg   : {:.1}", bpm);
    println!();
    assert!(bytes > 0);
    assert!(
        bpm < 2000.0,
        "Bytes-per-message ({:.1}) exceeds 2000 — likely a regression",
        bpm
    );
}

/// Measure file sizes for all three canonical scales in one test and print
/// a summary table. This is the "quick glance" test for CI output.
#[test]
fn filesize_summary_table() {
    println!();
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║        .acomm File Size Summary                     ║");
    println!("╠═══════════╦══════════════╦══════════════════════════╣");
    println!("║  Messages ║  File Bytes  ║  Bytes per Message       ║");
    println!("╠═══════════╬══════════════╬══════════════════════════╣");

    for &scale in &[100, 1_000, 10_000] {
        let (bytes, n) = measure_file_size(scale);
        let bpm = bytes as f64 / n as f64;
        println!(
            "║  {:>7} ║  {:>10} ║  {:>22.1} ║",
            n, bytes, bpm
        );
    }

    println!("╚═══════════╩══════════════╩══════════════════════════╝");
    println!();
}
