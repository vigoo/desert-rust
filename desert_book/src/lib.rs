use std::collections::BTreeMap;
use std::fmt::Write;

use anyhow::{anyhow, bail, Context};
use bigdecimal::BigDecimal;
use bit_vec::BitVec;
use bytes::Bytes;
use chrono::{NaiveDate, NaiveTime};
use desert_rust::{
    serialize_to_byte_vec, BinaryCodec, BinaryOutput, BinarySerializer, DeduplicatedString,
    Options, SerializationContext,
};
use flate2::Compression;
use mac_address::MacAddress;
use nonempty_collections::NEVec;
use serde_json::{json, Value};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Example {
    pub id: &'static str,
    pub title: &'static str,
    pub snippet: &'static str,
    pub bytes: Vec<u8>,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub label: &'static str,
    pub len: usize,
    pub description: &'static str,
}

impl Segment {
    fn new(label: &'static str, len: usize, description: &'static str) -> Self {
        Self {
            label,
            len,
            description,
        }
    }
}

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
struct User {
    id: u32,
    name: String,
    email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
enum Event {
    Started,
    Message(String),
    Moved { x: i32, y: i32 },
}

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
#[desert(evolution())]
struct PointV1 {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
#[desert(evolution(FieldAdded("label", "origin".to_string())))]
struct PointV2 {
    x: i32,
    label: String,
    y: i32,
}

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
#[desert(evolution(
    FieldAdded("label", Some("origin".to_string())),
    FieldMadeOptional("label")
))]
struct PointV3 {
    x: i32,
    label: Option<String>,
    y: i32,
}

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
#[desert(evolution(
    FieldAdded("label", Some("origin".to_string())),
    FieldMadeOptional("label"),
    FieldRemoved("label")
))]
struct PointV4 {
    x: i32,
    y: i32,
}

pub fn preprocess_json(input: &str) -> anyhow::Result<String> {
    let mut value: Value = serde_json::from_str(input).context("failed to parse mdBook input")?;
    let book = value
        .as_array_mut()
        .and_then(|items| items.get_mut(1))
        .ok_or_else(|| anyhow!("expected mdBook preprocessor input array"))?;
    preprocess_book(book)?;
    serde_json::to_string(book).context("failed to serialize mdBook output")
}

pub fn replace_directives(markdown: &str) -> anyhow::Result<String> {
    let mut output = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(start) = rest.find("{{#desert-bytes") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start..];
        let Some(end) = after_start.find("}}") else {
            bail!("unterminated desert-bytes directive");
        };
        let directive = &after_start[..end + 2];
        let id = directive
            .trim_start_matches("{{#desert-bytes")
            .trim_end_matches("}}")
            .trim();
        if id.is_empty() {
            bail!("empty desert-bytes directive");
        }
        let example = example_by_id(id).ok_or_else(|| anyhow!("unknown desert-bytes id: {id}"))?;
        output.push_str(&render_example(&example)?);
        rest = &after_start[end + 2..];
    }
    output.push_str(rest);
    Ok(output)
}

pub fn all_examples() -> anyhow::Result<Vec<Example>> {
    Ok(vec![
        primitive_i32()?,
        primitive_u16()?,
        primitive_bool()?,
        primitive_unit()?,
        primitive_char()?,
        primitive_string()?,
        option_some()?,
        option_none()?,
        result_ok()?,
        result_err()?,
        bytes_vec_u8()?,
        bytes_bytes()?,
        collection_vec_i32()?,
        collection_btree_map()?,
        tuple_pair()?,
        derived_struct()?,
        derived_enum()?,
        string_dedup()?,
        feature_uuid()?,
        feature_chrono_date()?,
        feature_chrono_time()?,
        feature_bigdecimal()?,
        feature_bit_vec()?,
        feature_mac_address()?,
        feature_url()?,
        feature_serde_json()?,
        feature_nonempty_vec()?,
        io_var_u32()?,
        io_var_i32()?,
        io_compressed()?,
        io_iterable_unknown()?,
        evolution_point_v1()?,
        evolution_point_v2()?,
        evolution_point_v3()?,
        evolution_point_v4()?,
    ])
}

