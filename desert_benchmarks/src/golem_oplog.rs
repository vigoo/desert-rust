//! Benchmark-only Golem oplog shapes.
//!
//! These types intentionally mirror the serialization surface used by Golem's
//! oplog and payload model without depending on the Golem workspace.

use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use desert_rust::{
    deserialize, serialize, serialize_to_byte_vec, BinaryCodec, BinaryDeserializer, BinaryInput,
    BinaryOutput, BinarySerializer, DeserializationContext, Error, SerializationContext,
};
use uuid::Uuid;

use crate::golem_schema::{
    large_binary_typed_value, medium_typed_value, SchemaValue, TypedSchemaValue,
};

pub const SERIALIZATION_VERSION_V3: u8 = 3;
const DEFAULT_SERIALIZATION_CAPACITY: usize = 128;

pub fn serialize_with_version<T: BinarySerializer>(value: &T) -> desert_rust::Result<Vec<u8>> {
    let mut result = Vec::with_capacity(DEFAULT_SERIALIZATION_CAPACITY + 1);
    result.push(SERIALIZATION_VERSION_V3);
    serialize(value, result)
}

pub fn deserialize_with_version<T: BinaryDeserializer>(bytes: &[u8]) -> desert_rust::Result<T> {
    let (version, payload) = bytes.split_first().ok_or(Error::InputEndedUnexpectedly)?;
    if *version != SERIALIZATION_VERSION_V3 {
        return Err(Error::DeserializationFailure(format!(
            "unsupported serialization version: {version}"
        )));
    }
    deserialize(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versioned_serialization_prefixes_version_and_roundtrips() {
        let value = 42u32;
        let bytes = serialize_with_version(&value).unwrap();

        assert_eq!(bytes.first().copied(), Some(SERIALIZATION_VERSION_V3));
        assert_eq!(deserialize_with_version::<u32>(&bytes).unwrap(), value);
    }

    #[test]
    fn versioned_deserialization_rejects_unknown_version() {
        let bytes = [SERIALIZATION_VERSION_V3 + 1, 0];

        assert!(deserialize_with_version::<u32>(&bytes).is_err());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, BinaryCodec)]
#[desert(transparent)]
pub struct OplogIndex(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, BinaryCodec)]
#[desert(transparent)]
pub struct ComponentRevision(pub u64);

#[derive(Clone, Debug, PartialEq, Eq, Hash, BinaryCodec)]
#[desert(transparent)]
pub struct PayloadId(pub Uuid);

#[derive(Clone, Debug, PartialEq, Eq, Hash, BinaryCodec)]
#[desert(transparent)]
pub struct SpanId(pub Uuid);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, BinaryCodec)]
#[desert(evolution())]
pub struct Timestamp {
    pub millis: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, BinaryCodec)]
#[desert(evolution())]
pub struct AgentId {
    pub component_id: Uuid,
    pub agent_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, BinaryCodec)]
