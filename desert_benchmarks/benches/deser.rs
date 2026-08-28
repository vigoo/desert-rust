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
#[desert(evolution(FieldAdded("value", 0)))]
struct EvolvedU64 {
    value: u64,
}

#[derive(BinaryCodec)]
enum TestEnum {
    A,
    B(u32),
    C { field: String },
}

macro_rules! large_enum {
    ($($variant:ident),* $(,)?) => {
        #[allow(dead_code)]
        #[derive(BinaryCodec)]
        enum LargeEnum {
            Custom(String),
            $($variant),*
        }
    };
}

large_enum!(
    V000, V001, V002, V003, V004, V005, V006, V007, V008, V009, V010, V011, V012, V013, V014, V015,
    V016, V017, V018, V019, V020, V021, V022, V023, V024, V025, V026, V027, V028, V029, V030, V031,
    V032, V033, V034, V035, V036, V037, V038, V039, V040, V041, V042, V043, V044, V045, V046, V047,
    V048, V049, V050, V051, V052, V053, V054, V055, V056, V057, V058, V059, V060, V061, V062, V063,
    V064, V065, V066, V067, V068, V069, V070, V071, V072, V073, V074, V075, V076, V077, V078, V079,
    V080, V081, V082, V083, V084, V085, V086, V087, V088, V089, V090, V091, V092, V093, V094, V095,
    V096, V097, V098, V099, V100, V101, V102, V103, V104, V105, V106, V107, V108, V109, V110, V111,
    V112, V113, V114, V115, V116, V117, V118, V119, V120, V121, V122, V123, V124, V125, V126, V127,
    V128, V129, V130, V131, V132, V133, V134, V135, V136, V137, V138, V139, V140, V141, V142, V143,
    V144, V145, V146, V147, V148, V149, V150, V151, V152, V153, V154, V155, V156, V157, V158, V159,
    V160, V161, V162, V163, V164, V165, V166, V167, V168, V169, V170, V171, V172,
);

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

fn bench_deserialize_large_enum(c: &mut Criterion) {
    bench_deserialize(
        "large enum payload variant",
        LargeEnum::Custom("test".to_string()),
        c,
    );
    bench_deserialize("large enum unit variant", LargeEnum::V172, c);
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
    bench_deserialize_enum,
    bench_deserialize_large_enum,
);
criterion_main!(benches);
