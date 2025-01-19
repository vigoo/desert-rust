
# Getting started with desert

Desert is a _binary serialization library_ for Rust (and Scala), focusing on creating small binaries
while still enabling binary compatible evolution of the data model.

It is suitable for any kind of short or long term storage.

First add `desert` as a dependency:

```toml
desert_rust = "0.1.0"
```

The most simple use case is to serialize a known type to an array of bytes and read it back:

```rust
# extern crate desert_rust;
use desert_rust::serialize_to_byte_vec;

# fn main() {
let data_or_failure = serialize_to_byte_vec(&"Hello world".to_string());
# }
```

```rust
# extern crate desert_rust;
# use desert_rust::serialize_to_byte_vec;
# use desert_rust::deserialize;

# fn main() {
#  let data_or_failure = serialize_to_byte_vec(&"Hello world".to_string());
let y = data_or_failure.and_then(|data| deserialize::<String>(&data));
# }
```

### Codecs

This works because the `BinaryCodec` (a combination of `BinarySerializer` and `BinaryDeserializer`) trait is implemented for `String`. Read
the [codecs page](./codecs.md) to learn about the available codecs and how to define custom ones.

### Low level input/output

The above example shows the convenient functions to work with `Vec<u8>` and `&[u8]` directly, but they have a more generic
version working on the low level `BinaryInput` and `BinaryOutput` interfaces. These are described on
the [input/output page](./input_output.md).

### Evolution

One of the primary features of the library is the support for _evolving the data model_. The possibilities
are described on a [separate page](./evolution.md).

### Type registry

For cases when the exact type to be deserialized is not known at compile type, the possibilities
can be registered to a [type registry](./type_registry.md).