#[desert(evolution())]
pub struct PromiseId {
    pub agent_id: AgentId,
    pub oplog_idx: OplogIndex,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub enum AttributeValue {
    String(String),
    U64(u64),
    Bool(bool),
    Json(serde_json::Value),
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub enum SpanData {
    LocalSpan {
        span_id: SpanId,
        start: Timestamp,
        parent_id: Option<SpanId>,
        attributes: HashMap<String, AttributeValue>,
        inherited: bool,
    },
    ExternalSpan {
        span_id: SpanId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum DurableFunctionType {
    ReadLocal,
    WriteLocal,
    ReadRemote,
    WriteRemote,
    WriteRemoteBatched(Option<OplogIndex>),
    WriteRemoteTransaction(Option<OplogIndex>),
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum AgentError {
    Unknown(String),
    InvalidRequest(String),
    Runtime(String),
    Interrupted,
    ShardAssignmentChanged,
    PreviousInvocationFailed(String),
    PreviousInvocationExited,
    ReadOnlyViolation {
        method: String,
        host_function: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum OplogPayload<T> {
    Inline(Box<T>),
    SerializedInline {
        bytes: Vec<u8>,
        cached: Option<Arc<T>>,
    },
    External {
        payload_id: PayloadId,
        md5_hash: Vec<u8>,
        cached: Option<Arc<T>>,
    },
}

impl<T: BinarySerializer> BinarySerializer for OplogPayload<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> desert_rust::Result<()> {
        match self {
            OplogPayload::Inline(value) => {
                context.write_u8(0);
                let bytes = serialize_with_version(value.as_ref())?;
                bytes.serialize(context)
            }
            OplogPayload::SerializedInline { bytes, .. } => {
                context.write_u8(0);
                bytes.serialize(context)
            }
            OplogPayload::External {
                payload_id,
                md5_hash,
                ..
            } => {
                context.write_u8(1);
                payload_id.serialize(context)?;
                md5_hash.serialize(context)
            }
        }
    }
}

impl<T: BinaryDeserializer> BinaryDeserializer for OplogPayload<T> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> desert_rust::Result<Self> {
        match context.read_u8()? {
            0 => Ok(OplogPayload::SerializedInline {
                bytes: Vec::<u8>::deserialize(context)?,
                cached: None,
            }),
            1 => Ok(OplogPayload::External {
                payload_id: PayloadId::deserialize(context)?,
                md5_hash: Vec::<u8>::deserialize(context)?,
                cached: None,
            }),
            tag => Err(Error::DeserializationFailure(format!(
                "unknown oplog payload tag: {tag}"
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum SerializableHttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct SerializableResponseHeaders {
    pub status: u16,
    pub headers: HashMap<String, Vec<Vec<u8>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum SerializableHttpErrorCode {
    Timeout,
    Dns(String),
    InvalidUrl(String),
    ConnectionRefused,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum SerializableHttpResponse {
    Pending,
    HeadersReceived(SerializableResponseHeaders),
    HttpError(SerializableHttpErrorCode),
    InternalError(Option<String>),
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub enum SerializableDbValue {
    Null,
    Boolean(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
    Numeric(BigDecimal),
    Date(NaiveDate),
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct SerializableDbColumn {
    pub name: String,
    pub db_type: String,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub struct SerializableDbResult {
    pub columns: Vec<SerializableDbColumn>,
    pub rows: Vec<Vec<SerializableDbValue>>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub enum SerializableRdbmsRequest {
    Query {
        statement: String,
        params: Vec<SerializableDbValue>,
    },
    Execute {
        statement: String,
    },
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub enum SerializableRdbmsResponse {
    Result(SerializableDbResult),
    Error(String),
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub struct HostRequestHttpRequest {
    pub uri: String,
    pub method: SerializableHttpMethod,
    pub headers: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub struct HostRequestRdbmsRequest {
    pub connection: String,
    pub request: SerializableRdbmsRequest,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub struct HostRequestRpcInvoke {
    pub agent_id: AgentId,
    pub method: String,
    pub input: TypedSchemaValue,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct HostRequestGetConfigValue {
    pub key: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct HostRequestWebsocketSend {
    pub connection_id: Uuid,
    pub message: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
#[allow(clippy::large_enum_variant)]
pub enum HostRequest {
    Custom(TypedSchemaValue),
    HttpRequest(HostRequestHttpRequest),
    RdbmsRequest(HostRequestRdbmsRequest),
    RpcInvoke(HostRequestRpcInvoke),
    GetConfigValue(HostRequestGetConfigValue),
    WebsocketSend(HostRequestWebsocketSend),
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct HostResponseHttpResponse {
    pub response: SerializableHttpResponse,
    pub body: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub struct HostResponseRdbmsResponse {
    pub response: SerializableRdbmsResponse,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub struct HostResponseRpcInvoke {
    pub result: Result<TypedSchemaValue, AgentError>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
#[allow(clippy::large_enum_variant)]
pub enum HostResponse {
    Custom(TypedSchemaValue),
    HttpResponse(HostResponseHttpResponse),
    RdbmsResponse(HostResponseRdbmsResponse),
    RpcInvokeResult(HostResponseRpcInvoke),
    ConfigValue(Option<SchemaValue>),
    WebsocketMessage(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
#[allow(clippy::large_enum_variant)]
pub enum GolemOplogEntry {
    Create {
        timestamp: Timestamp,
        agent_id: AgentId,
        component_revision: ComponentRevision,
        args: OplogPayload<TypedSchemaValue>,
        env: HashMap<String, String>,
    },
    ImportedFunctionInvoked {
        timestamp: Timestamp,
        function_name: String,
        request: OplogPayload<HostRequest>,
        response: OplogPayload<HostResponse>,
        function_type: DurableFunctionType,
    },
    ExportedFunctionInvoked {
        timestamp: Timestamp,
        idempotency_key: String,
        method_name: String,
        input: OplogPayload<TypedSchemaValue>,
        trace: Vec<SpanData>,
    },
    ExportedFunctionCompleted {
        timestamp: Timestamp,
        response: OplogPayload<SchemaValue>,
        consumed_fuel: i64,
    },
    PendingUpdate {
        timestamp: Timestamp,
        target_revision: ComponentRevision,
        payload: OplogPayload<Vec<u8>>,
        mime_type: String,
    },
    Error {
        timestamp: Timestamp,
        error: AgentError,
    },
    CompletePromise {
        timestamp: Timestamp,
        promise_id: PromiseId,
        data: OplogPayload<Vec<u8>>,
    },
    NoOp {
        timestamp: Timestamp,
    },
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub struct CompressedOplogChunk {
    pub entry_count: u64,
    pub first_oplog_index: OplogIndex,
    pub compressed_data: Vec<u8>,
}

pub fn golem_oplog_entry_cases() -> Vec<(&'static str, GolemOplogEntry)> {
    vec![
        ("create", create_entry(0)),
        ("imported_http", imported_http_entry(1)),
        ("imported_rdbms", imported_rdbms_entry(2)),
        ("exported_invoked", exported_invoked_entry(3)),
        ("exported_completed", exported_completed_entry(4)),
        ("pending_update", pending_update_entry(5)),
        ("error", error_entry(6)),
        ("complete_promise", complete_promise_entry(7)),
    ]
}

pub fn golem_oplog_batch(entry_count: usize) -> Vec<GolemOplogEntry> {
    (0..entry_count)
        .map(|idx| match idx % 8 {
            0 => create_entry(idx),
            1 => imported_http_entry(idx),
            2 => imported_rdbms_entry(idx),
            3 => exported_invoked_entry(idx),
            4 => exported_completed_entry(idx),
            5 => pending_update_entry(idx),
            6 => error_entry(idx),
            _ => complete_promise_entry(idx),
        })
        .collect()
}

pub fn golem_oplog_chunk(entry_count: usize) -> CompressedOplogChunk {
    let entries = golem_oplog_batch(entry_count);
    let compressed_data = serialize_to_byte_vec(&entries).unwrap();
    CompressedOplogChunk {
        entry_count: entries.len() as u64,
        first_oplog_index: OplogIndex(1_000_000),
        compressed_data,
    }
}

pub fn oplog_payload_cases() -> Vec<(&'static str, OplogPayload<HostRequest>)> {
    let request = HostRequest::HttpRequest(http_request(42));
    vec![
        ("inline", OplogPayload::Inline(Box::new(request.clone()))),
        (
            "serialized_inline",
            OplogPayload::SerializedInline {
                bytes: serialize_with_version(&request).unwrap(),
                cached: None,
            },
        ),
        (
            "external",
            OplogPayload::External {
                payload_id: PayloadId(uuid(9_001)),
                md5_hash: bytes(16, 0x42),
                cached: None,
            },
        ),
    ]
}

fn create_entry(idx: usize) -> GolemOplogEntry {
    GolemOplogEntry::Create {
        timestamp: timestamp(idx),
        agent_id: agent_id(idx),
        component_revision: ComponentRevision(17),
        args: inline(medium_typed_value()),
        env: HashMap::from([
            ("GOLEM_AGENT".to_string(), format!("bench-agent-{idx}")),
            ("REGION".to_string(), "local".to_string()),
        ]),
    }
}

fn imported_http_entry(idx: usize) -> GolemOplogEntry {
    GolemOplogEntry::ImportedFunctionInvoked {
        timestamp: timestamp(idx),
        function_name: "wasi:http/outgoing-handler.handle".to_string(),
        request: serialized_inline(HostRequest::HttpRequest(http_request(idx))),
        response: serialized_inline(HostResponse::HttpResponse(http_response(idx))),
        function_type: DurableFunctionType::WriteRemoteBatched(Some(OplogIndex(idx as u64))),
    }
}

fn imported_rdbms_entry(idx: usize) -> GolemOplogEntry {
    GolemOplogEntry::ImportedFunctionInvoked {
        timestamp: timestamp(idx),
        function_name: "golem:rdbms/query".to_string(),
        request: serialized_inline(HostRequest::RdbmsRequest(rdbms_request(idx))),
        response: serialized_inline(HostResponse::RdbmsResponse(rdbms_response())),
        function_type: DurableFunctionType::WriteRemoteTransaction(Some(OplogIndex(idx as u64))),
    }
}

fn exported_invoked_entry(idx: usize) -> GolemOplogEntry {
    GolemOplogEntry::ExportedFunctionInvoked {
        timestamp: timestamp(idx),
        idempotency_key: format!("idem-{idx:08}"),
        method_name: "do-work".to_string(),
        input: serialized_inline(large_binary_typed_value()),
        trace: span_data(idx),
    }
}

fn exported_completed_entry(idx: usize) -> GolemOplogEntry {
    GolemOplogEntry::ExportedFunctionCompleted {
        timestamp: timestamp(idx),
        response: serialized_inline(SchemaValue::Record {
            fields: vec![
                SchemaValue::U64(idx as u64),
                SchemaValue::String("completed".to_string()),
            ],
        }),
        consumed_fuel: 100_000 + idx as i64,
    }
}

fn pending_update_entry(idx: usize) -> GolemOplogEntry {
    GolemOplogEntry::PendingUpdate {
        timestamp: timestamp(idx),
        target_revision: ComponentRevision(20),
        payload: external_payload(64 * 1024, idx),
        mime_type: "application/wasm".to_string(),
    }
}

fn error_entry(idx: usize) -> GolemOplogEntry {
    GolemOplogEntry::Error {
        timestamp: timestamp(idx),
        error: AgentError::ReadOnlyViolation {
            method: "GET".to_string(),
            host_function: "wasi:http/outgoing-handler.handle".to_string(),
        },
    }
}

fn complete_promise_entry(idx: usize) -> GolemOplogEntry {
    GolemOplogEntry::CompletePromise {
        timestamp: timestamp(idx),
        promise_id: PromiseId {
            agent_id: agent_id(idx),
            oplog_idx: OplogIndex(idx as u64),
        },
        data: serialized_inline(bytes(4096, idx as u8)),
    }
}

fn http_request(idx: usize) -> HostRequestHttpRequest {
    HostRequestHttpRequest {
        uri: format!("https://api.example.com/v1/items/{idx}?include=details"),
        method: SerializableHttpMethod::Post,
        headers: HashMap::from([
            ("content-type".to_string(), "application/json".to_string()),
            ("x-request-id".to_string(), format!("request-{idx:08}")),
        ]),
    }
}

fn http_response(idx: usize) -> HostResponseHttpResponse {
    HostResponseHttpResponse {
        response: SerializableHttpResponse::HeadersReceived(SerializableResponseHeaders {
            status: 200,
            headers: HashMap::from([
                (
                    "content-type".to_string(),
                    vec![b"application/json".to_vec()],
                ),
                ("etag".to_string(), vec![format!("etag-{idx}").into_bytes()]),
            ]),
        }),
        body: Some(bytes(8192, idx as u8)),
    }
}

fn rdbms_request(idx: usize) -> HostRequestRdbmsRequest {
    HostRequestRdbmsRequest {
        connection: "postgres://bench/local".to_string(),
        request: SerializableRdbmsRequest::Query {
            statement: "select id, amount, payload from ledger where agent_id = $1 and idx < $2"
                .to_string(),
            params: vec![
                SerializableDbValue::Text(format!("agent-{idx}")),
                SerializableDbValue::Int(idx as i64 + 100),
            ],
        },
    }
}

fn rdbms_response() -> HostResponseRdbmsResponse {
    HostResponseRdbmsResponse {
        response: SerializableRdbmsResponse::Result(SerializableDbResult {
            columns: vec![
                SerializableDbColumn {
                    name: "id".to_string(),
                    db_type: "uuid".to_string(),
                },
                SerializableDbColumn {
                    name: "amount".to_string(),
                    db_type: "numeric".to_string(),
                },
                SerializableDbColumn {
                    name: "payload".to_string(),
                    db_type: "jsonb".to_string(),
                },
            ],
            rows: (0..24)
                .map(|idx| {
                    vec![
                        SerializableDbValue::Text(uuid(idx).to_string()),
                        SerializableDbValue::Numeric(BigDecimal::from_str("123456.7891").unwrap()),
                        SerializableDbValue::Json(serde_json::json!({
                            "idx": idx,
                            "status": "posted",
                            "labels": ["bench", "oplog", "rdbms"]
                        })),
                    ]
                })
                .collect(),
        }),
    }
}

fn span_data(idx: usize) -> Vec<SpanData> {
    vec![
        SpanData::ExternalSpan {
            span_id: SpanId(uuid(idx + 3_000)),
        },
        SpanData::LocalSpan {
            span_id: SpanId(uuid(idx + 3_001)),
            start: timestamp(idx),
            parent_id: Some(SpanId(uuid(idx + 3_000))),
            attributes: HashMap::from([
                (
                    "method".to_string(),
                    AttributeValue::String("do-work".to_string()),
                ),
                ("retry".to_string(), AttributeValue::Bool(false)),
                ("items".to_string(), AttributeValue::U64(64)),
                (
                    "context".to_string(),
                    AttributeValue::Json(serde_json::json!({
                        "tenant": "benchmark",
                        "priority": "normal"
                    })),
                ),
            ]),
            inherited: true,
        },
    ]
}

fn inline<T>(value: T) -> OplogPayload<T> {
    OplogPayload::Inline(Box::new(value))
}

fn serialized_inline<T: BinarySerializer>(value: T) -> OplogPayload<T> {
    OplogPayload::SerializedInline {
        bytes: serialize_with_version(&value).unwrap(),
        cached: None,
    }
}

fn external_payload(size: usize, seed: usize) -> OplogPayload<Vec<u8>> {
    OplogPayload::External {
        payload_id: PayloadId(uuid(seed + 10_000)),
        md5_hash: bytes(16, seed as u8),
        cached: Some(Arc::new(bytes(size, seed as u8))),
    }
}

fn agent_id(idx: usize) -> AgentId {
    AgentId {
        component_id: uuid(idx + 100),
        agent_name: format!("agent-{idx:08}"),
    }
}

fn timestamp(idx: usize) -> Timestamp {
    Timestamp {
        millis: 1_700_000_000_000 + idx as u64 * 37,
    }
}

fn uuid(idx: usize) -> Uuid {
    Uuid::from_u128(0x1234_5678_90ab_cdef_0000_0000_0000_0000u128 + idx as u128)
}

fn bytes(size: usize, seed: u8) -> Vec<u8> {
    (0..size)
        .map(|idx| seed.wrapping_add((idx % 251) as u8))
        .collect()
}
