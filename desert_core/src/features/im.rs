use crate::{
    deserialize_iterator, serialize_iterator, BinaryDeserializer, BinaryInput, BinaryOutput,
    BinarySerializer, DeserializationContext, Result, SerializationContext,
};
use castaway::cast;
use im::{OrdMap, Vector};
use std::any::TypeId;

impl<T: BinarySerializer + Clone + 'static> BinarySerializer for Vector<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        if let Ok(byte_vector) = cast!(self, &Vector<u8>) {
            context.write_var_u32(byte_vector.len().try_into()?);
            for chunk in byte_vector.leaves() {
                context.write_bytes(chunk);
            }
            Ok(())
        } else {
            serialize_iterator(&mut self.iter(), context)
        }
    }
}

impl<T: BinaryDeserializer + Clone + 'static> BinaryDeserializer for Vector<T> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        if TypeId::of::<T>() == TypeId::of::<u8>() {
            let length = context.read_var_u32()?;
            let bytes = context.read_bytes(length as usize)?;
            let vec = unsafe { std::mem::transmute::<Vec<u8>, Vec<T>>(bytes.to_vec()) };
            Ok(Vector::from(vec))
        } else {
            deserialize_iterator(context).0.collect()
        }
    }
}

impl<K: BinarySerializer + Ord, V: BinarySerializer> BinarySerializer for OrdMap<K, V> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        serialize_iterator(&mut self.iter(), context)
    }
}

impl<K, V> BinaryDeserializer for OrdMap<K, V>
where
    K: BinaryDeserializer + Ord + Clone,
    V: BinaryDeserializer + Clone,
{
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        deserialize_iterator(context).0.collect()
    }
}
