use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::deserializer::DeserializationContext;
use crate::serializer::SerializationContext;
use crate::{BinaryDeserializer, BinarySerializer, Result};
use mac_address::MacAddress;

impl BinarySerializer for MacAddress {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_bytes(self.bytes().as_slice());
        Ok(())
    }
}

impl BinaryDeserializer for MacAddress {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let bytes = context.read_bytes(6)?;
        let bytes: [u8; 6] = bytes.try_into()?;
        Ok(MacAddress::new(bytes))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::roundtrip;
    use mac_address::MacAddress;
    use proptest::prelude::*;
    use test_r::test;

    proptest! {
        #[test]
        fn test_mac_address(value: [u8; 6]) {
            let mac = MacAddress::new(value);
            roundtrip(mac);
        }
    }
}
