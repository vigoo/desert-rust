pub mod adt;
mod binary_input;
mod binary_output;
mod deserializer;
mod error;
mod evolution;
mod features;
mod serializer;
mod state;
mod storable;

use bytes::{Bytes, BytesMut};
use std::fmt::{Display, Formatter};

pub use binary_input::BinaryInput;
pub use binary_output::BinaryOutput;
pub use deserializer::{BinaryDeserializer, DeserializationContext};
pub use error::{Error, Result};
pub use evolution::Evolution;
pub use serializer::{serialize_iterator, BinarySerializer, SerializationContext};

pub trait BinaryCodec: BinarySerializer + BinaryDeserializer {}

pub fn serialize<T: BinarySerializer, O: BinaryOutput>(value: &T, output: O) -> Result<O> {
    let mut context = serializer::Serialization::new(output);
    value.serialize(&mut context)?;
    Ok(context.into_output())
}

pub fn deserialize<T: BinaryDeserializer, I: BinaryInput>(input: I) -> Result<T> {
    let mut context = deserializer::Deserialization::new(input);
    T::deserialize(&mut context)
}

pub fn serialize_to_bytes<T: BinarySerializer>(value: &T) -> Result<Bytes> {
    Ok(serialize(value, BytesMut::new())?.freeze())
}

/// Wrapper for strings, enabling desert's string deduplication mode.
///
/// The library have a simple deduplication system, without sacrificing any extra
/// bytes for cases when strings are not duplicate. In general, the strings are encoded by a variable length
/// int representing the length of the string in bytes, followed by its UTF-8 encoding.
/// When deduplication is enabled (the string values are wrapped in `DeduplicatedString`) , each serialized
/// string gets an ID and if it is serialized once more in the same stream, a negative number in place of the
/// length identifies it.
///
/// It is not turned on by default because it breaks backward compatibility when evolving data structures.
/// If a new string field is added, old versions of the application will skip it and would not assign the
/// same ID to the string if it is first seen.
pub struct DeduplicatedString(String);

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct StringId(pub i32);

impl StringId {
    fn next(&mut self) {
        self.0 += 1;
    }
}

impl Display for StringId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RefId(pub i32);

impl RefId {
    fn next(&mut self) {
        self.0 += 1;
    }
}

impl Display for RefId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{deserialize, serialize_to_bytes, BinaryDeserializer, BinarySerializer};
    use proptest::prelude::*;
    use std::collections::LinkedList;
    use std::fmt::Debug;

    pub(crate) fn roundtrip<T: BinarySerializer + BinaryDeserializer + Debug + PartialEq>(
        value: T,
    ) {
        let data = serialize_to_bytes(&value).unwrap();
        let result = deserialize::<T, _>(data).unwrap();
        assert_eq!(value, result);
    }

    fn is_supported_char(char: char) -> bool {
        let code = char as u32;
        let code: Result<u16, _> = code.try_into();
        code.is_ok()
    }

    proptest! {
        #[test]
        fn roundtrip_i8(value: i8) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_i16(value: i16) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_i32(value: i32) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_i64(value: i64) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_i128(value: i128) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_u8(value: u8) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_u16(value: u16) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_u32(value: u32) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_u64(value: u64) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_u128(value: u128) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_f32(value: f32) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_f64(value: f64) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_bool(value: bool) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_char(value in any::<char>().prop_filter("only chars that can be encoded in 16 bits", |c| is_supported_char(*c))) {
            // NOTE: we don't support arbitrary chars, just the ones that can be represented as u16, to keep binary compatibility with the Scala version
            roundtrip(value);
        }

        #[test]
        fn roundtrip_string(value: String) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_bytes(value: Vec<u8>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_option(value: Option<u32>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_vec(value: Vec<String>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple2(value: (u32, String)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple3(value: (u32, String, bool)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple4(value: (u32, String, bool, u64)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple5(value: (u32, String, bool, u64, i32)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple6(value: (u32, String, bool, u64, i32, i64)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple7(value: (u32, String, bool, u64, i32, i64, u128)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple8(value: (u32, String, bool, u64, i32, i64, u128, i128)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_sized_array(value: [u32; 3]) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_hashset(value: std::collections::HashSet<String>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_btreeset(value: std::collections::HashSet<u64>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_hashmap(value: std::collections::HashMap<String, u32>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_btreemap(value: std::collections::BTreeMap<String, u32>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_result(value: Result<u32, String>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_linked_list(value: LinkedList<String>) {
            roundtrip(value);
        }
    }
}
