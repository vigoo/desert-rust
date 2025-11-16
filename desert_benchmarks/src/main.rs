use std::hint::black_box;
use std::time::{Duration, Instant};

use rand::prelude::StdRng;
use rand::*;
use serde::{Deserialize, Serialize};

use desert_rust::serialize_to_byte_vec;
use model::OplogEntry;

use crate::model::{random_oplog_entry, Case};

mod model;

struct Report {
    name: String,
    total_size: usize,
    se_duration: Duration,
    de_duration: Duration,
}

fn benchmark<S: Fn(&OplogEntry) -> Vec<u8>, D: Fn(&[u8]) -> OplogEntry>(
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
            let data = serde_json::to_vec(entry).unwrap();
            let mut result = Vec::with_capacity(data.len() + 1);
            result.push(1u8);
            result.extend(data);
            result
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
            let mut result = Vec::with_capacity(data.len() + 1);
            result.push(1u8);
            result.extend(data);
            result
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
        "bincode without serde",
        |entry| {
            let data = bincode::encode_to_vec(entry, bincode::config::standard()).unwrap();
            let mut result = Vec::with_capacity(data.len() + 1);
            result.push(1u8);
            result.extend(data);
            result
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
            let data = rmp_serde::encode::to_vec(entry).unwrap();
            let mut result = Vec::with_capacity(data.len() + 1);
            result.push(1u8);
            result.extend(data);
            result
        },
        |bytes| {
            let (_, data) = bytes.split_at(1);
            rmp_serde::from_slice(data).unwrap()
        },
    )
}

#[allow(dead_code)]
fn dlhn_benchmark(case: &Case) -> Report {
    benchmark(
        case,
        "DLHN",
        |entry| {
            let mut writer = Vec::with_capacity(128);
            writer.push(1u8);
            let mut ser = dlhn::Serializer::new(&mut writer);
            entry.serialize(&mut ser).unwrap();
            writer
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
            writer
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
            writer
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
            let data = bitcode::serialize(&entry).unwrap();
            let mut result = Vec::with_capacity(data.len() + 1);
            result.push(1u8);
            result.extend(data);
            result
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
            let data = serialize_to_byte_vec(entry).unwrap();
            let mut result = Vec::with_capacity(data.len() + 1);
            result.push(1u8);
            result.extend(data);
            result
        },
        |bytes| {
            let (_, data) = bytes.split_at(1);
            desert_rust::deserialize(data).unwrap()
        },
    )
}

fn print_report(report: &Report) {
    if report.name.contains("desert") {
        println!("**{}**", report.name);
    } else {
        println!("{}", report.name);
    }
    println!("- total size:               {} bytes", report.total_size);
    println!("- serialization duration:   {:?}", report.se_duration);
    println!("- deserialization duration: {:?}", report.de_duration);
    println!();
}

fn main() {
    let desert_only = std::env::args().any(|arg| &arg == "--desert-only");

    println!("# Benchmark results without schema evolution");
    println!();
    println!("This benchmark serializes/deserializes 10000 \"oplog entries\", where oplog entries is a big enum type,");
    println!("taken from an early prototype of [Golem](https://github.com/golemcloud/golem). Some of the cases have");
    println!("an arbitrary dynamic 'Value' payload in them, which the benchmark sets to various sizes to see the effect on");
    println!("serialization speed.");
    println!();

    println!("Generating data set");
    let mut rng = StdRng::seed_from_u64(317826381);

    let payload_sizes = [16, 256, 1024, 16 * 1024];
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
            "## Benchmarking case with {} entries, payload size {} bytes",
            case.entries.len(),
            case.payload_size
        );
        println!();

        // let desert_bin = desert_rust::serialize_to_bytes(&case.entries).unwrap();
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
