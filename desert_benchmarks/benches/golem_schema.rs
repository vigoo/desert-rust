use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use desert_benchmarks::golem_schema::*;
use desert_rust::{deserialize, serialize_to_byte_vec, BinaryCodec};
use std::hint::black_box;

fn bench_roundtrip<T: BinaryCodec>(group_name: &str, id: &str, value: T, c: &mut Criterion) {
    let mut group = c.benchmark_group(group_name);
    group.bench_with_input(BenchmarkId::from_parameter(id), &value, |b, value| {
        b.iter(|| {
            let bytes = serialize_to_byte_vec(black_box(value)).unwrap();
            black_box(deserialize::<T>(black_box(&bytes)).unwrap())
        });
    });
    group.finish();
}

fn bench_serialize_typed_values(c: &mut Criterion) {
    let cases = typed_value_cases();
    let mut group = c.benchmark_group("golem schema serialize typed value");
    for (name, value) in &cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), value, |b, value| {
            b.iter(|| black_box(serialize_to_byte_vec(black_box(value)).unwrap()));
        });
    }
    group.finish();
}

fn bench_deserialize_typed_values(c: &mut Criterion) {
    let cases = typed_value_cases()
        .into_iter()
        .map(|(name, value)| (name, serialize_to_byte_vec(&value).unwrap()))
        .collect::<Vec<_>>();
    let mut group = c.benchmark_group("golem schema deserialize typed value");
    for (name, bytes) in &cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), bytes, |b, bytes| {
            b.iter(|| black_box(deserialize::<TypedSchemaValue>(black_box(bytes)).unwrap()));
        });
    }
    group.finish();
}

fn bench_schema_graph(c: &mut Criterion) {
    let (wide_graph, _) = wide_graph_with_recursive_value(64);
    bench_roundtrip(
        "golem schema graph roundtrip",
        "wide_64_defs",
        wide_graph,
        c,
    );
}

fn bench_schema_value_cache_key(c: &mut Criterion) {
    let value = cache_key_schema_value();
    let mut group = c.benchmark_group("golem schema cache key");
    group.bench_function("canonicalize_schema_value", |b| {
        b.iter(|| black_box(serialize_to_byte_vec(black_box(&value)).unwrap()));
    });
    group.finish();
}

fn bench_wide_typed_value(c: &mut Criterion) {
    bench_roundtrip(
        "golem schema typed value roundtrip",
        "wide_recursive",
        wide_typed_value(),
        c,
    );
}

fn bench_agent_type_schema(c: &mut Criterion) {
    bench_roundtrip(
        "golem schema agent type roundtrip",
        "representative",
        representative_agent_type_schema(),
        c,
    );
}

criterion_group!(
    benches,
    bench_serialize_typed_values,
    bench_deserialize_typed_values,
    bench_schema_graph,
    bench_schema_value_cache_key,
    bench_wide_typed_value,
    bench_agent_type_schema,
);
criterion_main!(benches);