pub fn example_by_id(id: &str) -> Option<Example> {
    all_examples()
        .ok()
        .and_then(|examples| examples.into_iter().find(|example| example.id == id))
}

fn preprocess_book(value: &mut Value) -> anyhow::Result<()> {
    match value {
        Value::Object(object) => {
            if let Some(Value::Object(chapter)) = object.get_mut("Chapter") {
                if let Some(Value::String(content)) = chapter.get_mut("content") {
                    *content = replace_directives(content)?;
                }
                if let Some(items) = chapter.get_mut("sub_items") {
                    preprocess_book(items)?;
                }
            } else {
                for value in object.values_mut() {
                    preprocess_book(value)?;
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                preprocess_book(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn render_example(example: &Example) -> anyhow::Result<String> {
    verify_example(example)?;

    let mut output = String::new();
    writeln!(output, "### {}", escape_html(example.title))?;
    writeln!(output)?;
    writeln!(output, "```rust,ignore")?;
    writeln!(output, "{}", example.snippet.trim())?;
    writeln!(output, "```")?;
    writeln!(output)?;
    writeln!(output, "```text")?;
    writeln!(output, "{}", format_byte_array(&example.bytes))?;
    writeln!(output, "```")?;
    writeln!(output)?;
    writeln!(output, "<div class=\"desert-bytes\">")?;
    writeln!(output, "<table>")?;
    writeln!(output, "<tbody>")?;

    let mut offset = 0;
    for row in rows_for(example) {
        writeln!(output, "<tr class=\"desert-byte-row\">")?;
        for cell in &row {
            let color_class = segment_color_class(cell.segment_index);
            write!(
                output,
                "<td class=\"desert-byte-cell {color_class}\" title=\"{}\"><code>{:02X}</code></td>",
                escape_html(cell.description),
                example.bytes[cell.index]
            )?;
        }
        writeln!(output, "</tr>")?;
        writeln!(output, "<tr class=\"desert-segment-row\">")?;
        let mut consumed = 0;
        while consumed < row.len() {
            let cell = &row[consumed];
            let span = row[consumed..]
                .iter()
                .take_while(|candidate| candidate.segment_index == cell.segment_index)
                .count();
            let color_class = segment_color_class(cell.segment_index);
            write!(
                output,
                "<td class=\"desert-segment {color_class}\" colspan=\"{}\"><strong>{}</strong><span>{}</span></td>",
                span,
                escape_html(cell.label),
                escape_html(cell.description)
            )?;
            consumed += span;
        }
        writeln!(output, "</tr>")?;
        offset += row.len();
    }
    debug_assert_eq!(offset, example.bytes.len());

    writeln!(output, "</tbody>")?;
    writeln!(output, "</table>")?;
    writeln!(output, "</div>")?;
    Ok(output)
}

fn segment_color_class(segment_index: usize) -> String {
    const SEGMENT_COLOR_COUNT: usize = 8;

    format!(
        "desert-segment-color-{}",
        segment_index % SEGMENT_COLOR_COUNT
    )
}

fn verify_example(example: &Example) -> anyhow::Result<()> {
    let covered: usize = example.segments.iter().map(|segment| segment.len).sum();
    if covered != example.bytes.len() {
        bail!(
            "{} annotations cover {covered} bytes, but serialization produced {} bytes",
            example.id,
            example.bytes.len()
        );
    }
    Ok(())
}

#[derive(Debug)]
struct Cell<'a> {
    index: usize,
    segment_index: usize,
    label: &'a str,
    description: &'a str,
}

fn rows_for(example: &Example) -> Vec<Vec<Cell<'_>>> {
    const MAX_ROW_BYTES: usize = 16;

    let mut cells = Vec::with_capacity(example.bytes.len());
    let mut offset = 0;
    for (segment_index, segment) in example.segments.iter().enumerate() {
        for index in offset..offset + segment.len {
            cells.push(Cell {
                index,
                segment_index,
                label: segment.label,
                description: segment.description,
            });
        }
        offset += segment.len;
    }

    let mut rows = Vec::new();
    let mut current = Vec::new();
    for cell in cells {
        let would_cross_segment = current
            .last()
            .is_some_and(|last: &Cell<'_>| last.segment_index != cell.segment_index);
        if current.len() >= MAX_ROW_BYTES || (current.len() >= 8 && would_cross_segment) {
            rows.push(current);
            current = Vec::new();
        }
        current.push(cell);
    }
    if !current.is_empty() {
        rows.push(current);
    }
    rows
}

fn example<T: BinarySerializer>(
    id: &'static str,
    title: &'static str,
    snippet: &'static str,
    value: T,
    segments: Vec<Segment>,
) -> anyhow::Result<Example> {
    let bytes = serialize_to_byte_vec(&value)?;
    Ok(Example {
        id,
        title,
        snippet,
        bytes,
        segments,
    })
}

fn primitive_i32() -> anyhow::Result<Example> {
    example(
        "primitive.i32",
        "i32",
        "let bytes = desert_rust::serialize_to_byte_vec(&42i32)?;",
        42i32,
        vec![Segment::new(
            "i32",
            4,
            "fixed-width big-endian signed integer",
        )],
    )
}

fn primitive_u16() -> anyhow::Result<Example> {
    example(
        "primitive.u16",
        "u16",
        "let bytes = desert_rust::serialize_to_byte_vec(&1000u16)?;",
        1000u16,
        vec![Segment::new(
            "u16",
            2,
            "fixed-width big-endian unsigned integer",
        )],
    )
}

fn primitive_bool() -> anyhow::Result<Example> {
    example(
        "primitive.bool",
        "bool",
        "let bytes = desert_rust::serialize_to_byte_vec(&true)?;",
        true,
        vec![Segment::new("true", 1, "one byte: 1 for true, 0 for false")],
    )
}

fn primitive_unit() -> anyhow::Result<Example> {
    example(
        "primitive.unit",
        "unit",
        "let bytes = desert_rust::serialize_to_byte_vec(&())?;",
        (),
        vec![],
    )
}

fn primitive_char() -> anyhow::Result<Example> {
    example(
        "primitive.char",
        "char",
        "let bytes = desert_rust::serialize_to_byte_vec(&'λ')?;",
        'λ',
        vec![Segment::new(
            "code point",
            2,
            "Unicode scalar value written as var_u32",
        )],
    )
}

fn primitive_string() -> anyhow::Result<Example> {
    let text = "desert";
    example(
        "primitive.string",
        "String",
        r#"let bytes = desert_rust::serialize_to_byte_vec(&"desert".to_string())?;"#,
        text.to_string(),
        string_segments("length", "UTF-8", text),
    )
}

fn option_some() -> anyhow::Result<Example> {
    example(
        "option.some",
        "Option::Some",
        "let bytes = desert_rust::serialize_to_byte_vec(&Some(7i32))?;",
        Some(7i32),
        vec![
            Segment::new("Some", 1, "presence marker"),
            Segment::new("value", 4, "inner i32 payload"),
        ],
    )
}

fn option_none() -> anyhow::Result<Example> {
    example(
        "option.none",
        "Option::None",
        "let bytes = desert_rust::serialize_to_byte_vec(&Option::<i32>::None)?;",
        Option::<i32>::None,
        vec![Segment::new("None", 1, "absence marker")],
    )
}

fn result_ok() -> anyhow::Result<Example> {
    example(
        "result.ok",
        "Result::Ok",
        "let value: Result<i32, String> = Ok(7);\nlet bytes = desert_rust::serialize_to_byte_vec(&value)?;",
        Ok::<i32, String>(7),
        vec![
            Segment::new("Ok", 1, "result marker"),
            Segment::new("value", 4, "success payload"),
        ],
    )
}

fn result_err() -> anyhow::Result<Example> {
    let text = "no";
    let mut segments = vec![Segment::new("Err", 1, "result marker")];
    segments.extend(string_segments("length", "error", text));
    example(
        "result.err",
        "Result::Err",
        r#"let value: Result<i32, String> = Err("no".to_string());
let bytes = desert_rust::serialize_to_byte_vec(&value)?;"#,
        Err::<i32, String>(text.to_string()),
        segments,
    )
}

fn bytes_vec_u8() -> anyhow::Result<Example> {
    example(
        "bytes.vec_u8",
        "Vec<u8>",
        "let bytes = desert_rust::serialize_to_byte_vec(&vec![1u8, 2, 3, 4])?;",
        vec![1u8, 2, 3, 4],
        vec![
            Segment::new("length", 1, "var_u32 byte count"),
            Segment::new("bytes", 4, "raw byte payload"),
        ],
    )
}

fn bytes_bytes() -> anyhow::Result<Example> {
    example(
        "bytes.bytes",
        "bytes::Bytes",
        "let value = bytes::Bytes::from_static(b\"abc\");\nlet bytes = desert_rust::serialize_to_byte_vec(&value)?;",
        Bytes::from_static(b"abc"),
        vec![
            Segment::new("length", 1, "var_u32 byte count"),
            Segment::new("bytes", 3, "raw byte payload"),
        ],
    )
}

fn collection_vec_i32() -> anyhow::Result<Example> {
    example(
        "collection.vec_i32",
        "Vec<i32>",
        "let bytes = desert_rust::serialize_to_byte_vec(&vec![1i32, 2, 3])?;",
        vec![1i32, 2, 3],
        vec![
            Segment::new("count", 1, "exact-size iterable count as var_i32"),
            Segment::new("item 0", 4, "first i32"),
            Segment::new("item 1", 4, "second i32"),
            Segment::new("item 2", 4, "third i32"),
        ],
    )
}

fn collection_btree_map() -> anyhow::Result<Example> {
    let value = BTreeMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)]);
    example(
        "collection.btree_map",
        "BTreeMap<String, i32>",
        r#"let value = std::collections::BTreeMap::from([
    ("a".to_string(), 1i32),
    ("b".to_string(), 2i32),
]);
let bytes = desert_rust::serialize_to_byte_vec(&value)?;"#,
        value,
        vec![
            Segment::new("count", 1, "exact-size iterable count as var_i32"),
            Segment::new("entry", 1, "tuple marker for key/value pair"),
            Segment::new("key a", 2, "string key: length plus UTF-8"),
            Segment::new("value 1", 4, "i32 map value"),
            Segment::new("entry", 1, "tuple marker for key/value pair"),
            Segment::new("key b", 2, "string key: length plus UTF-8"),
            Segment::new("value 2", 4, "i32 map value"),
        ],
    )
}

