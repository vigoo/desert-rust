use assert2::check;
use desert_rust::*;
use std::borrow::Cow;
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
enum GenericHelperNameCollision<DesertDeserializationHelpers> {
    Value(DesertDeserializationHelpers),
}

#[test]
fn generic_parameter_may_use_internal_helper_name() {
    let value = GenericHelperNameCollision::Value(42_u32);
    let bytes = serialize_to_bytes(&value).unwrap();
    let decoded: GenericHelperNameCollision<u32> = deserialize(&bytes).unwrap();
    assert_eq!(decoded, value);
}

#[derive(Debug, PartialEq, BinaryCodec)]
#[allow(dead_code)]
struct r#RawIdent {
    pub r#raw_field: u32,
}

#[derive(Debug, PartialEq, Clone)]
struct CustomWrapper<'a>(Cow<'a, str>);

impl<'a> BinarySerializer for CustomWrapper<'a> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        self.0.as_bytes().serialize(context)
    }
}

impl<'a> BinaryDeserializer for CustomWrapper<'a> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        Ok(CustomWrapper(Cow::Owned(
            String::from_utf8(Vec::<u8>::deserialize(context)?).unwrap(),
        )))
    }
}

#[derive(Debug, PartialEq, BinaryCodec)]
enum EnumWithCustom {
    #[desert(custom = CustomWrapper)]
    Wrapped(String),
    Regular(String),
}

#[derive(Debug, PartialEq, BinaryCodec)]
enum EnumWithTransparent {
    #[desert(transparent)]
    A,
    #[desert(transparent)]
    B,
    #[desert(transparent)]
    C(String),
    D {
        value: String,
    },
}

#[derive(Debug, PartialEq, BinaryCodec)]
enum EnumWithSelfDefault {
    #[desert(evolution(FieldAdded("value", Self::default_value())))]
    A { value: u32 },
}

impl EnumWithSelfDefault {
    fn default_value() -> u32 {
        42
    }
}

macro_rules! large_enum {
    ($($variant:ident),* $(,)?) => {
        #[allow(dead_code)]
        #[derive(Debug, PartialEq, BinaryCodec)]
        enum LargeEnum {
            #[desert(evolution(FieldAdded("value", String::new())))]
            Custom { value: String },
            $($variant),*
        }
    };
}

large_enum!(
    V000, V001, V002, V003, V004, V005, V006, V007, V008, V009, V010, V011, V012, V013, V014, V015,
    V016, V017, V018, V019, V020, V021, V022, V023, V024, V025, V026, V027, V028, V029, V030, V031,
    V032, V033, V034, V035, V036, V037, V038, V039, V040, V041, V042, V043, V044, V045, V046, V047,
    V048, V049, V050, V051, V052, V053, V054, V055, V056, V057, V058, V059, V060, V061, V062, V063,
    V064, V065, V066, V067, V068, V069, V070, V071, V072, V073, V074, V075, V076, V077, V078, V079,
    V080, V081, V082, V083, V084, V085, V086, V087, V088, V089, V090, V091, V092, V093, V094, V095,
    V096, V097, V098, V099, V100, V101, V102, V103, V104, V105, V106, V107, V108, V109, V110, V111,
    V112, V113, V114, V115, V116, V117, V118, V119, V120, V121, V122, V123, V124, V125, V126, V127,
    V128, V129, V130, V131, V132, V133, V134, V135, V136, V137, V138, V139, V140, V141, V142, V143,
    V144, V145, V146, V147, V148, V149, V150, V151, V152, V153, V154, V155, V156, V157, V158, V159,
    V160, V161, V162, V163, V164, V165, V166, V167, V168, V169, V170, V171, V172,
);

fn assert_large_enum_roundtrip_on_small_stack(value: LargeEnum) {
    let bytes = serialize_to_bytes(&value).unwrap();
    std::thread::Builder::new()
        .stack_size(384 * 1024)
        .spawn(move || {
            let decoded: LargeEnum = deserialize(&bytes).unwrap();
            assert_eq!(decoded, value);
        })
        .unwrap()
        .join()
        .unwrap();
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

    // Test custom wrapper
    let wrapped = EnumWithCustom::Wrapped("hello".to_string());
    let bytes_wrapped = serialize_to_bytes(&wrapped).unwrap();
    let wrapped2: EnumWithCustom = deserialize(&bytes_wrapped).unwrap();
    check!(wrapped == wrapped2);

    let regular = EnumWithCustom::Regular("test".to_string());
    let bytes_regular = serialize_to_bytes(&regular).unwrap();
    let regular2: EnumWithCustom = deserialize(&bytes_regular).unwrap();
    check!(regular == regular2);

    // Test transparent variants
    let transparent_a = EnumWithTransparent::A;
    let bytes_transparent_a = serialize_to_bytes(&transparent_a).unwrap();
    let transparent_a2: EnumWithTransparent = deserialize(&bytes_transparent_a).unwrap();
    check!(transparent_a == transparent_a2);

    let transparent_b = EnumWithTransparent::B;
    let bytes_transparent_b = serialize_to_bytes(&transparent_b).unwrap();
    let transparent_b2: EnumWithTransparent = deserialize(&bytes_transparent_b).unwrap();
    check!(transparent_b == transparent_b2);

    let transparent_c = EnumWithTransparent::C("hello".to_string());
    let bytes_transparent_c = serialize_to_bytes(&transparent_c).unwrap();
    let transparent_c2: EnumWithTransparent = deserialize(&bytes_transparent_c).unwrap();
    check!(transparent_c == transparent_c2);

    let transparent_d = EnumWithTransparent::D {
        value: "hello".to_string(),
    };
    let bytes_transparent_d = serialize_to_bytes(&transparent_d).unwrap();
    let transparent_d2: EnumWithTransparent = deserialize(&bytes_transparent_d).unwrap();
    check!(transparent_d == transparent_d2);
}

#[test]
fn large_enum_deserialization_fits_on_small_stack() {
    assert_large_enum_roundtrip_on_small_stack(LargeEnum::V172);
    assert_large_enum_roundtrip_on_small_stack(LargeEnum::Custom {
        value: "test".to_string(),
    });
}

#[test]
fn enum_evolution_default_can_reference_self() {
    let value = EnumWithSelfDefault::A { value: 7 };
    let bytes = serialize_to_bytes(&value).unwrap();
    let decoded: EnumWithSelfDefault = deserialize(&bytes).unwrap();
    assert_eq!(decoded, value);
}
