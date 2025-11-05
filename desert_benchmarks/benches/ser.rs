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

#[derive(BinaryCodec)]
enum TestEnum {
    A,
    B(u32),
    C { field: String },
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

fn bench_serialize_vec_u32(c: &mut Criterion) {
    let data: Vec<u32> = (0..1000).collect();
    bench_serialize("vec u32 (1000 elements)", data, c);
}

fn bench_serialize_vec_u32_large(c: &mut Criterion) {
    let data: Vec<u32> = (0..10000).collect();
    bench_serialize("vec u32 (10000 elements)", data, c);
}

fn bench_serialize_i32(c: &mut Criterion) {
    bench_serialize("i32", i32::MAX, c);
}

fn bench_serialize_f64(c: &mut Criterion) {
    bench_serialize("f64", std::f64::consts::PI, c);
}

fn bench_serialize_bool(c: &mut Criterion) {
    bench_serialize("bool", true, c);
}

fn bench_serialize_char(c: &mut Criterion) {
    bench_serialize("char", 'a', c);
}

fn bench_serialize_string(c: &mut Criterion) {
    bench_serialize("string", "hello world".to_string(), c);
}

fn bench_serialize_option_some(c: &mut Criterion) {
    bench_serialize("option some", Some(42u32), c);
}

fn bench_serialize_option_none(c: &mut Criterion) {
    bench_serialize("option none", None::<u32>, c);
}

fn bench_serialize_tuple(c: &mut Criterion) {
    bench_serialize("tuple", (42u32, "hello".to_string()), c);
}

fn bench_serialize_vec_string(c: &mut Criterion) {
    let data: Vec<String> = (0..100).map(|i| format!("item{}", i)).collect();
    bench_serialize("vec string (100 elements)", data, c);
}

fn bench_serialize_hashmap(c: &mut Criterion) {
    let data: std::collections::HashMap<String, u32> =
        (0..100).map(|i| (format!("key{}", i), i)).collect();
    bench_serialize("hashmap (100 elements)", data, c);
}

fn bench_serialize_hashset(c: &mut Criterion) {
    let data: std::collections::HashSet<u32> = (0..100).collect();
    bench_serialize("hashset (100 elements)", data, c);
}

fn bench_serialize_result_ok(c: &mut Criterion) {
    bench_serialize::<Result<u32, String>>("result ok", Ok(42u32), c);
}

fn bench_serialize_result_err(c: &mut Criterion) {
    bench_serialize::<Result<u32, String>>("result err", Err("error message".to_string()), c);
}

fn bench_serialize_linked_list(c: &mut Criterion) {
    let data: std::collections::LinkedList<String> =
        (0..100).map(|i| format!("item{}", i)).collect();
    bench_serialize("linked list (100 elements)", data, c);
}

fn bench_serialize_array(c: &mut Criterion) {
    let data: [u32; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    bench_serialize("array u32 (10 elements)", data, c);
}

fn bench_serialize_enum(c: &mut Criterion) {
    bench_serialize(
        "enum",
        TestEnum::C {
            field: "test".to_string(),
        },
        c,
    );
}

criterion_group!(
    benches,
    bench_serialize_u64,
    bench_serialize_wrapped_u64,
    bench_serialize_evolved_u64,
    bench_serialize_vec_u32,
    bench_serialize_vec_u32_large,
    bench_serialize_i32,
    bench_serialize_f64,
    bench_serialize_bool,
    bench_serialize_char,
    bench_serialize_string,
    bench_serialize_option_some,
    bench_serialize_option_none,
    bench_serialize_tuple,
    bench_serialize_vec_string,
    bench_serialize_hashmap,
    bench_serialize_hashset,
    bench_serialize_result_ok,
    bench_serialize_result_err,
    bench_serialize_linked_list,
    bench_serialize_array,
    bench_serialize_enum
);
criterion_main!(benches);
