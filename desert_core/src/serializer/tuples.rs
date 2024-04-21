use crate::{BinaryOutput, BinarySerializer, SerializationContext};

impl<T1: BinarySerializer> BinarySerializer for (T1,) {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> crate::Result<()> {
        self.0.serialize(context)
    }
}

impl<T1: BinarySerializer, T2: BinarySerializer> BinarySerializer for (T1, T2) {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> crate::Result<()> {
        context.write_u8(0);
        self.0.serialize(context)?;
        self.1.serialize(context)?;
        Ok(())
    }
}

impl<T1: BinarySerializer, T2: BinarySerializer, T3: BinarySerializer> BinarySerializer
    for (T1, T2, T3)
{
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> crate::Result<()> {
        context.write_u8(0);
        self.0.serialize(context)?;
        self.1.serialize(context)?;
        self.2.serialize(context)?;
        Ok(())
    }
}

impl<T1: BinarySerializer, T2: BinarySerializer, T3: BinarySerializer, T4: BinarySerializer>
    BinarySerializer for (T1, T2, T3, T4)
{
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> crate::Result<()> {
        context.write_u8(0);
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
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> crate::Result<()> {
        context.write_u8(0);
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
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> crate::Result<()> {
        context.write_u8(0);
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
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> crate::Result<()> {
        context.write_u8(0);
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
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> crate::Result<()> {
        context.write_u8(0);
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
