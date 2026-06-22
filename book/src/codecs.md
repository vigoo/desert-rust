# Codecs and derivation

A codec is the pair of traits that defines how a type is written and read:

```rust,ignore
use desert_rust::{BinaryDeserializer, BinarySerializer};

trait BinaryCodec: BinarySerializer + BinaryDeserializer {}
```

In the crate this is a blanket trait: any type implementing both serializer and
deserializer automatically implements `BinaryCodec`.

## Built-in codecs

The `desert_rust` crate re-exports the core implementations. The always
available codecs include:

- integers: `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `u128`,
  `i128`, `usize`, `isize`
- non-zero integers from `std::num`
- floats: `f32`, `f64`
- `bool`, `()`, `char`, `String`, `str`
- `std::time::Duration`
- `Option<T>` and `Result<T, E>`
- `std::ops::Bound<T>` and `Range<T>`
- `bytes::Bytes`
- arrays, `Vec<T>`, `VecDeque<T>`, `LinkedList<T>`
- `HashSet<T>`, `BTreeSet<T>`, `HashMap<K, V>`, `BTreeMap<K, V>`
- `Box<T>`, `Rc<T>`, `Arc<T>`, references, and `PhantomData<T>`
- `std::net::IpAddr`
- tuples from arity 1 to 8

Feature flags control codecs for third-party types:

| Feature | Types |
| --- | --- |
| `bigdecimal` | `bigdecimal::BigDecimal`, `bigdecimal::num_bigint::BigInt` |
| `bit-vec` | `bit_vec::BitVec` |
| `chrono` | `chrono` dates, times, offsets, `chrono_tz::Tz` |
| `mac_address` | `mac_address::MacAddress` |
| `nonempty-collections` | `nonempty_collections::NEVec<T>` |
| `serde-json` | `serde_json::Value` |
| `url` | `url::Url` |
| `uuid` | `uuid::Uuid` |

The facade currently pulls in the `desert_core` default feature set, so
`bigdecimal`, `chrono`, `uuid`, `nonempty-collections`, and `serde-json` are
enabled by default. Enable `bit-vec`, `mac_address`, or `url` explicitly when
you need those codecs.

The same generator is used for optional third-party codecs:

{{#desert-bytes feature.uuid}}

{{#desert-bytes feature.chrono_date}}

{{#desert-bytes feature.chrono_time}}

{{#desert-bytes feature.bigdecimal}}

{{#desert-bytes feature.bit_vec}}

{{#desert-bytes feature.mac_address}}

{{#desert-bytes feature.url}}

{{#desert-bytes feature.serde_json}}

{{#desert-bytes feature.nonempty_vec}}

## Primitive representation

Fixed-width numeric types are written in big-endian byte order:

```rust,ignore
use desert_rust::{serialize_to_byte_vec, Result};

fn main() -> Result<()> {
    assert_eq!(serialize_to_byte_vec(&100u16)?, vec![0, 100]);
    assert_eq!(serialize_to_byte_vec(&100u32)?, vec![0, 0, 0, 100]);
    Ok(())
}
```

`bool` is encoded as a single byte: `0` for `false`, `1` for `true`.
`String` and `str` are encoded as a variable-length signed byte count followed
by UTF-8 bytes.

{{#desert-bytes primitive.i32}}

{{#desert-bytes primitive.u16}}

{{#desert-bytes primitive.bool}}

{{#desert-bytes primitive.unit}}

{{#desert-bytes primitive.char}}

{{#desert-bytes primitive.string}}

`Option<T>` and `Result<T, E>` start with a single tag byte and then write only
the payload selected by that tag:

{{#desert-bytes option.some}}

{{#desert-bytes option.none}}

{{#desert-bytes result.ok}}

{{#desert-bytes result.err}}

`Vec<u8>`, `[u8]`, `[u8; N]`, `bytes::Bytes`, and `NEVec<u8>` use an optimized
byte-block encoding: a variable-length unsigned length followed by raw bytes.
This is intentionally compatible with the Scala library's byte chunk format.

{{#desert-bytes bytes.vec_u8}}

{{#desert-bytes bytes.bytes}}

## Collections

All generic iterable collection codecs share the same representation:

- If the iterator reports an exact size, desert writes that size as a
  variable-length signed integer and then all elements.
- If the size is not known, desert writes `-1`, then each element prefixed by a
  `1` byte, then a final `0` byte.

Because the representation is shared, many collection changes are binary
compatible. For example, a `Vec<i32>` can be read as a `LinkedList<i32>`, and a
`BTreeSet<i32>` can be read as a `HashSet<i32>`, as long as the target
collection's type constraints are satisfied.

{{#desert-bytes collection.vec_i32}}

{{#desert-bytes collection.btree_map}}

{{#desert-bytes tuple.pair}}

## Deriving structs

Use `#[derive(BinaryCodec)]` for ordinary named-field structs:

