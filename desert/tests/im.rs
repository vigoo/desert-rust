use desert_rust::{deserialize, serialize_to_byte_vec, BinaryCodec};
use im::{ordmap, vector, OrdMap, Vector};
use std::collections::BTreeMap;
use test_r::test;

test_r::enable!();

fn roundtrip<T>(value: T)
where
    T: desert_rust::BinarySerializer
        + desert_rust::BinaryDeserializer
        + std::fmt::Debug
        + PartialEq,
{
    let bytes = serialize_to_byte_vec(&value).unwrap();
    let decoded = deserialize::<T>(&bytes).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn roundtrip_vectors() {
    roundtrip(Vector::<String>::new());
    roundtrip(vector!["one".to_string(), "two".to_string()]);
    roundtrip(vector![1_u8, 2, 3]);
}

#[test]
fn roundtrip_ord_maps() {
    roundtrip(OrdMap::<String, u32>::new());
    roundtrip(ordmap!["one".to_string() => 1, "two".to_string() => 2]);
}

#[test]
fn uses_standard_collection_wire_encodings() {
    let vector = vector![1_u8, 2, 3];
    let vec = vec![1_u8, 2, 3];
    assert_eq!(
        serialize_to_byte_vec(&vector).unwrap(),
        serialize_to_byte_vec(&vec).unwrap()
    );

    let ord_map = ordmap!["one".to_string() => 1_u32, "two".to_string() => 2];
    let btree_map = BTreeMap::from([("one".to_string(), 1_u32), ("two".to_string(), 2_u32)]);
    assert_eq!(
        serialize_to_byte_vec(&ord_map).unwrap(),
        serialize_to_byte_vec(&btree_map).unwrap()
    );
}

#[derive(Debug, PartialEq, BinaryCodec)]
struct PersistentCollections {
    vectors: Vector<Vector<i32>>,
    maps: OrdMap<String, Vector<u32>>,
}

#[test]
fn derived_struct_with_nested_im_collections_roundtrips() {
    roundtrip(PersistentCollections {
        vectors: vector![vector![1, 2], vector![3, 4]],
        maps: ordmap![
            "even".to_string() => vector![2, 4],
            "odd".to_string() => vector![1, 3]
        ],
    });
}
