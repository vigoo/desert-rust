use bytes::Bytes;
use castaway::cast;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use crate::binary_output::BinaryOutput;
use crate::error::Result;
use crate::state::State;
use crate::storable::StorableRef;
use crate::{DeduplicatedString, Error, RefId, StringId};

pub trait BinarySerializer {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()>;
}

pub trait SerializationContext {
    type Output: BinaryOutput;

    fn output_mut(&mut self) -> &mut Self::Output;
    fn state_mut(&mut self) -> &mut State;

    fn store_ref_or_object(&mut self, value: Rc<dyn StorableRef>) -> Result<bool> {
        match self.state_mut().store_ref(value) {
            StoreRefResult::RefAlreadyStored { id } => {
                self.output_mut().write_var_u32(id.0);
                Ok(false)
            }
            StoreRefResult::RefIsNew { new_id, value } => {
                self.output_mut().write_var_u32(0);
                Ok(true)
            }
        }
    }
}

pub struct Serialization<Output: BinaryOutput> {
    output: Output,
    state: State,
}

impl<Output: BinaryOutput> Serialization<Output> {
    pub fn new(output: Output) -> Self {
        Serialization {
            output,
            state: State::default(),
        }
    }

    pub fn into_output(self) -> Output {
        self.output
    }
}

impl<Output: BinaryOutput> SerializationContext for Serialization<Output> {
    type Output = Output;

    fn output_mut(&mut self) -> &mut Self::Output {
        &mut self.output
    }

    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

pub enum StoreStringResult {
    StringAlreadyStored { id: StringId },
    StringIsNew { new_id: StringId, value: String },
}

pub enum StoreRefResult {
    RefAlreadyStored {
        id: RefId,
    },
    RefIsNew {
        new_id: RefId,
        value: Rc<dyn StorableRef>,
    },
}

impl BinarySerializer for u8 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u8(*self);
        Ok(())
    }
}

impl BinarySerializer for i8 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_i8(*self);
        Ok(())
    }
}

impl BinarySerializer for u16 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u16(*self);
        Ok(())
    }
}

impl BinarySerializer for i16 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_i16(*self);
        Ok(())
    }
}

impl BinarySerializer for u32 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u32(*self);
        Ok(())
    }
}

impl BinarySerializer for i32 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_i32(*self);
        Ok(())
    }
}

impl BinarySerializer for u64 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u64(*self);
        Ok(())
    }
}

impl BinarySerializer for i64 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_i64(*self);
        Ok(())
    }
}

impl BinarySerializer for u128 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u128(*self);
        Ok(())
    }
}

impl BinarySerializer for i128 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_i128(*self);
        Ok(())
    }
}

impl BinarySerializer for f32 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_f32(*self);
        Ok(())
    }
}

impl BinarySerializer for f64 {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_f64(*self);
        Ok(())
    }
}

impl BinarySerializer for bool {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u8(if *self { 1 } else { 0 });
        Ok(())
    }
}

impl BinarySerializer for () {
    fn serialize<Context: SerializationContext>(&self, _context: &mut Context) -> Result<()> {
        Ok(())
    }
}

impl BinarySerializer for char {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        let mut buf = [0; 2];
        let result = self.encode_utf16(&mut buf);
        if result.len() == 1 {
            context.output_mut().write_u16(result[0]);
            Ok(())
        } else {
            Err(Error::UnsupportedCharacter(*self))
        }
    }
}

impl BinarySerializer for str {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        let utf8 = self.as_bytes();
        context.output_mut().write_var_i32(utf8.len().try_into()?);
        context.output_mut().write_bytes(utf8);
        Ok(())
    }
}

impl BinarySerializer for String {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        let utf8 = self.as_bytes();
        context.output_mut().write_var_i32(utf8.len().try_into()?);
        context.output_mut().write_bytes(utf8);
        Ok(())
    }
}

