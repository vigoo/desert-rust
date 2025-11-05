use crate::{
    BinaryDeserializer, BinaryOutput, BinarySerializer, DeserializationContext,
    SerializationContext,
};
use serde_json::Value;

impl BinarySerializer for Value {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> crate::Result<()> {
        let bytes = serde_json::to_vec(self)
            .map_err(|err| crate::Error::SerializationFailure(err.to_string()))?;
        bytes.serialize(context)
    }
}

impl BinaryDeserializer for Value {
    fn deserialize(context: &mut DeserializationContext<'_>) -> crate::Result<Self> {
        let bytes = Vec::<u8>::deserialize(context)?;
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|err| crate::Error::DeserializationFailure(err.to_string()))?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use test_r::test;

    #[test]
    fn roundtrip_example1() {
        let input = json!({
            "hello": "world",
            "other": 10,
            "inner": [{"x": 1, "y": 0.5}, {"x": -1, "y": 100}]
        });
        let serialized = crate::serialize_to_bytes(&input).unwrap();
        let deserialized: Value = crate::deserialize(&serialized).unwrap();
        assert_eq!(input, deserialized);
    }
}
