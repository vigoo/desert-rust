// Model types copied from an early version of Golem Cloud to replicate an internal benchmark

use bincode::de::{BorrowDecoder, Decoder};
use bincode::enc::Encoder;
use bincode::error::{DecodeError, EncodeError};
use bincode::{Decode, Encode};
use desert_rust::deserializer::deserialize_iterator;
use desert_rust::{
    serialize_iterator, BinaryCodec, BinaryDeserializer, BinaryInput, BinaryOutput,
    BinarySerializer, DeserializationContext, SerializationContext,
};
use rand::distr::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize, Serializer};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Timestamp(pub iso8601_timestamp::Timestamp);

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            iso8601_timestamp::Timestamp::deserialize(deserializer).map(Self)
        } else {
            // For non-human-readable formats we assume it was an i64 representing milliseconds from epoch
            let timestamp: i64 = serde::Deserialize::deserialize(deserializer)?;
            Ok(Timestamp(
                iso8601_timestamp::Timestamp::UNIX_EPOCH
                    .add(Duration::from_millis(timestamp as u64)),
            ))
        }
    }
}

impl bincode::Encode for Timestamp {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (self
            .0
            .duration_since(iso8601_timestamp::Timestamp::UNIX_EPOCH)
            .whole_milliseconds() as u64)
            .encode(encoder)
    }
}

impl<Context> bincode::Decode<Context> for Timestamp {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let timestamp: u64 = bincode::Decode::decode(decoder)?;
        Ok(Timestamp(
            iso8601_timestamp::Timestamp::UNIX_EPOCH.add(Duration::from_millis(timestamp)),
        ))
    }
}

impl<'de, Context> bincode::BorrowDecode<'de, Context> for Timestamp {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let timestamp: u64 = bincode::BorrowDecode::borrow_decode(decoder)?;
        Ok(Timestamp(
            iso8601_timestamp::Timestamp::UNIX_EPOCH.add(Duration::from_millis(timestamp)),
        ))
    }
}

impl BinarySerializer for Timestamp {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> desert_rust::Result<()> {
        let ms = self
            .0
            .duration_since(iso8601_timestamp::Timestamp::UNIX_EPOCH)
            .whole_milliseconds() as u64;
        context.write_u64(ms);
        Ok(())
    }
}

impl BinaryDeserializer for Timestamp {
    fn deserialize(context: &mut DeserializationContext<'_>) -> desert_rust::Result<Self> {
        let ms = context.read_u64()?;
        Ok(Timestamp(
            iso8601_timestamp::Timestamp::UNIX_EPOCH.add(Duration::from_millis(ms)),
        ))
    }
}

#[derive(
    Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Encode, Decode, BinaryCodec,
)]
pub struct WorkerId {
    #[serde(rename = "component_id")]
    pub template_id: TemplateId,
    #[serde(rename = "instance_name")]
    pub worker_name: String,
}

#[derive(
    Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Encode, Decode, BinaryCodec,
)]
pub struct TemplateId {
    #[bincode(with_serde)]
    pub uuid: Uuid,
}

#[derive(
    Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Encode, Decode, BinaryCodec,
)]
pub struct PromiseId {
    #[serde(rename = "instance_id")]
    pub worker_id: WorkerId,
    pub oplog_idx: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode, BinaryCodec)]
#[desert(sorted_constructors)] // For being compatible with desert-scala
pub enum OplogEntry {
    ImportedFunctionInvoked {
        timestamp: Timestamp,
        function_name: String,
        response: Vec<u8>,
        side_effect: Vec<u8>,
        wrapped_function_type: WrappedFunctionType,
    },
    ExportedFunctionInvoked {
        timestamp: Timestamp,
        function_name: String,
        request: Vec<Value>,
        invocation_key: Option<InvocationKey>,
        calling_convention: Option<CallingConvention>,
    },
    ExportedFunctionCompleted {
        timestamp: Timestamp,
        response: Vec<Value>,
        consumed_fuel: i64,
    },
    CreatePromise {
        timestamp: Timestamp,
        promise_id: PromiseId,
    },
    CompletePromise {
        timestamp: Timestamp,
        promise_id: PromiseId,
        data: Vec<u8>,
    },
    Suspend {
        timestamp: Timestamp,
    },
    Error {
        timestamp: Timestamp,
    },
    Debug {
        timestamp: Timestamp,
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, BinaryCodec)]
#[desert(sorted_constructors)] // For being compatible with desert-scala
pub enum WrappedFunctionType {
    ReadLocal,
    WriteLocal,
    ReadRemote,
    WriteRemote,
}

