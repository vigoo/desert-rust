use arbitrary::Arbitrary;
use desert_macro::BinaryCodec;

test_r::enable!();

#[allow(dead_code)]
mod serialization_properties;

mod desert_rust {
    pub use desert_core::*;
}

//#[derive(Debug, Clone, PartialEq, BinaryCodec)]
//#[evolution()]
//struct TestId(String);

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
#[evolution()]
struct ProdV1 {
    field_a: String,
    field_b: i32,
}

#[derive(Debug, Clone, PartialEq, BinaryCodec, Arbitrary)]
#[evolution(FieldAdded("new_field_1", true))]
struct ProdV2 {
    field_a: String,
    new_field_1: bool,
    field_b: i32,
}

#[derive(Debug, Clone, PartialEq, BinaryCodec, Arbitrary)]
#[evolution(FieldAdded("new_field_1", true), FieldMadeOptional("field_b"))]
struct ProdV3 {
    field_a: String,
    new_field_1: bool,
    field_b: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, BinaryCodec, Arbitrary)]
#[evolution(
    FieldAdded("new_field_1", true),
    FieldMadeOptional("field_b"),
    FieldRemoved("field_b")
)]
struct ProdV4 {
    field_a: String,
    new_field_1: bool,
}

#[derive(Debug, Clone, PartialEq, BinaryCodec, Arbitrary)]
#[evolution(
    FieldAdded("new_field_1", true),
    FieldMadeOptional("field_b"),
    FieldRemoved("field_b"),
    FieldMadeTransient("field_a")
)]
struct ProdV5 {
    #[transient("unset".to_string())]
    field_a: String,
    new_field_1: bool,
}

#[derive(Debug, Clone, PartialEq, BinaryCodec, Arbitrary)]
#[evolution()]
enum Coprod1 {
    Case11(i32),
    Case21 { x: String },
}

#[derive(Debug, Clone, PartialEq, BinaryCodec, Arbitrary)]
#[evolution()]
enum Coprod2 {
    Case12(i32),
    #[transient]
    TransientCons,
    Case22 {
        x: String,
    },
}

mod tuples_vs_products {
    use crate::serialization_properties::compatibility_test;
    use crate::ProdV1;
    use test_r::test;

    #[test]
    fn tuple_can_be_read_as_struct() {
        compatibility_test(
            ("hello".to_string(), 42),
            ProdV1 {
                field_a: "hello".to_string(),
                field_b: 42,
            },
        );
    }

    #[test]
    fn simple_struct_can_be_read_as_tuple() {
        compatibility_test(
            ProdV1 {
                field_a: "hello".to_string(),
                field_b: 42,
            },
            ("hello".to_string(), 42),
        );
    }
}

mod newtypes {
    // TODO
}

mod collections {
    use crate::serialization_properties::{compatibility_test, custom_compatibility_test};
    use std::collections::LinkedList;
    use test_r::test;

    #[test]
    fn list_to_vector() {
        compatibility_test(
            [1, 2, 3, 4, 5].into_iter().collect::<LinkedList<_>>(),
            [1, 2, 3, 4, 5].into_iter().collect::<Vec<_>>(),
        )
    }

    #[test]
    fn vector_to_list() {
        compatibility_test(
            [1, 2, 3, 4, 5].into_iter().collect::<Vec<_>>(),
            [1, 2, 3, 4, 5].into_iter().collect::<LinkedList<_>>(),
        )
    }

    #[test]
    fn vector_to_hashset() {
        compatibility_test(
            [1, 2, 3, 4, 5].into_iter().collect::<Vec<_>>(),
            [1, 2, 3, 4, 5]
                .into_iter()
                .collect::<std::collections::HashSet<_>>(),
        )
    }

    #[test]
    fn hashset_to_vector() {
        custom_compatibility_test(
            [1, 2, 3, 4, 5]
                .into_iter()
                .collect::<std::collections::HashSet<_>>(),
            |vec: Vec<i32>| {
                vec.contains(&1)
                    && vec.contains(&2)
                    && vec.contains(&3)
                    && vec.contains(&4)
                    && vec.contains(&5)
            },
        )
    }

    #[test]
    fn btreeset_to_hashset() {
        compatibility_test(
            [1, 2, 3, 4, 5]
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>(),
            [1, 2, 3, 4, 5]
                .into_iter()
                .collect::<std::collections::HashSet<_>>(),
        )
    }

