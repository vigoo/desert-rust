use std::hint::black_box;
use std::time::{Duration, Instant};

use bytes::{BufMut, Bytes, BytesMut};
use rand::prelude::StdRng;
use rand::*;
use serde::{Deserialize, Serialize};

use crate::model::{random_oplog_entry, Case};
use desert::{serialize_to_byte_vec, SliceInput};
use model::OplogEntry;

mod model;

struct Report {
    name: String,
    total_size: usize,
    se_duration: Duration,
    de_duration: Duration,
}

fn benchmark<S: Fn(&OplogEntry) -> Bytes, D: Fn(&Bytes) -> OplogEntry>(
    case: &Case,
    name: &str,
    ser: S,
    deser: D,
) -> Report {
    println!("...{name}...");
    let mut total_size = 0;

    let mut se_duration = Duration::ZERO;
    let mut entries = Vec::with_capacity(10000);

    for _ in 0..10 {
        entries.clear();
        let start = Instant::now();
        for entry in &case.entries {
            let bytes = black_box(ser(entry));
            total_size += bytes.len();
            entries.push(bytes);
        }
        se_duration += start.elapsed();
    }
    se_duration /= 10;

    let mut de_duration = Duration::ZERO;
    let mut new_entries = Vec::with_capacity(10000);

    for _ in 0..10 {
        new_entries.clear();
        let deser_start = Instant::now();
        for bytes in &entries {
            let entry: OplogEntry = black_box(deser(bytes));
            new_entries.push(entry);
        }
        de_duration += deser_start.elapsed();
    }
    de_duration /= 10;

    for (e1, e2) in new_entries.iter().zip(case.entries.iter()) {
        assert_eq!(e1, e2);
    }

    Report {
        name: name.to_string(),
        total_size,
        se_duration,
        de_duration,
    }
}

fn json_benchmark(case: &Case) -> Report {
    benchmark(
        case,
        "JSON",
        |entry| {
            let mut writer = Vec::with_capacity(128);
            writer.push(1u8);
            serde_json::to_writer(&mut writer, entry).unwrap();
            Bytes::from(writer)
        },
        |bytes| {
            let (_, data) = bytes.split_at(1);
            serde_json::from_slice(data).unwrap()
        },
    )
}

fn bincode_benchmark(case: &Case) -> Report {
    benchmark(
        case,
        "bincode",
        |entry| {
            let data = bincode::serde::encode_to_vec(entry, bincode::config::standard()).unwrap();
            let mut bytes = BytesMut::new();
            bytes.put_u8(1);
            bytes.extend_from_slice(&data);
            bytes.freeze()
        },
        |bytes| {
            let (_, data) = bytes.split_at(1);
            let (entry, _) =
                bincode::serde::decode_from_slice(data, bincode::config::standard()).unwrap();
            entry
        },
    )
}

fn bincode_noserde_benchmark(case: &Case) -> Report {
    benchmark(
        case,
        "bincode without serde (current implementation)",
        |entry| {
            let data = bincode::encode_to_vec(entry, bincode::config::standard()).unwrap();
            let mut bytes = BytesMut::new();
            bytes.put_u8(1);
            bytes.extend_from_slice(&data);
            bytes.freeze()
        },
        |bytes| {
            let (_, data) = bytes.split_at(1);
            let (entry, _) = bincode::decode_from_slice(data, bincode::config::standard()).unwrap();
            entry
        },
    )
}

fn messagepack_benchmark(case: &Case) -> Report {
    benchmark(
        case,
        "MessagePack (rmp-serde)",
        |entry| {
            let mut writer = Vec::with_capacity(128);
            writer.push(1u8);
            rmp_serde::encode::write(&mut writer, entry).unwrap();
            Bytes::from(writer)
        },
        |bytes| {
            let (_, data) = bytes.split_at(1);
            rmp_serde::from_slice(data).unwrap()
        },
    )
}

