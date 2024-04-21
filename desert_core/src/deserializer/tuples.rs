use crate::adt::{AdtDeserializer, ChunkedInput, EMPTY_ADT_METADATA};
use crate::{BinaryDeserializer, BinaryInput, DeserializationContext};

fn deserialize_tuple1<Context: DeserializationContext, CI: ChunkedInput, T1: BinaryDeserializer>(
    deserializer: &mut AdtDeserializer<Context, CI>,
) -> crate::Result<(T1,)> {
    Ok((deserializer.read_field("_0", None)?,))
}

fn deserialize_tuple2<
    Context: DeserializationContext,
    CI: ChunkedInput,
    T1: BinaryDeserializer,
    T2: BinaryDeserializer,
>(
    deserializer: &mut AdtDeserializer<Context, CI>,
) -> crate::Result<(T1, T2)> {
    Ok((
        deserializer.read_field("_0", None)?,
        deserializer.read_field("_1", None)?,
    ))
}

fn deserialize_tuple3<
    Context: DeserializationContext,
    CI: ChunkedInput,
    T1: BinaryDeserializer,
    T2: BinaryDeserializer,
    T3: BinaryDeserializer,
>(
    deserializer: &mut AdtDeserializer<Context, CI>,
) -> crate::Result<(T1, T2, T3)> {
    Ok((
        deserializer.read_field("_0", None)?,
        deserializer.read_field("_1", None)?,
        deserializer.read_field("_2", None)?,
    ))
}

fn deserialize_tuple4<
    Context: DeserializationContext,
    CI: ChunkedInput,
    T1: BinaryDeserializer,
    T2: BinaryDeserializer,
    T3: BinaryDeserializer,
    T4: BinaryDeserializer,
>(
    deserializer: &mut AdtDeserializer<Context, CI>,
) -> crate::Result<(T1, T2, T3, T4)> {
    Ok((
        deserializer.read_field("_0", None)?,
        deserializer.read_field("_1", None)?,
        deserializer.read_field("_2", None)?,
        deserializer.read_field("_3", None)?,
    ))
}

fn deserialize_tuple5<
    Context: DeserializationContext,
    CI: ChunkedInput,
    T1: BinaryDeserializer,
    T2: BinaryDeserializer,
    T3: BinaryDeserializer,
    T4: BinaryDeserializer,
    T5: BinaryDeserializer,
>(
    deserializer: &mut AdtDeserializer<Context, CI>,
) -> crate::Result<(T1, T2, T3, T4, T5)> {
    Ok((
        deserializer.read_field("_0", None)?,
        deserializer.read_field("_1", None)?,
        deserializer.read_field("_2", None)?,
        deserializer.read_field("_3", None)?,
        deserializer.read_field("_4", None)?,
    ))
}

fn deserialize_tuple6<
    Context: DeserializationContext,
    CI: ChunkedInput,
    T1: BinaryDeserializer,
    T2: BinaryDeserializer,
    T3: BinaryDeserializer,
    T4: BinaryDeserializer,
    T5: BinaryDeserializer,
    T6: BinaryDeserializer,
>(
    deserializer: &mut AdtDeserializer<Context, CI>,
) -> crate::Result<(T1, T2, T3, T4, T5, T6)> {
    Ok((
        deserializer.read_field("_0", None)?,
        deserializer.read_field("_1", None)?,
        deserializer.read_field("_2", None)?,
        deserializer.read_field("_3", None)?,
        deserializer.read_field("_4", None)?,
        deserializer.read_field("_5", None)?,
    ))
}

fn deserialize_tuple7<
    Context: DeserializationContext,
    CI: ChunkedInput,
    T1: BinaryDeserializer,
    T2: BinaryDeserializer,
    T3: BinaryDeserializer,
    T4: BinaryDeserializer,
    T5: BinaryDeserializer,
    T6: BinaryDeserializer,
    T7: BinaryDeserializer,
>(
    deserializer: &mut AdtDeserializer<Context, CI>,
) -> crate::Result<(T1, T2, T3, T4, T5, T6, T7)> {
    Ok((
        deserializer.read_field("_0", None)?,
        deserializer.read_field("_1", None)?,
        deserializer.read_field("_2", None)?,
        deserializer.read_field("_3", None)?,
        deserializer.read_field("_4", None)?,
        deserializer.read_field("_5", None)?,
        deserializer.read_field("_6", None)?,
    ))
}

fn deserialize_tuple8<
    Context: DeserializationContext,
    CI: ChunkedInput,
    T1: BinaryDeserializer,
    T2: BinaryDeserializer,
    T3: BinaryDeserializer,
    T4: BinaryDeserializer,
    T5: BinaryDeserializer,
    T6: BinaryDeserializer,
    T7: BinaryDeserializer,
    T8: BinaryDeserializer,
