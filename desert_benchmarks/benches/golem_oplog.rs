use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use desert_benchmarks::golem_oplog::*;
use desert_rust::{
    deserialize, serialize_into_byte_vec, serialize_to_byte_vec, serialize_to_byte_vec_exact,
    serialize_to_byte_vec_with_capacity,
};

fn bench_single_entries(c: &mut Criterion) {
    let cases = golem_oplog_entry_cases();

    let mut group = c.benchmark_group("golem oplog entry roundtrip");
    for (name, entry) in &cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), entry, |b, entry| {
            b.iter(|| {
                let bytes = serialize_to_byte_vec(black_box(entry)).unwrap();
                black_box(deserialize::<GolemOplogEntry>(black_box(&bytes)).unwrap())
            });
        });
    }
    group.finish();
}

fn bench_oplog_payloads(c: &mut Criterion) {
    let cases = oplog_payload_cases();

    let mut group = c.benchmark_group("golem oplog payload roundtrip");
    for (name, payload) in &cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), payload, |b, payload| {
            b.iter(|| {
                let bytes = serialize_to_byte_vec(black_box(payload)).unwrap();
                black_box(deserialize::<OplogPayload<HostRequest>>(black_box(&bytes)).unwrap())
            });
        });
    }
    group.finish();
}

fn bench_versioned_wrapper(c: &mut Criterion) {
    let entry = golem_oplog_entry_cases().remove(1).1;
    let batch = golem_oplog_batch(512);
    let entry_bytes = serialize_with_version(&entry).unwrap();
    let batch_bytes = serialize_with_version(&batch).unwrap();

    let mut group = c.benchmark_group("golem serialization wrapper");
    group.bench_function("serialize_entry_with_version", |b| {
        b.iter(|| black_box(serialize_with_version(black_box(&entry)).unwrap()));
    });
    group.bench_function("deserialize_entry_with_version", |b| {
        b.iter(|| {
            black_box(deserialize_with_version::<GolemOplogEntry>(black_box(&entry_bytes)).unwrap())
        });
    });
    group.bench_function("serialize_batch_with_version_512", |b| {
        b.iter(|| black_box(serialize_with_version(black_box(&batch)).unwrap()));
    });
    group.bench_function("deserialize_batch_with_version_512", |b| {
        b.iter(|| {
            black_box(
                deserialize_with_version::<Vec<GolemOplogEntry>>(black_box(&batch_bytes)).unwrap(),
            )
        });
    });
    group.finish();
}

fn bench_batch_paths(c: &mut Criterion) {
    let batch = golem_oplog_batch(1024);
    let batch_bytes = serialize_to_byte_vec(&batch).unwrap();
    let batch_capacity = batch_bytes.len();

    let mut group = c.benchmark_group("golem oplog batch");
    group.bench_function("serialize_vec_entries_1024", |b| {
        b.iter(|| black_box(serialize_to_byte_vec(black_box(&batch)).unwrap()));
    });
    group.bench_function("serialize_vec_entries_1024_with_capacity", |b| {
        b.iter(|| {
            black_box(
                serialize_to_byte_vec_with_capacity(black_box(&batch), batch_capacity).unwrap(),
            )
        });
    });
    group.bench_function("serialize_vec_entries_1024_exact", |b| {
        b.iter(|| black_box(serialize_to_byte_vec_exact(black_box(&batch)).unwrap()));
    });
    group.bench_function("serialize_vec_entries_1024_reuse_buffer", |b| {
        let mut output = Vec::with_capacity(batch_capacity);
        b.iter(|| {
            serialize_into_byte_vec(black_box(&batch), &mut output).unwrap();
            black_box(output.len())
        });
    });
    group.bench_function("deserialize_vec_entries_1024", |b| {
        b.iter(|| black_box(deserialize::<Vec<GolemOplogEntry>>(black_box(&batch_bytes)).unwrap()));
    });
    group.bench_function("roundtrip_vec_entries_1024", |b| {
        b.iter(|| {
            let bytes = serialize_to_byte_vec(black_box(&batch)).unwrap();
            black_box(deserialize::<Vec<GolemOplogEntry>>(black_box(&bytes)).unwrap())
        });
    });
    group.finish();
}

fn bench_chunk_path(c: &mut Criterion) {
    let chunk = golem_oplog_chunk(1024);
    let chunk_bytes = serialize_to_byte_vec(&chunk).unwrap();

    let mut group = c.benchmark_group("golem compressed oplog chunk envelope");
    group.bench_function("serialize_chunk_envelope_1024", |b| {
        b.iter(|| black_box(serialize_to_byte_vec(black_box(&chunk)).unwrap()));
    });
    group.bench_function("deserialize_chunk_envelope_1024", |b| {
        b.iter(|| black_box(deserialize::<CompressedOplogChunk>(black_box(&chunk_bytes)).unwrap()));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_single_entries,
    bench_oplog_payloads,
    bench_versioned_wrapper,
    bench_batch_paths,
    bench_chunk_path,
);
criterion_main!(benches);
