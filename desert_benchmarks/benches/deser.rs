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

#[derive(BinaryCodec)]
enum TestEnum {
    A,
    B(u32),
    C { field: String },
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

fn bench_deserialize_vec_u32(c: &mut Criterion) {
    let data: Vec<u32> = (0..1000).collect();
    bench_deserialize("vec u32 (1000 elements)", data, c);
}

fn bench_deserialize_vec_u32_large(c: &mut Criterion) {
    let data: Vec<u32> = (0..10000).collect();
    bench_deserialize("vec u32 (10000 elements)", data, c);
}

fn bench_deserialize_i32(c: &mut Criterion) {
    bench_deserialize("i32", i32::MAX, c);
}

fn bench_deserialize_f64(c: &mut Criterion) {
    bench_deserialize("f64", std::f64::consts::PI, c);
}

fn bench_deserialize_bool(c: &mut Criterion) {
    bench_deserialize("bool", true, c);
}

fn bench_deserialize_char(c: &mut Criterion) {
    bench_deserialize("char", 'a', c);
}

fn bench_deserialize_string(c: &mut Criterion) {
    bench_deserialize("string", "hello world".to_string(), c);
}

fn bench_deserialize_option_some(c: &mut Criterion) {
    bench_deserialize("option some", Some(42u32), c);
}

fn bench_deserialize_option_none(c: &mut Criterion) {
    bench_deserialize("option none", None::<u32>, c);
}

fn bench_deserialize_tuple(c: &mut Criterion) {
    bench_deserialize("tuple", (42u32, "hello".to_string()), c);
}

fn bench_deserialize_vec_string(c: &mut Criterion) {
    let data: Vec<String> = (0..100).map(|i| format!("item{}", i)).collect();
    bench_deserialize("vec string (100 elements)", data, c);
}

fn bench_deserialize_hashmap(c: &mut Criterion) {
    let data: std::collections::HashMap<String, u32> =
        (0..100).map(|i| (format!("key{}", i), i)).collect();
    bench_deserialize("hashmap (100 elements)", data, c);
}

fn bench_deserialize_hashset(c: &mut Criterion) {
    let data: std::collections::HashSet<u32> = (0..100).collect();
    bench_deserialize("hashset (100 elements)", data, c);
}

fn bench_deserialize_result_ok(c: &mut Criterion) {
    bench_deserialize::<Result<u32, String>>("result ok", Ok(42u32), c);
}

fn bench_deserialize_result_err(c: &mut Criterion) {
    bench_deserialize::<Result<u32, String>>("result err", Err("error message".to_string()), c);
}

fn bench_deserialize_linked_list(c: &mut Criterion) {
    let data: std::collections::LinkedList<String> =
        (0..100).map(|i| format!("item{}", i)).collect();
    bench_deserialize("linked list (100 elements)", data, c);
}

fn bench_deserialize_array(c: &mut Criterion) {
    let data: [u32; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    bench_deserialize("array u32 (10 elements)", data, c);
}

fn bench_deserialize_enum(c: &mut Criterion) {
    bench_deserialize(
        "enum",
        TestEnum::C {
            field: "test".to_string(),
        },
        c,
    );
}

criterion_group!(
    benches,
    bench_deserialize_u64,
    bench_deserialize_wrapped_u64,
    bench_deserialize_evolved_u64,
    bench_deserialize_vec_u32,
    bench_deserialize_vec_u32_large,
    bench_deserialize_i32,
    bench_deserialize_f64,
    bench_deserialize_bool,
    bench_deserialize_char,
    bench_deserialize_string,
    bench_deserialize_option_some,
    bench_deserialize_option_none,
    bench_deserialize_tuple,
    bench_deserialize_vec_string,
    bench_deserialize_hashmap,
    bench_deserialize_hashset,
    bench_deserialize_result_ok,
    bench_deserialize_result_err,
    bench_deserialize_linked_list,
    bench_deserialize_array,
    bench_deserialize_enum
);
criterion_main!(benches);