>(
    deserializer: &mut AdtDeserializer<Context, CI>,
) -> crate::Result<(T1, T2, T3, T4, T5, T6, T7, T8)> {
    Ok((
        deserializer.read_field("_0", None)?,
        deserializer.read_field("_1", None)?,
        deserializer.read_field("_2", None)?,
        deserializer.read_field("_3", None)?,
        deserializer.read_field("_4", None)?,
        deserializer.read_field("_5", None)?,
        deserializer.read_field("_6", None)?,
        deserializer.read_field("_7", None)?,
    ))
}

impl<T1: BinaryDeserializer> BinaryDeserializer for (T1,) {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> crate::Result<Self> {
        let stored_version = context.input_mut().read_u8()?;

        if stored_version == 0 {
            let mut deserializer = AdtDeserializer::new_v0(&EMPTY_ADT_METADATA, context)?;
            deserialize_tuple1(&mut deserializer)
        } else {
            let mut deserializer =
                AdtDeserializer::new(&EMPTY_ADT_METADATA, context, stored_version)?;
            deserialize_tuple1(&mut deserializer)
        }
    }
}

impl<T1: BinaryDeserializer, T2: BinaryDeserializer> BinaryDeserializer for (T1, T2) {
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> crate::Result<Self> {
        let stored_version = context.input_mut().read_u8()?;

        if stored_version == 0 {
            let mut deserializer = AdtDeserializer::new_v0(&EMPTY_ADT_METADATA, context)?;
            deserialize_tuple2(&mut deserializer)
        } else {
            let mut deserializer =
                AdtDeserializer::new(&EMPTY_ADT_METADATA, context, stored_version)?;
            deserialize_tuple2(&mut deserializer)
        }
    }
}

impl<T1: BinaryDeserializer, T2: BinaryDeserializer, T3: BinaryDeserializer> BinaryDeserializer
    for (T1, T2, T3)
{
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> crate::Result<Self> {
        let stored_version = context.input_mut().read_u8()?;

        if stored_version == 0 {
            let mut deserializer = AdtDeserializer::new_v0(&EMPTY_ADT_METADATA, context)?;
            deserialize_tuple3(&mut deserializer)
        } else {
            let mut deserializer =
                AdtDeserializer::new(&EMPTY_ADT_METADATA, context, stored_version)?;
            deserialize_tuple3(&mut deserializer)
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
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> crate::Result<Self> {
        let stored_version = context.input_mut().read_u8()?;

        if stored_version == 0 {
            let mut deserializer = AdtDeserializer::new_v0(&EMPTY_ADT_METADATA, context)?;
            deserialize_tuple4(&mut deserializer)
        } else {
            let mut deserializer =
                AdtDeserializer::new(&EMPTY_ADT_METADATA, context, stored_version)?;
            deserialize_tuple4(&mut deserializer)
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
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> crate::Result<Self> {
        let stored_version = context.input_mut().read_u8()?;

        if stored_version == 0 {
            let mut deserializer = AdtDeserializer::new_v0(&EMPTY_ADT_METADATA, context)?;
            deserialize_tuple5(&mut deserializer)
        } else {
            let mut deserializer =
                AdtDeserializer::new(&EMPTY_ADT_METADATA, context, stored_version)?;
            deserialize_tuple5(&mut deserializer)
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
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> crate::Result<Self> {
        let stored_version = context.input_mut().read_u8()?;

        if stored_version == 0 {
            let mut deserializer = AdtDeserializer::new_v0(&EMPTY_ADT_METADATA, context)?;
            deserialize_tuple6(&mut deserializer)
        } else {
            let mut deserializer =
                AdtDeserializer::new(&EMPTY_ADT_METADATA, context, stored_version)?;
            deserialize_tuple6(&mut deserializer)
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
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> crate::Result<Self> {
        let stored_version = context.input_mut().read_u8()?;

        if stored_version == 0 {
            let mut deserializer = AdtDeserializer::new_v0(&EMPTY_ADT_METADATA, context)?;
            deserialize_tuple7(&mut deserializer)
        } else {
            let mut deserializer =
                AdtDeserializer::new(&EMPTY_ADT_METADATA, context, stored_version)?;
            deserialize_tuple7(&mut deserializer)
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
    fn deserialize<Context: DeserializationContext>(context: &mut Context) -> crate::Result<Self> {
        let stored_version = context.input_mut().read_u8()?;

        if stored_version == 0 {
            let mut deserializer = AdtDeserializer::new_v0(&EMPTY_ADT_METADATA, context)?;
            deserialize_tuple8(&mut deserializer)
        } else {
            let mut deserializer =
                AdtDeserializer::new(&EMPTY_ADT_METADATA, context, stored_version)?;
            deserialize_tuple8(&mut deserializer)
        }
    }
}
