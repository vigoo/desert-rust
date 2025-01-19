# Codecs

A `BinaryCodec` is a type class that defines both the _serializer_ and the _deserializer_ for a given type:

```rust
# extern crate desert_rust;
# use desert_rust::{BinarySerializer, BinaryDeserializer};

trait BinaryCodec: BinarySerializer + BinaryDeserializer {}
```

## Primitive types

The library defines implementations of `BinaryCodec` for the following primitive types:

```rust
# extern crate desert_rust;
# use desert_rust::*;
# fn main() {
let byte = serialize_to_byte_vec(&100u8).unwrap();    
# }
```

<table class="binary"><tr>
<td>100</td>
</tr></table>

```rust
# extern crate desert_rust;
# use desert_rust::*;
# fn main() {
let short = serialize_to_byte_vec(&100u16).unwrap();    
# }
```

<table class="binary"><tr>
<td>0</td>
<td>100</td>
</tr></table>

```rust
# extern crate desert_rust;
# use desert_rust::*;
# fn main() {
let int = serialize_to_byte_vec(&100u32).unwrap();    
# }
```

<table class="binary"><tr>
<td>0</td>
<td>0</td>
<td>0</td>
<td>100</td>
</tr></table>

```rust
# extern crate desert_rust;
# use desert_rust::*;
# fn main() {
let long = serialize_to_byte_vec(&100u64).unwrap();    
# }
```

<table class="binary"><tr>
<td>0</td>
<td>0</td>
<td>0</td>
<td>0</td>
<td>0</td>
<td>0</td>
<td>0</td>
<td>100</td>
</tr></table>


```rust
# extern crate desert_rust;
# use desert_rust::*;
# fn main() {
let float = serialize_to_byte_vec(&3.14f32).unwrap();    
# }
```

<table class="binary"><tr>
<td>64</td>
<td>72</td>
<td>245</td>
<td>195</td>
</tr></table>
