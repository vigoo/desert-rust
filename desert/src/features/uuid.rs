use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::deserializer::DeserializationContext;
use crate::serializer::SerializationContext;
use crate::{BinaryDeserializer, BinarySerializer, Result};
use uuid::Uuid;

impl BinarySerializer for Uuid {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context
            .output_mut()
            .write_bytes(self.into_bytes().as_slice());
        Ok(())
    }
}

impl BinaryDeserializer for Uuid {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let bytes = context.input_mut().read_bytes(16)?;
        let bytes: [u8; 16] = bytes.as_slice().try_into()?;
        Ok(Uuid::from_bytes(uuid::Bytes::from(bytes)))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::roundtrip;
    use proptest::proptest;
    use proptest_arbitrary_interop::arb;
    use uuid::Uuid;

    proptest! {
        #[test]
        fn test_uuid(value in arb::<Uuid>()) {
            roundtrip(value);
        }
    }
}