fn tuple_pair() -> anyhow::Result<Example> {
    example(
        "tuple.pair",
        "(i32, bool)",
        "let bytes = desert_rust::serialize_to_byte_vec(&(42i32, true))?;",
        (42i32, true),
        vec![
            Segment::new(
                "version",
                1,
                "tuple payload marker compatible with version-0 structs",
            ),
            Segment::new("field 0", 4, "first tuple item"),
            Segment::new("field 1", 1, "second tuple item"),
        ],
    )
}

fn derived_struct() -> anyhow::Result<Example> {
    let text = "Ada";
    example(
        "derived.struct",
        "derived struct",
        r#"#[derive(Debug, Clone, PartialEq, desert_rust::BinaryCodec)]
struct User {
    id: u32,
    name: String,
    email: Option<String>,
}

let value = User {
    id: 7,
    name: "Ada".to_string(),
    email: None,
};
let bytes = desert_rust::serialize_to_byte_vec(&value)?;"#,
        User {
            id: 7,
            name: text.to_string(),
            email: None,
        },
        vec![
            Segment::new("version", 1, "version-0 struct marker"),
            Segment::new("id", 4, "u32 field"),
            Segment::new("name length", 1, "string byte count as var_i32"),
            Segment::new("name UTF-8", text.len(), "string bytes"),
            Segment::new("email", 1, "Option::None marker"),
        ],
    )
}

