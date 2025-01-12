use crate::deserializer::DeserializationContext;
use crate::serializer::SerializationContext;
use crate::{BinaryDeserializer, BinaryOutput, BinarySerializer, Error, Result};
use bigdecimal::num_bigint::BigInt;
use bigdecimal::num_traits::ToBytes;
use bigdecimal::BigDecimal;

impl BinarySerializer for BigDecimal {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        self.to_string().serialize(context)
    }
}

impl BinaryDeserializer for BigDecimal {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let string = String::deserialize(context)?;
        string.parse().map_err(|err| {
            Error::DeserializationFailure(format!("Failed to deserialize BigDecimal: {err}"))
        })
    }
}

impl BinarySerializer for BigInt {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        self.to_be_bytes().serialize(context)
    }
}

impl BinaryDeserializer for BigInt {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let bytes = Vec::<u8>::deserialize(context)?;
        Ok(BigInt::from_signed_bytes_be(&bytes))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::roundtrip;
    use bigdecimal::num_bigint::BigInt;
    use bigdecimal::{BigDecimal, Num};
    use proptest::collection::vec;
    use proptest::prelude::*;
    use test_r::test;

    fn bigdecimal_strategy() -> impl Strategy<Value = BigDecimal> {
        ((0..u128::MAX), (0..u128::MAX), any::<bool>()).prop_map(|(a, b, has_fractional)| {
            let a = a.to_string();
            let b = b.to_string();
            let string = if has_fractional {
                format!("{}.{}", a, b)
            } else {
                a
            };
            string.parse().unwrap()
        })
    }

    fn bigint_strategy() -> impl Strategy<Value = BigInt> {
        vec(any::<u8>(), 1..1000).prop_map(|nums| {
            let s = nums.into_iter().map(|n| n.to_string()).collect::<String>();
            BigInt::from_str_radix(&s, 10).unwrap()
        })
    }

    proptest! {
        #[test]
        fn roundtrip_bigdecimal(value in bigdecimal_strategy()) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_bigint(value in bigint_strategy()) {
            roundtrip(value);
        }
    }
}
