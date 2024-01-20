use crate::StringId;
use std::array::TryFromSliceError;
use std::char::DecodeUtf16Error;
use std::fmt::{Display, Formatter};
use std::num::TryFromIntError;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    UnsupportedCharacter(char),
    FailedToDecodeCharacter(u16),
    LengthTooLarge,
    InvalidTimeZone(String),
    InputEndedUnexpectedly,
    CompressionFailure(String),
    DecompressionFailure(String),
    FailedToDecodeString(String),
    InvalidStringId(StringId),
    DeserializationFailure(String),
    UnknownFieldReferenceInEvolutionStep(String),
    InvalidConstructorName {
        constructor_name: String,
        type_name: String,
    },
    DeserializingNonExistingChunk(u8),
    FieldRemovedInSerializedVersion(String),
    FieldWithoutDefaultValueIsMissing(String),
    NonOptionalFieldSerializedAsNone(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnsupportedCharacter(c) => write!(f, "Unsupported character: {}", c),
            Error::FailedToDecodeCharacter(c) => write!(f, "Failed to decode character: {}", c),
            Error::LengthTooLarge => write!(f, "Length too large"),
            Error::InvalidTimeZone(msg) => write!(f, "Invalid timezone: {}", msg),
            Error::InputEndedUnexpectedly => write!(f, "Input ended unexpectedly"),
            Error::CompressionFailure(msg) => write!(f, "Compression failure: {}", msg),
            Error::DecompressionFailure(msg) => write!(f, "Decompression failure: {}", msg),
            Error::FailedToDecodeString(msg) => write!(f, "Failed to decode string: {}", msg),
            Error::InvalidStringId(id) => write!(f, "Invalid string id: {}", id),
            Error::DeserializationFailure(msg) => write!(f, "Deserialization failure: {}", msg),
            Error::UnknownFieldReferenceInEvolutionStep(msg) => {
                write!(f, "Unknown field reference in evolution step: {msg}")
            }
            Error::InvalidConstructorName {
                constructor_name,
                type_name,
            } => write!(
                f,
                "Invalid constructor name: {constructor_name} for type: {type_name}"
            ),
            Error::DeserializingNonExistingChunk(chunk_id) => {
                write!(f, "Deserializing non existing chunk: {chunk_id}")
            }
            Error::FieldRemovedInSerializedVersion(field_name) => {
                write!(f, "Field removed in serialized version: {field_name}")
            }
            Error::FieldWithoutDefaultValueIsMissing(field_name) => {
                write!(f, "Field without default value is missing: {field_name}")
            }
            Error::NonOptionalFieldSerializedAsNone(field_name) => {
                write!(f, "Non optional field serialized as None: {field_name}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<TryFromIntError> for Error {
    fn from(_: TryFromIntError) -> Self {
        Error::LengthTooLarge
    }
}

#[cfg(feature = "chrono")]
impl From<chrono_tz::ParseError> for Error {
    fn from(msg: chrono_tz::ParseError) -> Self {
        Error::InvalidTimeZone(msg.to_string())
    }
}

impl From<TryFromSliceError> for Error {
    fn from(_: TryFromSliceError) -> Self {
        Error::InputEndedUnexpectedly
    }
}

impl From<DecodeUtf16Error> for Error {
    fn from(err: DecodeUtf16Error) -> Self {
        Error::FailedToDecodeCharacter(err.unpaired_surrogate())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Error::FailedToDecodeString(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
