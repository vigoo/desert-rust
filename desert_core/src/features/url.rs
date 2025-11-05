use crate::binary_output::BinaryOutput;
use crate::deserializer::DeserializationContext;
use crate::serializer::SerializationContext;
use crate::{BinaryDeserializer, BinarySerializer, Error, Result};
use url::Url;

impl BinarySerializer for Url {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        self.as_str().serialize(context)
    }
}

impl BinaryDeserializer for Url {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let s = String::deserialize(context)?;
        Url::parse(&s)
            .map_err(|e| Error::DeserializationFailure(format!("Failed to parse URL: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::roundtrip;
    use proptest::prelude::*;
    use test_r::test;
    use url::Url;

    fn url_strategy() -> impl Strategy<Value = Url> {
        prop_oneof![
            Just(Url::parse("http://example.com").unwrap()),
            Just(Url::parse("https://www.google.com/search?q=test").unwrap()),
            Just(Url::parse("ftp://ftp.example.com/file.txt").unwrap()),
            Just(Url::parse("mailto:user@example.com").unwrap()),
            any::<String>().prop_filter_map("valid url", |s| {
                if s.is_empty() || s.contains('\0') || s.contains('\n') {
                    None
                } else {
                    Url::parse(&s).ok()
                }
            })
        ]
    }

    proptest! {
        #[test]
        fn test_url(value in url_strategy()) {
            roundtrip(value);
        }
    }
}
