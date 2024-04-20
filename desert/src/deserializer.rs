use crate::binary_input::BinaryInput;
use crate::error::Result;
use crate::state::State;
use crate::{DeduplicatedString, Error, RefId, StringId};
use bytes::Bytes;
use castaway::cast;
use std::any::Any;
use std::char::DecodeUtf16Error;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList};
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

pub trait BinaryDeserializer: Sized {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self>;
}

pub trait DeserializationContext {
    type Input: BinaryInput;

    fn input_mut(&mut self) -> &mut Self::Input;
    fn state(&self) -> &State;
    fn state_mut(&mut self) -> &mut State;

    fn try_read_ref(&mut self) -> Result<Option<&dyn Any>> {
        let id = self.input_mut().read_var_u32()?;
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
}

pub struct Deserialization<Input: BinaryInput> {
    input: Input,
    state: State,
}

impl<Input: BinaryInput> Deserialization<Input> {
    pub fn new(input: Input) -> Self {
        Self {
            input,
            state: State::default(),
        }
    }
}

impl<Input: BinaryInput> DeserializationContext for Deserialization<Input> {
    type Input = Input;

    fn input_mut(&mut self) -> &mut Self::Input {
        &mut self.input
    }

    fn state(&self) -> &State {
        &self.state
    }

    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

impl BinaryDeserializer for u8 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_u8()
    }
}

impl BinaryDeserializer for i8 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_i8()
    }
}

impl BinaryDeserializer for u16 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_u16()
    }
}

impl BinaryDeserializer for i16 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_i16()
    }
}

impl BinaryDeserializer for u32 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_u32()
    }
}

impl BinaryDeserializer for i32 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_i32()
    }
}

impl BinaryDeserializer for u64 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_u64()
    }
}

impl BinaryDeserializer for i64 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_i64()
    }
}

impl BinaryDeserializer for u128 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_u128()
    }
}

impl BinaryDeserializer for i128 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_i128()
    }
}

impl BinaryDeserializer for f32 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_f32()
    }
}

impl BinaryDeserializer for f64 {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        context.input_mut().read_f64()
    }
}

impl BinaryDeserializer for bool {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        Ok(context.input_mut().read_u8()? != 0)
    }
}

impl BinaryDeserializer for () {
    fn deserialize<Context: DeserializationContext>(_: &mut Context) -> Result<Self> {
        Ok(())
    }
}

impl BinaryDeserializer for char {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let code = context.input_mut().read_u16()?;
        let chars: std::result::Result<Vec<char>, DecodeUtf16Error> =
            char::decode_utf16([code]).collect();
        Ok(chars?[0])
    }
}

impl BinaryDeserializer for String {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let id = context.input_mut().read_var_i32()?;
        let bytes = context.input_mut().read_bytes(id as usize)?;
        Ok(String::from_utf8(bytes)?)
    }
}

impl BinaryDeserializer for DeduplicatedString {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let count_or_id = context.input_mut().read_var_i32()?;
        if count_or_id < 0 {
            let id = StringId(-count_or_id);
            match context.state().get_string_by_id(id) {
                Some(s) => Ok(DeduplicatedString(s.to_string())),
                None => Err(Error::InvalidStringId(id)),
            }
        } else {
            let bytes = context.input_mut().read_bytes(count_or_id as usize)?;
            let s = String::from_utf8(bytes)?;
            context.state_mut().store_string(s.clone());
            Ok(DeduplicatedString(s))
        }
    }
}

impl BinaryDeserializer for Duration {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let seconds = context.input_mut().read_u64()?;
        let nanos = context.input_mut().read_u32()?;
        Ok(Duration::new(seconds, nanos))
    }
}

impl<T1: BinaryDeserializer, T2: BinaryDeserializer> BinaryDeserializer for (T1, T2) {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let version = context.input_mut().read_u8()?;
        if version == 0 {
            Ok((T1::deserialize(context)?, T2::deserialize(context)?))
        } else {
            // TODO: handle the struct evolution if possible
            Err(Error::DeserializationFailure(
                "Failed to deserialize tuple".to_string(),
            ))
        }
    }
}

