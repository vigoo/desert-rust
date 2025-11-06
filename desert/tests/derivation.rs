use assert2::check;
use desert_rust::*;
use test_r::test;

test_r::enable!();

#[derive(Debug, PartialEq, BinaryCodec)]
#[desert(evolution(FieldAdded("x", 0), FieldRemoved("z")))]
struct Point {
    pub x: i32,
    pub y: i32,
    #[transient(None::<String>)]
    _cached_str: Option<String>,
}

#[derive(Debug, PartialEq, BinaryCodec)]
#[desert(evolution(FieldAdded("x", 0), FieldRemoved("z"), FieldAdded("description", Some("hello".to_string())), FieldMadeOptional("description")))]
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

#[derive(Debug, PartialEq, BinaryCodec)]
#[desert(transparent)]
struct MyInt(i32);

#[derive(Debug, PartialEq, BinaryCodec)]
#[desert(transparent)]
struct MyString {
    pub value: String,
}

#[derive(Debug, PartialEq, BinaryCodec)]
struct GenericStruct<T> {
    pub value: T,
}

#[derive(Debug, PartialEq, BinaryCodec)]
enum GenericEnum<T> {
    A(T),
    B,
}

#[derive(Debug, PartialEq, BinaryCodec)]
struct r#RawIdent {
    pub r#raw_field: u32,
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

    let my_int = MyInt(42);
    let bytes4 = serialize_to_bytes(&my_int).unwrap();
    let my_int2: MyInt = deserialize(&bytes4).unwrap();
    check!(my_int == my_int2);

    // Check that transparent serialization matches the inner type
    let inner: i32 = 42;
    let bytes_inner = serialize_to_bytes(&inner).unwrap();
    check!(bytes4 == bytes_inner);

    let my_string = MyString {
        value: "hello".to_string(),
    };
    let bytes5 = serialize_to_bytes(&my_string).unwrap();
    let my_string2: MyString = deserialize(&bytes5).unwrap();
    check!(my_string == my_string2);

    let inner2: String = "hello".to_string();
    let bytes_inner2 = serialize_to_bytes(&inner2).unwrap();
    check!(bytes5 == bytes_inner2);

    // Test generic struct
    let generic_struct = GenericStruct { value: 42 };
    let bytes6 = serialize_to_bytes(&generic_struct).unwrap();
    let generic_struct2: GenericStruct<i32> = deserialize(&bytes6).unwrap();
    check!(generic_struct == generic_struct2);

    // Test generic enum
    let generic_enum = GenericEnum::A("hello".to_string());
    let bytes7 = serialize_to_bytes(&generic_enum).unwrap();
    let generic_enum2: GenericEnum<String> = deserialize(&bytes7).unwrap();
    check!(generic_enum == generic_enum2);

    let generic_enum_b = GenericEnum::<String>::B;
    let bytes8 = serialize_to_bytes(&generic_enum_b).unwrap();
    let generic_enum_b2: GenericEnum<String> = deserialize(&bytes8).unwrap();
    check!(generic_enum_b == generic_enum_b2);
}
