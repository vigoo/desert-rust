use crate::deserializer::DeserializationContext;
use crate::serializer::SerializationContext;
use crate::{BinaryDeserializer, BinaryInput, BinaryOutput, BinarySerializer, Error, Result};
use bigdecimal::num_bigint::BigInt;
use bigdecimal::num_traits::ToBytes;
use bigdecimal::BigDecimal;

impl BinarySerializer for BigDecimal {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        if context.options().bigdecimal_binary {
            let (bigint, scale) = self.as_bigint_and_exponent();
            scale.serialize(context)?;
            bigint.serialize(context)
        } else {
            self.to_string().serialize(context)
        }
    }
}

impl BinaryDeserializer for BigDecimal {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        if context.options().bigdecimal_binary {
            let scale = i64::deserialize(context)?;
            let bigint = BigInt::deserialize(context)?;
            Ok(BigDecimal::new(bigint, scale))
        } else {
            let length = context.read_var_i32()?;
            if length < 0 {
                return Err(Error::DeserializationFailure(
                    "Failed to deserialize BigDecimal: negative string length".to_string(),
                ));
            }
            let bytes = context.read_bytes(length as usize)?;
            let string = std::str::from_utf8(bytes).map_err(|err| {
                Error::FailedToDecodeString(format!("Failed to decode BigDecimal string: {err}"))
            })?;
            string.parse().map_err(|err| {
                Error::DeserializationFailure(format!("Failed to deserialize BigDecimal: {err}"))
            })
        }
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
    use crate::tests::{roundtrip, roundtrip_with_options};
    use crate::{
        deserialize_with_options, serialize_to_byte_vec, serialize_to_byte_vec_with_options,
        Options,
    };
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
        fn roundtrip_bigdecimal_binary(value in bigdecimal_strategy()) {
            roundtrip_with_options(value, Options::default().with_binary_bigdecimal());
        }

        #[test]
        fn roundtrip_bigint(value in bigint_strategy()) {
            roundtrip(value);
        }
    }

    #[test]
    fn binary_bigdecimal_roundtrip() {
        let value = "123456789012345678901234567890.123456789"
            .parse::<BigDecimal>()
            .unwrap();
        let options = Options::default().with_binary_bigdecimal();
        let bytes = serialize_to_byte_vec_with_options(&value, options.clone()).unwrap();
        let result: BigDecimal = deserialize_with_options(&bytes, options).unwrap();

        assert_eq!(result, value);
    }

    #[test]
    fn default_bigdecimal_format_is_unchanged() {
        let value = "123456789012345678901234567890.123456789"
            .parse::<BigDecimal>()
            .unwrap();

        assert_eq!(
            serialize_to_byte_vec(&value).unwrap(),
            serialize_to_byte_vec_with_options(&value, Options::default()).unwrap()
        );
    }
}
