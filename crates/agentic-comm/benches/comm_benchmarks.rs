use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use agentic_comm::{
    AffectProcessor, AffectState, ChannelType, CognitiveEdge, CognitiveEdgeType, CognitiveNode,
    CognitiveNodeType, CollectiveDecisionMode, CommStore, CommTrustLevel, ConsentScope, HiveRole,
    MessageType, SemanticProcessor,
};

fn bench_message_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_throughput");

    for count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut store = CommStore::new();
                let ch = store
                    .create_channel("bench", ChannelType::Direct, None)
                    .unwrap();
                for i in 0..count {
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
        });
    }
    group.finish();
}

fn bench_channel_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("channel_creation");

    for count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut store = CommStore::new();
                for i in 0..count {
                    store
                        .create_channel(
                            &format!("ch-{}", i),
                            ChannelType::Direct,
                            None,
                        )
                        .unwrap();
                }
            });
        });
    }
    group.finish();
}

fn bench_message_search(c: &mut Criterion) {
    c.bench_function("search_1000_messages", |b| {
        let mut store = CommStore::new();
        let ch = store
            .create_channel("search-bench", ChannelType::Direct, None)
            .unwrap();
        for i in 0..1000 {
            store
                .send_message(
                    ch.id,
                    "sender",
                    &format!("message number {} content", i),
                    MessageType::Text,
                )
                .unwrap();
        }
        b.iter(|| {
            store.search_messages("number 500", 10);
        });
    });
}

fn bench_save_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_load");

    for msg_count in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("save", msg_count),
            msg_count,
            |b, &count| {
                let mut store = CommStore::new();
                let ch = store
                    .create_channel("save-bench", ChannelType::Direct, None)
                    .unwrap();
                for i in 0..count {
                    store
                        .send_message(
                            ch.id,
                            "sender",
                            &format!("msg {}", i),
                            MessageType::Text,
                        )
                        .unwrap();
                }
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("bench.acomm");
                b.iter(|| {
                    store.save(&path).unwrap();
                });
            },
        );
    }
    group.finish();
}

fn bench_consent_check(c: &mut Criterion) {
    c.bench_function("consent_check_100_gates", |b| {
        let mut store = CommStore::new();
        for i in 0..100 {
            store
                .grant_consent(
                    &format!("agent_{}", i),
                    "grantee_default",
                    ConsentScope::ReadMessages,
                    None,
                    None,
                )
                .unwrap();
        }
        b.iter(|| {
            store.check_consent("agent_50", "grantee_default", &ConsentScope::ReadMessages);
        });
    });
}

fn bench_hive_operations(c: &mut Criterion) {
    c.bench_function("hive_form_and_join", |b| {
        b.iter(|| {
            let mut store = CommStore::new();
            let hive = store
                .form_hive(
                    "bench-hive",
                    "coordinator",
                    CollectiveDecisionMode::Consensus,
                )
                .unwrap();
            let hive_id = hive.id;
            for i in 0..10 {
                store
                    .join_hive(hive_id, &format!("agent_{}", i), HiveRole::Member)
                    .unwrap();
            }
        });
    });
}

// ---------------------------------------------------------------------------
// Semantic operations benchmark
// ---------------------------------------------------------------------------

