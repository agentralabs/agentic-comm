use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

use agentic_comm::{
    AffectProcessor, AffectState, ChannelType, CognitiveEdge, CognitiveEdgeType, CognitiveNode,
    CognitiveNodeType, CollectiveDecisionMode, CommStore, CommTrustLevel, ConsentScope,
    EncryptionKey, HiveRole, MessageFilter, MessageType, SemanticProcessor,
};

/// Create a CommStore with rate limits disabled (for benchmark throughput).
fn unlocked_store() -> CommStore {
    let mut store = CommStore::new();
    store.rate_limit_config.messages_per_minute = u32::MAX;
    store.rate_limit_config.semantic_per_minute = u32::MAX;
    store.rate_limit_config.affect_per_minute = u32::MAX;
    store.rate_limit_config.hive_per_hour = u32::MAX;
    store.rate_limit_config.federation_per_minute = u32::MAX;
    store
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a CommStore with `n` messages in a single channel. Returns (store, channel_id).
fn store_with_messages(n: usize) -> (CommStore, u64) {
    let mut store = unlocked_store();
    let ch = store
        .create_channel("bench", ChannelType::Direct, None)
        .unwrap();
    let ch_id = ch.id;
    for i in 0..n {
        store
            .send_message(
                ch_id,
                &format!("agent-{}", i % 10),
                &format!("message number {} with some realistic content payload", i),
                MessageType::Text,
            )
            .unwrap();
    }
    (store, ch_id)
}

/// Canonical scales required by the research paper spec.
const SCALES: [usize; 3] = [100, 1_000, 10_000];

// ===========================================================================
// Category 1: File I/O — create, save, load at 100 / 1K / 10K
// ===========================================================================

fn bench_file_io(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_io");
    group.sample_size(10); // 10K can be slow; keep iterations reasonable

    // --- Create (populate store in memory) ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("create", scale),
            &scale,
            |b, &n| {
                b.iter(|| {
                    black_box(store_with_messages(n));
                });
            },
        );
    }

    // --- Save ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("save", scale),
            &scale,
            |b, &n| {
                let (store, _) = store_with_messages(n);
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("bench.acomm");
                b.iter(|| {
                    store.save(&path).unwrap();
                });
            },
        );
    }

    // --- Load ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("load", scale),
            &scale,
            |b, &n| {
                let (store, _) = store_with_messages(n);
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("bench.acomm");
                store.save(&path).unwrap();
                b.iter(|| {
                    black_box(CommStore::load(&path).unwrap());
                });
            },
        );
    }

    group.finish();
}

// ===========================================================================
// Category 2: Entity Operations — message send, channel create, hive form
// ===========================================================================