```rust,ignore
use desert_rust::BinaryCodec;

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
struct User {
    id: u64,
    name: String,
    email: Option<String>,
}
```

The generated format starts with a version byte. Version `0` structs are
compatible with tuples of the same field order and arity, which allows simple
tuple-to-struct migrations.

{{#desert-bytes derived.struct}}

For generic types, the derive macro adds serializer and deserializer bounds for
generic parameters:

```rust,ignore
use desert_rust::BinaryCodec;

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
struct Wrapper<T> {
    value: T,
}
```

## Deriving enums

Enums are encoded as a constructor id followed by constructor payload data:

```rust,ignore
use desert_rust::BinaryCodec;

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
enum Event {
    Started,
    Message(String),
    Moved { x: i32, y: i32 },
}
```

Constructor ids are assigned from the enum variant order, skipping transient
variants. Adding new variants at the end is compatible with old data, but old
code cannot read values using the new variant.

{{#desert-bytes derived.enum}}

You can ask the derive macro to assign constructor ids by sorted variant name:

```rust,ignore
use desert_rust::BinaryCodec;

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
#[desert(sorted_constructors)]
enum StableByName {
    B,
    A,
}
```

Use this only when all versions agree on the same naming scheme. Reordering
without `sorted_constructors` changes constructor ids and breaks compatibility.

## Transparent wrappers

Single-field structs can be encoded exactly as their inner type:

```rust,ignore
use desert_rust::BinaryCodec;

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
#[desert(transparent)]
struct UserId(u64);
```

This is the Rust equivalent of using the Scala wrapper derivation. It is useful
when a primitive value is promoted to a domain-specific newtype without changing
the wire format.

Transparent enum variants are also supported for unit variants and single-field
variants:

```rust,ignore
use desert_rust::BinaryCodec;

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
enum Value {
    #[desert(transparent)]
    Text(String),
    Structured { value: String },
}
```

The transparent variant still has an enum constructor id. The attribute affects
how the variant payload is encoded.

## Transient fields and variants

A transient field is not serialized. It must provide a default expression used
when deserializing:

```rust,ignore
use desert_rust::BinaryCodec;

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
struct Cached {
    value: String,
    #[transient(None::<usize>)]
    cached_len: Option<usize>,
}
```

Transient enum variants are not assigned constructor ids. Serializing such a
variant returns `Error::SerializingTransientConstructor`.

```rust,ignore
use desert_rust::BinaryCodec;

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
enum State {
    Stored,
    #[transient]
    RuntimeOnly,
}
```

Transient variants can be inserted or removed without shifting the ids of stored
variants.

## Custom codecs

Implement `BinarySerializer` and `BinaryDeserializer` manually when the derived
format is not appropriate:

```rust,ignore
use desert_rust::{
    BinaryDeserializer, BinaryOutput, BinarySerializer, DeserializationContext,
    Result, SerializationContext,
};

#[derive(Debug, PartialEq)]
struct Lowercase(String);

impl BinarySerializer for Lowercase {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        self.0.to_lowercase().serialize(context)
    }
}

impl BinaryDeserializer for Lowercase {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        Ok(Self(String::deserialize(context)?))
    }
}
```

The enum derive macro can also wrap a single-field variant through a custom
type. The wrapper type must be constructible from a borrowed value in the shape
expected by the macro, so this is mainly useful for specialized string wrappers.

## String deduplication

Normal `String` serialization does not deduplicate values. This keeps schema
evolution safe: when an older reader skips a newly added string field, it does
not accidentally miss a string id assignment needed by a later field.

For streams where the writer and reader agree that deduplication is safe, wrap
values in `DeduplicatedString`:

```rust,ignore
use bytes::BytesMut;
use desert_rust::{
    BinaryDeserializer, BinarySerializer, DeduplicatedString, DeserializationContext,
    Options, Result, SerializationContext,
};

fn main() -> Result<()> {
    let mut output = SerializationContext::new(BytesMut::new(), Options::default());

    DeduplicatedString("same".to_string()).serialize(&mut output)?;
    DeduplicatedString("same".to_string()).serialize(&mut output)?;

    let bytes = output.into_output();
    let mut input = DeserializationContext::new(&bytes, Options::default());

    let first = DeduplicatedString::deserialize(&mut input)?.0;
    let second = DeduplicatedString::deserialize(&mut input)?.0;

    assert_eq!(first, second);
    Ok(())
}
```

The first occurrence is encoded like a normal string. Later occurrences in the
same serialization context are encoded as a negative id.

{{#desert-bytes string.dedup}}
