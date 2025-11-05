use crate::Error::DeserializationFailure;
use crate::{
    serialize_iterator, BinaryDeserializer, BinaryInput, BinaryOutput, BinarySerializer,
    DeserializationContext, SerializationContext,
};
use castaway::cast;
use nonempty_collections::NEVec;

impl<T: BinarySerializer + 'static> BinarySerializer for NEVec<T> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> crate::Result<()> {
        if let Ok(byte_vec) = cast!(self, &NEVec<u8>) {
            context.write_var_u32(byte_vec.len().get().try_into()?); // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
            context.write_bytes(byte_vec.as_nonempty_slice().as_ref());
            Ok(())
        } else {
            serialize_iterator(&mut self.iter(), context)
        }
    }
}

impl<T: BinaryDeserializer + 'static> BinaryDeserializer for NEVec<T> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> crate::Result<Self> {
        use std::any::TypeId;

        if TypeId::of::<T>() == TypeId::of::<u8>() {
            let length = context.read_var_u32()?; // NOTE: this is inconsistent with the generic case, but this way it is compatible with the Scala version's Chunk serializer
            let bytes = context.read_bytes(length as usize)?;
            let vec = unsafe { std::mem::transmute::<Vec<u8>, Vec<T>>(bytes.to_vec()) };
            Ok(NEVec::try_from_vec(vec)
                .ok_or_else(|| DeserializationFailure("NEVec was empty".to_string()))?)
        } else {
            let mut vec = Vec::new();
            for item in crate::deserializer::deserialize_iterator(context) {
                vec.push(item?);
            }
            Ok(NEVec::try_from_vec(vec)
                .ok_or_else(|| DeserializationFailure("NEVec was empty".to_string()))?)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::roundtrip;
    use nonempty_collections::NEVec;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use test_r::test;

    proptest! {
        #[test]
        fn roundtrip_u8(value in vec(any::<u8>(), 1..=1024)) {
            let nevec = NEVec::try_from_vec(value).unwrap();
            roundtrip(nevec);
        }

        #[test]
        fn roundtrip_string(value in vec(any::<String>(), 1..=100)) {
            let nevec = NEVec::try_from_vec(value).unwrap();
            roundtrip(nevec);
        }
    }
}
