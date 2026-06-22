use bigdecimal::BigDecimal;
use bit_vec::BitVec;
use chrono::{NaiveDate, TimeZone, Utc};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use desert_rust::{
    deserialize_with_options, serialize_to_byte_vec_with_options, BinaryCodec, Options,
};
use mac_address::MacAddress;
use nonempty_collections::NEVec;
use serde_json::json;
use std::hint::black_box;
use url::Url;

fn bench_feature_roundtrip<T: BinaryCodec>(name: &str, value: T, c: &mut Criterion) {
    bench_feature_roundtrip_with_options(name, value, Options::default(), c)
}

fn bench_feature_roundtrip_with_options<T: BinaryCodec>(
    name: &str,
    value: T,
    options: Options,
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group("feature codec roundtrip");
    group.bench_with_input(BenchmarkId::from_parameter(name), &value, |b, value| {
        let options = options.clone();
        b.iter(|| {
            let bytes =
                serialize_to_byte_vec_with_options(black_box(value), options.clone()).unwrap();
            black_box(deserialize_with_options::<T>(black_box(&bytes), options.clone()).unwrap())
        });
    });
    group.finish();
}

fn bench_bigdecimal(c: &mut Criterion) {
    let value = "123456789012345678901234567890.123456789"
        .parse::<BigDecimal>()
        .unwrap();
    bench_feature_roundtrip("bigdecimal_text", value.clone(), c);
    bench_feature_roundtrip_with_options(
        "bigdecimal_binary",
        value,
        Options::default().with_binary_bigdecimal(),
        c,
    );
}

fn bench_bit_vec(c: &mut Criterion) {
    let mut bits = BitVec::from_elem(4096, false);
    for idx in (0..4096).step_by(3) {
        bits.set(idx, true);
    }
    bench_feature_roundtrip("bit_vec_4096", bits, c);
}

fn bench_chrono(c: &mut Criterion) {
    bench_feature_roundtrip(
        "chrono_datetime_utc",
        Utc.timestamp_opt(1_700_000_000, 123_456_789).unwrap(),
        c,
    );
    bench_feature_roundtrip(
        "chrono_naive_date",
        NaiveDate::from_ymd_opt(2026, 6, 22).unwrap(),
        c,
    );
}

fn bench_mac_address(c: &mut Criterion) {
    bench_feature_roundtrip("mac_address", MacAddress::new([0, 1, 2, 3, 4, 5]), c);
}

fn bench_nonempty_collections(c: &mut Criterion) {
    bench_feature_roundtrip(
        "nevec_u8_4096",
        NEVec::try_from_vec((0..4096).map(|idx| (idx % 256) as u8).collect()).unwrap(),
        c,
    );
    bench_feature_roundtrip(
        "nevec_string_128",
        NEVec::try_from_vec((0..128).map(|idx| format!("item-{idx}")).collect()).unwrap(),
        c,
    );
}

fn bench_serde_json(c: &mut Criterion) {
    let value = json!({
        "agent": "bench-agent",
        "items": (0..128).collect::<Vec<_>>(),
        "nested": {
            "flag": true,
            "label": "schema-value"
        }
    });
    bench_feature_roundtrip("serde_json_value_text", value.clone(), c);
    bench_feature_roundtrip_with_options(
        "serde_json_value_binary",
        value,
        Options::default().with_binary_json(),
        c,
    );
}

fn bench_url(c: &mut Criterion) {
    bench_feature_roundtrip(
        "url",
        Url::parse("https://example.com/api/v1/agents/bench?limit=100").unwrap(),
        c,
    );
}

criterion_group!(
    benches,
    bench_bigdecimal,
    bench_bit_vec,
    bench_chrono,
    bench_mac_address,
    bench_nonempty_collections,
    bench_serde_json,
    bench_url,
);
criterion_main!(benches);