impl<T1: BinaryDeserializer, T2: BinaryDeserializer, T3: BinaryDeserializer> BinaryDeserializer
    for (T1, T2, T3)
{
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let header = context.input_mut().read_u8()?;
        if header == 0 {
            Ok((
                T1::deserialize(context)?,
                T2::deserialize(context)?,
                T3::deserialize(context)?,
            ))
        } else {
            // TODO: handle the struct evolution if possible
            Err(Error::DeserializationFailure(
                "Failed to deserialize tuple".to_string(),
            ))
        }
    }
}

impl<
        T1: BinaryDeserializer,
        T2: BinaryDeserializer,
        T3: BinaryDeserializer,
        T4: BinaryDeserializer,
    > BinaryDeserializer for (T1, T2, T3, T4)
{
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let header = context.input_mut().read_u8()?;
        if header == 0 {
            Ok((
                T1::deserialize(context)?,
                T2::deserialize(context)?,
                T3::deserialize(context)?,
                T4::deserialize(context)?,
            ))
        } else {
            // TODO: handle the struct evolution if possible
            Err(Error::DeserializationFailure(
                "Failed to deserialize tuple".to_string(),
            ))
        }
    }
}

impl<
        T1: BinaryDeserializer,
        T2: BinaryDeserializer,
        T3: BinaryDeserializer,
        T4: BinaryDeserializer,
        T5: BinaryDeserializer,
    > BinaryDeserializer for (T1, T2, T3, T4, T5)
{
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let header = context.input_mut().read_u8()?;
        if header == 0 {
            Ok((
                T1::deserialize(context)?,
                T2::deserialize(context)?,
                T3::deserialize(context)?,
                T4::deserialize(context)?,
                T5::deserialize(context)?,
            ))
        } else {
            // TODO: handle the struct evolution if possible
            Err(Error::DeserializationFailure(
                "Failed to deserialize tuple".to_string(),
            ))
        }
    }
}

impl<
        T1: BinaryDeserializer,
        T2: BinaryDeserializer,
        T3: BinaryDeserializer,
        T4: BinaryDeserializer,
        T5: BinaryDeserializer,
        T6: BinaryDeserializer,
    > BinaryDeserializer for (T1, T2, T3, T4, T5, T6)
{
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let header = context.input_mut().read_u8()?;
        if header == 0 {
            Ok((
                T1::deserialize(context)?,
                T2::deserialize(context)?,
                T3::deserialize(context)?,
                T4::deserialize(context)?,
                T5::deserialize(context)?,
                T6::deserialize(context)?,
            ))
        } else {
            // TODO: handle the struct evolution if possible
            Err(Error::DeserializationFailure(
                "Failed to deserialize tuple".to_string(),
            ))
        }
    }
}

impl<
        T1: BinaryDeserializer,
        T2: BinaryDeserializer,
        T3: BinaryDeserializer,
        T4: BinaryDeserializer,
        T5: BinaryDeserializer,
        T6: BinaryDeserializer,
        T7: BinaryDeserializer,
    > BinaryDeserializer for (T1, T2, T3, T4, T5, T6, T7)
{
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let header = context.input_mut().read_u8()?;
        if header == 0 {
            Ok((
                T1::deserialize(context)?,
                T2::deserialize(context)?,
                T3::deserialize(context)?,
                T4::deserialize(context)?,
                T5::deserialize(context)?,
                T6::deserialize(context)?,
                T7::deserialize(context)?,
            ))
        } else {
            // TODO: handle the struct evolution if possible
            Err(Error::DeserializationFailure(
                "Failed to deserialize tuple".to_string(),
            ))
        }
    }
}

impl<
        T1: BinaryDeserializer,
        T2: BinaryDeserializer,
        T3: BinaryDeserializer,
        T4: BinaryDeserializer,
        T5: BinaryDeserializer,
        T6: BinaryDeserializer,
        T7: BinaryDeserializer,
        T8: BinaryDeserializer,
    > BinaryDeserializer for (T1, T2, T3, T4, T5, T6, T7, T8)
{
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let header = context.input_mut().read_u8()?;
        if header == 0 {
            Ok((
                T1::deserialize(context)?,
                T2::deserialize(context)?,
                T3::deserialize(context)?,
                T4::deserialize(context)?,
                T5::deserialize(context)?,
                T6::deserialize(context)?,
                T7::deserialize(context)?,
                T8::deserialize(context)?,
            ))
        } else {
            // TODO: handle the struct evolution if possible
            Err(Error::DeserializationFailure(
                "Failed to deserialize tuple".to_string(),
            ))
        }
    }
}

