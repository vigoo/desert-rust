// Tests deserialization of a binary from the original Scala desert library

use assert2::check;
use desert_core::{
    deserialize, serialize_to_byte_vec, BinaryDeserializer, BinaryInput, BinaryOutput,
    BinarySerializer, DeserializationContext, SerializationContext,
};
use desert_macro::BinaryCodec;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use test_r::test;
use uuid::Uuid;

test_r::enable!();

mod desert_rust {
    pub use desert_core::*;
}

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
#[evolution(FieldMadeOptional("option"), FieldAdded("string", "default string".to_string()), FieldAdded("set", HashSet::new()))]
struct TestModel1 {
    byte: i8,
    short: i16,
    int: i32,
    long: i64,
    float: f32,
    double: f64,
    boolean: bool,
    unit: (),
    string: String,
    uuid: Uuid,
    exception: Throwable,
    list: Vec<ListElement1>,
    array: Vec<i64>,
    vector: Vec<ListElement1>,
    set: HashSet<String>,
    either: Result<bool, String>,
    tried: Result<ListElement2, Throwable>,
    option: Option<HashMap<String, ListElement2>>,
}

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
struct ListElement1 {
    id: String,
}

#[derive(Debug, Clone, PartialEq, BinaryCodec)]
#[sorted_constructors]
enum ListElement2 {
    First {
        elem: ListElement1,
    },
    #[evolution(FieldMadeTransient("cached"))]
    Second {
        uuid: Uuid,
        desc: Option<String>,
        #[transient(None)]
        _cached: Option<String>,
    },
    #[transient]
    #[allow(dead_code)]
    Third {
        _file: PathBuf,
    },
}

// Corresponds to desert-scala's PersistedThrowable structure it uses for serializing arbitrary Throwables
#[derive(Debug, Clone, PartialEq, BinaryCodec)]
struct Throwable {
    class_name: String,
    message: String,
    stack_trace: Vec<StackTraceElement>,
    cause: Option<Box<Throwable>>,
}

#[derive(Debug, Clone, PartialEq)]
struct StackTraceElement {
    class_name: Option<String>,
    method_name: Option<String>,
    file_name: Option<String>,
    line_number: u32,
}

impl BinarySerializer for StackTraceElement {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> desert_rust::Result<()> {
        context.write_u8(0);
        self.class_name.serialize(context)?;
        self.method_name.serialize(context)?;
        self.file_name.serialize(context)?;
        context.write_var_u32(self.line_number);
        Ok(())
    }
}

impl BinaryDeserializer for StackTraceElement {
    fn deserialize(context: &mut DeserializationContext<'_>) -> desert_rust::Result<Self> {
        let hdr = context.read_u8()?;
        assert_eq!(hdr, 0);
        let class_name = Option::<String>::deserialize(context)?;
        let method_name = Option::<String>::deserialize(context)?;
        let file_name = Option::<String>::deserialize(context)?;
        let line_number = context.read_var_u32()?;
        Ok(StackTraceElement {
            class_name,
            method_name,
            file_name,
            line_number,
        })
    }
}