fn bench_entity_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_ops");

    // --- Single message send (amortised across scale) ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("message_send", scale),
            &scale,
            |b, &n| {
                b.iter(|| {
                    let mut store = unlocked_store();
                    let ch = store
                        .create_channel("bench", ChannelType::Direct, None)
                        .unwrap();
                    for i in 0..n {
                        store
                            .send_message(
                                ch.id,
                                "benchuser",
                                &format!("msg {}", i),
                                MessageType::Text,
                            )
                            .unwrap();
                    }
                });
            },
        );
    }

    // --- Channel creation at scale ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("channel_create", scale),
            &scale,
            |b, &n| {
                b.iter(|| {
                    let mut store = unlocked_store();
                    for i in 0..n {
                        store
                            .create_channel(
                                &format!("ch-{}", i),
                                ChannelType::Direct,
                                None,
                            )
                            .unwrap();
                    }
                });
            },
        );
    }

    // --- Hive form + join ---
    for &member_count in &[10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("hive_form_join", member_count),
            &member_count,
            |b, &n| {
                b.iter(|| {
                    let mut store = unlocked_store();
                    let hive = store
                        .form_hive(
                            "bench-hive",
                            "coordinator",
                            CollectiveDecisionMode::Consensus,
                        )
                        .unwrap();
                    let hive_id = hive.id;
                    for i in 0..n {
                        store
                            .join_hive(
                                hive_id,
                                &format!("agent_{}", i),
                                HiveRole::Member,
                            )
                            .unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

// ===========================================================================
// Category 3: Core Computations — topic matching, trust lookup,
//             consent check, encryption
// ===========================================================================

fn bench_core_computations(c: &mut Criterion) {
    let mut group = c.benchmark_group("core_computations");

    // --- Topic matching (pub/sub publish) ---
    for &sub_count in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("topic_match_publish", sub_count),
            &sub_count,
            |b, &n| {
                let mut store = unlocked_store();
                for i in 0..n {
                    store
                        .subscribe(&format!("topic-{}", i % 50), &format!("agent-{}", i))
                        .unwrap();
                }
                b.iter(|| {
                    black_box(
                        store.publish("topic-25", "publisher", "benchmark payload"),
                    );
                });
            },
        );
    }

    // --- Trust lookup ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("trust_lookup", scale),
            &scale,
            |b, &n| {
                let mut store = unlocked_store();
                for i in 0..n {
                    store
                        .set_trust_level(&format!("agent-{}", i), CommTrustLevel::High)
                        .unwrap();
                }
                b.iter(|| {
                    // Lookup a mid-range agent
                    black_box(store.get_trust_level(&format!("agent-{}", n / 2)));
                });
            },
        );
    }

    // --- Consent check ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("consent_check", scale),
            &scale,
            |b, &n| {
                let mut store = unlocked_store();
                for i in 0..n {
                    store
                        .grant_consent(
                            &format!("agent-{}", i),
                            "grantee-default",
                            ConsentScope::ReadMessages,
                            None,
                            None,
                        )
                        .unwrap();
                }
                b.iter(|| {
                    black_box(store.check_consent(
                        &format!("agent-{}", n / 2),
                        "grantee-default",
                        &ConsentScope::ReadMessages,
                    ));
                });
            },
        );
    }

    // --- Encryption round-trip ---
    {
        let key = EncryptionKey::generate();
        let plaintext = "The quick brown fox jumps over the lazy dog. Benchmark payload.";
        group.bench_function("encrypt_chacha20", |b| {
            b.iter(|| {
                black_box(agentic_comm::encrypt(&key, plaintext).unwrap());
            });
        });

        let encrypted = agentic_comm::encrypt(&key, plaintext).unwrap();
        group.bench_function("decrypt_chacha20", |b| {
            b.iter(|| {
                black_box(agentic_comm::decrypt(&key, &encrypted).unwrap());
            });
        });

        let aes_key = EncryptionKey::generate_for(&agentic_comm::EncryptionScheme::Aes256Gcm);
        group.bench_function("encrypt_aes256gcm", |b| {
            b.iter(|| {
                black_box(agentic_comm::encrypt_aes(&aes_key, plaintext).unwrap());
            });
        });

        let aes_encrypted = agentic_comm::encrypt_aes(&aes_key, plaintext).unwrap();
        group.bench_function("decrypt_aes256gcm", |b| {
            b.iter(|| {
                black_box(agentic_comm::decrypt_aes(&aes_key, &aes_encrypted).unwrap());
            });
        });
    }

    group.finish();
}

// ===========================================================================
// Category 4: Queries — search, relationships, timeline, history
// ===========================================================================

fn bench_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("queries");
    group.sample_size(10);

    // --- Search at scale ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("search", scale),
            &scale,
            |b, &n| {
                let (store, _) = store_with_messages(n);
                b.iter(|| {
                    black_box(store.search_messages("number 500", 10));
                });
            },
        );
    }

    // --- Relationship query ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("relationships", scale),
            &scale,
            |b, &n| {
                let mut store = unlocked_store();
                // Set up trust + consent + channels for agents
                for i in 0..n {
                    let agent = format!("agent-{}", i);
                    store
                        .set_trust_level(&agent, CommTrustLevel::High)
                        .unwrap();
                    if i < n / 2 {
                        store
                            .grant_consent(
                                &agent,
                                "agent-0",
                                ConsentScope::ReadMessages,
                                None,
                                None,
                            )
                            .unwrap();
                    }
                }
                b.iter(|| {
                    black_box(store.query_relationships("agent-0", None, 2));
                });
            },
        );
    }

    // --- Timeline (conversation_at_time) ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("timeline", scale),
            &scale,
            |b, &n| {
                let (store, ch_id) = store_with_messages(n);
                let now_ts = chrono::Utc::now().timestamp() as u64;
                b.iter(|| {
                    black_box(store.conversation_at_time(ch_id, now_ts));
                });
            },
        );
    }

    // --- History (query_history with filter) ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("history", scale),
            &scale,
            |b, &n| {
                let (store, ch_id) = store_with_messages(n);
                let filter = MessageFilter {
                    sender: Some("agent-5".to_string()),
                    limit: Some(50),
                    ..Default::default()
                };
                b.iter(|| {
                    black_box(store.query_history(ch_id, &filter));
                });
            },
        );
    }

    group.finish();
}