fn derived_enum() -> anyhow::Result<Example> {
    let text = "hi";
    example(
        "derived.enum",
        "derived enum",
        r#"#[derive(Debug, Clone, PartialEq, desert_rust::BinaryCodec)]
enum Event {
    Started,
    Message(String),
    Moved { x: i32, y: i32 },
}

let bytes = desert_rust::serialize_to_byte_vec(&Event::Message("hi".to_string()))?;"#,
        Event::Message(text.to_string()),
        {
            let mut segments = vec![
                Segment::new("version", 1, "outer enum version"),
                Segment::new("constructor", 1, "variant id as var_u32"),
                Segment::new("case version", 1, "variant payload version"),
            ];
            segments.extend(string_segments("length", "payload", text));
            segments
        },
    )
}

fn string_dedup() -> anyhow::Result<Example> {
    let mut context = SerializationContext::new(Vec::new(), Options::default());
    DeduplicatedString("same".to_string()).serialize(&mut context)?;
    DeduplicatedString("same".to_string()).serialize(&mut context)?;
    Ok(Example {
        id: "string.dedup",
        title: "DeduplicatedString",
        snippet: r#"let mut context = desert_rust::SerializationContext::new(
    Vec::new(),
    desert_rust::Options::default(),
);
desert_rust::DeduplicatedString("same".to_string()).serialize(&mut context)?;
desert_rust::DeduplicatedString("same".to_string()).serialize(&mut context)?;
let bytes = context.into_output();"#,
        bytes: context.into_output(),
        segments: vec![
            Segment::new(
                "first length",
                1,
                "first occurrence uses normal string length",
            ),
            Segment::new("first UTF-8", 4, "first string bytes"),
            Segment::new("repeat id", 1, "negative string id encoded as var_i32"),
        ],
    })
}

