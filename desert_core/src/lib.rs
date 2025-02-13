pub mod adt;
mod binary_input;
mod binary_output;
mod deserializer;
mod error;
mod evolution;
mod features;
pub mod serializer;
mod state;

use bytes::{Bytes, BytesMut};
use std::fmt::{Display, Formatter};

pub use binary_input::{BinaryInput, OwnedInput, SliceInput};
pub use binary_output::{BinaryOutput, SizeCalculator};
pub use deserializer::{BinaryDeserializer, DeserializationContext};
pub use error::{Error, Result};
pub use evolution::Evolution;
pub use serializer::{serialize_iterator, BinarySerializer, SerializationContext};

#[cfg(test)]
test_r::enable!();

pub trait BinaryCodec: BinarySerializer + BinaryDeserializer {}

impl<T: BinarySerializer + BinaryDeserializer> BinaryCodec for T {}

const DEFAULT_CAPACITY: usize = 128;

pub fn serialize<T: BinarySerializer, O: BinaryOutput>(value: &T, output: O) -> Result<O> {
    let mut context = SerializationContext::new(output);
    value.serialize(&mut context)?;
    Ok(context.into_output())
}

pub fn deserialize<T: BinaryDeserializer>(input: &[u8]) -> Result<T> {
    let mut context = DeserializationContext::new(input);
    T::deserialize(&mut context)
}

pub fn serialize_to_bytes<T: BinarySerializer>(value: &T) -> Result<Bytes> {
    Ok(serialize(value, BytesMut::with_capacity(DEFAULT_CAPACITY))?.freeze())
}

pub fn serialize_to_byte_vec<T: BinarySerializer>(value: &T) -> Result<Vec<u8>> {
    serialize(value, Vec::with_capacity(DEFAULT_CAPACITY))
}

/// Wrapper for strings, enabling desert's string deduplication mode.
///
/// The library have a simple deduplication system, without sacrificing any extra
/// bytes for cases when strings are not duplicate. In general, the strings are encoded by a variable length
/// int representing the length of the string in bytes, followed by its UTF-8 encoding.
/// When deduplication is enabled (the string values are wrapped in `DeduplicatedString`) , each serialized
/// string gets an ID and if it is serialized once more in the same stream, a negative number in place of the
/// length identifies it.
///
/// It is not turned on by default because it breaks backward compatibility when evolving data structures.
/// If a new string field is added, old versions of the application will skip it and would not assign the
/// same ID to the string if it is first seen.
pub struct DeduplicatedString(pub String);

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct StringId(pub i32);

impl StringId {
    fn next(&mut self) {
        self.0 += 1;
    }
}

impl Display for StringId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RefId(pub u32);

impl RefId {
    fn next(&mut self) {
        self.0 += 1;
    }
}

