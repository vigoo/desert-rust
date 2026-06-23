# Binary input/output

The high-level helpers are enough for most use cases, but the core library is
built around two low-level traits:

- `BinaryOutput` writes bytes.
- `BinaryInput` reads bytes and can skip regions.

Serialization and deserialization contexts implement these traits and add shared
state for features such as string deduplication, reference tracking, options,
and ADT evolution.

## High-level output helpers

Use `serialize_to_byte_vec` when you want a `Vec<u8>`:

```rust,ignore
use desert_rust::{serialize_to_byte_vec, Result};

fn main() -> Result<()> {
    let bytes = serialize_to_byte_vec(&42i32)?;
    assert_eq!(bytes, vec![0, 0, 0, 42]);
    Ok(())
}
```

Use `serialize_to_bytes` when you want `bytes::Bytes`:

```rust,ignore
use desert_rust::{serialize_to_bytes, Result};

fn main() -> Result<()> {
    let bytes = serialize_to_bytes(&"hello".to_string())?;
    assert_eq!(&bytes[..], &[10, b'h', b'e', b'l', b'l', b'o']);
    Ok(())
}
```

The string length byte is `10` because signed variable integers use zig-zag
encoding internally. The logical string length is `5`.

### Choosing a `Vec<u8>` helper

`serialize_to_byte_vec` starts with a small default capacity and serializes the
value once. This is usually the right choice for small payloads, such as single
commands, individual events, request/response fragments, or anything likely to
fit within the default 128-byte buffer.

For larger values, repeated `Vec` growth can become visible. There are three
ways to avoid that:

- Use `serialize_to_byte_vec_with_capacity` when you already know a good
  capacity estimate.
- Use `serialize_into_byte_vec` when serializing repeatedly and you can reuse an
  existing buffer.
- Use `serialize_to_byte_vec_exact` when the value is probably large but the
  caller does not know its serialized size.

`serialize_to_byte_vec_exact` first computes the exact serialized length with
`serialized_size`, then serializes into a `Vec<u8>` allocated with that capacity:

```rust,ignore
use desert_rust::{serialize_to_byte_vec_exact, Result};

fn main() -> Result<()> {
    let batch = (0..10_000).collect::<Vec<u32>>();
    let bytes = serialize_to_byte_vec_exact(&batch)?;

    assert!(!bytes.is_empty());
    Ok(())
}
```

This is a two-pass tradeoff. It avoids growth for large, one-off values, but it
does more work for small values. If the value fits in the default buffer,
`serialize_to_byte_vec` is normally faster. If you already know the size or can
reuse a buffer, the capacity or reusable-buffer helpers are usually faster than
the exact helper.

## Writing to a custom output

The generic `serialize` function accepts any `BinaryOutput`. The built-in
outputs are:

- `Vec<u8>`
- `bytes::BytesMut`
- `SizeCalculator`

`SizeCalculator` computes the number of bytes that would be written without
storing them:

```rust,ignore
use desert_rust::{serialize, Result, SizeCalculator};

fn main() -> Result<()> {
    let output = serialize(&1234i32, SizeCalculator::new())?;
    assert_eq!(output.size(), 4);
    Ok(())
}
```

To write to a new destination, implement `BinaryOutput`:

```rust,ignore
use desert_rust::{BinaryOutput, Result};

struct CountingOutput {
    count: usize,
}

impl BinaryOutput for CountingOutput {
    fn write_u8(&mut self, _value: u8) {
        self.count += 1;
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.count += bytes.len();
    }
}
```

The trait provides default implementations for fixed-width integers, floats,
variable-length integers, and compressed byte blocks.

## Reading input

The public top-level `deserialize` helper reads from `&[u8]`:

```rust,ignore
use desert_rust::{deserialize, Result};

fn main() -> Result<()> {
    let value: i32 = deserialize(&[0, 0, 0, 42])?;
    assert_eq!(value, 42);
    Ok(())
}
```

For low-level code, `SliceInput` borrows bytes and `OwnedInput` owns a `Vec<u8>`:

```rust,ignore
use desert_rust::{BinaryInput, Result, SliceInput};

fn main() -> Result<()> {
    let mut input = SliceInput::new(&[0, 0, 0, 42]);
    let value = input.read_i32()?;

    assert_eq!(value, 42);
    Ok(())
}
```

Most custom deserializers should use `DeserializationContext` rather than
`SliceInput` directly, because the context also carries options and shared
state:

```rust,ignore
use desert_rust::{BinaryDeserializer, DeserializationContext, Options, Result};

fn main() -> Result<()> {
    let mut context = DeserializationContext::new(&[0, 0, 0, 42], Options::default());
    let value = i32::deserialize(&mut context)?;

    assert_eq!(value, 42);
    Ok(())
}
```

## Variable-length integers

`BinaryOutput::write_var_u32` writes a `u32` in 1 to 5 bytes. Smaller positive
values use fewer bytes:

```rust,ignore
use desert_rust::{BinaryOutput, Result};

fn main() -> Result<()> {
    let mut bytes = Vec::new();
    bytes.write_var_u32(1);
    bytes.write_var_u32(4096);

    assert_eq!(bytes, vec![1, 128, 32]);
    Ok(())
}
```

`write_var_i32` uses zig-zag encoding before writing the value as `var_u32`.
This keeps small negative values compact too:

```rust,ignore
use desert_rust::BinaryOutput;

let mut bytes = Vec::new();
bytes.write_var_i32(-1);
assert_eq!(bytes, vec![1]);
```

The library uses variable-length integers for lengths, ids, and ADT evolution
metadata. Fixed-width Rust integer codecs such as `i32` still use fixed-width
big-endian bytes.

{{#desert-bytes io.var_u32}}

{{#desert-bytes io.var_i32}}

## Compression helpers

`BinaryOutput::write_compressed` stores:

1. the uncompressed length as `var_u32`
2. the compressed length as `var_u32`
3. the deflate-compressed bytes

`BinaryInput::read_compressed` reverses that representation:

```rust,ignore
use desert_rust::{BinaryInput, BinaryOutput, OwnedInput, Result};

fn main() -> Result<()> {
    let data = b"hello hello hello";
    let mut bytes = Vec::new();

    bytes.write_compressed(data, Default::default())?;

    let mut input = OwnedInput::new(bytes);
    let decoded = input.read_compressed()?;

    assert_eq!(decoded, data);
    Ok(())
}
```

Compression is a low-level helper. The built-in type codecs do not compress
their payloads automatically.

{{#desert-bytes io.compressed}}

## Byte blocks and iterables

Byte-oriented containers use a compact block format: a `var_u32` byte count
followed by the bytes. Generic iterable containers use an item format that can
also represent streams with no exact size hint.

{{#desert-bytes bytes.vec_u8}}

{{#desert-bytes io.iterable_unknown}}

## Options

`Options` currently controls Scala-compatible character encoding:

```rust,ignore
use desert_rust::Options;

let default_options = Options::default();
let scala_options = Options::scala_compatible();

assert!(!default_options.chars_as_u16);
assert!(scala_options.chars_as_u16);
```

Pass options through `serialize_with_options`,
`serialize_to_byte_vec_with_options`, `serialize_to_bytes_with_options`, or
`deserialize_with_options`.

## Context state

`SerializationContext` and `DeserializationContext` also hold per-stream state.
Built-in uses include:

- string ids for `DeduplicatedString`
- reference ids for custom reference-aware codecs
- nested buffer stacks used by ADT evolution encoding

If you implement a custom codec and only need ordinary values, call the existing
`BinarySerializer` and `BinaryDeserializer` implementations. Touching context
state directly is an advanced use case.
