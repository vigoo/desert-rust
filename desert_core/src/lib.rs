pub mod adt;
mod binary_input;
mod binary_output;
mod deserializer;
mod error;
mod evolution;
mod features;
pub mod serializer;
mod state;

#[cfg(test)]
mod tests;

use bytes::{Bytes, BytesMut};
use std::fmt::{Display, Formatter};

pub use binary_input::{BinaryInput, OwnedInput, SliceInput};
pub use binary_output::{BinaryOutput, SizeCalculator};
pub use deserializer::{BinaryDeserializer, DeserializationContext};
pub use error::{Error, Result};
pub use evolution::Evolution;
pub use serializer::{serialize_iterator, BinarySerializer, SerializationContext};

#[cfg(test)]
test_r::enable!();

pub trait BinaryCodec: BinarySerializer + BinaryDeserializer {}

impl<T: BinarySerializer + BinaryDeserializer> BinaryCodec for T {}

const DEFAULT_CAPACITY: usize = 128;

pub fn serialize<T: BinarySerializer, O: BinaryOutput>(value: &T, output: O) -> Result<O> {
    let mut context = SerializationContext::new(output);
    value.serialize(&mut context)?;
    Ok(context.into_output())
}

pub fn deserialize<T: BinaryDeserializer>(input: &[u8]) -> Result<T> {
    let mut context = DeserializationContext::new(input);
    T::deserialize(&mut context)
}

pub fn serialize_to_bytes<T: BinarySerializer>(value: &T) -> Result<Bytes> {
    Ok(serialize(value, BytesMut::with_capacity(DEFAULT_CAPACITY))?.freeze())
}

pub fn serialize_to_byte_vec<T: BinarySerializer>(value: &T) -> Result<Vec<u8>> {
    serialize(value, Vec::with_capacity(DEFAULT_CAPACITY))
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
pub struct DeduplicatedString(pub String);

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
pub struct RefId(pub u32);

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

#[doc(hidden)]
pub use lazy_static::lazy_static;
