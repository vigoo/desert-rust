use crate::{
    BinaryDeserializer, BinaryInput, BinaryOutput, BinarySerializer, DeserializationContext, Error,
    Result, SerializationContext,
};
use serde_json::{Map, Number, Value};

const JSON_NULL: u8 = 0;
const JSON_FALSE: u8 = 1;
const JSON_TRUE: u8 = 2;
const JSON_I64: u8 = 3;
const JSON_U64: u8 = 4;
const JSON_F64: u8 = 5;
const JSON_DECIMAL_STRING: u8 = 6;
const JSON_STRING: u8 = 7;
const JSON_ARRAY: u8 = 8;
const JSON_OBJECT: u8 = 9;

impl BinarySerializer for Value {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        if context.options().json_binary {
            serialize_binary_json_value(self, context)
        } else {
            let bytes = serde_json::to_vec(self)
                .map_err(|err| Error::SerializationFailure(err.to_string()))?;
            bytes.serialize(context)
        }
    }
}

impl BinaryDeserializer for Value {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        if context.options().json_binary {
            deserialize_binary_json_value(context)
        } else {
            let length = context.read_var_u32()?;
            let bytes = context.read_bytes(length as usize)?;
            let value: Value = serde_json::from_slice(bytes)
                .map_err(|err| Error::DeserializationFailure(err.to_string()))?;
            Ok(value)
        }
    }
}

fn serialize_binary_json_value<Output: BinaryOutput>(
    value: &Value,
    context: &mut SerializationContext<Output>,
) -> Result<()> {
    match value {
        Value::Null => {
            context.write_u8(JSON_NULL);
            Ok(())
        }
        Value::Bool(false) => {
            context.write_u8(JSON_FALSE);
            Ok(())
        }
        Value::Bool(true) => {
            context.write_u8(JSON_TRUE);
            Ok(())
        }
        Value::Number(number) => serialize_binary_json_number(number, context),
        Value::String(string) => {
            context.write_u8(JSON_STRING);
            string.serialize(context)
        }
        Value::Array(values) => {
            context.write_u8(JSON_ARRAY);
            context.write_var_i32(values.len().try_into()?);
            for value in values {
                serialize_binary_json_value(value, context)?;
            }
            Ok(())
        }
        Value::Object(entries) => {
            context.write_u8(JSON_OBJECT);
            context.write_var_i32(entries.len().try_into()?);
            for (key, value) in entries {
                key.serialize(context)?;
                serialize_binary_json_value(value, context)?;
            }
            Ok(())
        }
    }
}

fn serialize_binary_json_number<Output: BinaryOutput>(
    number: &Number,
    context: &mut SerializationContext<Output>,
) -> Result<()> {
    if let Some(value) = number.as_i64() {
        context.write_u8(JSON_I64);
        value.serialize(context)
    } else if let Some(value) = number.as_u64() {
        context.write_u8(JSON_U64);
        value.serialize(context)
    } else if let Some(value) = number.as_f64() {
        context.write_u8(JSON_F64);
        value.serialize(context)
    } else {
        context.write_u8(JSON_DECIMAL_STRING);
        number.to_string().serialize(context)
    }
}

