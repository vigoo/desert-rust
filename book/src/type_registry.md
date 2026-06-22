# Type registry

The original Scala `desert` library includes a type registry for cases where the
concrete type is not known at compile time. A serialized value carries a compact
type id, and the registry maps that id back to a codec during deserialization.

`desert-rust` does not currently expose an equivalent public type registry API.
The placeholder remains in the book because this is an important concept in the
Scala documentation and a likely future Rust feature.

## What to use today

For closed sets of alternatives, use an enum:

```rust,ignore
use desert_rust::BinaryCodec;

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
enum Message {
    Ping,
    Rename { id: u64, name: String },
    Delete { id: u64 },
}
```

This is the most idiomatic Rust option when all possible variants are known to
the crate defining the protocol.

For open sets, define an application-level tag and implement the codec traits
manually:

```rust,ignore
use desert_rust::{
    BinaryDeserializer, BinaryOutput, BinarySerializer, DeserializationContext,
    Error, Result, SerializationContext,
};

trait PluginMessage {}

struct TextMessage(String);
impl PluginMessage for TextMessage {}

enum AnyMessage {
    Text(TextMessage),
}

impl BinarySerializer for AnyMessage {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        match self {
            AnyMessage::Text(TextMessage(text)) => {
                1u32.serialize(context)?;
                text.serialize(context)
            }
        }
    }
}

impl BinaryDeserializer for AnyMessage {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        match u32::deserialize(context)? {
            1 => Ok(AnyMessage::Text(TextMessage(String::deserialize(context)?))),
            other => Err(Error::InvalidConstructorId {
                constructor_id: other,
                type_name: "AnyMessage".to_string(),
            }),
        }
    }
}
```

Keep ids stable once data has been written. If an id is retired, leave it
reserved so newer variants do not take over old meanings.

## Difference from Scala

In Scala, the type registry is part of the public library API and is used by
integrations such as actor serializers. In Rust, framework integration is
currently expected to happen at the byte boundary: serialize a statically known
type, or define your own dynamic envelope type as shown above.