fn feature_uuid() -> anyhow::Result<Example> {
    example(
        "feature.uuid",
        "uuid",
        "let value = uuid::Uuid::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);\nlet bytes = desert_rust::serialize_to_byte_vec(&value)?;",
        Uuid::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
        vec![Segment::new("uuid", 16, "raw UUID bytes")],
    )
}

fn feature_chrono_date() -> anyhow::Result<Example> {
    example(
        "feature.chrono_date",
        "chrono::NaiveDate",
        "let value = chrono::NaiveDate::from_ymd_opt(2024, 6, 22).unwrap();\nlet bytes = desert_rust::serialize_to_byte_vec(&value)?;",
        NaiveDate::from_ymd_opt(2024, 6, 22).unwrap(),
        vec![
            Segment::new("year", 2, "year as var_u32"),
            Segment::new("month", 1, "month byte"),
            Segment::new("day", 1, "day byte"),
        ],
    )
}

fn feature_chrono_time() -> anyhow::Result<Example> {
    example(
        "feature.chrono_time",
        "chrono::NaiveTime",
        "let value = chrono::NaiveTime::from_hms_nano_opt(9, 30, 5, 125).unwrap();\nlet bytes = desert_rust::serialize_to_byte_vec(&value)?;",
        NaiveTime::from_hms_nano_opt(9, 30, 5, 125).unwrap(),
        vec![
            Segment::new("hour", 1, "hour byte"),
            Segment::new("minute", 1, "minute byte"),
            Segment::new("second", 1, "second byte"),
            Segment::new("nanos", 1, "nanosecond fraction as var_u32"),
        ],
    )
}