impl BinarySerializer for DeduplicatedString {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        match context.state_mut().store_string(self.0.clone()) {
            StoreStringResult::StringAlreadyStored { id } => {
                context.output_mut().write_var_i32(-id.0);
                Ok(())
            }
            StoreStringResult::StringIsNew { value, .. } => value.serialize(context),
        }
    }
}

impl BinarySerializer for Duration {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u64(self.as_secs());
        context.output_mut().write_u32(self.subsec_nanos());
        Ok(())
    }
}

impl<T1: BinarySerializer, T2: BinarySerializer> BinarySerializer for (T1, T2) {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u8(0);
        self.0.serialize(context)?;
        self.1.serialize(context)?;
        Ok(())
    }
}

impl<T1: BinarySerializer, T2: BinarySerializer, T3: BinarySerializer> BinarySerializer
    for (T1, T2, T3)
{
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u8(0);
        self.0.serialize(context)?;
        self.1.serialize(context)?;
        self.2.serialize(context)?;
        Ok(())
    }
}

impl<T1: BinarySerializer, T2: BinarySerializer, T3: BinarySerializer, T4: BinarySerializer>
    BinarySerializer for (T1, T2, T3, T4)
{
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u8(0);
        self.0.serialize(context)?;
        self.1.serialize(context)?;
        self.2.serialize(context)?;
        self.3.serialize(context)?;
        Ok(())
    }
}

impl<
        T1: BinarySerializer,
        T2: BinarySerializer,
        T3: BinarySerializer,
        T4: BinarySerializer,
        T5: BinarySerializer,
    > BinarySerializer for (T1, T2, T3, T4, T5)
{
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u8(0);
        self.0.serialize(context)?;
        self.1.serialize(context)?;
        self.2.serialize(context)?;
        self.3.serialize(context)?;
        self.4.serialize(context)?;
        Ok(())
    }
}

impl<
        T1: BinarySerializer,
        T2: BinarySerializer,
        T3: BinarySerializer,
        T4: BinarySerializer,
        T5: BinarySerializer,
        T6: BinarySerializer,
    > BinarySerializer for (T1, T2, T3, T4, T5, T6)
{
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u8(0);
        self.0.serialize(context)?;
        self.1.serialize(context)?;
        self.2.serialize(context)?;
        self.3.serialize(context)?;
        self.4.serialize(context)?;
        self.5.serialize(context)?;
        Ok(())
    }
}

impl<
        T1: BinarySerializer,
        T2: BinarySerializer,
        T3: BinarySerializer,
        T4: BinarySerializer,
        T5: BinarySerializer,
        T6: BinarySerializer,
        T7: BinarySerializer,
    > BinarySerializer for (T1, T2, T3, T4, T5, T6, T7)
{
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u8(0);
        self.0.serialize(context)?;
        self.1.serialize(context)?;
        self.2.serialize(context)?;
        self.3.serialize(context)?;
        self.4.serialize(context)?;
        self.5.serialize(context)?;
        self.6.serialize(context)?;
        Ok(())
    }
}

impl<
        T1: BinarySerializer,
        T2: BinarySerializer,
        T3: BinarySerializer,
        T4: BinarySerializer,
        T5: BinarySerializer,
        T6: BinarySerializer,
        T7: BinarySerializer,
        T8: BinarySerializer,
    > BinarySerializer for (T1, T2, T3, T4, T5, T6, T7, T8)
{
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_u8(0);
        self.0.serialize(context)?;
        self.1.serialize(context)?;
        self.2.serialize(context)?;
        self.3.serialize(context)?;
        self.4.serialize(context)?;
        self.5.serialize(context)?;
        self.6.serialize(context)?;
        self.7.serialize(context)?;
        Ok(())
    }
}

impl<T: BinarySerializer> BinarySerializer for Option<T> {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        match self {
            Some(value) => {
                context.output_mut().write_u8(1);
                value.serialize(context)
            }
            None => {
                context.output_mut().write_u8(0);
                Ok(())
            }
        }
    }
}

