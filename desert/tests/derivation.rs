use desert_rust::*;
use assert2::check;
use test_r::test;

test_r::enable!();

#[derive(Debug, PartialEq, BinaryCodec)]
#[evolution(FieldAdded("x", 0), FieldRemoved("z"))]
struct Point {
    pub x: i32,
    pub y: i32,
    #[transient(None::<String>)]
    _cached_str: Option<String>,
}

#[derive(Debug, PartialEq, BinaryCodec)]
#[evolution(FieldAdded("x", 0), FieldRemoved("z"), FieldAdded("description", Some("hello".to_string())), FieldMadeOptional("description"))]
struct Point2 {
    pub x: i32,
    pub y: i32,
    #[transient(None::<String>)]
    _cached_str: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, BinaryCodec)]
enum Choices {
    A,
    B(String),
    C { pt: Option<Point>, z: u64 },
}

#[test]
fn debug() {
    let pt = Point {
        x: 1,
        y: -10,
        _cached_str: None,
    };
    let bytes = serialize_to_bytes(&pt).unwrap();
    check!(
        bytes.to_vec()
            == vec![0x02, 0x08, 0x08, 0x03, 0x02, 0x7a, 0xff, 0xff, 0xff, 0xf6, 0, 0, 0, 1]
    );

    let pt2 = deserialize(&bytes).unwrap();
    check!(pt == pt2);

    let pt3 = Point2 {
        x: 1,
        y: -10,
        _cached_str: None,
        description: Some("Hello world".to_string()),
    };
    let bytes2 = serialize_to_bytes(&pt3).unwrap();
    let pt4 = deserialize(&bytes2).unwrap();
    check!(pt3 == pt4);

    let choices = Choices::C {
        pt: Some(Point {
            x: 1,
            y: 2,
            _cached_str: None,
        }),
        z: 3,
    };
    let bytes3 = serialize_to_bytes(&choices).unwrap();
    println!("{:?}", bytes3);
    let choices2 = deserialize(&bytes3).unwrap();
    check!(choices == choices2);
}