fn feature_bigdecimal() -> anyhow::Result<Example> {
    let text = "123.45";
    example(
        "feature.bigdecimal",
        "bigdecimal",
        r#"let value: bigdecimal::BigDecimal = "123.45".parse().unwrap();
let bytes = desert_rust::serialize_to_byte_vec(&value)?;"#,
        text.parse::<BigDecimal>()?,
        string_segments("length", "decimal", text),
    )
}

fn feature_bit_vec() -> anyhow::Result<Example> {
    example(
        "feature.bit_vec",
        "bit-vec",
        "let value = bit_vec::BitVec::from_bytes(&[0b1010_0000]);\nlet bytes = desert_rust::serialize_to_byte_vec(&value)?;",
        BitVec::from_bytes(&[0b1010_0000]),
        vec![
            Segment::new("length", 1, "byte count for packed bits"),
            Segment::new("bits", 1, "packed bit payload"),
        ],
    )
}

fn feature_mac_address() -> anyhow::Result<Example> {
    example(
        "feature.mac_address",
        "mac_address",
        "let value = mac_address::MacAddress::new([0, 17, 34, 51, 68, 85]);\nlet bytes = desert_rust::serialize_to_byte_vec(&value)?;",
        MacAddress::new([0, 17, 34, 51, 68, 85]),
        vec![Segment::new("mac", 6, "six raw address bytes")],
    )
}

fn feature_url() -> anyhow::Result<Example> {
    let text = "https://desert-rust.vigoo.dev/";
    example(
        "feature.url",
        "url",
        r#"let value = url::Url::parse("https://desert-rust.vigoo.dev/").unwrap();
let bytes = desert_rust::serialize_to_byte_vec(&value)?;"#,
        Url::parse(text)?,
        string_segments("length", "URL", text),
    )
}

fn feature_serde_json() -> anyhow::Result<Example> {
    let text = r#"{"a":1}"#;
    example(
        "feature.serde_json",
        "serde_json",
        r#"let value = serde_json::json!({ "a": 1 });
let bytes = desert_rust::serialize_to_byte_vec(&value)?;"#,
        json!({ "a": 1 }),
        vec![
            Segment::new("length", 1, "JSON byte count as var_u32"),
            Segment::new("JSON", text.len(), "compact JSON bytes"),
        ],
    )
}

fn feature_nonempty_vec() -> anyhow::Result<Example> {
    example(
        "feature.nonempty_vec",
        "nonempty-collections",
        "let value = nonempty_collections::NEVec::try_from_vec(vec![1u8, 2, 3]).unwrap();\nlet bytes = desert_rust::serialize_to_byte_vec(&value)?;",
        NEVec::try_from_vec(vec![1u8, 2, 3]).unwrap(),
        vec![
            Segment::new("length", 1, "non-empty byte count as var_u32"),
            Segment::new("bytes", 3, "raw byte payload"),
        ],
    )
}

fn io_var_u32() -> anyhow::Result<Example> {
    let mut bytes = Vec::new();
    bytes.write_var_u32(16_384);
    Ok(Example {
        id: "io.var_u32",
        title: "var_u32",
        snippet: "let mut bytes = Vec::new();\nbytes.write_var_u32(16_384);",
        bytes,
        segments: vec![Segment::new(
            "var_u32",
            3,
            "7-bit groups with continuation bits",
        )],
    })
}

fn io_var_i32() -> anyhow::Result<Example> {
    let mut bytes = Vec::new();
    bytes.write_var_i32(-64);
    Ok(Example {
        id: "io.var_i32",
        title: "var_i32",
        snippet: "let mut bytes = Vec::new();\nbytes.write_var_i32(-64);",
        bytes,
        segments: vec![Segment::new(
            "zig-zag",
            1,
            "signed value zig-zag encoded, then var_u32",
        )],
    })
}

