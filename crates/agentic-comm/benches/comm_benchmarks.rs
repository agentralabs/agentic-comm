use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use agentic_comm::{
    ChannelType, CollectiveDecisionMode, CommStore, ConsentScope, HiveRole, MessageType,
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

criterion_group!(
    benches,
    bench_message_throughput,
    bench_channel_creation,
    bench_message_search,
    bench_save_load,
    bench_consent_check,
    bench_hive_operations,
);
criterion_main!(benches);