fn dlhn_benchmark(case: &Case) -> Report {
    benchmark(
        case,
        "DLHN",
        |entry| {
            let mut writer = Vec::with_capacity(128);
            writer.push(1u8);
            let mut ser = dlhn::Serializer::new(&mut writer);
            entry.serialize(&mut ser).unwrap();
            Bytes::from(writer)
        },
        |bytes| {
            let (_, data) = bytes.split_at(1);
            let mut reader = data;
            let mut de = dlhn::Deserializer::new(&mut reader);
            OplogEntry::deserialize(&mut de).unwrap()
        },
    )
}

fn postcard_benchmark(case: &Case) -> Report {
    benchmark(
        case,
        "postcard",
        |entry| {
            let mut writer = Vec::with_capacity(128);
            writer.push(1u8);
            postcard::to_io(entry, &mut writer).unwrap();
            Bytes::from(writer)
        },
        |bytes| {
            let (_, data) = bytes.split_at(1);
            postcard::from_bytes(data).unwrap()
        },
    )
}

fn bare_benchmark(case: &Case) -> Report {
    benchmark(
        case,
        "BARE",
        |entry| {
            let mut writer = Vec::with_capacity(128);
            writer.push(1u8);
            serde_bare::to_writer(&mut writer, entry).unwrap();
            Bytes::from(writer)
        },
        |bytes| {
            let (_, data) = bytes.split_at(1);
            serde_bare::from_slice(data).unwrap()
        },
    )
}

fn bitcode_benchmark(case: &Case) -> Report {
    benchmark(
        case,
        "bitcode",
        |entry| {
            let mut bytes = BytesMut::new();
            bytes.put_u8(1);
            let encoded = bitcode::serialize(&entry).unwrap();
            bytes.extend_from_slice(&encoded);
            bytes.freeze()
        },
        |bytes| {
            let (_, data) = bytes.split_at(1);
            bitcode::deserialize(data).unwrap()
        },
    )
}

fn desert_benchmark(case: &Case) -> Report {
    benchmark(
        case,
        "desert",
        |entry| {
            let encoded = serialize_to_byte_vec(entry).unwrap();
            let mut bytes = BytesMut::new();
            bytes.put_u8(1);
            bytes.extend_from_slice(&encoded);
            bytes.freeze()
        },
        |bytes| {
            let (_, data) = bytes.split_at(1);
            desert::deserialize(SliceInput::new(data)).unwrap()
        },
    )
}

fn print_report(report: &Report) {
    println!("{}", report.name);
    println!(" - total size:               {} bytes", report.total_size);
    println!(" - serialization duration:   {:?}", report.se_duration);
    println!(" - deserialization duration: {:?}", report.de_duration);
}

fn main() {
    let desert_only = std::env::args()
        .find(|arg| arg == "--desert-only")
        .is_some();

    println!("Generating data set");
    let mut rng = StdRng::seed_from_u64(317826381);

    let payload_sizes = [16, 256, 1024, 128 * 1024];
    let cases = payload_sizes
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
        .collect::<Vec<_>>();

    for case in cases {
        println!(
            "Benchmarking case with {} entries, payload size {} bytes",
            case.entries.len(),
            case.payload_size
        );

        // let desert_bin = desert::serialize_to_bytes(&case.entries).unwrap();
        // std::fs::write(Path::new(&format!("desert_bin_{}.bin", case.payload_size)), desert_bin).unwrap();

        let mut reports = vec![];
        if !desert_only {
            reports.push(json_benchmark(&case));
            reports.push(bincode_benchmark(&case));
            reports.push(bincode_noserde_benchmark(&case));
            reports.push(messagepack_benchmark(&case));
            // reports.push(dlhn_benchmark(&case));
            reports.push(postcard_benchmark(&case));
            reports.push(bare_benchmark(&case));
            reports.push(bitcode_benchmark(&case));
        }
        reports.push(desert_benchmark(&case));

        for report in reports {
            print_report(&report);
        }
    }
}
