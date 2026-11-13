use desert_core::{deserialize, serialize_to_byte_vec, BinaryDeserializer, BinarySerializer};
use proptest::prelude::*;
use std::fmt::Debug;

pub fn arb<T: for<'a> arbitrary::Arbitrary<'a> + Debug>() -> impl Strategy<Value = T> {
    // Keep the former interop adapter's 256-byte input budget, while letting
    // proptest shrink the input and reject invalid arbitrary values.
    any::<[u8; 256]>().prop_filter_map("valid arbitrary value", |bytes| {
        T::arbitrary(&mut arbitrary::Unstructured::new(&bytes)).ok()
    })
}

pub fn roundtrip<T: BinarySerializer + BinaryDeserializer + Debug + Clone + PartialEq>(value: T) {
    roundtrip_custom(value, |a, b| assert_eq!(a, b));
}

pub fn roundtrip_custom<T: BinarySerializer + BinaryDeserializer + Debug + Clone + PartialEq>(
    value: T,
    check: impl Fn(T, T),
) {
    let data = serialize_to_byte_vec(&value).unwrap();
    let result = deserialize::<T>(&data).unwrap();
    check(value, result);
}

pub fn compatibility_test<
    Old: BinarySerializer + Debug + PartialEq,
    New: BinaryDeserializer + Debug + PartialEq,
>(
    old: Old,
    expected: New,
) {
    let data = serialize_to_byte_vec(&old).unwrap();
    let result = deserialize::<New>(&data).unwrap();
    assert_eq!(result, expected);
}

pub fn custom_compatibility_test<
    Old: BinarySerializer + Debug + PartialEq,
    New: BinaryDeserializer + Debug + PartialEq,
>(
    old: Old,
    check: impl Fn(New) -> bool,
) {
    let data = serialize_to_byte_vec(&old).unwrap();
    let result = deserialize::<New>(&data).unwrap();
    assert!(check(result));
}

pub fn incompatibility_test<Old: BinarySerializer + Debug + PartialEq, New: BinaryDeserializer>(
    old: Old,
) {
    let data = serialize_to_byte_vec(&old).unwrap();
    let result = deserialize::<New>(&data);
    assert!(result.is_err());
}
