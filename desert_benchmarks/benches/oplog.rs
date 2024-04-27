use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::prelude::StdRng;
use rand::SeedableRng;

use desert::*;
use desert_benchmarks::model::*;

fn generate_cases() -> Vec<Case> {
    let mut rng = StdRng::seed_from_u64(317826381);

    let payload_sizes = [16, 256, 1024, 128 * 1024];
    payload_sizes
        .iter()
        .map(|payload_size| {
            let entries = (0..10000)
                .map(|_| random_oplog_entry(&mut rng, *payload_size))
                .collect::<Vec<_>>();
            Case {
                payload_size: *payload_size,
                entries,
            }
        })
        .collect::<Vec<_>>()
}

fn bench_serialize(c: &mut Criterion) {
    let cases = generate_cases();
    let mut group = c.benchmark_group("serialize oplog");
    for case in cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(case.payload_size),
            &case,
            |b, case| {
                b.iter(|| {
                    let mut entries = Vec::with_capacity(10000);
                    for entry in &case.entries {
                        //let bytes = black_box(serialize_to_bytes(black_box(&entry)).unwrap());
                        let bytes = black_box(serialize_to_byte_vec(black_box(&entry)).unwrap());
                        entries.push(bytes);
                    }
                    entries
                });
            },
        );
    }
    group.finish()
}

fn bench_deserialize(c: &mut Criterion) {
    let cases = generate_cases();
    let serialized_cases = cases
        .into_iter()
        .map(|case| {
            let mut entries = Vec::new();
            for entry in &case.entries {
                entries.push(serialize_to_bytes(&entry).unwrap());
            }
            (entries, case.payload_size)
        })
        .collect::<Vec<_>>();
    let mut group = c.benchmark_group("deserialize oplog");
    for (serialized_entries, payload_size) in serialized_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(payload_size),
            &serialized_entries,
            |b, serialized_entries| {
                b.iter(|| {
                    let mut results = Vec::with_capacity(10000);
                    for bytes in serialized_entries {
                        let entry: OplogEntry = black_box(deserialize(black_box(bytes)).unwrap());
                        results.push(entry);
                    }
                    results
                });
            },
        );
    }
    group.finish()
}

fn bench_serialize_bincode(c: &mut Criterion) {
    let cases = generate_cases();
    let mut group = c.benchmark_group("bincode serialize oplog");
    for case in cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(case.payload_size),
            &case,
            |b, case| {
                b.iter(|| {
                    let mut entries = Vec::with_capacity(10000);
                    for entry in &case.entries {
                        let bytes = black_box(
                            bincode::encode_to_vec(black_box(entry), bincode::config::standard())
                                .unwrap(),
                        );
                        entries.push(bytes);
                    }
                    entries
                });
            },
        );
    }
    group.finish()
}

fn bench_deserialize_bincode(c: &mut Criterion) {
    let cases = generate_cases();
    let serialized_cases = cases
        .into_iter()
        .map(|case| {
            let mut entries = Vec::new();
            for entry in &case.entries {
                entries.push(serialize_to_bytes(&entry).unwrap());
            }
            (entries, case.payload_size)
        })
        .collect::<Vec<_>>();
    let mut group = c.benchmark_group("bincode deserialize oplog");
    for (serialized_entries, payload_size) in serialized_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(payload_size),
            &serialized_entries,
            |b, serialized_entries| {
                b.iter(|| {
                    let mut results = Vec::with_capacity(10000);
                    for bytes in serialized_entries {
                        let (entry, _): (OplogEntry, usize) = black_box(
                            bincode::decode_from_slice(
                                black_box(bytes),
                                bincode::config::standard(),
                            )
                            .unwrap(),
                        );
                        results.push(entry);
                    }
                    results
                });
            },
        );
    }
    group.finish()
}

criterion_group!(
    benches,
    bench_serialize,
    bench_serialize_bincode,
    bench_deserialize,
    bench_deserialize_bincode
);
criterion_main!(benches);