#[test]
fn golden_test_1() {
    let bytes = include_bytes!("../golden/dataset1.bin");
    let value: TestModel1 = deserialize(bytes).unwrap();

    #[allow(clippy::approx_constant)]
    let expected = TestModel1 {
        byte: -10,
        short: 10000,
        int: -2000000000,
        long: 100000000001i64,
        float: 3.14,
        double: 0.1234e-10,
        boolean: false,
        unit: (),
        string: "Example data set".to_string(),
        uuid: Uuid::parse_str("d90c4285-544d-424d-885c-3940fe00883d").unwrap(),
        exception: Throwable {
            class_name: "java.lang.RuntimeException".to_string(),
            message: "Example exception".to_string(),
            stack_trace: vec![
                StackTraceElement {
                    class_name: Some("io.github.vigoo.desert.golden.TestModel1$".to_string()),
                    method_name: Some("generateException".to_string()),
                    file_name: Some("TestModel1.scala".to_string()),
                    line_number: 67,
                },
                StackTraceElement {
                    class_name: Some("io.github.vigoo.desert.golden.TestModel1$".to_string()),
                    method_name: Some("<clinit>".to_string()),
                    file_name: Some("TestModel1.scala".to_string()),
                    line_number: 84,
                },
                StackTraceElement {
                    class_name: Some("io.github.vigoo.desert.golden.Main$".to_string()),
                    method_name: Some("$anonfun$run$3".to_string()),
                    file_name: Some("Main.scala".to_string()),
                    line_number: 31,
                },
                StackTraceElement {
                    class_name: Some("zio.ZIO$FlatMap".to_string()),
                    method_name: Some("apply".to_string()),
                    file_name: Some("ZIO.scala".to_string()),
                    line_number: 5210,
                },
                StackTraceElement {
                    class_name: Some("zio.ZIO$FlatMap".to_string()),
                    method_name: Some("apply".to_string()),
                    file_name: Some("ZIO.scala".to_string()),
                    line_number: 5199,
                },
                StackTraceElement {
                    class_name: Some("zio.internal.FiberContext".to_string()),
                    method_name: Some("runUntil".to_string()),
                    file_name: Some("FiberContext.scala".to_string()),
                    line_number: 901,
                },
                StackTraceElement {
                    class_name: Some("zio.internal.FiberContext".to_string()),
                    method_name: Some("run".to_string()),
                    file_name: Some("FiberContext.scala".to_string()),
                    line_number: 111,
                },
                StackTraceElement {
                    class_name: Some("zio.Runtime".to_string()),
                    method_name: Some("unsafeRunWithRefs".to_string()),
                    file_name: Some("Runtime.scala".to_string()),
                    line_number: 400,
                },
                StackTraceElement {
                    class_name: Some("zio.Runtime".to_string()),
                    method_name: Some("unsafeRunWith".to_string()),
                    file_name: Some("Runtime.scala".to_string()),
                    line_number: 355,
                },
                StackTraceElement {
                    class_name: Some("zio.Runtime".to_string()),
                    method_name: Some("unsafeRunAsyncCancelable".to_string()),
                    file_name: Some("Runtime.scala".to_string()),
                    line_number: 308,
                },
                StackTraceElement {
                    class_name: Some("zio.Runtime".to_string()),
                    method_name: Some("unsafeRunAsyncCancelable$".to_string()),
                    file_name: Some("Runtime.scala".to_string()),
                    line_number: 304,
                },
                StackTraceElement {
                    class_name: Some("zio.Runtime$$anon$2".to_string()),
                    method_name: Some("unsafeRunAsyncCancelable".to_string()),
                    file_name: Some("Runtime.scala".to_string()),
                    line_number: 425,
                },
                StackTraceElement {
                    class_name: Some("zio.Runtime".to_string()),
                    method_name: Some("$anonfun$run$2".to_string()),
                    file_name: Some("Runtime.scala".to_string()),
                    line_number: 78,
                },
                StackTraceElement {
                    class_name: Some("zio.internal.FiberContext".to_string()),
                    method_name: Some("runUntil".to_string()),
                    file_name: Some("FiberContext.scala".to_string()),
                    line_number: 316,
                },
                StackTraceElement {
                    class_name: Some("zio.internal.FiberContext".to_string()),
                    method_name: Some("run".to_string()),
                    file_name: Some("FiberContext.scala".to_string()),
                    line_number: 111,
                },
                StackTraceElement {
                    class_name: Some("zio.internal.ZScheduler$$anon$3".to_string()),
                    method_name: Some("run".to_string()),
                    file_name: Some("ZScheduler.scala".to_string()),
                    line_number: 415,
                },
            ],
            cause: Some(Box::new(Throwable {
                class_name: "java.lang.IllegalArgumentException".to_string(),
                message: "param should not be negative".to_string(),
                stack_trace: vec![
                    StackTraceElement {
                        class_name: Some("io.github.vigoo.desert.golden.TestModel1$".to_string()),
                        method_name: Some("generateException".to_string()),
                        file_name: Some("TestModel1.scala".to_string()),
                        line_number: 67,
                    },
                    StackTraceElement {
                        class_name: Some("io.github.vigoo.desert.golden.TestModel1$".to_string()),
                        method_name: Some("<clinit>".to_string()),
                        file_name: Some("TestModel1.scala".to_string()),
                        line_number: 84,
                    },
                    StackTraceElement {
                        class_name: Some("io.github.vigoo.desert.golden.Main$".to_string()),
                        method_name: Some("$anonfun$run$3".to_string()),
                        file_name: Some("Main.scala".to_string()),
                        line_number: 31,
                    },
                    StackTraceElement {
                        class_name: Some("zio.ZIO$FlatMap".to_string()),
                        method_name: Some("apply".to_string()),
                        file_name: Some("ZIO.scala".to_string()),
                        line_number: 5210,
                    },
                    StackTraceElement {
                        class_name: Some("zio.ZIO$FlatMap".to_string()),
                        method_name: Some("apply".to_string()),
                        file_name: Some("ZIO.scala".to_string()),
                        line_number: 5199,
                    },
                    StackTraceElement {
                        class_name: Some("zio.internal.FiberContext".to_string()),
                        method_name: Some("runUntil".to_string()),
                        file_name: Some("FiberContext.scala".to_string()),
                        line_number: 901,
                    },
                    StackTraceElement {
                        class_name: Some("zio.internal.FiberContext".to_string()),
                        method_name: Some("run".to_string()),
                        file_name: Some("FiberContext.scala".to_string()),
                        line_number: 111,
                    },
                    StackTraceElement {
                        class_name: Some("zio.Runtime".to_string()),
                        method_name: Some("unsafeRunWithRefs".to_string()),
                        file_name: Some("Runtime.scala".to_string()),
                        line_number: 400,
                    },
                    StackTraceElement {
                        class_name: Some("zio.Runtime".to_string()),
                        method_name: Some("unsafeRunWith".to_string()),
                        file_name: Some("Runtime.scala".to_string()),
                        line_number: 355,
                    },
                    StackTraceElement {
                        class_name: Some("zio.Runtime".to_string()),
                        method_name: Some("unsafeRunAsyncCancelable".to_string()),
                        file_name: Some("Runtime.scala".to_string()),
                        line_number: 308,
                    },
                    StackTraceElement {
                        class_name: Some("zio.Runtime".to_string()),
                        method_name: Some("unsafeRunAsyncCancelable$".to_string()),
                        file_name: Some("Runtime.scala".to_string()),
                        line_number: 304,
                    },
                    StackTraceElement {
                        class_name: Some("zio.Runtime$$anon$2".to_string()),
                        method_name: Some("unsafeRunAsyncCancelable".to_string()),
                        file_name: Some("Runtime.scala".to_string()),
                        line_number: 425,
                    },
                    StackTraceElement {
                        class_name: Some("zio.Runtime".to_string()),
                        method_name: Some("$anonfun$run$2".to_string()),
                        file_name: Some("Runtime.scala".to_string()),
                        line_number: 78,
                    },
                    StackTraceElement {
                        class_name: Some("zio.internal.FiberContext".to_string()),
                        method_name: Some("runUntil".to_string()),
                        file_name: Some("FiberContext.scala".to_string()),
                        line_number: 316,
                    },
                    StackTraceElement {
                        class_name: Some("zio.internal.FiberContext".to_string()),
                        method_name: Some("run".to_string()),
                        file_name: Some("FiberContext.scala".to_string()),
                        line_number: 111,
                    },
                    StackTraceElement {
                        class_name: Some("zio.internal.ZScheduler$$anon$3".to_string()),
                        method_name: Some("run".to_string()),
                        file_name: Some("ZScheduler.scala".to_string()),
                        line_number: 415,
                    },
                ],
                cause: None,
            })),
        },
        list: vec![
            ListElement1 {
                id: "a".to_string(),
            },
            ListElement1 {
                id: "aa".to_string(),
            },
            ListElement1 {
                id: "aaa".to_string(),
            },
        ],
        array: (1i64..=30000i64).collect::<Vec<_>>(),
        vector: (1..=100)
            .map(|i| ListElement1 { id: i.to_string() })
            .collect(),
        set: HashSet::from_iter(["hello".to_string(), "world".to_string()]),
        either: Ok(true),
        tried: Ok(ListElement2::First {
            elem: ListElement1 { id: "".to_string() },
        }),
        option: Some(HashMap::from_iter(vec![
            (
                "first".to_string(),
                ListElement2::First {
                    elem: ListElement1 {
                        id: "1st".to_string(),
                    },
                },
            ),
            (
                "second".to_string(),
                ListElement2::Second {
                    uuid: Uuid::parse_str("0ca26648-edee-4a2d-bd88-eebf92d19c30").unwrap(),
                    desc: None,
                    _cached: None,
                },
            ),
            (
                "third".to_string(),
                ListElement2::Second {
                    uuid: Uuid::parse_str("0ca26648-edee-4a2d-bd88-eebf92d19c30").unwrap(),
                    desc: Some("some description".to_string()),
                    _cached: None,
                },
            ),
        ])),
    };

    check!(value.byte == expected.byte);
    check!(value.short == expected.short);
    check!(value.int == expected.int);
    check!(value.long == expected.long);
    check!(value.float == expected.float);
    check!(value.double == expected.double);
    check!(value.boolean == expected.boolean);
    check!(value.unit == expected.unit);
    check!(value.string == expected.string);
    check!(value.uuid == expected.uuid);
    check!(value.exception == expected.exception);
    check!(value.list == expected.list);
    check!(value.array == expected.array);
    check!(value.vector == expected.vector);
    check!(value.set == expected.set);
    check!(value.either == expected.either);
    check!(value.tried == expected.tried);
    check!(value.option == expected.option);

    let serialized = serialize_to_byte_vec(&value).unwrap();
    let value2 = deserialize(&serialized).unwrap();

    assert_eq!(value, value2);
}
