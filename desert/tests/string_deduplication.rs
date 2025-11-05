use crate::serialization_properties::compatibility_test;
use bytes::BytesMut;
use desert_rust::*;
use lazy_static::lazy_static;
use test_r::test;

test_r::enable!();

#[allow(dead_code)]
mod serialization_properties;

#[derive(Debug, PartialEq, BinaryCodec)]
struct DataV1;

#[derive(Debug, PartialEq, BinaryCodec)]
#[evolution(FieldAdded("new_field", "context".to_string()))]
struct DataV2 {
    new_field: String,
}

#[derive(Debug, PartialEq, BinaryCodec)]
struct OuterV1 {
    data: DataV1,
    other: String,
}

#[derive(Debug, PartialEq, BinaryCodec)]
struct OuterV2 {
    data: DataV2,
    other: String,
}

lazy_static! {
    static ref S1: String = "this is a test string".to_string();
    static ref S2: String = "and another one".to_string();
    static ref S3: String = "and another one".to_string();
}

fn test_dedup_ser<Output: BinaryOutput>(context: &mut SerializationContext<Output>) -> Result<()> {
    DeduplicatedString(S1.clone()).serialize(context)?;
    DeduplicatedString(S2.clone()).serialize(context)?;
    DeduplicatedString(S3.clone()).serialize(context)?;
    DeduplicatedString(S1.clone()).serialize(context)?;
    DeduplicatedString(S2.clone()).serialize(context)?;
    DeduplicatedString(S3.clone()).serialize(context)?;
    Ok(())
}

fn test_dedup_deser(
    context: &mut DeserializationContext,
) -> Result<(String, String, String, String, String, String)> {
    let s1 = DeduplicatedString::deserialize(context)?.0;
    let s2 = DeduplicatedString::deserialize(context)?.0;
    let s3 = DeduplicatedString::deserialize(context)?.0;
    let s4 = DeduplicatedString::deserialize(context)?.0;
    let s5 = DeduplicatedString::deserialize(context)?.0;
    let s6 = DeduplicatedString::deserialize(context)?.0;
    Ok((s1, s2, s3, s4, s5, s6))
}

fn test_non_dedup_ser<Output: BinaryOutput>(
    context: &mut SerializationContext<Output>,
) -> Result<()> {
    S1.serialize(context)?;
    S2.serialize(context)?;
    S3.serialize(context)?;
    S1.serialize(context)?;
    S2.serialize(context)?;
    S3.serialize(context)?;
    Ok(())
}

#[test]
fn reads_back_duplicated_strings_currently() {
    let mut context = SerializationContext::new(BytesMut::new());
    test_dedup_ser(&mut context).unwrap();
    let bytes = context.into_output();
    let mut context = DeserializationContext::new(&bytes);
    let (s1, s2, s3, s4, s5, s6) = test_dedup_deser(&mut context).unwrap();
    assert_eq!(s1, *S1);
    assert_eq!(s2, *S2);
    assert_eq!(s3, *S3);
    assert_eq!(s4, *S1);
    assert_eq!(s5, *S2);
    assert_eq!(s6, *S3);
}

#[test]
fn reduces_serialized_size() {
    let mut context = SerializationContext::new(BytesMut::new());
    test_dedup_ser(&mut context).unwrap();
    let bytes = context.into_output();
    let dedup_len = bytes.len();

    let mut context = SerializationContext::new(BytesMut::new());
    test_non_dedup_ser(&mut context).unwrap();
    let bytes = context.into_output();
    let non_dedup_len = bytes.len();

    assert!(dedup_len < non_dedup_len);
}

#[test]
fn default_string_serialization_does_not_break_data_evolution() {
    compatibility_test(
        OuterV2 {
            data: DataV2 {
                new_field: "hello world".to_string(),
            },
            other: "hello world".to_string(),
        },
        OuterV1 {
            data: DataV1,
            other: "hello world".to_string(),
        },
    );
}