fn io_compressed() -> anyhow::Result<Example> {
    let data = b"hello hello hello";
    let mut bytes = Vec::new();
    bytes.write_compressed(data, Compression::fast())?;
    let compressed_len = bytes.len() - 2;
    Ok(Example {
        id: "io.compressed",
        title: "compressed block",
        snippet: r#"let data = b"hello hello hello";
let mut bytes = Vec::new();
bytes.write_compressed(data, flate2::Compression::fast())?;"#,
        bytes,
        segments: vec![
            Segment::new("plain len", 1, "uncompressed byte count as var_u32"),
            Segment::new("deflate len", 1, "compressed byte count as var_u32"),
            Segment::new("deflate", compressed_len, "deflate payload"),
        ],
    })
}

fn io_iterable_unknown() -> anyhow::Result<Example> {
    let mut iter = [1i32, 2].into_iter().filter(|value| *value > 0);
    let mut context = SerializationContext::new(Vec::new(), Options::default());
    desert_rust::serialize_iterator(&mut iter, &mut context)?;
    let bytes = context.into_output();
    Ok(Example {
        id: "io.iterable_unknown",
        title: "unknown-size iterable",
        snippet: r#"let mut iter = [1i32, 2].into_iter().filter(|value| *value > 0);
desert_rust::serialize_iterator(
    &mut iter,
    &mut desert_rust::SerializationContext::new(&mut bytes, desert_rust::Options::default()),
)?;"#,
        bytes,
        segments: vec![
            Segment::new("unknown", 1, "-1 count marker as var_i32"),
            Segment::new("item", 1, "item-present marker"),
            Segment::new("value", 4, "first i32"),
            Segment::new("item", 1, "item-present marker"),
            Segment::new("value", 4, "second i32"),
            Segment::new("end", 1, "end marker"),
        ],
    })
}

fn evolution_point_v1() -> anyhow::Result<Example> {
    example(
        "evolution.point_v1",
        "PointV1",
        r#"#[derive(Debug, Clone, PartialEq, desert_rust::BinaryCodec)]
#[desert(evolution())]
struct PointV1 {
    x: i32,
    y: i32,
}

let bytes = desert_rust::serialize_to_byte_vec(&PointV1 { x: 10, y: 20 })?;"#,
        PointV1 { x: 10, y: 20 },
        vec![
            Segment::new("version", 1, "version 0"),
            Segment::new("x", 4, "first field"),
            Segment::new("y", 4, "second field"),
        ],
    )
}

fn evolution_point_v2() -> anyhow::Result<Example> {
    let label = "origin";
    example(
        "evolution.point_v2",
        "PointV2 adds label",
        r#"#[derive(Debug, Clone, PartialEq, desert_rust::BinaryCodec)]
#[desert(evolution(FieldAdded("label", "origin".to_string())))]
struct PointV2 {
    x: i32,
    label: String,
    y: i32,
}

let value = PointV2 { x: 10, label: "origin".to_string(), y: 20 };
let bytes = desert_rust::serialize_to_byte_vec(&value)?;"#,
        PointV2 {
            x: 10,
            label: label.to_string(),
            y: 20,
        },
        vec![
            Segment::new("version", 1, "version 1"),
            Segment::new("v0 size", 1, "byte length of original-field chunk"),
            Segment::new("v1 size", 1, "byte length of added-field chunk"),
            Segment::new("x", 4, "version-0 field"),
            Segment::new("y", 4, "version-0 field"),
            Segment::new("label length", 1, "added string length"),
            Segment::new("label UTF-8", label.len(), "added field data"),
        ],
    )
}

fn evolution_point_v3() -> anyhow::Result<Example> {
    example(
        "evolution.point_v3",
        "PointV3 makes label optional",
        r#"#[derive(Debug, Clone, PartialEq, desert_rust::BinaryCodec)]
#[desert(evolution(
    FieldAdded("label", "origin".to_string()),
    FieldMadeOptional("label")
))]
struct PointV3 {
    x: i32,
    label: Option<String>,
    y: i32,
}