// ===========================================================================
// Category 5: Write Engine — end-to-end write ops including save
// ===========================================================================

fn bench_write_engine(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_engine");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(15));

    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("e2e_write", scale),
            &scale,
            |b, &n| {
                b.iter(|| {
                    let mut store = unlocked_store();
                    let ch = store
                        .create_channel("bench", ChannelType::Direct, None)
                        .unwrap();
                    for i in 0..n {
                        store
                            .send_message(
                                ch.id,
                                &format!("agent-{}", i % 10),
                                &format!("msg {} payload content", i),
                                MessageType::Text,
                            )
                            .unwrap();
                    }
                    let dir = tempfile::tempdir().unwrap();
                    let path = dir.path().join("bench.acomm");
                    store.save(&path).unwrap();
                });
            },
        );
    }

    // --- Write with rich state (messages + trust + consent + affect) ---
    for &scale in &SCALES {
        group.bench_with_input(
            BenchmarkId::new("e2e_write_rich", scale),
            &scale,
            |b, &n| {
                b.iter(|| {
                    let mut store = unlocked_store();
                    // Channels + messages
                    let ch = store
                        .create_channel("bench", ChannelType::Direct, None)
                        .unwrap();
                    for i in 0..n {
                        store
                            .send_message(
                                ch.id,
                                &format!("agent-{}", i % 10),
                                &format!("msg {}", i),
                                MessageType::Text,
                            )
                            .unwrap();
                    }
                    // Trust
                    for i in 0..(n.min(500)) {
                        store
                            .set_trust_level(
                                &format!("agent-{}", i),
                                CommTrustLevel::High,
                            )
                            .unwrap();
                    }
                    // Consent
                    for i in 0..(n.min(200)) {
                        store
                            .grant_consent(
                                &format!("agent-{}", i),
                                "agent-0",
                                ConsentScope::ReadMessages,
                                None,
                                None,
                            )
                            .unwrap();
                    }
                    // Affect
                    for i in 0..(n.min(100)) {
                        store.affect_states.insert(
                            format!("agent-{}", i),
                            AffectState {
                                valence: (i as f64 / 100.0) * 2.0 - 1.0,
                                arousal: i as f64 / 100.0,
                                dominance: 0.5,
                                ..Default::default()
                            },
                        );
                    }
                    let dir = tempfile::tempdir().unwrap();
                    let path = dir.path().join("bench.acomm");
                    store.save(&path).unwrap();
                });
            },
        );
    }

    group.finish();
}

// ===========================================================================
// Legacy benchmarks (retained from original suite, kept for continuity)
// ===========================================================================

fn bench_semantic_transfer(c: &mut Criterion) {
    let mut group = c.benchmark_group("semantic_transfer");

    for size in [10, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("send_and_search", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut store = unlocked_store();
                    let ch = store
                        .create_channel("bench-chan", ChannelType::Direct, None)
                        .unwrap();
                    for i in 0..size {
                        store
                            .send_message(
                                ch.id,
                                &format!("agent-{}", i % 10),
                                &format!("Message {}", i),
                                MessageType::Text,
                            )
                            .unwrap();
                    }
                    black_box(store.search_messages("Message", 50))
                });
            },
        );
    }

    for count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("graft_semantic", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut store = unlocked_store();
                    for i in 0..count {
                        store
                            .graft_semantic(i as u64, (i + 1) as u64, "merge")
                            .unwrap();
                    }
                });
            },
        );
    }

    group.bench_function("extract_fragment_50_nodes", |b| {
        let mut processor = SemanticProcessor::new();
        for i in 0..50 {
            processor.add_node(CognitiveNode {
                id: format!("node-{}", i),
                label: format!("concept-{}", i),
                node_type: CognitiveNodeType::Concept,
                confidence: 0.0,
                source: None,
                content: None,
                metadata: std::collections::HashMap::new(),
            });
        }
        for i in 0..49 {
            processor.add_edge(CognitiveEdge {
                from: format!("node-{}", i),
                to: format!("node-{}", i + 1),
                edge_type: CognitiveEdgeType::RelatedTo,
                weight: 0.5,
            });
        }
        let focus = vec!["node-25".to_string()];
        b.iter(|| {
            black_box(processor.extract_fragment(&focus, 3, "bench-agent"));
        });
    });

    group.finish();
}