#[derive(
    Clone, Debug, Serialize, Deserialize, Encode, Decode, Eq, Hash, PartialEq, BinaryCodec,
)]
pub struct InvocationKey {
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Encode, Decode, BinaryCodec)]
#[desert(sorted_constructors)] // For being compatible with desert-scala
pub enum CallingConvention {
    Component,
    Stdio,
    StdioEventloop,
}

fn random_wrapped_function_type(rng: &mut impl Rng) -> WrappedFunctionType {
    let case = rng.random_range(0..4);
    match case {
        0 => WrappedFunctionType::ReadLocal,
        1 => WrappedFunctionType::WriteLocal,
        2 => WrappedFunctionType::ReadRemote,
        3 => WrappedFunctionType::WriteRemote,
        _ => unreachable!(),
    }
}

fn random_invocation_key(rng: &mut impl Rng) -> Option<InvocationKey> {
    let some: bool = rng.random();
    if some {
        Some(InvocationKey {
            value: rng
                .sample_iter(Alphanumeric)
                .take(16)
                .map(char::from)
                .collect(),
        })
    } else {
        None
    }
}

fn random_calling_convention(rng: &mut impl Rng) -> Option<CallingConvention> {
    let case = rng.random_range(0..4);
    match case {
        0 => None,
        1 => Some(CallingConvention::Component),
        2 => Some(CallingConvention::Stdio),
        3 => Some(CallingConvention::StdioEventloop),
        _ => unreachable!(),
    }
}

fn random_timestamp(rng: &mut impl Rng) -> Timestamp {
    Timestamp(iso8601_timestamp::Timestamp::from(
        SystemTime::UNIX_EPOCH.add(Duration::from_secs(rng.random_range(0..3000000000))),
    ))
}

fn random_promise_id(rng: &mut impl Rng) -> PromiseId {
    PromiseId {
        worker_id: WorkerId {
            template_id: TemplateId {
                uuid: Uuid::new_v4(),
            },
            worker_name: rng
                .sample_iter(Alphanumeric)
                .take(8)
                .map(char::from)
                .collect(),
        },
        oplog_idx: rng.random(),
    }
}

pub fn random_oplog_entry(rng: &mut impl Rng, payload_size: usize) -> OplogEntry {
    let case = rng.random_range(0..7);
    match case {
        0 => {
            let mut response: Vec<u8> = vec![0; payload_size];
            let mut side_effect: Vec<u8> = vec![0; payload_size];

            rng.fill_bytes(&mut response);
            rng.fill_bytes(&mut side_effect);

            OplogEntry::ImportedFunctionInvoked {
                timestamp: random_timestamp(rng),
                function_name: rng
                    .sample_iter(Alphanumeric)
                    .take(16)
                    .map(char::from)
                    .collect(),
                response,
                side_effect,
                wrapped_function_type: random_wrapped_function_type(rng),
            }
        }
        1 => {
            let mut request: Vec<u8> = vec![0; payload_size];

            rng.fill_bytes(&mut request);

            OplogEntry::ExportedFunctionInvoked {
                timestamp: random_timestamp(rng),
                function_name: rng
                    .sample_iter(Alphanumeric)
                    .take(16)
                    .map(char::from)
                    .collect(),
                request: vec![Value::List(request.into_iter().map(Value::U8).collect())],
                invocation_key: random_invocation_key(rng),
                calling_convention: random_calling_convention(rng),
            }
        }
        2 => {
            let mut response: Vec<u8> = vec![0; payload_size];
            rng.fill_bytes(&mut response);

            OplogEntry::ExportedFunctionCompleted {
                timestamp: random_timestamp(rng),
                response: vec![Value::List(response.into_iter().map(Value::U8).collect())],
                consumed_fuel: rng.random(),
            }
        }
        3 => OplogEntry::CreatePromise {
            timestamp: random_timestamp(rng),
            promise_id: random_promise_id(rng),
        },
        4 => {
            let mut data: Vec<u8> = vec![0; payload_size];
            rng.fill_bytes(&mut data);

            OplogEntry::CompletePromise {
                timestamp: random_timestamp(rng),
                promise_id: random_promise_id(rng),
                data,
            }
        }
        5 => OplogEntry::Suspend {
            timestamp: random_timestamp(rng),
        },
        6 => OplogEntry::Error {
            timestamp: random_timestamp(rng),
        },
        _ => unreachable!(),
    }
}