    #[test]
    fn hashset_to_btreeset() {
        compatibility_test(
            [1, 2, 3, 4, 5]
                .into_iter()
                .collect::<std::collections::HashSet<_>>(),
            [1, 2, 3, 4, 5]
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>(),
        )
    }
}

mod adding_new_field {
    use crate::serialization_properties::{compatibility_test, roundtrip};
    use crate::{ProdV1, ProdV2};
    use proptest::proptest;
    use proptest_arbitrary_interop::arb;
    use test_r::test;

    proptest! {
        #[test]
        fn product_with_added_field_is_serializable(value in arb::<ProdV2>()) {
            roundtrip(value);
        }
    }

    #[test]
    fn old_version_can_read_new() {
        let serialized = ProdV2 {
            field_a: "hello".to_string(),
            new_field_1: true,
            field_b: 42,
        };
        let expected = ProdV1 {
            field_a: "hello".to_string(),
            field_b: 42,
        };
        compatibility_test(serialized, expected);
    }

    #[test]
    fn new_version_can_read_old() {
        let serialized = ProdV1 {
            field_a: "hello".to_string(),
            field_b: 42,
        };
        let expected = ProdV2 {
            field_a: "hello".to_string(),
            new_field_1: true,
            field_b: 42,
        };
        compatibility_test(serialized, expected);
    }
}

mod making_a_field_optional {
    use crate::serialization_properties::{compatibility_test, incompatibility_test, roundtrip};
    use crate::{ProdV1, ProdV2, ProdV3};
    use proptest::proptest;
    use proptest_arbitrary_interop::arb;
    use test_r::test;

    proptest! {
        #[test]
        fn product_with_field_made_optional_is_serializable(value in arb::<ProdV3>()) {
            roundtrip(value);
        }
    }

    #[test]
    fn v1_can_read_new_if_it_is_not_none() {
        let serialized = ProdV3 {
            field_a: "hello".to_string(),
            new_field_1: true,
            field_b: Some(200),
        };
        let expected = ProdV1 {
            field_a: "hello".to_string(),
            field_b: 200,
        };
        compatibility_test(serialized, expected);
    }

    #[test]
    fn v2_can_read_new_if_it_is_not_none() {
        let serialized = ProdV3 {
            field_a: "hello".to_string(),
            new_field_1: false,
            field_b: Some(200),
        };
        let expected = ProdV2 {
            field_a: "hello".to_string(),
            new_field_1: false,
            field_b: 200,
        };
        compatibility_test(serialized, expected);
    }

    #[test]
    fn old_cannot_read_new_if_it_is_none() {
        let serialized = ProdV3 {
            field_a: "hello".to_string(),
            new_field_1: false,
            field_b: None,
        };
        incompatibility_test::<ProdV3, ProdV2>(serialized);
    }

    #[test]
    fn new_version_can_read_v1() {
        let serialized = ProdV1 {
            field_a: "hello".to_string(),
            field_b: 200,
        };
        let expected = ProdV3 {
            field_a: "hello".to_string(),
            new_field_1: true,
            field_b: Some(200),
        };
        compatibility_test(serialized, expected);
    }

    #[test]
    fn new_version_can_read_v2() {
        let serialized = ProdV2 {
            field_a: "hello".to_string(),
            new_field_1: false,
            field_b: 200,
        };
        let expected = ProdV3 {
            field_a: "hello".to_string(),
            new_field_1: false,
            field_b: Some(200),
        };
        compatibility_test(serialized, expected);
    }
}

mod removing_a_field {
    use crate::serialization_properties::{compatibility_test, incompatibility_test, roundtrip};
    use crate::{ProdV1, ProdV2, ProdV3, ProdV4};
    use proptest::proptest;
    use proptest_arbitrary_interop::arb;
    use test_r::test;

    proptest! {
        #[test]
        fn product_with_field_removed_is_serializable(value in arb::<ProdV4>()) {
            roundtrip(value);
        }
    }

    #[test]
    fn can_read_v1_by_skipping_the_field() {
        let serialized = ProdV1 {
            field_a: "hello".to_string(),
            field_b: 200,
        };
        let expected = ProdV4 {
            field_a: "hello".to_string(),
            new_field_1: true,
        };
        compatibility_test(serialized, expected);
    }

    #[test]
    fn can_read_v2_by_skipping_the_field() {
        let serialized = ProdV2 {
            field_a: "hello".to_string(),
            new_field_1: false,
            field_b: 200,
        };
        let expected = ProdV4 {
            field_a: "hello".to_string(),
            new_field_1: false,
        };
        compatibility_test(serialized, expected);
    }

