use crate::deserializer::DeserializationContext;
use crate::serializer::SerializationContext;
use crate::{BinaryDeserializer, BinarySerializer, Error, Result};
use bigdecimal::num_bigint::BigInt;
use bigdecimal::num_traits::ToBytes;
use bigdecimal::BigDecimal;

impl BinarySerializer for BigDecimal {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        self.to_string().serialize(context)
    }
}

impl BinaryDeserializer for BigDecimal {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let string = String::deserialize(context)?;
        Ok(string.parse().map_err(|err| {
            Error::DeserializationFailure(format!("Failed to deserialize BigDecimal: {err}"))
        })?)
    }
}

impl BinarySerializer for BigInt {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        self.to_be_bytes().serialize(context)
    }
}

impl BinaryDeserializer for BigInt {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let bytes = Vec::<u8>::deserialize(context)?;
        Ok(BigInt::from_signed_bytes_be(&bytes))
    }
}