pub struct Case {
    pub payload_size: usize,
    pub entries: Vec<OplogEntry>,
}

/// A tree representation of Value - isomorphic to the protobuf Val type but easier to work with in Rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode, BinaryCodec)]
pub enum Value {
    #[desert(transparent)]
    Bool(bool),
    #[desert(transparent)]
    U8(u8),
    #[desert(transparent)]
    U16(u16),
    #[desert(transparent)]
    U32(u32),
    #[desert(transparent)]
    U64(u64),
    #[desert(transparent)]
    S8(i8),
    #[desert(transparent)]
    S16(i16),
    #[desert(transparent)]
    S32(i32),
    #[desert(transparent)]
    S64(i64),
    #[desert(transparent)]
    F32(f32),
    #[desert(transparent)]
    F64(f64),
    #[desert(transparent)]
    Char(char),
    #[desert(transparent)]
    String(String),
    #[desert(transparent)]
    // NOTE: custom case disabled for fair comparison in benchmarks // #[desert(custom = VecValueWrapper)]
    List(Vec<Value>),
    #[desert(transparent)]
    Tuple(Vec<Value>),
    #[desert(transparent)]
    Record(Vec<Value>),
    Variant {
        case_idx: u32,
        case_value: Option<Box<Value>>,
    },
    #[desert(transparent)]
    Enum(u32),
    #[desert(transparent)]
    Flags(Vec<bool>),
    #[desert(transparent)]
    Option(Option<Box<Value>>),
    #[desert(transparent)]
    Result(Result<Option<Box<Value>>, Option<Box<Value>>>),
    Handle {
        uri: String,
        resource_id: u64,
    },
}

#[allow(dead_code)]
pub struct VecValueWrapper<'a>(pub Cow<'a, [Value]>);

impl<'a> BinarySerializer for VecValueWrapper<'a> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> desert_rust::Result<()> {
        let all_u8 = self.0.iter().all(|v| matches!(v, Value::U8(_)));
        if all_u8 {
            context.write_u8(1); // special case 1
            let bytes = self
                .0
                .iter()
                .map(|v| match v {
                    Value::U8(b) => *b,
                    _ => unreachable!(),
                })
                .collect::<Vec<u8>>();
            context.write_var_u32(bytes.len() as u32);
            context.write_bytes(&bytes);
            Ok(())
        } else {
            context.write_u8(0); // default case 0
            serialize_iterator(&mut self.0.iter(), context)
        }
    }
}

impl<'a> BinaryDeserializer for VecValueWrapper<'a> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> desert_rust::Result<Self> {
        let tag = context.read_u8()?;
        match tag {
            0 => {
                let (iter, maybe_size) = deserialize_iterator(context);
                let mut vec = Vec::with_capacity(maybe_size.unwrap_or_default());
                for item in iter {
                    vec.push(item?);
                }

                Ok(Self(Cow::Owned(vec)))
            }
            1 => {
                let length = context.read_var_u32()? as usize;
                let bytes = context.read_bytes(length)?;
                Ok(Self(Cow::Owned(
                    bytes.iter().map(|b| Value::U8(*b)).collect(),
                )))
            }
            other => Err(desert_rust::Error::DeserializationFailure(format!(
                "Invalid Vec<Value> tag: {}",
                other
            ))),
        }
    }
}
