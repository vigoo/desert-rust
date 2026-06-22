use std::collections::HashMap;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use desert_rust::{deserialize, serialize_to_byte_vec, BinaryCodec, BinaryDeserializer};

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
struct AddedChunkV1 {
    id: u64,
    name: String,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution(
    FieldAdded("metadata", HashMap::<String, String>::new()),
    FieldAdded("tags", Vec::<String>::new()),
    FieldAdded("retry_count", 0_u32),
    FieldAdded("enabled", true)
))]
struct AddedChunkV2 {
    id: u64,
    name: String,
    metadata: HashMap<String, String>,
    tags: Vec<String>,
    retry_count: u32,
    enabled: bool,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
struct RemovedPayloadV1 {
    id: u64,
    label: String,
    tags: Vec<String>,
    payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution(FieldMadeOptional("payload"), FieldRemoved("payload")))]
struct RemovedPayloadV2 {
    id: u64,
    label: String,
    tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
struct OptionalV1 {
    id: u64,
    label: String,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution(FieldMadeOptional("label")))]
struct OptionalV2 {
    id: u64,
    label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
struct TransientV1 {
    id: u64,
    name: String,
    cached: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution(FieldMadeTransient("cached")))]
struct TransientV2 {
    id: u64,
    name: String,
    #[transient(Vec::new())]
    cached: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
enum NestedEnumV1 {
    Running {
        request_id: String,
        payload: Vec<u8>,
    },
    Finished {
        result: String,
    },
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
enum NestedEnumV2 {
    #[desert(evolution(
        FieldMadeOptional("payload"),
        FieldRemoved("payload"),
        FieldAdded("trace_id", "trace-default".to_string())
    ))]
    Running {
        request_id: String,
        trace_id: String,
    },
    #[desert(evolution(FieldMadeOptional("result"), FieldAdded("metrics", Vec::<u64>::new())))]
    Finished {
        result: Option<String>,
        metrics: Vec<u64>,
    },
    #[transient]
    #[allow(dead_code)]
    Cancelled,
}

fn added_v1() -> AddedChunkV1 {
    AddedChunkV1 {
        id: 42,
        name: "bench-added-v1".to_string(),
    }
}

fn added_v2() -> AddedChunkV2 {
    AddedChunkV2 {
        id: 42,
        name: "bench-added-v2".to_string(),
        metadata: HashMap::from([
            ("tenant".to_string(), "benchmark".to_string()),
            ("source".to_string(), "golem-oplog".to_string()),
        ]),
        tags: (0..32).map(|idx| format!("tag-{idx}")).collect(),
        retry_count: 3,
        enabled: true,
    }
}

fn removed_payload_v1() -> RemovedPayloadV1 {
    RemovedPayloadV1 {
        id: 100,
        label: "large payload that should be skipped".to_string(),
        tags: (0..128).map(|idx| format!("skip-tag-{idx}")).collect(),
        payload: large_bytes(256 * 1024, 0xA5),
    }
}

fn optional_v1() -> OptionalV1 {
    OptionalV1 {
        id: 200,
        label: "was-required".to_string(),
    }
}

fn optional_v2_some() -> OptionalV2 {
    OptionalV2 {
        id: 200,
        label: Some("still-present".to_string()),
    }
}

fn transient_v1() -> TransientV1 {
    TransientV1 {
        id: 300,
        name: "transient-cache".to_string(),
        cached: large_bytes(128 * 1024, 0x5A),
    }
}

fn nested_running_v1() -> NestedEnumV1 {
    NestedEnumV1::Running {
        request_id: "request-0001".to_string(),
        payload: large_bytes(64 * 1024, 0x11),
    }
}

fn nested_finished_v1() -> NestedEnumV1 {
    NestedEnumV1::Finished {
        result: "ok".to_string(),
    }
}

fn nested_finished_v2() -> NestedEnumV2 {
    NestedEnumV2::Finished {
        result: Some("ok".to_string()),
        metrics: (0..128).collect(),
    }
}

