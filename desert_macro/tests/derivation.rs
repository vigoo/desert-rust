use assert2::check;
use desert_macro::BinaryCodec;

#[derive(Debug, PartialEq, BinaryCodec)]
#[evolution(FieldAdded("x", 0), FieldRemoved("z"))]
struct Point {
    pub x: i32,
    pub y: i32,
    #[transient(None::<String>)]
    cached_str: Option<String>,
}

#[test]
fn debug() {
    let pt = Point {
        x: 1,
        y: -10,
        cached_str: None,
    };
    let bytes = desert::serialize_to_bytes(&pt).unwrap();
    check!(
        bytes.to_vec()
            == vec![0x02, 0x08, 0x08, 0x03, 0x02, 0x7a, 0xff, 0xff, 0xff, 0xf6, 0, 0, 0, 1]
    );

    let pt2 = desert::deserialize(bytes).unwrap();
    check!(pt == pt2);
}
