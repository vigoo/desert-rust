use bytes::BytesMut;
use desert_core::serializer::Serialization;
use desert_core::{BinarySerializer, DeduplicatedString, Result, SerializationContext};
use desert_macro::BinaryCodec;
use lazy_static::lazy_static;

mod desert {
    pub use desert_core::*;
}

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

fn test_dedup_ser<Context: SerializationContext>(context: &mut Context) -> Result<()> {
    DeduplicatedString(S1.clone()).serialize(context)?;
    DeduplicatedString(S2.clone()).serialize(context)?;
    DeduplicatedString(S3.clone()).serialize(context)?;
    DeduplicatedString(S1.clone()).serialize(context)?;
    DeduplicatedString(S2.clone()).serialize(context)?;
    DeduplicatedString(S3.clone()).serialize(context)?;
    Ok(())
}

fn test_non_dedup_ser<Context: SerializationContext>(context: &mut Context) -> Result<()> {
    S1.serialize(context)?;
    S2.serialize(context)?;
    S3.serialize(context)?;
    S1.serialize(context)?;
    S2.serialize(context)?;
    S3.serialize(context)?;
    Ok(())
}

#[test]
fn reduces_serialized_size() {
    let mut context = Serialization::new(BytesMut::new());
    test_dedup_ser(&mut context).unwrap();
    let bytes = context.into_output();
    let dedup_len = bytes.len();

    let mut context = Serialization::new(BytesMut::new());
    test_non_dedup_ser(&mut context).unwrap();
    let bytes = context.into_output();
    let non_dedup_len = bytes.len();

    assert!(dedup_len < non_dedup_len);
}