impl Display for RefId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        deserialize, serialize_to_byte_vec, serialize_to_bytes, BinaryDeserializer, BinaryOutput,
        BinarySerializer, DeserializationContext, SerializationContext,
    };
    use proptest::prelude::*;
    use std::cell::RefCell;
    use std::collections::LinkedList;
    use std::fmt::Debug;
    use std::ops::Deref;
    use std::rc::Rc;
    use test_r::test;

    pub(crate) fn roundtrip<
        T: BinarySerializer + BinaryDeserializer + Debug + Clone + PartialEq,
    >(
        value: T,
    ) {
        let data = serialize_to_byte_vec(&value).unwrap();
        let result = deserialize::<T>(&data).unwrap();
        assert_eq!(value, result);
    }

    fn is_supported_char(char: char) -> bool {
        let code = char as u32;
        let code: Result<u16, _> = code.try_into();
        code.is_ok()
    }

    proptest! {
        #[test]
        fn roundtrip_i8(value: i8) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_i16(value: i16) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_i32(value: i32) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_i64(value: i64) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_i128(value: i128) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_u8(value: u8) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_u16(value: u16) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_u32(value: u32) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_u64(value: u64) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_u128(value: u128) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_f32(value: f32) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_f64(value: f64) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_bool(value: bool) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_char(value in any::<char>().prop_filter("only chars that can be encoded in 16 bits", |c| is_supported_char(*c))) {
            // NOTE: we don't support arbitrary chars, just the ones that can be represented as u16, to keep binary compatibility with the Scala version
            roundtrip(value);
        }

        #[test]
        fn roundtrip_string(value: String) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_bytes(value: Vec<u8>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_option(value: Option<u32>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_vec(value: Vec<String>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple2(value: (u32, String)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple3(value: (u32, String, bool)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple4(value: (u32, String, bool, u64)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple5(value: (u32, String, bool, u64, i32)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple6(value: (u32, String, bool, u64, i32, i64)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple7(value: (u32, String, bool, u64, i32, i64, u128)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tuple8(value: (u32, String, bool, u64, i32, i64, u128, i128)) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_sized_array(value: [u32; 3]) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_hashset(value: std::collections::HashSet<String>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_btreeset(value: std::collections::HashSet<u64>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_hashmap(value: std::collections::HashMap<String, u32>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_btreemap(value: std::collections::BTreeMap<String, u32>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_result(value: Result<u32, String>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_linked_list(value: LinkedList<String>) {
            roundtrip(value);
        }
    }

    #[derive(Debug, Clone)]
    struct Node {
        label: String,
        next: Option<Rc<RefCell<Node>>>,
    }

    impl BinarySerializer for Rc<RefCell<Node>> {
        fn serialize<Output: BinaryOutput>(
            &self,
            context: &mut SerializationContext<Output>,
        ) -> crate::Result<()> {
            let node = self.borrow();
            node.label.serialize(context)?;
            match &node.next {
                Some(next) => {
                    true.serialize(context)?;
                    if context.store_ref_or_object(next)? {
                        next.serialize(context)?;
                    }
                }
                None => {
                    false.serialize(context)?;
                }
            }
            Ok(())
        }
    }

    impl BinaryDeserializer for Rc<RefCell<Node>> {
        fn deserialize(context: &mut DeserializationContext<'_>) -> crate::Result<Self> {
            let label = String::deserialize(context)?;
            let result = Rc::new(RefCell::new(Node { label, next: None }));
            context.state_mut().store_ref(&result);
            let has_next = bool::deserialize(context)?;
            if has_next {
                match context.try_read_ref()? {
                    Some(next) => {
                        result.borrow_mut().next =
                            Some(next.downcast_ref::<Rc<RefCell<Node>>>().unwrap().clone())
                    }
                    None => {
                        result.borrow_mut().next = Some(Rc::<RefCell<Node>>::deserialize(context)?)
                    }
                }
            }
            Ok(result)
        }
    }

    #[derive(Debug, Clone)]
    struct Root {
        node: Rc<RefCell<Node>>,
    }

    impl BinarySerializer for Root {
        fn serialize<Output: BinaryOutput>(
            &self,
            context: &mut SerializationContext<Output>,
        ) -> crate::Result<()> {
            if context.store_ref_or_object(&self.node)? {
                self.node.serialize(context)?;
            }
            Ok(())
        }
    }

    impl BinaryDeserializer for Root {
        fn deserialize(context: &mut DeserializationContext<'_>) -> crate::Result<Self> {
            let node = match context.try_read_ref()? {
                Some(node) => node.downcast_ref::<Rc<RefCell<Node>>>().unwrap().clone(),
                None => Rc::<RefCell<Node>>::deserialize(context)?,
            };
            Ok(Root { node })
        }
    }

    #[test]
    fn known_sized_collection_is_stack_safe() {
        let big_vec = (0..1_000_000).collect::<Vec<_>>();
        roundtrip(big_vec);
    }

    #[test]
    fn reference_tracking_serializes_cycles() {
        let a = Rc::new(RefCell::new(Node {
            label: "a".to_string(),
            next: None,
        }));
        let b = Rc::new(RefCell::new(Node {
            label: "b".to_string(),
            next: None,
        }));
        let c = Rc::new(RefCell::new(Node {
            label: "c".to_string(),
            next: None,
        }));

        a.borrow_mut().next = Some(b.clone());
        b.borrow_mut().next = Some(c.clone());
        c.borrow_mut().next = Some(a.clone());

        let root = Root { node: a.clone() };

        let data = serialize_to_bytes(&root).unwrap();
        let _result = deserialize::<Root>(&data).unwrap();

        let a = root.node;
        let b = a.borrow().next.clone().unwrap();
        let c = b.borrow().next.clone().unwrap();

        assert_eq!(a.borrow().label, "a".to_string());
        assert_eq!(b.borrow().label, "b".to_string());
        assert_eq!(c.borrow().label, "c".to_string());

        let d = c.borrow().next.clone().unwrap();
        assert!(std::ptr::eq(d.borrow().deref(), a.borrow().deref()));
    }
}
