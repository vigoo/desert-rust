use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::deserializer::DeserializationContext;
use crate::serializer::SerializationContext;
use crate::{BinaryDeserializer, BinarySerializer, Result};
use uuid::Uuid;

impl BinarySerializer for Uuid {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_bytes(self.into_bytes().as_slice());
        Ok(())
    }
}

impl BinaryDeserializer for Uuid {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let bytes = context.read_bytes(16)?;
        let bytes: [u8; 16] = bytes.try_into()?;
        Ok(Uuid::from_bytes(uuid::Bytes::from(bytes)))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::roundtrip;
    use proptest::proptest;
    use proptest_arbitrary_interop::arb;
    use test_r::test;
    use uuid::Uuid;

    proptest! {
        #[test]
        fn test_uuid(value in arb::<Uuid>()) {
            roundtrip(value);
        }
    }
}
