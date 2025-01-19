use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use desert_rust::{deserialize, serialize_to_byte_vec, BinaryCodec};
use std::hint::black_box;

fn bench_deserialize<T: BinaryCodec>(name: &str, data: T, c: &mut Criterion) {
    let bytes = serialize_to_byte_vec(&data).unwrap();

    let mut group = c.benchmark_group("deserialize");
    group.bench_with_input(BenchmarkId::from_parameter(name), &bytes, |b, bytes| {
        b.iter(|| {
            black_box(deserialize::<T>(black_box(bytes)).unwrap());
        });
    });
    group.finish()
}

#[derive(BinaryCodec)]
struct WrappedU64 {
    value: u64,
}

#[derive(BinaryCodec)]
#[evolution(FieldAdded("value", 0))]
struct EvolvedU64 {
    value: u64,
}

fn bench_deserialize_u64(c: &mut Criterion) {
    bench_deserialize("u64", u64::MAX, c);
}

fn bench_deserialize_wrapped_u64(c: &mut Criterion) {
    bench_deserialize("wrapped u64", WrappedU64 { value: u64::MAX }, c);
}

fn bench_deserialize_evolved_u64(c: &mut Criterion) {
    bench_deserialize("evolved u64", EvolvedU64 { value: u64::MAX }, c);
}

criterion_group!(
    benches,
    bench_deserialize_u64,
    bench_deserialize_wrapped_u64,
    bench_deserialize_evolved_u64
);
criterion_main!(benches);
