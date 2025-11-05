use crate::binary_output::BinaryOutput;
use crate::deserializer::DeserializationContext;
use crate::serializer::SerializationContext;
use crate::{BinaryDeserializer, BinarySerializer, Result};
use bit_vec::BitVec;

impl BinarySerializer for BitVec {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        let bytes = self.to_bytes();
        bytes.serialize(context)
    }
}

impl BinaryDeserializer for BitVec {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let bytes: Vec<u8> = Vec::deserialize(context)?;
        Ok(BitVec::from_bytes(&bytes))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::roundtrip;
    use bit_vec::BitVec;
    use proptest::prelude::*;
    use test_r::test;

    fn bitvec_strategy() -> impl Strategy<Value = BitVec> {
        any::<Vec<u8>>().prop_map(|bytes| BitVec::from_bytes(&bytes))
    }

    proptest! {
        #[test]
        fn test_bit_vec(value in bitvec_strategy()) {
            roundtrip(value);
        }
    }
}