fn bench_old_bytes_into_new(c: &mut Criterion) {
    let added_v1_bytes = serialized(&added_v1());
    let removed_payload_v1_bytes = serialized(&removed_payload_v1());
    let optional_v1_bytes = serialized(&optional_v1());
    let transient_v1_bytes = serialized(&transient_v1());
    let nested_running_v1_bytes = serialized(&nested_running_v1());
    let nested_finished_v1_bytes = serialized(&nested_finished_v1());

    assert_deserializes::<AddedChunkV2>(&added_v1_bytes, "added_v1_to_v2");
    assert_deserializes::<RemovedPayloadV2>(&removed_payload_v1_bytes, "removed_payload_v1_to_v2");
    assert_deserializes::<OptionalV2>(&optional_v1_bytes, "optional_v1_to_v2");
    assert_deserializes::<TransientV2>(&transient_v1_bytes, "transient_v1_to_v2");
    assert_deserializes::<NestedEnumV2>(&nested_running_v1_bytes, "nested_running_v1_to_v2");
    assert_deserializes::<NestedEnumV2>(&nested_finished_v1_bytes, "nested_finished_v1_to_v2");

    let mut group = c.benchmark_group("evolution old bytes into new type");
    group.bench_function("field_added_chunk", |b| {
        b.iter(|| black_box(deserialize::<AddedChunkV2>(black_box(&added_v1_bytes)).unwrap()));
    });
    group.bench_function("removed_large_payload_skip", |b| {
        b.iter(|| {
            black_box(
                deserialize::<RemovedPayloadV2>(black_box(&removed_payload_v1_bytes)).unwrap(),
            )
        });
    });
    group.bench_function("required_field_to_optional", |b| {
        b.iter(|| black_box(deserialize::<OptionalV2>(black_box(&optional_v1_bytes)).unwrap()));
    });
    group.bench_function("field_made_transient_skip", |b| {
        b.iter(|| black_box(deserialize::<TransientV2>(black_box(&transient_v1_bytes)).unwrap()));
    });
    group.bench_function("nested_enum_removed_payload", |b| {
        b.iter(|| {
            black_box(deserialize::<NestedEnumV2>(black_box(&nested_running_v1_bytes)).unwrap())
        });
    });
    group.bench_function("nested_enum_optional_and_added", |b| {
        b.iter(|| {
            black_box(deserialize::<NestedEnumV2>(black_box(&nested_finished_v1_bytes)).unwrap())
        });
    });
    group.finish();
}

fn bench_new_bytes_into_old(c: &mut Criterion) {
    let added_v2_bytes = serialized(&added_v2());
    let optional_v2_some_bytes = serialized(&optional_v2_some());
    let nested_finished_v2_bytes = serialized(&nested_finished_v2());

    assert_deserializes::<AddedChunkV1>(&added_v2_bytes, "added_v2_to_v1");
    assert_deserializes::<OptionalV1>(&optional_v2_some_bytes, "optional_v2_to_v1");
    assert_deserializes::<NestedEnumV1>(&nested_finished_v2_bytes, "nested_finished_v2_to_v1");

    let mut group = c.benchmark_group("evolution new bytes into old type");
    group.bench_function("field_added_chunk_skipped_by_old", |b| {
        b.iter(|| black_box(deserialize::<AddedChunkV1>(black_box(&added_v2_bytes)).unwrap()));
    });
    group.bench_function("optional_some_back_to_required", |b| {
        b.iter(|| {
            black_box(deserialize::<OptionalV1>(black_box(&optional_v2_some_bytes)).unwrap())
        });
    });
    group.bench_function("nested_enum_added_fields_skipped_by_old", |b| {
        b.iter(|| {
            black_box(deserialize::<NestedEnumV1>(black_box(&nested_finished_v2_bytes)).unwrap())
        });
    });
    group.finish();
}

fn bench_roundtrip_evolved_shapes(c: &mut Criterion) {
    let added = added_v2();
    let optional = optional_v2_some();
    let transient = TransientV2 {
        id: 300,
        name: "transient-cache".to_string(),
        cached: large_bytes(128 * 1024, 0x77),
    };
    let nested = nested_finished_v2();

    let mut group = c.benchmark_group("evolution evolved shape roundtrip");
    group.bench_function("field_added_chunk_v2", |b| {
        b.iter(|| {
            let bytes = serialize_to_byte_vec(black_box(&added)).unwrap();
            black_box(deserialize::<AddedChunkV2>(black_box(&bytes)).unwrap())
        });
    });
    group.bench_function("optional_v2", |b| {
        b.iter(|| {
            let bytes = serialize_to_byte_vec(black_box(&optional)).unwrap();
            black_box(deserialize::<OptionalV2>(black_box(&bytes)).unwrap())
        });
    });
    group.bench_function("transient_v2", |b| {
        b.iter(|| {
            let bytes = serialize_to_byte_vec(black_box(&transient)).unwrap();
            black_box(deserialize::<TransientV2>(black_box(&bytes)).unwrap())
        });
    });
    group.bench_function("nested_enum_v2", |b| {
        b.iter(|| {
            let bytes = serialize_to_byte_vec(black_box(&nested)).unwrap();
            black_box(deserialize::<NestedEnumV2>(black_box(&bytes)).unwrap())
        });
    });
    group.finish();
}

fn serialized<T: desert_rust::BinarySerializer>(value: &T) -> Vec<u8> {
    serialize_to_byte_vec(value).unwrap()
}

fn assert_deserializes<T: BinaryDeserializer>(bytes: &[u8], label: &str) {
    let _: T = deserialize(bytes).unwrap_or_else(|err| panic!("{label}: {err}"));
}

fn large_bytes(size: usize, seed: u8) -> Vec<u8> {
    (0..size)
        .map(|idx| seed.wrapping_add((idx % 251) as u8))
        .collect()
}

criterion_group!(
    benches,
    bench_old_bytes_into_new,
    bench_new_bytes_into_old,
    bench_roundtrip_evolved_shapes,
);
criterion_main!(benches);