fn deserialize_binary_json_value(context: &mut DeserializationContext<'_>) -> Result<Value> {
    match context.read_u8()? {
        JSON_NULL => Ok(Value::Null),
        JSON_FALSE => Ok(Value::Bool(false)),
        JSON_TRUE => Ok(Value::Bool(true)),
        JSON_I64 => Ok(Value::Number(Number::from(i64::deserialize(context)?))),
        JSON_U64 => Ok(Value::Number(Number::from(u64::deserialize(context)?))),
        JSON_F64 => {
            let value = f64::deserialize(context)?;
            let number = Number::from_f64(value).ok_or_else(|| {
                Error::DeserializationFailure(format!(
                    "Failed to deserialize JSON number from f64: {value}"
                ))
            })?;
            Ok(Value::Number(number))
        }
        JSON_DECIMAL_STRING => {
            let value = String::deserialize(context)?;
            let number = value.parse::<Number>().map_err(|err| {
                Error::DeserializationFailure(format!(
                    "Failed to deserialize JSON decimal number: {err}"
                ))
            })?;
            Ok(Value::Number(number))
        }
        JSON_STRING => Ok(Value::String(String::deserialize(context)?)),
        JSON_ARRAY => {
            let length = read_json_collection_len(context)?;
            let mut values = Vec::with_capacity(length);
            for _ in 0..length {
                values.push(deserialize_binary_json_value(context)?);
            }
            Ok(Value::Array(values))
        }
        JSON_OBJECT => {
            let length = read_json_collection_len(context)?;
            let mut entries = Map::new();
            for _ in 0..length {
                let key = String::deserialize(context)?;
                let value = deserialize_binary_json_value(context)?;
                entries.insert(key, value);
            }
            Ok(Value::Object(entries))
        }
        tag => Err(Error::DeserializationFailure(format!(
            "Failed to deserialize JSON value: invalid tag {tag}"
        ))),
    }
}

fn read_json_collection_len(context: &mut DeserializationContext<'_>) -> Result<usize> {
    let length = context.read_var_i32()?;
    if length < 0 {
        Err(Error::DeserializationFailure(
            "Failed to deserialize JSON collection: negative length".to_string(),
        ))
    } else {
        Ok(length as usize)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::roundtrip_with_options;
    use crate::{
        deserialize_with_options, serialize_to_byte_vec, serialize_to_byte_vec_with_options,
        Options,
    };
    use proptest::collection::{btree_map, vec};
    use proptest::prelude::*;
    use proptest::string::string_regex;
    use serde_json::{json, Value};
    use test_r::test;

    fn json_string_strategy() -> impl Strategy<Value = String> {
        string_regex("[ -~]{0,32}").unwrap()
    }

    fn json_number_strategy() -> impl Strategy<Value = serde_json::Number> {
        prop_oneof![
            any::<i64>().prop_map(serde_json::Number::from),
            any::<u64>().prop_map(serde_json::Number::from),
            (-1.0e12f64..1.0e12f64)
                .prop_filter_map("finite JSON f64", serde_json::Number::from_f64),
        ]
    }

    fn json_value_strategy() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            json_number_strategy().prop_map(Value::Number),
            json_string_strategy().prop_map(Value::String),
        ];

        leaf.prop_recursive(4, 64, 8, |inner| {
            prop_oneof![
                vec(inner.clone(), 0..12).prop_map(Value::Array),
                btree_map(json_string_strategy(), inner, 0..12)
                    .prop_map(|entries| Value::Object(entries.into_iter().collect())),
            ]
        })
    }

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

    #[test]
    fn binary_json_roundtrip() {
        let input = json!({
            "null": null,
            "false": false,
            "true": true,
            "negative": -42,
            "unsigned": 18446744073709551615u64,
            "float": 12.5,
            "text": "hello \"json\"",
            "array": [1, null, "item"],
            "object": {
                "nested": true
            }
        });
        let options = Options::default().with_binary_json();
        let serialized = serialize_to_byte_vec_with_options(&input, options.clone()).unwrap();
        let deserialized: Value = deserialize_with_options(&serialized, options).unwrap();

        assert_eq!(input, deserialized);
    }

    #[test]
    fn default_json_format_is_unchanged() {
        let input = json!({
            "hello": "world",
            "other": 10,
            "inner": [{"x": 1, "y": 0.5}, {"x": -1, "y": 100}]
        });

        assert_eq!(
            serialize_to_byte_vec(&input).unwrap(),
            serialize_to_byte_vec_with_options(&input, Options::default()).unwrap()
        );
    }

    proptest! {
        #[test]
        fn binary_json_roundtrips_generated_values(value in json_value_strategy()) {
            roundtrip_with_options(value, Options::default().with_binary_json());
        }
    }
}
