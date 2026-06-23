use desert_rust::{
    serialize, serialize_to_byte_vec, BinaryCodec, BinaryDeserializer, BinaryOutput,
    BinarySerializer, DeduplicatedString, DeserializationContext, Result, SerializationContext,
};
use test_r::test;

test_r::enable!();

#[derive(Default)]
struct NonEditableOutput(Vec<u8>);

impl BinaryOutput for NonEditableOutput {
    fn write_u8(&mut self, value: u8) {
        self.0.push(value);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

fn fallback_bytes<T: BinarySerializer>(value: &T) -> Vec<u8> {
    serialize(value, NonEditableOutput::default()).unwrap().0
}

fn assert_fast_matches_fallback<T>(value: &T)
where
    T: BinarySerializer,
{
    assert_eq!(serialize_to_byte_vec(value).unwrap(), fallback_bytes(value));
}

#[derive(Debug, PartialEq, BinaryCodec)]
#[desert(evolution(FieldAdded("metadata", String::new()), FieldAdded("enabled", true)))]
struct MultiChunk {
    id: u64,
    metadata: String,
    count: u32,
    enabled: bool,
}

#[derive(Debug, PartialEq, BinaryCodec)]
#[desert(evolution(FieldMadeOptional("label")))]
struct OptionalField {
    id: u64,
    label: Option<String>,
}

#[derive(Debug, PartialEq, BinaryCodec)]
#[desert(evolution(FieldMadeOptional("payload"), FieldRemoved("payload")))]
struct RemovedField {
    id: u64,
    label: String,
}

#[derive(Debug, PartialEq, BinaryCodec)]
#[desert(evolution(FieldMadeTransient("cached")))]
struct TransientField {
    id: u64,
    #[transient(Vec::new())]
    cached: Vec<u8>,
    label: String,
}

#[derive(Debug, PartialEq, BinaryCodec)]
enum EvolvedVariant {
    #[desert(evolution(
        FieldAdded("trace_id", "trace-default".to_string()),
        FieldMadeOptional("payload")
    ))]
    Running {
        id: u64,
        trace_id: String,
        payload: Option<String>,
    },
    Done,
}

#[derive(Debug, Clone, PartialEq)]
struct DedupField(String);

impl BinarySerializer for DedupField {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        DeduplicatedString(self.0.clone()).serialize(context)
    }
}

impl BinaryDeserializer for DedupField {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        Ok(Self(DeduplicatedString::deserialize(context)?.0))
    }
}

#[derive(Debug, PartialEq, BinaryCodec)]
#[desert(evolution(FieldRemoved("gone")))]
struct DedupHeaderSensitive {
    first: DedupField,
    second: DedupField,
}

#[test]
fn multi_chunk_struct_fast_path_is_byte_identical_to_fallback() {
    assert_fast_matches_fallback(&MultiChunk {
        id: 42,
        metadata: "source-order middle field".to_string(),
        count: 7,
        enabled: true,
    });
}

#[test]
fn optional_field_fast_path_is_byte_identical_to_fallback() {
    assert_fast_matches_fallback(&OptionalField {
        id: 100,
        label: Some("now optional".to_string()),
    });
}

#[test]
fn removed_field_fast_path_is_byte_identical_to_fallback() {
    assert_fast_matches_fallback(&RemovedField {
        id: 200,
        label: "kept".to_string(),
    });
}

#[test]
fn transient_field_fast_path_is_byte_identical_to_fallback() {
    assert_fast_matches_fallback(&TransientField {
        id: 300,
        cached: vec![1, 2, 3],
        label: "visible".to_string(),
    });
}

#[test]
fn evolved_enum_variant_fast_path_is_byte_identical_to_fallback() {
    assert_fast_matches_fallback(&EvolvedVariant::Running {
        id: 400,
        trace_id: "trace-1".to_string(),
        payload: Some("payload".to_string()),
    });
}

#[test]
fn deduplicated_string_state_order_matches_fallback() {
    assert_fast_matches_fallback(&DedupHeaderSensitive {
        first: DedupField("gone".to_string()),
        second: DedupField("gone".to_string()),
    });
}

#[test]
fn non_editable_output_falls_back_and_roundtrips() {
    let value = MultiChunk {
        id: 500,
        metadata: "fallback".to_string(),
        count: 11,
        enabled: false,
    };

    let bytes = fallback_bytes(&value);
    let decoded: MultiChunk = desert_rust::deserialize(&bytes).unwrap();
    assert_eq!(decoded, value);
}
