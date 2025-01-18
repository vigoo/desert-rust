use arbitrary::Arbitrary;
use desert_macro::BinaryCodec;
use proptest::prelude::*;
use proptest_arbitrary_interop::arb;
use test_r::test;

#[allow(dead_code)]
mod serialization_properties;

use crate::serialization_properties::{compatibility_test, incompatibility_test, roundtrip};

test_r::enable!();

mod desert {
    pub use desert_core::*;
}

#[derive(Debug, Clone, PartialEq, BinaryCodec, Arbitrary)]
#[evolution()]
enum TypeV1 {
    Cons1V1,
    Cons2V1(String),
}

#[derive(Debug, Clone, PartialEq, BinaryCodec, Arbitrary)]
#[evolution()]
enum TypeV2 {
    Cons1V2,
    Cons2V2(String),
    Cons2V3 { value: i64 },
}

proptest! {
    #[test]
    fn serialization_works(v1 in arb::<TypeV1>()) {
        roundtrip(v1);
    }
}

#[test]
fn can_read_old_data_after_adding_new_constructor() {
    let old = TypeV1::Cons1V1;
    let new = TypeV2::Cons1V2;
    compatibility_test(old, new);
}

#[test]
fn can_read_new_data_after_adding_new_constructor_if_it_is_not_the_new_constructor() {
    let old = TypeV2::Cons2V2("x".to_string());
    let new = TypeV1::Cons2V1("x".to_string());
    compatibility_test(old, new);
}

#[test]
fn cannot_read_new_constructor_as_old_data() {
    incompatibility_test::<TypeV2, TypeV1>(TypeV2::Cons2V3 { value: 42 });
}
