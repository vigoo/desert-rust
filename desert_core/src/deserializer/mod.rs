use std::any::Any;
use std::char::DecodeUtf16Error;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList};
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use castaway::cast;
use once_cell::unsync::Lazy;

use crate::binary_input::BinaryInput;
use crate::error::Result;
use crate::state::State;
use crate::{DeduplicatedString, Error, RefId, StringId};

#[allow(clippy::type_complexity)]
mod tuples;

pub trait BinaryDeserializer: Sized {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self>;
}

pub struct DeserializationContext<'a> {
    input: &'a [u8],
    state: Lazy<State>,
    region_stack: Vec<ResolvedInputRegion>,
    current: ResolvedInputRegion,
}

impl<'a> DeserializationContext<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        let whole_input = ResolvedInputRegion {
            start: 0,
            pos: 0,
            end: input.len(),
            delta: 0,
        };
        Self {
            input,
            state: Lazy::new(State::default),
            region_stack: vec![],
            current: whole_input,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn try_read_ref(&mut self) -> Result<Option<&dyn Any>> {
        let id = self.read_var_u32()?;
        if id == 0 {
            Ok(None)
        } else {
            let id = RefId(id);
            match self.state().get_ref_by_id(id) {
                Some(r) => Ok(Some(r)),
                None => Err(Error::InvalidRefId(id)),
            }
        }
    }

    pub(crate) fn push_region(&mut self, region: InputRegion) {
        let resolved_region = ResolvedInputRegion {
            start: self.current.start + region.start,
            pos: region.pos,
            end: self.current.start + region.end,
            delta: self.current.start,
        };
        self.region_stack.push(self.current);
        self.current = resolved_region;
    }

    pub(crate) fn pop_region(&mut self) -> InputRegion {
        let result = self.current.unresolve();
        self.current = self.region_stack.pop().unwrap();
        result
    }

    pub(crate) fn pos(&self) -> usize {
        self.current.pos
    }
}

impl BinaryInput for DeserializationContext<'_> {
    fn read_u8(&mut self) -> Result<u8> {
        if self.current.pos == self.current.end {
            Err(Error::InputEndedUnexpectedly)
        } else {
            self.current.pos += 1;
            Ok(self.input[self.current.start + self.current.pos - 1])
        }
    }

    fn read_bytes(&mut self, count: usize) -> Result<&[u8]> {
        if self.current.pos + count > self.current.end {
            Err(Error::InputEndedUnexpectedly)
        } else {
            let start = self.current.start + self.current.pos;
            self.current.pos += count;
            Ok(&self.input[start..(self.current.start + self.current.pos)])
        }
    }

    fn skip(&mut self, count: usize) -> Result<()> {
        if self.current.pos + count > self.current.end {
            Err(Error::InputEndedUnexpectedly)
        } else {
            self.current.pos += count;
            Ok(())
        }
    }
}

impl BinaryDeserializer for u8 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_u8()
    }
}

impl BinaryDeserializer for i8 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_i8()
    }
}

impl BinaryDeserializer for u16 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_u16()
    }
}

impl BinaryDeserializer for i16 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_i16()
    }
}

impl BinaryDeserializer for u32 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_u32()
    }
}

impl BinaryDeserializer for i32 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_i32()
    }
}

impl BinaryDeserializer for u64 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_u64()
    }
}

impl BinaryDeserializer for i64 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_i64()
    }
}

impl BinaryDeserializer for u128 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_u128()
    }
}

impl BinaryDeserializer for i128 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_i128()
    }
}

impl BinaryDeserializer for f32 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_f32()
    }
}

impl BinaryDeserializer for f64 {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_f64()
    }
}

impl BinaryDeserializer for usize {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        context.read_u64().map(|r| r as usize)
    }
}

impl BinaryDeserializer for bool {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        Ok(context.read_u8()? != 0)
    }
}

impl BinaryDeserializer for () {
    fn deserialize(_: &mut DeserializationContext<'_>) -> Result<Self> {
        Ok(())
    }
}

impl BinaryDeserializer for char {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let code = context.read_u16()?;
        let chars: std::result::Result<Vec<char>, DecodeUtf16Error> =
            char::decode_utf16([code]).collect();
        Ok(chars?[0])
    }
}

impl BinaryDeserializer for String {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let id = context.read_var_i32()?;
        let bytes = context.read_bytes(id as usize)?;
        Ok(String::from_utf8(bytes.to_vec())?)
    }
}

impl BinaryDeserializer for DeduplicatedString {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let count_or_id = context.read_var_i32()?;
        if count_or_id < 0 {
            let id = StringId(-count_or_id);
            match context.state().get_string_by_id(id) {
                Some(s) => Ok(DeduplicatedString(s.to_string())),
                None => Err(Error::InvalidStringId(id)),
            }
        } else {
            let bytes = context.read_bytes(count_or_id as usize)?;
            let s = String::from_utf8(bytes.to_vec())?;
            context.state_mut().store_string(s.clone());
            Ok(DeduplicatedString(s))
        }
    }
}

impl BinaryDeserializer for Duration {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let seconds = context.read_u64()?;
        let nanos = context.read_u32()?;
        Ok(Duration::new(seconds, nanos))
    }
}

impl<T: BinaryDeserializer> BinaryDeserializer for Option<T> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        match context.read_u8()? {
            0 => Ok(None),
            1 => Ok(Some(T::deserialize(context)?)),
            other => Err(Error::DeserializationFailure(format!(
                "Failed to deserialize Option: invalid tag: {other}"
            ))),
        }
    }
}

