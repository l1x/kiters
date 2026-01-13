//! Benchmarks for the eid (External ID) module.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use kiters::eid::ExternalId;
use std::hint::black_box;
use uuid::Uuid;

fn bench_eid(c: &mut Criterion) {
    let mut group = c.benchmark_group("eid_generation");
    group.throughput(Throughput::Elements(1));

    // Baseline: just UUID generation
    group.bench_function("uuid/v4_only", |b| {
        b.iter(|| black_box(Uuid::new_v4()))
    });

    // UUID to string
    group.bench_function("uuid/v4_to_string", |b| {
        b.iter(|| black_box(Uuid::new_v4().to_string()))
    });

    // Our ExternalId::new (UUID + prefix allocation)
    group.bench_function("eid/new", |b| {
        b.iter(|| black_box(ExternalId::new(black_box("user"))))
    });

    // ExternalId to_string (base36 encoding + format)
    group.bench_function("eid/to_string", |b| {
        let id = ExternalId::new("user");
        b.iter(|| black_box(id.to_string()))
    });

    // Full flow: new + to_string
    group.bench_function("eid/new_and_to_string", |b| {
        b.iter(|| {
            let id = ExternalId::new(black_box("user"));
            black_box(id.to_string())
        })
    });

    group.finish();
}

fn bench_eid_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("eid_batch_1000");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("uuid/v4_only", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(Uuid::new_v4());
            }
        })
    });

    group.bench_function("eid/new", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(ExternalId::new("user"));
            }
        })
    });

    group.bench_function("eid/new_and_to_string", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let id = ExternalId::new("user");
                black_box(id.to_string());
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_eid, bench_eid_batch);
criterion_main!(benches);