impl<T: BinaryDeserializer> BinaryDeserializer for Option<T> {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        match context.input_mut().read_u8()? {
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
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        match context.input_mut().read_u8()? {
            0 => Ok(Err(E::deserialize(context)?)),
            1 => Ok(Ok(R::deserialize(context)?)),
            other => Err(Error::DeserializationFailure(format!(
                "Failed to deserialize Result: invalid tag: {other}"
            ))),
        }
    }
}

impl BinaryDeserializer for Bytes {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let length = context.input_mut().read_var_u32()?; // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
        let bytes = context.input_mut().read_bytes(length as usize)?;
        Ok(Bytes::from(bytes))
    }
}

impl<T: BinaryDeserializer, const L: usize> BinaryDeserializer for [T; L] {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let empty: [T; 0] = [];
        if cast!(empty, [u8; 0]).is_ok() {
            let length = context.input_mut().read_var_u32()?; // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
            let bytes = context.input_mut().read_bytes(length as usize)?;
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

impl<T: BinaryDeserializer> BinaryDeserializer for Vec<T> {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        let empty: Self = Vec::new();
        if let Ok(_) = cast!(empty, Vec<u8>) {
            let length = context.input_mut().read_var_u32()?; // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
            let bytes = context.input_mut().read_bytes(length as usize)?;
            unsafe { Ok(std::mem::transmute(bytes)) }
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
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        deserialize_iterator(context).collect()
    }
}

impl<T: BinaryDeserializer + Ord> BinaryDeserializer for BTreeSet<T> {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        deserialize_iterator(context).collect()
    }
}

impl<K: BinaryDeserializer + Eq + Hash, V: BinaryDeserializer> BinaryDeserializer
    for HashMap<K, V>
{
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        deserialize_iterator(context).collect()
    }
}

impl<K: BinaryDeserializer + Ord, V: BinaryDeserializer> BinaryDeserializer for BTreeMap<K, V> {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        deserialize_iterator(context).collect()
    }
}

impl<T: BinaryDeserializer + Eq + Hash> BinaryDeserializer for LinkedList<T> {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        deserialize_iterator(context).collect()
    }
}

impl<T: BinaryDeserializer> BinaryDeserializer for Box<T> {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        Ok(Box::new(T::deserialize(context)?))
    }
}

impl<T: BinaryDeserializer> BinaryDeserializer for Rc<T> {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        Ok(Rc::new(T::deserialize(context)?))
    }
}

impl<T: BinaryDeserializer> BinaryDeserializer for Arc<T> {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> Result<Self> {
        Ok(Arc::new(T::deserialize(context)?))
    }
}

impl<T> BinaryDeserializer for PhantomData<T> {
    fn deserialize<Context: DeserializationContext>(_: &mut Context) -> Result<Self> {
        Ok(PhantomData)
    }
}

fn deserialize_iterator<'a, T: BinaryDeserializer + 'a, Context: DeserializationContext>(
    context: &'a mut Context,
) -> impl Iterator<Item = Result<T>> + 'a {
    match context.input_mut().read_var_i32() {
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

enum DeserializerIterator<'a, T: BinaryDeserializer + 'a, Context: DeserializationContext> {
    KnownSize {
        context: &'a mut Context,
        remaining: usize,
        element: PhantomData<T>,
    },
    UnknownSize {
        context: &'a mut Context,
        element: PhantomData<T>,
    },
    InputEndedUnexpectedly,
}

impl<'a, T: BinaryDeserializer, Context: DeserializationContext> Iterator
    for DeserializerIterator<'a, T, Context>
{
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
                    Some(T::deserialize(*context))
                }
            }
            DeserializerIterator::UnknownSize {
                ref mut context, ..
            } => match Option::<T>::deserialize(*context) {
                Ok(Some(item)) => Some(Ok(item)),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            },
        }
    }
}
