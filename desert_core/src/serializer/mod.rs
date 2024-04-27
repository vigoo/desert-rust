mod tuples;

use bytes::Bytes;
use castaway::cast;
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use crate::binary_output::BinaryOutput;
use crate::error::Result;
use crate::state::State;
use crate::{DeduplicatedString, Error, RefId, StringId};

pub trait BinarySerializer {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()>;
}

pub struct SerializationContext<Output: BinaryOutput> {
    output: Output,
    state: State,
    buffer_stack: Vec<Vec<u8>>, // TODO: remove it once AdtSerializer does not need it anymore
}

impl<Output: BinaryOutput> SerializationContext<Output> {
    pub fn new(output: Output) -> Self {
        Self {
            output,
            state: State::default(),
            buffer_stack: Vec::new(),
        }
    }

    pub fn into_output(self) -> Output {
        self.output
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn store_ref_or_object(&mut self, value: &impl Any) -> Result<bool> {
        match self.state_mut().store_ref(value) {
            StoreRefResult::RefAlreadyStored { id } => {
                self.write_var_u32(id.0);
                Ok(false)
            }
            StoreRefResult::RefIsNew { .. } => {
                self.write_var_u32(0);
                Ok(true)
            }
        }
    }

    pub fn push_buffer(&mut self, buffer: Vec<u8>) {
        self.buffer_stack.push(buffer);
    }

    pub fn pop_buffer(&mut self) -> Vec<u8> {
        self.buffer_stack.pop().unwrap()
    }
}

impl<Output: BinaryOutput> BinaryOutput for SerializationContext<Output> {
    fn write_u8(&mut self, value: u8) {
        match self.buffer_stack.last_mut() {
            Some(buffer) => buffer.write_u8(value),
            None => self.output.write_u8(value),
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        match self.buffer_stack.last_mut() {
            Some(buffer) => buffer.write_bytes(bytes),
            None => self.output.write_bytes(bytes),
        }
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
        value: *const dyn Any,
    },
}

impl BinarySerializer for u8 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u8(*self);
        Ok(())
    }
}

impl BinarySerializer for i8 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_i8(*self);
        Ok(())
    }
}

impl BinarySerializer for u16 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u16(*self);
        Ok(())
    }
}

impl BinarySerializer for i16 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_i16(*self);
        Ok(())
    }
}

impl BinarySerializer for u32 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u32(*self);
        Ok(())
    }
}

impl BinarySerializer for i32 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_i32(*self);
        Ok(())
    }
}

impl BinarySerializer for u64 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u64(*self);
        Ok(())
    }
}

impl BinarySerializer for i64 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_i64(*self);
        Ok(())
    }
}

impl BinarySerializer for u128 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u128(*self);
        Ok(())
    }
}

impl BinarySerializer for i128 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_i128(*self);
        Ok(())
    }
}

impl BinarySerializer for f32 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_f32(*self);
        Ok(())
    }
}

impl BinarySerializer for f64 {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_f64(*self);
        Ok(())
    }
}

impl BinarySerializer for bool {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u8(if *self { 1 } else { 0 });
        Ok(())
    }
}

impl BinarySerializer for () {
    fn serialize<Output: BinaryOutput>(
        &self,
        _context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        Ok(())
    }
}

impl BinarySerializer for char {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        let mut buf = [0; 2];
        let result = self.encode_utf16(&mut buf);
        if result.len() == 1 {
            context.write_u16(result[0]);
            Ok(())
        } else {
            Err(Error::UnsupportedCharacter(*self))
        }
    }
}

impl BinarySerializer for str {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        let utf8 = self.as_bytes();
        context.write_var_i32(utf8.len().try_into()?);
        context.write_bytes(utf8);
        Ok(())
    }
}

impl BinarySerializer for String {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        let utf8 = self.as_bytes();
        context.write_var_i32(utf8.len().try_into()?);
        context.write_bytes(utf8);
        Ok(())
    }
}

impl BinarySerializer for DeduplicatedString {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        match context.state_mut().store_string(self.0.clone()) {
            StoreStringResult::StringAlreadyStored { id } => {
                context.write_var_i32(-id.0);
                Ok(())
            }
            StoreStringResult::StringIsNew { value, .. } => value.serialize(context),
        }
    }
}

