// Tests deserialization of a binary from the original Scala desert library

use desert_macro::BinaryCodec;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use uuid::Uuid;
use desert_core::{BinaryCodec, BinaryDeserializer, BinaryInput, BinaryOutput, BinarySerializer, deserialize_slice};

mod desert {
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
    list: Vec<Vec<ListElement1>>,
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
        cached: Option<String>,
    },
    #[transient]
    Third {
        file: PathBuf,
    },
}

// Corresponds to desert-scala's PersistedThrowable structure it uses for serializing arbitrary Throwables
#[derive(Debug, Clone, PartialEq, BinaryCodec)]
struct Throwable {
    class_name: String,
    message: String,
    stack_trace: Vec<StackTraceElement>,
    cause: Option<Box<Throwable>>
}

#[derive(Debug, Clone, PartialEq)]
struct StackTraceElement {
    class_name: String,
    method_name: String,
    file_name: String,
    line_number: u32,
}

impl BinarySerializer for StackTraceElement {
    fn serialize<Context: desert::SerializationContext>(&self, context: &mut Context) -> desert::Result<()> {
        context.output_mut().write_u8(0);
        self.class_name.serialize(context)?;
        self.method_name.serialize(context)?;
        self.file_name.serialize(context)?;
        context.output_mut().write_var_u32(self.line_number);
        Ok(())
    }
}

impl BinaryDeserializer for StackTraceElement {
    fn deserialize<Context: desert::DeserializationContext>(context: &mut Context) -> desert::Result<Self> {
        let _ = context.input_mut().read_u8()?;
        let class_name = String::deserialize(context)?;
        let method_name = String::deserialize(context)?;
        let file_name = String::deserialize(context)?;
        let line_number = context.input_mut().read_var_u32()?;
        Ok(StackTraceElement { class_name, method_name, file_name, line_number })
    }
}

#[test]
fn golden_test_1() {
    let bytes = include_bytes!("../golden/dataset1.bin");
    let value: TestModel1 = deserialize_slice(bytes).unwrap();
    println!("{:?}", value);
}
