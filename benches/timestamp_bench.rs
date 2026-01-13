//! Benchmarks for the timestamp module.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use kiters::timestamp::{get_utc_formatter, get_utc_timestamp};
use std::hint::black_box;
use time::OffsetDateTime;

fn bench_timestamp(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestamp");
    group.throughput(Throughput::Elements(1));

    // Baseline: just get current time (no formatting)
    group.bench_function("time/now_utc", |b| {
        b.iter(|| black_box(OffsetDateTime::now_utc()))
    });

    // Get formatter (should be essentially free - static)
    group.bench_function("timestamp/get_formatter", |b| {
        b.iter(|| black_box(get_utc_formatter()))
    });

    // Format pre-obtained time
    group.bench_function("timestamp/format_only", |b| {
        let now = OffsetDateTime::now_utc();
        let formatter = get_utc_formatter();
        b.iter(|| black_box(now.format(formatter).unwrap()))
    });

    // Full flow: get time + format
    group.bench_function("timestamp/get_utc_timestamp", |b| {
        b.iter(|| black_box(get_utc_timestamp()))
    });

    // Compare with time crate's built-in RFC3339 formatting
    group.bench_function("time/rfc3339", |b| {
        use time::format_description::well_known::Rfc3339;
        b.iter(|| {
            let now = OffsetDateTime::now_utc();
            black_box(now.format(&Rfc3339).unwrap())
        })
    });

    group.finish();
}

fn bench_timestamp_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestamp_batch_1000");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("time/now_utc", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(OffsetDateTime::now_utc());
            }
        })
    });

    group.bench_function("timestamp/get_utc_timestamp", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(get_utc_timestamp());
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_timestamp, bench_timestamp_batch);
criterion_main!(benches);
