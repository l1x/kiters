//! Benchmarks comparing nanoid vs our request_id implementation.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use kiters::request_id::{
    as_str, encode_request_id, encode_request_id_mixed,
    encode_request_id_wide, encode_request_id_mixed_wide,
    RequestIdGenerator, WideRequestIdGenerator,
};

fn bench_request_id(c: &mut Criterion) {
    let mut group = c.benchmark_group("id_generation");
    group.throughput(Throughput::Elements(1));

    // Our implementations — 6 chars
    group.bench_function("request_id/encode_plain", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            counter += 1;
            black_box(encode_request_id(black_box(counter)))
        })
    });

    group.bench_function("request_id/encode_mixed", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            counter += 1;
            black_box(encode_request_id_mixed(black_box(counter)))
        })
    });

    // Our implementations — 11 chars (wide)
    group.bench_function("request_id/encode_wide", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            counter += 1;
            black_box(encode_request_id_wide(black_box(counter)))
        })
    });

    group.bench_function("request_id/encode_mixed_wide", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            counter += 1;
            black_box(encode_request_id_mixed_wide(black_box(counter)))
        })
    });

    // Generators — 6 chars
    group.bench_function("request_id/generator_next_id", |b| {
        let generator: RequestIdGenerator = RequestIdGenerator::new();
        b.iter(|| black_box(generator.next_id()))
    });

    group.bench_function("request_id/generator_mixed", |b| {
        let generator: RequestIdGenerator = RequestIdGenerator::new_mixed();
        b.iter(|| black_box(generator.next_id()))
    });

    group.bench_function("request_id/generator_to_string", |b| {
        let generator: RequestIdGenerator = RequestIdGenerator::new();
        b.iter(|| black_box(generator.next_id_string()))
    });

    // Generators — 11 chars (wide)
    group.bench_function("request_id/generator_wide", |b| {
        let generator = WideRequestIdGenerator::new();
        b.iter(|| black_box(generator.next_id()))
    });

    group.bench_function("request_id/generator_mixed_wide", |b| {
        let generator = WideRequestIdGenerator::new_mixed();
        b.iter(|| black_box(generator.next_id()))
    });

    group.bench_function("request_id/generator_wide_to_string", |b| {
        let generator = WideRequestIdGenerator::new();
        b.iter(|| black_box(generator.next_id_string()))
    });

    // nanoid - default 21 chars
    group.bench_function("nanoid/default_21", |b| {
        b.iter(|| black_box(nanoid::nanoid!()))
    });

    // nanoid - 6 chars (same length as our request_id)
    group.bench_function("nanoid/6_chars", |b| {
        b.iter(|| black_box(nanoid::nanoid!(6)))
    });

    group.finish();
}

fn bench_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_1000");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("request_id/encode_plain", |b| {
        b.iter(|| {
            for i in 0u64..1000 {
                black_box(encode_request_id(black_box(i)));
            }
        })
    });

    group.bench_function("request_id/encode_wide", |b| {
        b.iter(|| {
            for i in 0u64..1000 {
                black_box(encode_request_id_wide(black_box(i)));
            }
        })
    });

    group.bench_function("request_id/generator", |b| {
        let generator: RequestIdGenerator = RequestIdGenerator::new();
        b.iter(|| {
            for _ in 0..1000 {
                black_box(generator.next_id());
            }
        })
    });

    group.bench_function("request_id/generator_wide", |b| {
        let generator = WideRequestIdGenerator::new();
        b.iter(|| {
            for _ in 0..1000 {
                black_box(generator.next_id());
            }
        })
    });

    group.bench_function("nanoid/6_chars", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(nanoid::nanoid!(6));
            }
        })
    });

    group.finish();
}

fn bench_to_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("to_string");
    group.throughput(Throughput::Elements(1));

    group.bench_function("request_id/as_str", |b| {
        let id = encode_request_id(12345);
        b.iter(|| black_box(as_str(black_box(&id))))
    });

    group.bench_function("request_id/as_str_wide", |b| {
        let id = encode_request_id_wide(12345);
        b.iter(|| black_box(as_str(black_box(&id))))
    });

    group.bench_function("request_id/to_owned", |b| {
        let id = encode_request_id(12345);
        b.iter(|| black_box(as_str(&id).to_owned()))
    });

    group.bench_function("request_id/to_owned_wide", |b| {
        let id = encode_request_id_wide(12345);
        b.iter(|| black_box(as_str(&id).to_owned()))
    });

    group.bench_function("nanoid/already_string", |b| {
        // nanoid returns String directly, so this measures just the generation
        b.iter(|| black_box(nanoid::nanoid!(6)))
    });

    group.finish();
}

criterion_group!(benches, bench_request_id, bench_batch, bench_to_string);
criterion_main!(benches);