let value = PointV3 { x: 10, label: None, y: 20 };
let bytes = desert_rust::serialize_to_byte_vec(&value)?;"#,
        PointV3 {
            x: 10,
            label: None,
            y: 20,
        },
        vec![
            Segment::new("version", 1, "version 2"),
            Segment::new("v0 size", 1, "byte length of original-field chunk"),
            Segment::new("v1 size", 1, "byte length of optional-field chunk"),
            Segment::new(
                "optional",
                2,
                "FieldMadeOptional marker plus field position",
            ),
            Segment::new("x", 4, "version-0 field"),
            Segment::new("y", 4, "version-0 field"),
            Segment::new("label", 1, "Option::None marker"),
        ],
    )
}

fn evolution_point_v4() -> anyhow::Result<Example> {
    let label = "label";
    example(
        "evolution.point_v4",
        "PointV4 removes label",
        r#"#[derive(Debug, Clone, PartialEq, desert_rust::BinaryCodec)]
#[desert(evolution(
    FieldAdded("label", "origin".to_string()),
    FieldMadeOptional("label"),
    FieldRemoved("label")
))]
struct PointV4 {
    x: i32,
    y: i32,
}

let bytes = desert_rust::serialize_to_byte_vec(&PointV4 { x: 10, y: 20 })?;"#,
        PointV4 { x: 10, y: 20 },
        vec![
            Segment::new("version", 1, "version 3"),
            Segment::new("v0 size", 1, "byte length of original-field chunk"),
            Segment::new(
                "v1 size",
                1,
                "removed field leaves no data in the added chunk",
            ),
            Segment::new("removed", 1, "FieldRemoved marker for optional step"),
            Segment::new("name length", 1, "removed field name length"),
            Segment::new("name UTF-8", label.len(), "removed field name"),
            Segment::new("removed", 1, "FieldRemoved marker"),
            Segment::new("name ref", 1, "deduplicated reference to field name"),
            Segment::new("x", 4, "version-0 field"),
            Segment::new("y", 4, "version-0 field"),
        ],
    )
}

fn string_segments(
    length_label: &'static str,
    payload_label: &'static str,
    text: &str,
) -> Vec<Segment> {
    let mut bytes = Vec::new();
    bytes.write_var_i32(text.len() as i32);
    vec![
        Segment::new(length_label, bytes.len(), "byte length encoded as var_i32"),
        Segment::new(payload_label, text.len(), "UTF-8 bytes"),
    ]
}

fn format_byte_array(bytes: &[u8]) -> String {
    let items = bytes
        .iter()
        .map(|byte| format!("0x{byte:02X}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{items}]")
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_annotations_cover_generated_bytes() {
        for example in all_examples().unwrap() {
            verify_example(&example).unwrap();
        }
    }

    #[test]
    fn directive_replaces_known_id() {
        let rendered =
            replace_directives("before\n{{#desert-bytes primitive.i32}}\nafter").unwrap();
        assert!(rendered.contains("before"));
        assert!(rendered.contains("let bytes = desert_rust::serialize_to_byte_vec(&42i32)?;"));
        assert!(rendered.contains("after"));
        assert!(!rendered.contains("{{#desert-bytes"));
    }

    #[test]
    fn directive_rejects_unknown_id() {
        let error = replace_directives("{{#desert-bytes missing.example}}").unwrap_err();
        assert!(error.to_string().contains("unknown desert-bytes id"));
    }

    #[test]
    fn preprocessor_updates_chapter_content() {
        let input = serde_json::json!([
            {},
            {
                "sections": [
                    {
                        "Chapter": {
                            "name": "Test",
                            "content": "{{#desert-bytes primitive.bool}}",
                            "sub_items": []
                        }
                    }
                ]
            }
        ]);

        let output = preprocess_json(&input.to_string()).unwrap();
        assert!(output.contains("one byte: 1 for true, 0 for false"));
    }
}
