use crate::{
    deserialize, serialize_to_byte_vec, serialize_to_bytes, BinaryDeserializer, BinaryOutput,
    BinarySerializer, DeserializationContext, SerializationContext,
};
use proptest::prelude::*;
use std::cell::RefCell;
use std::collections::{Bound, LinkedList};
use std::fmt::Debug;
use std::net::IpAddr;
use std::num::*;
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
        fn roundtrip_isize(value: isize) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_u8(value in (1u8..=u8::MAX).prop_map(|x| NonZeroU8::new(x).unwrap())) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_i8(value in any::<i8>().prop_filter("non-zero", |&x| x != 0).prop_map(|x| NonZeroI8::new(x).unwrap())) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_u16(value in (1u16..=u16::MAX).prop_map(|x| NonZeroU16::new(x).unwrap())) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_i16(value in any::<i16>().prop_filter("non-zero", |&x| x != 0).prop_map(|x| NonZeroI16::new(x).unwrap())) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_u32(value in (1u32..=u32::MAX).prop_map(|x| NonZeroU32::new(x).unwrap())) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_i32(value in any::<i32>().prop_filter("non-zero", |&x| x != 0).prop_map(|x| NonZeroI32::new(x).unwrap())) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_u64(value in (1u64..=u64::MAX).prop_map(|x| NonZeroU64::new(x).unwrap())) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_i64(value in any::<i64>().prop_filter("non-zero", |&x| x != 0).prop_map(|x| NonZeroI64::new(x).unwrap())) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_u128(value in (1u128..=u128::MAX).prop_map(|x| NonZeroU128::new(x).unwrap())) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_i128(value in any::<i128>().prop_filter("non-zero", |&x| x != 0).prop_map(|x| NonZeroI128::new(x).unwrap())) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_usize(value in (1usize..=usize::MAX).prop_map(|x| NonZeroUsize::new(x).unwrap())) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_non_zero_isize(value in any::<isize>().prop_filter("non-zero", |&x| x != 0).prop_map(|x| NonZeroIsize::new(x).unwrap())) {
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
        fn roundtrip_vec_u8(value: Vec<u8>) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_vec_u32(value: Vec<u32>) {
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

        #[test]
        fn roundtrip_ipaddr(value: IpAddr) {
            roundtrip(value);
        }

        #[test]
        fn bound_roundtrip_i32(bound in bound_strategy::<i32>()) {
            roundtrip(bound);
        }

        #[test]
        fn bound_roundtrip_string(bound in bound_strategy::<String>()) {
            roundtrip(bound);
        }
    }

fn bound_strategy<T: Arbitrary + Clone + 'static>() -> impl Strategy<Value = Bound<T>> {
    prop_oneof![
            Just(Bound::Unbounded),
            any::<T>().prop_map(Bound::Included),
            any::<T>().prop_map(Bound::Excluded),
        ]
}

#[test]
fn bound_unbounded_i32() {
    let bound: Bound<i32> = Bound::Unbounded;
    roundtrip(bound);
}

#[test]
fn bound_included_i32() {
    let bound: Bound<i32> = Bound::Included(42);
    roundtrip(bound);
}

#[test]
fn bound_excluded_i32() {
    let bound: Bound<i32> = Bound::Excluded(-10);
    roundtrip(bound);
}

#[test]
fn bound_unbounded_string() {
    let bound: Bound<String> = Bound::Unbounded;
    roundtrip(bound);
}

#[test]
fn bound_included_string() {
    let bound: Bound<String> = Bound::Included("hello".to_string());
    roundtrip(bound);
}

#[test]
fn bound_excluded_string() {
    let bound: Bound<String> = Bound::Excluded("world".to_string());
    roundtrip(bound);
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