impl<R: BinarySerializer, E: BinarySerializer> BinarySerializer for std::result::Result<R, E> {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        match self {
            Ok(value) => {
                context.output_mut().write_u8(1);
                value.serialize(context)
            }
            Err(error) => {
                context.output_mut().write_u8(0);
                error.serialize(context)
            }
        }
    }
}

impl BinarySerializer for Bytes {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_var_i32(self.len().try_into()?);
        context.output_mut().write_bytes(self);
        Ok(())
    }
}

impl<T: BinarySerializer + 'static> BinarySerializer for [T] {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_var_i32(self.len().try_into()?);
        if let Ok(byte_slice) = cast!(self, &[u8]) {
            context.output_mut().write_bytes(byte_slice);
        } else {
            for elem in self {
                elem.serialize(context)?;
            }
        }
        Ok(())
    }
}

impl<T: BinarySerializer, const L: usize> BinarySerializer for [T; L] {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        context.output_mut().write_var_i32(self.len().try_into()?);
        if let Ok(byte_slice) = cast!(self, &[u8; L]) {
            context.output_mut().write_bytes(byte_slice);
        } else {
            for elem in self {
                elem.serialize(context)?;
            }
        }
        Ok(())
    }
}

impl<T: BinarySerializer> BinarySerializer for Vec<T> {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        if let Ok(byte_vec) = cast!(self, &Vec<u8>) {
            context
                .output_mut()
                .write_var_i32(byte_vec.len().try_into()?);
            context.output_mut().write_bytes(byte_vec);
            Ok(())
        } else {
            serialize_iterator(&mut self.iter(), context)
        }
    }
}

impl<T: BinarySerializer> BinarySerializer for HashSet<T> {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        serialize_iterator(&mut self.iter(), context)
    }
}

impl<T: BinarySerializer> BinarySerializer for BTreeSet<T> {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        serialize_iterator(&mut self.iter(), context)
    }
}

impl<K: BinarySerializer, V: BinarySerializer> BinarySerializer for HashMap<K, V> {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        serialize_iterator(&mut self.iter(), context)
    }
}

impl<K: BinarySerializer, V: BinarySerializer> BinarySerializer for BTreeMap<K, V> {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        serialize_iterator(&mut self.iter(), context)
    }
}

impl<T: BinarySerializer> BinarySerializer for LinkedList<T> {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        serialize_iterator(&mut self.iter(), context)
    }
}

impl<T: BinarySerializer> BinarySerializer for Box<T> {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        (**self).serialize(context)
    }
}

impl<T: BinarySerializer + ?Sized> BinarySerializer for Rc<T> {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        (**self).serialize(context)
    }
}

impl<T: BinarySerializer> BinarySerializer for Arc<T> {
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        (**self).serialize(context)
    }
}

impl<T> BinarySerializer for PhantomData<T> {
    fn serialize<Context: SerializationContext>(&self, _context: &mut Context) -> Result<()> {
        Ok(())
    }
}

impl<'a, T> BinarySerializer for &'a T
where
    T: BinarySerializer + ?Sized,
{
    fn serialize<Context: SerializationContext>(&self, context: &mut Context) -> Result<()> {
        (*self).serialize(context)
    }
}

/// Helper function for implementing serialization of any iterable data source, keeping a format which is
/// compatible with both known and unknown sized iterables, allowing replacing data structures without breaking
/// the serialization format.
///
/// All the built-in `BinarySerializer` implementations for iterables use this function (or at least the same binary format).
pub fn serialize_iterator<
    I: Iterator<Item = T>,
    T: BinarySerializer,
    Context: SerializationContext,
>(
    iter: &mut I,
    context: &mut Context,
) -> Result<()> {
    match iter.size_hint() {
        (min, Some(max)) if min == max => {
            context.output_mut().write_var_i32(min.try_into()?);
            for item in iter {
                item.serialize(context)?;
            }
        }
        _ => {
            context.output_mut().write_var_i32(-1);
            for item in iter {
                context.output_mut().write_u8(1);
                item.serialize(context)?;
            }
            context.output_mut().write_u8(0);
        }
    }
    Ok(())
}
