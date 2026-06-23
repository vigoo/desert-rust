//! Benchmark-only copy of the Golem schema/value carrier shape.
//!
//! This mirrors the `value-type-refactoring-5` branch enough to benchmark
//! Desert binary serialization costs without adding Golem as a dependency.

use chrono::{DateTime, TimeZone, Utc};
use desert_rust::BinaryCodec;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, BinaryCodec)]
#[desert(transparent)]
pub struct TypeId(pub String);

impl TypeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct MetadataEnvelope {
    pub doc: Option<String>,
    pub aliases: Vec<String>,
    pub examples: Vec<String>,
    pub deprecated: Option<String>,
    pub role: Option<Role>,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum Role {
    Multimodal,
    UnstructuredText,
    UnstructuredBinary,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct SchemaGraph {
    pub defs: Vec<SchemaTypeDef>,
    pub root: SchemaType,
}

impl SchemaGraph {
    pub fn anonymous(root: SchemaType) -> Self {
        Self {
            defs: Vec::new(),
            root,
        }
    }

    pub fn empty() -> Self {
        Self::anonymous(SchemaType::record(Vec::new()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct SchemaTypeDef {
    pub id: TypeId,
    pub name: Option<String>,
    pub body: SchemaType,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub struct TypedSchemaValue {
    pub graph: SchemaGraph,
    pub value: SchemaValue,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum SchemaType {
    Ref {
        id: TypeId,
        metadata: MetadataEnvelope,
    },
    Bool {
        metadata: MetadataEnvelope,
    },
    S8 {
        metadata: MetadataEnvelope,
    },
    S16 {
        metadata: MetadataEnvelope,
    },
    S32 {
        metadata: MetadataEnvelope,
    },
    S64 {
        metadata: MetadataEnvelope,
    },
    U8 {
        metadata: MetadataEnvelope,
    },
    U16 {
        metadata: MetadataEnvelope,
    },
    U32 {
        metadata: MetadataEnvelope,
    },
    U64 {
        metadata: MetadataEnvelope,
    },
    F32 {
        metadata: MetadataEnvelope,
    },
    F64 {
        metadata: MetadataEnvelope,
    },
    Char {
        metadata: MetadataEnvelope,
    },
    String {
        metadata: MetadataEnvelope,
    },
    Record {
        fields: Vec<NamedFieldType>,
        metadata: MetadataEnvelope,
    },
    Variant {
        cases: Vec<VariantCaseType>,
        metadata: MetadataEnvelope,
    },
    Enum {
        cases: Vec<String>,
        metadata: MetadataEnvelope,
    },
    Flags {
        flags: Vec<String>,
        metadata: MetadataEnvelope,
    },
    Tuple {
        elements: Vec<SchemaType>,
        metadata: MetadataEnvelope,
    },
    List {
        element: Box<SchemaType>,
        metadata: MetadataEnvelope,
    },
    FixedList {
        element: Box<SchemaType>,
        length: u32,
        metadata: MetadataEnvelope,
    },
    Map {
        key: Box<SchemaType>,
        value: Box<SchemaType>,
        metadata: MetadataEnvelope,
    },
    Option {
        inner: Box<SchemaType>,
        metadata: MetadataEnvelope,
    },
    Result {
        spec: ResultSpec,
        metadata: MetadataEnvelope,
    },
    Text {
        restrictions: TextRestrictions,
        metadata: MetadataEnvelope,
    },
    Binary {
        restrictions: BinaryRestrictions,
        metadata: MetadataEnvelope,
    },
    Path {
        spec: PathSpec,
        metadata: MetadataEnvelope,
    },
    Url {
        restrictions: UrlRestrictions,
        metadata: MetadataEnvelope,
    },
    Datetime {
        metadata: MetadataEnvelope,
    },
    Duration {
        metadata: MetadataEnvelope,
    },
    Quantity {
        spec: QuantitySpec,
        metadata: MetadataEnvelope,
    },
    Union {
        spec: UnionSpec,
        metadata: MetadataEnvelope,
    },
    Secret {
        spec: SecretSpec,
        metadata: MetadataEnvelope,
    },
    QuotaToken {
        spec: QuotaTokenSpec,
        metadata: MetadataEnvelope,
    },
    Future {
        inner: Option<Box<SchemaType>>,
        metadata: MetadataEnvelope,
    },
    Stream {
        inner: Option<Box<SchemaType>>,
        metadata: MetadataEnvelope,
    },
}

impl SchemaType {
    pub fn bool() -> Self {
        Self::Bool {
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn s32() -> Self {
        Self::S32 {
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn u8() -> Self {
        Self::U8 {
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn u32() -> Self {
        Self::U32 {
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn u64() -> Self {
        Self::U64 {
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn f64() -> Self {
        Self::F64 {
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn string() -> Self {
        Self::String {
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn ref_to(id: impl Into<String>) -> Self {
        Self::Ref {
            id: TypeId::new(id),
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn record(fields: Vec<NamedFieldType>) -> Self {
        Self::Record {
            fields,
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn tuple(elements: Vec<SchemaType>) -> Self {
        Self::Tuple {
            elements,
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn list(element: SchemaType) -> Self {
        Self::List {
            element: Box::new(element),
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn option(inner: SchemaType) -> Self {
        Self::Option {
            inner: Box::new(inner),
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn variant(cases: Vec<VariantCaseType>) -> Self {
        Self::Variant {
            cases,
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn flags(flags: Vec<String>) -> Self {
        Self::Flags {
            flags,
            metadata: MetadataEnvelope::default(),
        }
    }

    pub fn result(ok: Option<SchemaType>, err: Option<SchemaType>) -> Self {
        Self::Result {
            spec: ResultSpec {
                ok: ok.map(Box::new),
                err: err.map(Box::new),
            },
            metadata: MetadataEnvelope::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct NamedFieldType {
    pub name: String,
    pub body: SchemaType,
    pub metadata: MetadataEnvelope,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct VariantCaseType {
    pub name: String,
    pub payload: Option<SchemaType>,
    pub metadata: MetadataEnvelope,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct ResultSpec {
    pub ok: Option<Box<SchemaType>>,
    pub err: Option<Box<SchemaType>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct TextRestrictions {
    pub languages: Option<Vec<String>>,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    pub regex: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct BinaryRestrictions {
    pub mime_types: Option<Vec<String>>,
    pub min_bytes: Option<u32>,
    pub max_bytes: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum PathDirection {
    Input,
    Output,
    InOut,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum PathKind {
    File,
    Directory,
    Any,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct PathSpec {
    pub direction: PathDirection,
    pub kind: PathKind,
    pub allowed_mime_types: Option<Vec<String>>,
    pub allowed_extensions: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct UrlRestrictions {
    pub allowed_schemes: Option<Vec<String>>,
    pub allowed_hosts: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct QuantityValue {
    pub mantissa: i64,
    pub scale: i32,
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct QuantitySpec {
    pub base_unit: String,
    pub allowed_suffixes: Vec<String>,
    pub min: Option<QuantityValue>,
    pub max: Option<QuantityValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct UnionSpec {
    pub branches: Vec<UnionBranch>,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct UnionBranch {
    pub tag: String,
    pub body: SchemaType,
    pub discriminator: DiscriminatorRule,
    pub metadata: MetadataEnvelope,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum DiscriminatorRule {
    Prefix { prefix: String },
    Suffix { suffix: String },
    Contains { substring: String },
    Regex { regex: String },
    FieldEquals(FieldDiscriminator),
    FieldAbsent { field_name: String },
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct FieldDiscriminator {
    pub field_name: String,
    pub literal: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct SecretSpec {
    pub category: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct QuotaTokenSpec {
    pub resource_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub enum SchemaValue {
    Bool(bool),
    S8(i8),
    S16(i16),
    S32(i32),
    S64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    String(String),
    Record {
        fields: Vec<SchemaValue>,
    },
    Variant(VariantValuePayload),
    Enum {
        case: u32,
    },
    Flags {
        bits: Vec<bool>,
    },
    Tuple {
        elements: Vec<SchemaValue>,
    },
    List {
        elements: Vec<SchemaValue>,
    },
    FixedList {
        elements: Vec<SchemaValue>,
    },
    Map {
        entries: Vec<(SchemaValue, SchemaValue)>,
    },
    Option {
        inner: Option<Box<SchemaValue>>,
    },
    Result(ResultValuePayload),
    Text(TextValuePayload),
    Binary(BinaryValuePayload),
    Path {
        path: String,
    },
    Url {
        url: String,
    },
    Datetime {
        value: DateTime<Utc>,
    },
    Duration(DurationValuePayload),
    Quantity(QuantityValue),
    Union(UnionValuePayload),
    Secret(SecretValuePayload),
    QuotaToken(QuotaTokenValuePayload),
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub struct VariantValuePayload {
    pub case: u32,
    pub payload: Option<Box<SchemaValue>>,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub enum ResultValuePayload {
    Ok { value: Option<Box<SchemaValue>> },
    Err { value: Option<Box<SchemaValue>> },
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct TextValuePayload {
    pub text: String,
    pub language: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct BinaryValuePayload {
    pub bytes: Vec<u8>,
    pub mime_type: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct DurationValuePayload {
    pub nanoseconds: i64,
}

#[derive(Clone, Debug, PartialEq, BinaryCodec)]
#[desert(evolution())]
pub struct UnionValuePayload {
    pub tag: String,
    pub body: Box<SchemaValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct SecretValuePayload {
    pub secret_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct QuotaTokenValuePayload {
    pub environment_id: String,
    pub resource_name: String,
    pub expected_use: u64,
    pub last_credit: i64,
    pub last_credit_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(transparent)]
pub struct AgentTypeName(pub String);

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum AgentMode {
    Durable,
    Ephemeral,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum AutoInjectedKind {
    Principal,
    AgentId,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum FieldSource {
    UserSupplied,
    AutoInjected(AutoInjectedKind),
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct NamedField {
    pub name: String,
    pub schema: SchemaType,
    pub metadata: MetadataEnvelope,
    pub source: FieldSource,
}

impl NamedField {
    pub fn user_supplied(name: impl Into<String>, schema: SchemaType) -> Self {
        Self {
            name: name.into(),
            schema,
            metadata: MetadataEnvelope::default(),
            source: FieldSource::UserSupplied,
        }
    }

    pub fn auto_injected(
        name: impl Into<String>,
        kind: AutoInjectedKind,
        schema: SchemaType,
    ) -> Self {
        Self {
            name: name.into(),
            schema,
            metadata: MetadataEnvelope::default(),
            source: FieldSource::AutoInjected(kind),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub enum InputSchema {
    Parameters(Vec<NamedField>),
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
#[allow(clippy::large_enum_variant)]
pub enum OutputSchema {
    Unit,
    Single(SchemaType),
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct AgentConstructorSchema {
    pub name: Option<String>,
    pub description: String,
    pub prompt_hint: Option<String>,
    pub input_schema: InputSchema,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct AgentMethodSchema {
    pub name: String,
    pub description: String,
    pub prompt_hint: Option<String>,
    pub input_schema: InputSchema,
    pub output_schema: OutputSchema,
    pub read_only: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct AgentConfigDeclarationSchema {
    pub source: String,
    pub path: Vec<String>,
    pub value_type: SchemaType,
}

#[derive(Clone, Debug, PartialEq, Eq, BinaryCodec)]
#[desert(evolution())]
pub struct AgentTypeSchema {
    pub type_name: AgentTypeName,
    pub description: String,
    pub source_language: String,
    pub schema: SchemaGraph,
    pub constructor: AgentConstructorSchema,
    pub methods: Vec<AgentMethodSchema>,
    pub dependencies: Vec<AgentTypeName>,
    pub mode: AgentMode,
    pub http_mount: Option<String>,
    pub snapshotting: bool,
    pub config: Vec<AgentConfigDeclarationSchema>,
}

fn field(name: &str, body: SchemaType) -> NamedFieldType {
    NamedFieldType {
        name: name.to_string(),
        body,
        metadata: MetadataEnvelope::default(),
    }
}

fn case(name: &str, payload: Option<SchemaType>) -> VariantCaseType {
    VariantCaseType {
        name: name.to_string(),
        payload,
        metadata: MetadataEnvelope::default(),
    }
}

fn def(id: &str, body: SchemaType) -> SchemaTypeDef {
    SchemaTypeDef {
        id: TypeId::new(id),
        name: None,
        body,
    }
}

fn typed(root: SchemaType, value: SchemaValue) -> TypedSchemaValue {
    TypedSchemaValue {
        graph: SchemaGraph::anonymous(SchemaType::record(vec![field("p0", root)])),
        value: SchemaValue::Record {
            fields: vec![value],
        },
    }
}

pub fn simple_typed_value() -> TypedSchemaValue {
    typed(SchemaType::u32(), SchemaValue::U32(42))
}

pub fn medium_typed_value() -> TypedSchemaValue {
    let root = SchemaType::record(vec![
        field("id", SchemaType::u32()),
        field("name", SchemaType::string()),
        field("tag", SchemaType::option(SchemaType::string())),
        field("scores", SchemaType::list(SchemaType::u8())),
    ]);
    let value = SchemaValue::Record {
        fields: vec![
            SchemaValue::U32(12345),
            SchemaValue::String("benchmark-test".into()),
            SchemaValue::Option {
                inner: Some(Box::new(SchemaValue::String("tagged".into()))),
            },
            SchemaValue::List {
                elements: (0..64).map(SchemaValue::U8).collect(),
            },
        ],
    };
    typed(root, value)
}

pub fn complex_typed_value() -> TypedSchemaValue {
    let inner_record = SchemaType::record(vec![
        field("a", SchemaType::u32()),
        field("b", SchemaType::string()),
    ]);
    let root = SchemaType::tuple(vec![
        SchemaType::variant(vec![
            case("ok", Some(SchemaType::u32())),
            case("error", Some(SchemaType::string())),
        ]),
        SchemaType::result(Some(SchemaType::u32()), Some(SchemaType::string())),
        SchemaType::flags(vec!["read".into(), "write".into(), "execute".into()]),
        SchemaType::list(inner_record),
        SchemaType::option(SchemaType::f64()),
    ]);
    let value = SchemaValue::Tuple {
        elements: vec![
            SchemaValue::Variant(VariantValuePayload {
                case: 1,
                payload: Some(Box::new(SchemaValue::String("something went wrong".into()))),
            }),
            SchemaValue::Result(ResultValuePayload::Ok {
                value: Some(Box::new(SchemaValue::U32(200))),
            }),
            SchemaValue::Flags {
                bits: vec![true, false, true],
            },
            SchemaValue::List {
                elements: (0..32)
                    .map(|i| SchemaValue::Record {
                        fields: vec![
                            SchemaValue::U32(i),
                            SchemaValue::String(format!("item-{i}")),
                        ],
                    })
                    .collect(),
            },
            SchemaValue::Option {
                inner: Some(Box::new(SchemaValue::F64(1.23456789))),
            },
        ],
    };
    typed(root, value)
}

pub fn large_list_typed_value() -> TypedSchemaValue {
    let root = SchemaType::list(SchemaType::record(vec![
        field("idx", SchemaType::u32()),
        field("label", SchemaType::string()),
    ]));
    let elements = (0..1000)
        .map(|i| SchemaValue::Record {
            fields: vec![
                SchemaValue::U32(i),
                SchemaValue::String(format!("item-{i}")),
            ],
        })
        .collect();
    typed(root, SchemaValue::List { elements })
}

pub fn large_binary_typed_value() -> TypedSchemaValue {
    typed(
        SchemaType::Binary {
            restrictions: BinaryRestrictions::default(),
            metadata: MetadataEnvelope::default(),
        },
        SchemaValue::Binary(BinaryValuePayload {
            bytes: vec![0xAB; 65536],
            mime_type: Some("application/octet-stream".to_string()),
        }),
    )
}

pub fn semantic_typed_value() -> TypedSchemaValue {
    let root = SchemaType::record(vec![
        field(
            "created_at",
            SchemaType::Datetime {
                metadata: MetadataEnvelope::default(),
            },
        ),
        field(
            "quota",
            SchemaType::QuotaToken {
                spec: QuotaTokenSpec {
                    resource_name: Some("tokens".to_string()),
                },
                metadata: MetadataEnvelope::default(),
            },
        ),
        field(
            "quantity",
            SchemaType::Quantity {
                spec: QuantitySpec {
                    base_unit: "ms".to_string(),
                    allowed_suffixes: vec!["s".to_string(), "ms".to_string()],
                    min: Some(QuantityValue {
                        mantissa: 0,
                        scale: 0,
                        unit: "ms".to_string(),
                    }),
                    max: None,
                },
                metadata: MetadataEnvelope::default(),
            },
        ),
    ]);
    let timestamp = Utc.timestamp_opt(1_700_000_000, 123_000_000).unwrap();
    let value = SchemaValue::Record {
        fields: vec![
            SchemaValue::Datetime { value: timestamp },
            SchemaValue::QuotaToken(QuotaTokenValuePayload {
                environment_id: "env-123".to_string(),
                resource_name: "tokens".to_string(),
                expected_use: 5,
                last_credit: 99,
                last_credit_at: timestamp,
            }),
            SchemaValue::Quantity(QuantityValue {
                mantissa: 1500,
                scale: 0,
                unit: "ms".to_string(),
            }),
        ],
    };
    typed(root, value)
}

pub fn typed_value_cases() -> Vec<(&'static str, TypedSchemaValue)> {
    vec![
        ("simple", simple_typed_value()),
        ("medium", medium_typed_value()),
        ("complex", complex_typed_value()),
        ("large_list_1000", large_list_typed_value()),
        ("large_binary_64k", large_binary_typed_value()),
        ("semantic", semantic_typed_value()),
    ]
}

fn tree_value(depth: usize) -> SchemaValue {
    if depth == 0 {
        SchemaValue::Variant(VariantValuePayload {
            case: 0,
            payload: Some(Box::new(SchemaValue::S32(7))),
        })
    } else {
        SchemaValue::Variant(VariantValuePayload {
            case: 1,
            payload: Some(Box::new(SchemaValue::Tuple {
                elements: vec![tree_value(depth - 1), tree_value(depth - 1)],
            })),
        })
    }
}

pub fn wide_graph_with_recursive_value(def_count: usize) -> (SchemaGraph, SchemaValue) {
    let mut defs = Vec::with_capacity(def_count + 1);
    for i in 0..def_count {
        defs.push(def(
            &format!("t{i:03}"),
            SchemaType::record(vec![
                field("a", SchemaType::u32()),
                field("b", SchemaType::string()),
                field("c", SchemaType::option(SchemaType::u64())),
            ]),
        ));
    }
    defs.push(def(
        "tree",
        SchemaType::variant(vec![
            case("leaf", Some(SchemaType::s32())),
            case(
                "node",
                Some(SchemaType::tuple(vec![
                    SchemaType::ref_to("tree"),
                    SchemaType::ref_to("tree"),
                ])),
            ),
        ]),
    ));

    let root = SchemaType::record(vec![
        field("first", SchemaType::ref_to("t000")),
        field(
            "middle",
            SchemaType::ref_to(format!("t{:03}", def_count / 2)),
        ),
        field("last", SchemaType::ref_to(format!("t{:03}", def_count - 1))),
        field("trees", SchemaType::list(SchemaType::ref_to("tree"))),
    ]);
    let named_record_value = SchemaValue::Record {
        fields: vec![
            SchemaValue::U32(1),
            SchemaValue::String("x".to_string()),
            SchemaValue::Option {
                inner: Some(Box::new(SchemaValue::U64(2))),
            },
        ],
    };
    let value = SchemaValue::Record {
        fields: vec![
            named_record_value.clone(),
            named_record_value.clone(),
            named_record_value,
            SchemaValue::List {
                elements: (0..16).map(|_| tree_value(6)).collect(),
            },
        ],
    };
    (SchemaGraph { defs, root }, value)
}

pub fn wide_typed_value() -> TypedSchemaValue {
    let (graph, value) = wide_graph_with_recursive_value(64);
    TypedSchemaValue { graph, value }
}

pub fn cache_key_schema_value() -> SchemaValue {
    complex_typed_value().value
}

pub fn representative_agent_type_schema() -> AgentTypeSchema {
    let (schema, _) = wide_graph_with_recursive_value(32);
    let method_fields = vec![
        NamedField::user_supplied("count", SchemaType::u32()),
        NamedField::user_supplied("label", SchemaType::string()),
        NamedField::user_supplied("items", SchemaType::list(SchemaType::u8())),
        NamedField::user_supplied("maybe", SchemaType::option(SchemaType::bool())),
        NamedField::user_supplied("first", SchemaType::ref_to("t000")),
        NamedField::user_supplied("last", SchemaType::ref_to("t031")),
        NamedField::auto_injected(
            "principal",
            AutoInjectedKind::Principal,
            SchemaType::string(),
        ),
    ];

    AgentTypeSchema {
        type_name: AgentTypeName("bench-agent".to_string()),
        description: "benchmark agent".to_string(),
        source_language: "rust".to_string(),
        schema,
        constructor: AgentConstructorSchema {
            name: None,
            description: String::new(),
            prompt_hint: None,
            input_schema: InputSchema::Parameters(vec![NamedField::user_supplied(
                "seed",
                SchemaType::u64(),
            )]),
        },
        methods: vec![AgentMethodSchema {
            name: "do-work".to_string(),
            description: String::new(),
            prompt_hint: None,
            input_schema: InputSchema::Parameters(method_fields),
            output_schema: OutputSchema::Single(SchemaType::result(Some(SchemaType::u64()), None)),
            read_only: Some(true),
        }],
        dependencies: vec![AgentTypeName("dependency-agent".to_string())],
        mode: AgentMode::Durable,
        http_mount: Some("/agents/bench".to_string()),
        snapshotting: false,
        config: vec![AgentConfigDeclarationSchema {
            source: "environment".to_string(),
            path: vec!["runtime".to_string(), "limit".to_string()],
            value_type: SchemaType::ref_to("t000"),
        }],
    }
}
