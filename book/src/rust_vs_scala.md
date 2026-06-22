# Rust and Scala differences

The Rust library follows the same design goals as the original Scala `desert`
library: compact binary data, ADT support, and schema evolution. The APIs are
different because Rust and Scala expose different language tools.

This page is a guide for readers who know the Scala documentation and want to
understand what changed in `desert-rust`.

## Crate layout

Rust has three workspace crates:

- `desert_rust`: public facade crate, re-exporting the core library and derive
  macro
- `desert_core`: serialization, deserialization, state, codecs, and evolution
  logic
- `desert_macro`: `#[derive(BinaryCodec)]`

Scala has separate modules for core codecs, derivation implementations, and
ecosystem integrations such as Akka, Pekko, Cats, ZIO, and Shardcake. Those
integration modules do not exist in the Rust project.

## Dependency model

Scala users choose between derivation modules such as Shapeless or ZIO Schema.
Rust users depend on `desert_rust` and use one derive macro:

```rust,ignore
use desert_rust::BinaryCodec;

#[derive(BinaryCodec)]
struct Point {
    x: i32,
    y: i32,
}
```

Third-party Rust codecs are controlled by Cargo feature flags, for example
`uuid`, `chrono`, and `serde-json`.

## Codec discovery

Scala uses implicit `BinaryCodec[T]` values. Rust uses trait implementations:

- `BinarySerializer` writes a type.
- `BinaryDeserializer` reads a type.
- `BinaryCodec` is implemented automatically when both are present.

There is no implicit search at runtime. If a generic function needs a codec, it
uses Rust trait bounds:

```rust,ignore
use desert_rust::{BinarySerializer, Result, serialize_to_byte_vec};

fn encode<T: BinarySerializer>(value: &T) -> Result<Vec<u8>> {
    serialize_to_byte_vec(value)
}
```

## Error handling

Scala APIs return an effect or an `Either`-like result depending on the module.
Rust APIs return `desert_rust::Result<T>`, whose error type is
`desert_rust::Error`:

```rust,ignore
use desert_rust::{deserialize, Error};

let result: Result<i32, Error> = deserialize(&[0, 0, 0, 1]);
```

## Derivation attributes

Scala evolution uses annotations such as `@evolutionSteps` and
`@transientField`. Rust uses derive helper attributes:

```rust,ignore
use desert_rust::BinaryCodec;

#[derive(BinaryCodec)]
#[desert(evolution(
    FieldAdded("description", Some("new".to_string())),
    FieldMadeOptional("description")
))]
struct Item {
    name: String,
    description: Option<String>,
    #[transient(0usize)]
    cached_hash: usize,
}
```

The Rust derive macro currently supports:

- `#[desert(evolution(...))]`
- `FieldAdded("field", default_expr)`
- `FieldMadeOptional("field")`
- `FieldRemoved("field")`
- `FieldMadeTransient("field")`
- `#[transient(default_expr)]` on fields
- `#[transient]` on enum variants
- `#[desert(transparent)]` on single-field structs and enum variants
- `#[desert(sorted_constructors)]` on enums

## Character encoding

The Scala library encoded characters as 16-bit Unicode values. The Rust default
encodes `char` as a Unicode scalar value using a variable-length unsigned
integer.

Use `Options::scala_compatible()` when Rust must read or write data compatible
with Scala character encoding:

```rust,ignore
use desert_rust::{Options, serialize_to_byte_vec_with_options};

fn main() -> desert_rust::Result<()> {
    let bytes = serialize_to_byte_vec_with_options(&'A', Options::scala_compatible())?;
    assert_eq!(bytes, vec![0, 65]);
    Ok(())
}
```

In Scala-compatible mode, characters outside a single UTF-16 code unit cannot be
serialized as `char`.

## Strings and byte arrays

Normal strings use the same basic shape: a compact byte length followed by
UTF-8. `DeduplicatedString` also follows the same idea as Scala: first
occurrences are written normally, repeated occurrences in the same context are
written as negative ids.

Raw byte blocks in Rust are represented by `Vec<u8>`, `[u8]`, `[u8; N]`,
`bytes::Bytes`, and `NEVec<u8>`. These use a compact unsigned length plus raw
bytes for compatibility with Scala byte chunks.

## Collections

Both implementations use a shared iterable representation so collection type
changes can be compatible. Rust collection compatibility is constrained by Rust
trait bounds: for example, deserializing into `HashSet<T>` requires `T: Eq +
Hash`, while deserializing into `BTreeSet<T>` requires `T: Ord`.

## Type registry

The Scala library has a type registry for serializing values whose concrete type
is not known statically.

The Rust crate does not currently expose an equivalent type registry API. For
now, model closed sets of runtime alternatives as enums, or define an
application-level tag plus custom `BinarySerializer` and `BinaryDeserializer`
implementations.

## Ecosystem integrations

Scala-specific modules described in the original documentation are not part of
`desert-rust`:

- Akka and Pekko serializers
- Cats and Cats Effect codecs
- ZIO effect wrappers and codecs
- ZIO Prelude API
- Shardcake serializer

Rust integration points are currently lower-level: implement the codec traits,
use the byte helper functions, and integrate those bytes with your framework of
choice.

## Compatibility expectation

The projects are similar, but not every Rust type has a Scala equivalent and not
every Scala codec has a Rust equivalent. For cross-language data, prefer a small
golden dataset and test it from both sides. Pay special attention to:

- `char` encoding and `Options::scala_compatible()`
- enabled Rust feature flags
- enum constructor order
- byte collection formats
- derived evolution history