fn bench_affect_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("affect_pipeline");

    group.bench_function("encode_affect_state_100", |b| {
        b.iter(|| {
            let mut store = unlocked_store();
            for i in 0..100 {
                let agent_id = format!("agent-{}", i);
                store.affect_states.insert(
                    agent_id,
                    AffectState {
                        valence: 0.5,
                        arousal: 0.3,
                        dominance: 0.7,
                        ..Default::default()
                    },
                );
            }
        });
    });

    group.bench_function("read_affect_states_100", |b| {
        let mut store = unlocked_store();
        for i in 0..100 {
            let agent_id = format!("agent-{}", i);
            store.affect_states.insert(
                agent_id,
                AffectState {
                    valence: 0.5,
                    arousal: 0.3,
                    dominance: 0.7,
                    ..Default::default()
                },
            );
        }
        b.iter(|| {
            for i in 0..100 {
                let agent_id = format!("agent-{}", i);
                black_box(store.get_affect_state(&agent_id));
            }
        });
    });

    group.bench_function("affect_contagion_pipeline", |b| {
        b.iter(|| {
            let mut store = unlocked_store();
            let ch = store
                .create_channel("affect-bench", ChannelType::Direct, None)
                .unwrap();
            store.join_channel(ch.id, "sender-a").unwrap();
            store.join_channel(ch.id, "receiver-b").unwrap();
            store.join_channel(ch.id, "receiver-c").unwrap();

            store.affect_states.insert(
                "receiver-b".to_string(),
                AffectState::default(),
            );
            store.affect_states.insert(
                "receiver-c".to_string(),
                AffectState::default(),
            );

            for i in 0..20 {
                let msg = store
                    .send_message(
                        ch.id,
                        "sender-a",
                        &format!("affect msg {}", i),
                        MessageType::Text,
                    )
                    .unwrap();
                let msg_id = msg.id;
                if let Some(stored) = store.messages.get_mut(&msg_id) {
                    stored.metadata.insert("valence".to_string(), "0.8".to_string());
                    stored.metadata.insert("arousal".to_string(), "0.6".to_string());
                    stored.metadata.insert("dominance".to_string(), "0.5".to_string());
                }
            }

            black_box(store.process_affect_contagion(ch.id));
        });
    });

    group.bench_function("affect_processor_register_100", |b| {
        b.iter(|| {
            let mut processor = AffectProcessor::new();
            for i in 0..100 {
                processor.register_agent(
                    &format!("agent-{}", i),
                    AffectState {
                        valence: 0.0,
                        arousal: 0.5,
                        dominance: 0.5,
                        ..Default::default()
                    },
                    0.3,
                );
            }
        });
    });

    group.finish();
}

fn bench_affect_processor(c: &mut Criterion) {
    let mut group = c.benchmark_group("affect_processor");

    group.bench_function("register_100_agents", |b| {
        b.iter(|| {
            let mut processor = AffectProcessor::new();
            for i in 0..100 {
                processor.register_agent(
                    &format!("agent-{}", i),
                    AffectState {
                        valence: 0.0,
                        arousal: 0.5,
                        dominance: 0.5,
                        ..Default::default()
                    },
                    0.3,
                );
            }
        });
    });

    group.bench_function("lookup_in_100_agents", |b| {
        let mut processor = AffectProcessor::new();
        for i in 0..100 {
            processor.register_agent(
                &format!("agent-{}", i),
                AffectState {
                    valence: 0.0,
                    arousal: 0.5,
                    dominance: 0.5,
                    ..Default::default()
                },
                0.3,
            );
        }
        b.iter(|| {
            for i in 0..100 {
                black_box(processor.get_state(&format!("agent-{}", i)));
            }
        });
    });

    group.bench_function("set_resistance_100", |b| {
        let mut processor = AffectProcessor::new();
        for i in 0..100 {
            processor.register_agent(
                &format!("agent-{}", i),
                AffectState::default(),
                0.3,
            );
        }
        b.iter(|| {
            for i in 0..100 {
                black_box(processor.set_resistance(&format!("agent-{}", i), 0.8));
            }
        });
    });

    group.finish();
}

// ===========================================================================
// Criterion wiring
// ===========================================================================

criterion_group!(
    benches,
    // Canonical spec categories
    bench_file_io,
    bench_entity_ops,
    bench_core_computations,
    bench_queries,
    bench_write_engine,
    // Legacy / extended benchmarks
    bench_semantic_transfer,
    bench_affect_pipeline,
    bench_affect_processor,
);
criterion_main!(benches);
