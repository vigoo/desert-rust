use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use desert_rust::{serialize_to_byte_vec, BinaryCodec};
use std::hint::black_box;

fn bench_serialize<T: BinaryCodec>(name: &str, data: T, c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize");
    group.bench_with_input(BenchmarkId::from_parameter(name), &data, |b, data| {
        b.iter(|| {
            black_box(serialize_to_byte_vec(black_box(data)).unwrap());
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

fn bench_serialize_u64(c: &mut Criterion) {
    bench_serialize("u64", u64::MAX, c);
}

fn bench_serialize_wrapped_u64(c: &mut Criterion) {
    bench_serialize("wrapped u64", WrappedU64 { value: u64::MAX }, c);
}

fn bench_serialize_evolved_u64(c: &mut Criterion) {
    bench_serialize("evolved u64", EvolvedU64 { value: u64::MAX }, c);
}

criterion_group!(
    benches,
    bench_serialize_u64,
    bench_serialize_wrapped_u64,
    bench_serialize_evolved_u64
);
criterion_main!(benches);