    #[test]
    fn can_read_v3_by_skipping_the_field() {
        let serialized = ProdV3 {
            field_a: "hello".to_string(),
            new_field_1: false,
            field_b: Some(200),
        };
        let expected = ProdV4 {
            field_a: "hello".to_string(),
            new_field_1: false,
        };
        compatibility_test(serialized, expected);
    }

    #[test]
    fn cannot_read_as_v1_because_of_missing_field() {
        let serialized = ProdV4 {
            field_a: "hello".to_string(),
            new_field_1: false,
        };
        incompatibility_test::<ProdV4, ProdV1>(serialized);
    }

    #[test]
    fn cannot_read_as_v2_because_of_missing_field() {
        let serialized = ProdV4 {
            field_a: "hello".to_string(),
            new_field_1: false,
        };
        incompatibility_test::<ProdV4, ProdV2>(serialized);
    }

    #[test]
    fn can_read_as_v3_missing_field_becomes_none() {
        let serialized = ProdV4 {
            field_a: "hello".to_string(),
            new_field_1: false,
        };
        let expected = ProdV3 {
            field_a: "hello".to_string(),
            new_field_1: false,
            field_b: None,
        };
        compatibility_test(serialized, expected);
    }
}

mod making_a_field_transient {
    use crate::serialization_properties::{
        compatibility_test, incompatibility_test, roundtrip_custom,
    };
    use crate::{ProdV1, ProdV2, ProdV3, ProdV4, ProdV5};
    use proptest::proptest;
    use proptest_arbitrary_interop::arb;
    use test_r::test;

    proptest! {
        #[test]
        fn product_with_field_removed_is_serializable(value in arb::<ProdV5>()) {
            roundtrip_custom(value, |a, b| {
                assert_eq!(a.new_field_1, b.new_field_1);
                assert_eq!(b.field_a, "unset");
            });
        }
    }

    #[test]
    fn can_read_v1_value_by_skipping_the_field_and_using_the_provided_default() {
        let serialized = ProdV1 {
            field_a: "hello".to_string(),
            field_b: 200,
        };
        let expected = ProdV5 {
            field_a: "unset".to_string(),
            new_field_1: true,
        };
        compatibility_test(serialized, expected);
    }

    #[test]
    fn can_read_v2_value_by_skipping_the_field_and_using_the_provided_default() {
        let serialized = ProdV2 {
            field_a: "hello".to_string(),
            new_field_1: false,
            field_b: 200,
        };
        let expected = ProdV5 {
            field_a: "unset".to_string(),
            new_field_1: false,
        };
        compatibility_test(serialized, expected);
    }

    #[test]
    fn can_read_v3_value_by_skipping_the_field_and_using_the_provided_default() {
        let serialized = ProdV3 {
            field_a: "hello".to_string(),
            new_field_1: false,
            field_b: Some(200),
        };
        let expected = ProdV5 {
            field_a: "unset".to_string(),
            new_field_1: false,
        };
        compatibility_test(serialized, expected);
    }

    #[test]
    fn can_read_v4_value_by_skipping_the_field_and_using_the_provided_default() {
        let serialized = ProdV4 {
            field_a: "hello".to_string(),
            new_field_1: false,
        };
        let expected = ProdV5 {
            field_a: "unset".to_string(),
            new_field_1: false,
        };
        compatibility_test(serialized, expected);
    }

    #[test]
    fn cannot_read_as_v4_because_of_missing_field() {
        let serialized = ProdV5 {
            field_a: "unset".to_string(),
            new_field_1: false,
        };
        incompatibility_test::<ProdV5, ProdV4>(serialized);
    }
}

mod adding_new_transient_constructors {
    use crate::serialization_properties::compatibility_test;
    use crate::{Coprod1, Coprod2};
    use test_r::test;

    #[test]
    fn adding_a_new_transient_constructor_keeps_binary_compatibility() {
        compatibility_test(Coprod1::Case11(5), Coprod2::Case12(5));
        compatibility_test(Coprod2::Case12(5), Coprod1::Case11(5));
        compatibility_test(
            Coprod1::Case21 {
                x: "hello".to_string(),
            },
            Coprod2::Case22 {
                x: "hello".to_string(),
            },
        );
        compatibility_test(
            Coprod2::Case22 {
                x: "hello".to_string(),
            },
            Coprod1::Case21 {
                x: "hello".to_string(),
            },
        );
    }
}