impl<R: BinaryDeserializer, E: BinaryDeserializer> BinaryDeserializer
    for std::result::Result<R, E>
{
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        match context.read_u8()? {
            0 => Ok(Err(E::deserialize(context)?)),
            1 => Ok(Ok(R::deserialize(context)?)),
            other => Err(Error::DeserializationFailure(format!(
                "Failed to deserialize Result: invalid tag: {other}"
            ))),
        }
    }
}

impl BinaryDeserializer for Bytes {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let length = context.read_var_u32()?; // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
        let bytes = context.read_bytes(length as usize)?;
        Ok(Bytes::from(bytes.to_vec()))
    }
}

impl<T: BinaryDeserializer, const L: usize> BinaryDeserializer for [T; L] {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let empty: [T; 0] = [];
        if cast!(empty, [u8; 0]).is_ok() {
            let length = context.read_var_u32()?; // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
            let bytes = context.read_bytes(length as usize)?;
            Ok(unsafe { std::mem::transmute_copy::<_, [T; L]>(&bytes) })
        } else {
            let mut array: [MaybeUninit<T>; L] = unsafe { MaybeUninit::uninit().assume_init() };
            for (target, item) in array.iter_mut().zip(deserialize_iterator(context)) {
                *target = MaybeUninit::new(item?);
            }
            let array: [T; L] = unsafe { std::mem::transmute_copy(&array) };
            Ok(array)
        }
    }
}

impl<T: BinaryDeserializer + 'static> BinaryDeserializer for Vec<T> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        use std::any::TypeId;

        if TypeId::of::<T>() == TypeId::of::<u8>() {
            let length = context.read_var_u32()?; // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
            let bytes = context.read_bytes(length as usize)?;
            unsafe { Ok(std::mem::transmute::<Vec<u8>, Vec<T>>(bytes.to_vec())) }
        } else {
            let mut vec = Vec::new();
            for item in deserialize_iterator(context) {
                vec.push(item?);
            }
            Ok(vec)
        }
    }
}

impl<T: BinaryDeserializer + Eq + Hash> BinaryDeserializer for HashSet<T> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        deserialize_iterator(context).collect()
    }
}

impl<T: BinaryDeserializer + Ord> BinaryDeserializer for BTreeSet<T> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        deserialize_iterator(context).collect()
    }
}

impl<K: BinaryDeserializer + Eq + Hash, V: BinaryDeserializer> BinaryDeserializer
    for HashMap<K, V>
{
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        deserialize_iterator(context).collect()
    }
}

impl<K: BinaryDeserializer + Ord, V: BinaryDeserializer> BinaryDeserializer for BTreeMap<K, V> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        deserialize_iterator(context).collect()
    }
}

impl<T: BinaryDeserializer + Eq + Hash> BinaryDeserializer for LinkedList<T> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        deserialize_iterator(context).collect()
    }
}

impl<T: BinaryDeserializer> BinaryDeserializer for Box<T> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        Ok(Box::new(T::deserialize(context)?))
    }
}

impl<T: BinaryDeserializer> BinaryDeserializer for Rc<T> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        Ok(Rc::new(T::deserialize(context)?))
    }
}

impl<T: BinaryDeserializer> BinaryDeserializer for Arc<T> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        Ok(Arc::new(T::deserialize(context)?))
    }
}

impl<T> BinaryDeserializer for PhantomData<T> {
    fn deserialize(_: &mut DeserializationContext<'_>) -> Result<Self> {
        Ok(PhantomData)
    }
}

pub fn deserialize_iterator<'a, 'b, T: BinaryDeserializer + 'a>(
    context: &'a mut DeserializationContext<'b>,
) -> DeserializerIterator<'a, 'b, T> {
    match context.read_var_i32() {
        Err(_) => DeserializerIterator::InputEndedUnexpectedly,
        Ok(-1) => DeserializerIterator::UnknownSize {
            context,
            element: PhantomData,
        },
        Ok(length) => DeserializerIterator::KnownSize {
            context,
            remaining: length as usize,
            element: PhantomData,
        },
    }
}

pub enum DeserializerIterator<'a, 'b, T: BinaryDeserializer + 'a> {
    KnownSize {
        context: &'a mut DeserializationContext<'b>,
        remaining: usize,
        element: PhantomData<T>,
    },
    UnknownSize {
        context: &'a mut DeserializationContext<'b>,
        element: PhantomData<T>,
    },
    InputEndedUnexpectedly,
}

impl<'a, T: BinaryDeserializer + 'a> Iterator for DeserializerIterator<'a, '_, T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DeserializerIterator::InputEndedUnexpectedly => {
                Some(Err(Error::InputEndedUnexpectedly))
            }
            DeserializerIterator::KnownSize {
                ref mut context,
                remaining,
                ..
            } => {
                if *remaining == 0 {
                    None
                } else {
                    *remaining -= 1;
                    Some(T::deserialize(context))
                }
            }
            DeserializerIterator::UnknownSize {
                ref mut context, ..
            } => match Option::<T>::deserialize(context) {
                Ok(Some(item)) => Some(Ok(item)),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            },
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct InputRegion {
    start: usize,
    pos: usize,
    end: usize,
}

impl InputRegion {
    pub(crate) fn new(start: usize, length: usize) -> Self {
        Self {
            start,
            pos: 0,
            end: start + length,
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            start: 0,
            pos: 0,
            end: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct ResolvedInputRegion {
    start: usize,
    pos: usize,
    end: usize,
    delta: usize,
}

impl ResolvedInputRegion {
    fn unresolve(self) -> InputRegion {
        InputRegion {
            start: self.start - self.delta,
            pos: self.pos,
            end: self.end - self.delta,
        }
    }
}