fn bench_semantic_transfer(c: &mut Criterion) {
    let mut group = c.benchmark_group("semantic_transfer");

    for size in [10, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("send_and_search", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut store = CommStore::new();
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

    // Benchmark semantic graft operations
    for count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("graft_semantic", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut store = CommStore::new();
                    for i in 0..count {
                        store
                            .graft_semantic(i as u64, (i + 1) as u64, "merge")
                            .unwrap();
                    }
                });
            },
        );
    }

    // Benchmark SemanticProcessor extract_fragment
    group.bench_function("extract_fragment_50_nodes", |b| {
        let mut processor = SemanticProcessor::new();
        for i in 0..50 {
            processor.add_node(CognitiveNode {
                id: format!("node-{}", i),
                label: format!("concept-{}", i),
                node_type: CognitiveNodeType::Concept,
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

// ---------------------------------------------------------------------------
// Affect pipeline benchmark
// ---------------------------------------------------------------------------

fn bench_affect_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("affect_pipeline");

    group.bench_function("encode_affect_state_100", |b| {
        b.iter(|| {
            let mut store = CommStore::new();
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
        let mut store = CommStore::new();
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

    // Benchmark the contagion pipeline with messages carrying affect metadata
    group.bench_function("affect_contagion_pipeline", |b| {
        b.iter(|| {
            let mut store = CommStore::new();
            let ch = store
                .create_channel("affect-bench", ChannelType::Direct, None)
                .unwrap();
            store.join_channel(ch.id, "sender-a").unwrap();
            store.join_channel(ch.id, "receiver-b").unwrap();
            store.join_channel(ch.id, "receiver-c").unwrap();

            // Pre-set affect states for participants
            store.affect_states.insert(
                "receiver-b".to_string(),
                AffectState::default(),
            );
            store.affect_states.insert(
                "receiver-c".to_string(),
                AffectState::default(),
            );

            // Send messages with affect metadata
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
                // Inject affect metadata into the stored message
                if let Some(stored) = store.messages.get_mut(&msg_id) {
                    stored.metadata.insert("valence".to_string(), "0.8".to_string());
                    stored.metadata.insert("arousal".to_string(), "0.6".to_string());
                    stored.metadata.insert("dominance".to_string(), "0.5".to_string());
                }
            }

            black_box(store.process_affect_contagion(ch.id));
        });
    });

    // Benchmark the AffectProcessor (standalone) registration and contagion
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

// ---------------------------------------------------------------------------
// Trust/consent benchmark
// ---------------------------------------------------------------------------

fn bench_trust_consent(c: &mut Criterion) {
    let mut group = c.benchmark_group("trust_consent");

    group.bench_function("trust_set_100_agents", |b| {
        b.iter(|| {
            let mut store = CommStore::new();
            for i in 0..100 {
                store
                    .set_trust_level(&format!("agent-{}", i), CommTrustLevel::High)
                    .unwrap();
            }
        });
    });

    group.bench_function("trust_check_100_agents", |b| {
        let mut store = CommStore::new();
        for i in 0..100 {
            store
                .set_trust_level(&format!("agent-{}", i), CommTrustLevel::High)
                .unwrap();
        }
        b.iter(|| {
            for i in 0..100 {
                black_box(store.get_trust_level(&format!("agent-{}", i)));
            }
        });
    });

    group.bench_function("consent_grant_100_gates", |b| {
        b.iter(|| {
            let mut store = CommStore::new();
            for i in 0..100 {
                store
                    .grant_consent(
                        &format!("agent-{}", i),
                        "agent-0",
                        ConsentScope::SendMessages,
                        None,
                        None,
                    )
                    .unwrap();
            }
        });
    });

    group.bench_function("consent_check_in_100_gates", |b| {
        let mut store = CommStore::new();
        for i in 0..100 {
            store
                .grant_consent(
                    &format!("agent-{}", i),
                    "agent-0",
                    ConsentScope::SendMessages,
                    None,
                    None,
                )
                .unwrap();
        }
        b.iter(|| {
            black_box(store.check_consent("agent-50", "agent-0", &ConsentScope::SendMessages));
        });
    });

    // Mixed trust levels benchmark
    group.bench_function("trust_mixed_levels_100", |b| {
        let levels = [
            CommTrustLevel::None,
            CommTrustLevel::Minimal,
            CommTrustLevel::Basic,
            CommTrustLevel::Standard,
            CommTrustLevel::High,
            CommTrustLevel::Full,
            CommTrustLevel::Absolute,
        ];
        b.iter(|| {
            let mut store = CommStore::new();
            for i in 0..100 {
                store
                    .set_trust_level(
                        &format!("agent-{}", i),
                        levels[i % levels.len()],
                    )
                    .unwrap();
            }
            // Then read them all back
            for i in 0..100 {
                black_box(store.get_trust_level(&format!("agent-{}", i)));
            }
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// File format benchmark (expanded save/load)
// ---------------------------------------------------------------------------

fn bench_file_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_format");

    for msg_count in [100, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("save", msg_count),
            msg_count,
            |b, &count| {
                let mut store = CommStore::new();
                let ch = store
                    .create_channel("bench", ChannelType::Direct, None)
                    .unwrap();
                for i in 0..count {
                    store
                        .send_message(
                            ch.id,
                            "agent-a",
                            &format!("msg-{}", i),
                            MessageType::Text,
                        )
                        .unwrap();
                }
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("bench.acomm");
                b.iter(|| {
                    store.save(&path).unwrap();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("load", msg_count),
            msg_count,
            |b, &count| {
                let mut store = CommStore::new();
                let ch = store
                    .create_channel("bench", ChannelType::Direct, None)
                    .unwrap();
                for i in 0..count {
                    store
                        .send_message(
                            ch.id,
                            "agent-a",
                            &format!("msg-{}", i),
                            MessageType::Text,
                        )
                        .unwrap();
                }
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("bench.acomm");
                store.save(&path).unwrap();
                b.iter(|| {
                    black_box(CommStore::load(&path).unwrap());
                });
            },
        );
    }

    // Save/load with rich store state (channels + consent + trust + affect)
    group.bench_function("save_rich_state", |b| {
        let mut store = CommStore::new();
        // Create multiple channels with messages
        for c_idx in 0..5 {
            let ch = store
                .create_channel(
                    &format!("ch-{}", c_idx),
                    ChannelType::Direct,
                    None,
                )
                .unwrap();
            for i in 0..100 {
                store
                    .send_message(
                        ch.id,
                        &format!("agent-{}", i % 10),
                        &format!("msg-{}-{}", c_idx, i),
                        MessageType::Text,
                    )
                    .unwrap();
            }
        }
        // Add trust levels
        for i in 0..50 {
            store
                .set_trust_level(&format!("agent-{}", i), CommTrustLevel::High)
                .unwrap();
        }
        // Add consent gates
        for i in 0..20 {
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
        // Add affect states
        for i in 0..30 {
            store.affect_states.insert(
                format!("agent-{}", i),
                AffectState {
                    valence: (i as f64 / 30.0) * 2.0 - 1.0,
                    arousal: i as f64 / 30.0,
                    dominance: 0.5,
                    ..Default::default()
                },
            );
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rich.acomm");
        b.iter(|| {
            store.save(&path).unwrap();
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// AffectProcessor registry benchmark
// ---------------------------------------------------------------------------

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

criterion_group!(
    benches,
    bench_message_throughput,
    bench_channel_creation,
    bench_message_search,
    bench_save_load,
    bench_consent_check,
    bench_hive_operations,
    bench_semantic_transfer,
    bench_affect_pipeline,
    bench_trust_consent,
    bench_file_format,
    bench_affect_processor,
);
criterion_main!(benches);
