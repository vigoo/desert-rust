pub mod adt;
mod binary_input;
mod binary_output;
pub mod deserializer;
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
pub use deserializer::{deserialize_iterator, BinaryDeserializer, DeserializationContext};
pub use error::{Error, Result};
pub use evolution::Evolution;
pub use serializer::{serialize_iterator, BinarySerializer, SerializationContext};

#[cfg(test)]
test_r::enable!();

pub trait BinaryCodec: BinarySerializer + BinaryDeserializer {}

impl<T: BinarySerializer + BinaryDeserializer> BinaryCodec for T {}

const DEFAULT_CAPACITY: usize = 128;

pub fn serialize<T: BinarySerializer, O: BinaryOutput>(value: &T, output: O) -> Result<O> {
    serialize_with_options(value, output, Options::default())
}

pub fn serialize_with_options<T: BinarySerializer, O: BinaryOutput>(
    value: &T,
    output: O,
    options: Options,
) -> Result<O> {
    let mut context = SerializationContext::new(output, options);
    value.serialize(&mut context)?;
    Ok(context.into_output())
}

pub fn deserialize<T: BinaryDeserializer>(input: &[u8]) -> Result<T> {
    deserialize_with_options(input, Options::default())
}

pub fn deserialize_with_options<T: BinaryDeserializer>(
    input: &[u8],
    options: Options,
) -> Result<T> {
    let mut context = DeserializationContext::new(input, options);
    T::deserialize(&mut context)
}

pub fn serialize_to_bytes<T: BinarySerializer>(value: &T) -> Result<Bytes> {
    serialize_to_bytes_with_capacity(value, DEFAULT_CAPACITY)
}

pub fn serialize_to_bytes_with_options<T: BinarySerializer>(
    value: &T,
    options: Options,
) -> Result<Bytes> {
    serialize_to_bytes_with_options_and_capacity(value, options, DEFAULT_CAPACITY)
}

pub fn serialize_to_bytes_with_capacity<T: BinarySerializer>(
    value: &T,
    capacity: usize,
) -> Result<Bytes> {
    serialize_to_bytes_with_options_and_capacity(value, Options::default(), capacity)
}

pub fn serialize_to_bytes_with_options_and_capacity<T: BinarySerializer>(
    value: &T,
    options: Options,
    capacity: usize,
) -> Result<Bytes> {
    Ok(serialize_with_options(value, BytesMut::with_capacity(capacity), options)?.freeze())
}

pub fn serialize_to_byte_vec<T: BinarySerializer>(value: &T) -> Result<Vec<u8>> {
    serialize_to_byte_vec_with_capacity(value, DEFAULT_CAPACITY)
}

pub fn serialize_to_byte_vec_with_options<T: BinarySerializer>(
    value: &T,
    options: Options,
) -> Result<Vec<u8>> {
    serialize_to_byte_vec_with_options_and_capacity(value, options, DEFAULT_CAPACITY)
}

pub fn serialize_to_byte_vec_with_capacity<T: BinarySerializer>(
    value: &T,
    capacity: usize,
) -> Result<Vec<u8>> {
    serialize_to_byte_vec_with_options_and_capacity(value, Options::default(), capacity)
}

pub fn serialize_to_byte_vec_with_options_and_capacity<T: BinarySerializer>(
    value: &T,
    options: Options,
    capacity: usize,
) -> Result<Vec<u8>> {
    serialize_with_options(value, Vec::with_capacity(capacity), options)
}

pub fn serialize_into_byte_vec<T: BinarySerializer>(value: &T, output: &mut Vec<u8>) -> Result<()> {
    serialize_into_byte_vec_with_options(value, output, Options::default())
}

pub fn serialize_into_byte_vec_with_options<T: BinarySerializer>(
    value: &T,
    output: &mut Vec<u8>,
    options: Options,
) -> Result<()> {
    output.clear();
    let mut context = SerializationContext::new(output, options);
    value.serialize(&mut context)
}

pub fn serialized_size<T: BinarySerializer>(value: &T) -> Result<usize> {
    serialized_size_with_options(value, Options::default())
}

pub fn serialized_size_with_options<T: BinarySerializer>(
    value: &T,
    options: Options,
) -> Result<usize> {
    Ok(serialize_with_options(value, SizeCalculator::new(), options)?.size())
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

#[derive(Clone, Default)]
pub struct Options {
    /// The Scala version of desert represented characters as 16-bit Unicode characters. Enabling this
    /// flag makes desert-rust compatible with that encoding, but serialization will fail on characters
    /// not fitting into this encoding.
    pub chars_as_u16: bool,
}

impl Options {
    /// Settings for binary compatibility with the Scala version of desert
    pub fn scala_compatible() -> Self {
        Self { chars_as_u16: true }
    }
}

#[doc(hidden)]
pub use lazy_static::lazy_static;