impl BinarySerializer for Duration {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u64(self.as_secs());
        context.write_u32(self.subsec_nanos());
        Ok(())
    }
}

impl<T: BinarySerializer> BinarySerializer for Option<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        match self {
            Some(value) => {
                context.write_u8(1);
                value.serialize(context)
            }
            None => {
                context.write_u8(0);
                Ok(())
            }
        }
    }
}

impl<R: BinarySerializer, E: BinarySerializer> BinarySerializer for std::result::Result<R, E> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        match self {
            Ok(value) => {
                context.write_u8(1);
                value.serialize(context)
            }
            Err(error) => {
                context.write_u8(0);
                error.serialize(context)
            }
        }
    }
}

impl BinarySerializer for Bytes {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_var_u32(self.len().try_into()?); // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
        context.write_bytes(self);
        Ok(())
    }
}

impl<T: BinarySerializer + 'static> BinarySerializer for [T] {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        if let Ok(byte_slice) = cast!(self, &[u8]) {
            context.write_var_u32(self.len().try_into()?); // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
            context.write_bytes(byte_slice);
        } else {
            context.write_var_i32(self.len().try_into()?);
            for elem in self {
                elem.serialize(context)?;
            }
        }
        Ok(())
    }
}

impl<T: BinarySerializer, const L: usize> BinarySerializer for [T; L] {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        if let Ok(byte_slice) = cast!(self, &[u8; L]) {
            context.write_var_u32(self.len().try_into()?); // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
            context.write_bytes(byte_slice);
        } else {
            context.write_var_i32(self.len().try_into()?);
            for elem in self {
                elem.serialize(context)?;
            }
        }
        Ok(())
    }
}

impl<T: BinarySerializer> BinarySerializer for Vec<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        if let Ok(byte_vec) = cast!(self, &Vec<u8>) {
            context.write_var_u32(byte_vec.len().try_into()?); // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
            context.write_bytes(byte_vec);
            Ok(())
        } else {
            serialize_iterator(&mut self.iter(), context)
        }
    }
}

impl<T: BinarySerializer> BinarySerializer for HashSet<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        serialize_iterator(&mut self.iter(), context)
    }
}

impl<T: BinarySerializer> BinarySerializer for BTreeSet<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        serialize_iterator(&mut self.iter(), context)
    }
}

impl<K: BinarySerializer, V: BinarySerializer> BinarySerializer for HashMap<K, V> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        serialize_iterator(&mut self.iter(), context)
    }
}

impl<K: BinarySerializer, V: BinarySerializer> BinarySerializer for BTreeMap<K, V> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        serialize_iterator(&mut self.iter(), context)
    }
}

impl<T: BinarySerializer> BinarySerializer for LinkedList<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        serialize_iterator(&mut self.iter(), context)
    }
}

impl<T: BinarySerializer> BinarySerializer for Box<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        (**self).serialize(context)
    }
}

impl<T: BinarySerializer + ?Sized> BinarySerializer for Rc<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        (**self).serialize(context)
    }
}

impl<T: BinarySerializer> BinarySerializer for Arc<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        (**self).serialize(context)
    }
}

impl<T> BinarySerializer for PhantomData<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        _context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        Ok(())
    }
}

impl<'a, T> BinarySerializer for &'a T
where
    T: BinarySerializer + ?Sized,
{
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        (*self).serialize(context)
    }
}

/// Helper function for implementing serialization of any iterable data source, keeping a format which is
/// compatible with both known and unknown sized iterables, allowing replacing data structures without breaking
/// the serialization format.
///
/// All the built-in `BinarySerializer` implementations for iterables use this function (or at least the same binary format).
pub fn serialize_iterator<I: Iterator<Item = T>, T: BinarySerializer, Output: BinaryOutput>(
    iter: &mut I,
    context: &mut SerializationContext<Output>,
) -> Result<()> {
    match iter.size_hint() {
        (min, Some(max)) if min == max => {
            context.write_var_i32(min.try_into()?);
            for item in iter {
                item.serialize(context)?;
            }
        }
        _ => {
            context.write_var_i32(-1);
            for item in iter {
                context.write_u8(1);
                item.serialize(context)?;
            }
            context.write_u8(0);
        }
    }
    Ok(())
}
