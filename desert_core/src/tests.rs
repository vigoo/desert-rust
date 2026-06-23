use crate::{
    deserialize, deserialize_with_options, serialize_into_byte_vec,
    serialize_into_byte_vec_with_options, serialize_to_byte_vec, serialize_to_byte_vec_exact,
    serialize_to_byte_vec_with_capacity, serialize_to_byte_vec_with_options,
    serialize_to_byte_vec_with_options_and_capacity, serialize_to_byte_vec_with_options_exact,
    serialize_to_bytes, serialized_size, serialized_size_with_options, BinaryDeserializer,
    BinaryOutput, BinarySerializer, DeserializationContext, Options, SerializationContext,
};
use proptest::prelude::*;
use std::cell::RefCell;
use std::collections::{Bound, LinkedList};
use std::fmt::Debug;
use std::net::IpAddr;
use std::num::*;
use std::ops::Deref;
use std::ops::Range;
use std::rc::Rc;
use test_r::test;

pub(crate) fn roundtrip<T: BinarySerializer + BinaryDeserializer + Debug + Clone + PartialEq>(
    value: T,
) {
    roundtrip_with_options(value, Options::default());
}

pub(crate) fn roundtrip_with_options<
    T: BinarySerializer + BinaryDeserializer + Debug + Clone + PartialEq,
>(
    value: T,
    options: Options,
) {
    let data = serialize_to_byte_vec_with_options(&value, options.clone()).unwrap();
    let result = deserialize_with_options::<T>(&data, options).unwrap();
    assert_eq!(value, result);
}

#[test]
fn serialize_to_byte_vec_with_capacity_matches_default() {
    let value = (0..1024).collect::<Vec<u32>>();

    assert_eq!(
        serialize_to_byte_vec(&value).unwrap(),
        serialize_to_byte_vec_with_capacity(&value, 4096).unwrap()
    );
}

#[test]
fn serialize_to_byte_vec_with_options_and_capacity_matches_default() {
    let value = 'a';
    let options = Options::scala_compatible();

    assert_eq!(
        serialize_to_byte_vec_with_options(&value, options.clone()).unwrap(),
        serialize_to_byte_vec_with_options_and_capacity(&value, options, 16).unwrap()
    );
}

#[test]
fn serialize_to_byte_vec_exact_matches_default_and_size() {
    let value = (0..1024).collect::<Vec<u32>>();
    let exact = serialize_to_byte_vec_exact(&value).unwrap();

    assert_eq!(serialize_to_byte_vec(&value).unwrap(), exact);
    assert_eq!(serialized_size(&value).unwrap(), exact.len());
}

#[test]
fn serialize_to_byte_vec_with_options_exact_matches_default_and_size() {
    let value = 'a';
    let options = Options::scala_compatible();
    let exact = serialize_to_byte_vec_with_options_exact(&value, options.clone()).unwrap();

    assert_eq!(
        serialize_to_byte_vec_with_options(&value, options.clone()).unwrap(),
        exact
    );
    assert_eq!(
        serialized_size_with_options(&value, options).unwrap(),
        exact.len()
    );
}

#[test]
fn fixed_width_numeric_vec_serialization_keeps_iterable_format() {
    let value = (0..32).collect::<Vec<u32>>();
    let mut expected = Vec::new();
    expected.write_var_i32(value.len().try_into().unwrap());
    for item in &value {
        expected.write_u32(*item);
    }

    assert_eq!(serialize_to_byte_vec(&value).unwrap(), expected);
}

#[test]
fn serialize_into_byte_vec_reuses_capacity_and_clears_existing_data() {
    let value = (0..1024).collect::<Vec<u32>>();
    let expected = serialize_to_byte_vec(&value).unwrap();
    let mut output = Vec::with_capacity(expected.len() * 2);
    let original_capacity = output.capacity();
    output.extend_from_slice(b"stale data");

    serialize_into_byte_vec(&value, &mut output).unwrap();

    assert_eq!(output, expected);
    assert_eq!(output.capacity(), original_capacity);
}

#[test]
fn serialize_into_byte_vec_with_options_matches_default() {
    let value = 'a';
    let options = Options::scala_compatible();
    let expected = serialize_to_byte_vec_with_options(&value, options.clone()).unwrap();
    let mut output = Vec::with_capacity(16);

    serialize_into_byte_vec_with_options(&value, &mut output, options).unwrap();

    assert_eq!(output, expected);
}

#[test]
fn serialized_size_matches_serialized_len() {
    let value = (0..1024).collect::<Vec<u32>>();

    assert_eq!(
        serialized_size(&value).unwrap(),
        serialize_to_byte_vec(&value).unwrap().len()
    );
}

#[test]
fn serialized_size_with_options_matches_serialized_len() {
    let value = 'a';
    let options = Options::scala_compatible();

    assert_eq!(
        serialized_size_with_options(&value, options.clone()).unwrap(),
        serialize_to_byte_vec_with_options(&value, options)
            .unwrap()
            .len()
    );
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
    fn roundtrip_char_scala_compatible(value in any::<char>().prop_filter("only chars that can be encoded in 16 bits", |c| is_supported_char(*c))) {
        // NOTE: we don't support arbitrary chars when keeping scala compatibility, just the ones that can be represented as u16, to keep binary compatibility with the Scala version
        roundtrip_with_options(value, Options::scala_compatible());
    }

    #[test]
    fn roundtrip_char(value in any::<char>()) {
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
    fn roundtrip_byte_array(value: [u8; 16]) {
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
    fn roundtrip_string_array(value: [String; 16]) {
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

    #[test]
    fn range_roundtrip_i32(range in range_strategy::<i32>()) {
        roundtrip(range);
    }

    #[test]
    fn range_roundtrip_string(range in range_strategy::<String>()) {
        roundtrip(range);
    }
}

fn bound_strategy<T: Arbitrary + Clone + 'static>() -> impl Strategy<Value = Bound<T>> {
    prop_oneof![
        Just(Bound::Unbounded),
        any::<T>().prop_map(Bound::Included),
        any::<T>().prop_map(Bound::Excluded),
    ]
}

fn range_strategy<T: Arbitrary + Clone + 'static>() -> impl Strategy<Value = Range<T>> {
    (any::<T>(), any::<T>()).prop_map(|(start, end)| Range { start, end })
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

#[test]
fn range_i32() {
    let range: Range<i32> = Range { start: 10, end: 20 };
    roundtrip(range);
}

#[test]
fn range_string() {
    let range: Range<String> = Range {
        start: "hello".to_string(),
        end: "world".to_string(),
    };
    roundtrip(range);
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
                None => result.borrow_mut().next = Some(Rc::<RefCell<Node>>::deserialize(context)?),
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
