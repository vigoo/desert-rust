use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::deserializer::DeserializationContext;
use crate::serializer::SerializationContext;
use crate::{BinaryDeserializer, BinarySerializer, Error, Result};
use bigdecimal::FromPrimitive;
use chrono::{
    DateTime, FixedOffset, Local, Month, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike,
    Utc, Weekday,
};
use chrono_tz::{OffsetName, Tz};
use std::str::FromStr;

impl BinarySerializer for Weekday {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        (self.number_from_monday() as i8).serialize(context)
    }
}

impl BinaryDeserializer for Weekday {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        Weekday::from_i8(i8::deserialize(context)? - 1).ok_or_else(|| {
            Error::DeserializationFailure("Failed to deserialize Weekday".to_string())
        })
    }
}

impl BinarySerializer for Month {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        (self.number_from_month() as i8).serialize(context)
    }
}

impl BinaryDeserializer for Month {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        Month::from_i8(i8::deserialize(context)?)
            .ok_or_else(|| Error::DeserializationFailure("Failed to deserialize Month".to_string()))
    }
}

impl BinarySerializer for FixedOffset {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u8(0);
        context.write_var_i32(self.local_minus_utc());
        Ok(())
    }
}

impl BinaryDeserializer for FixedOffset {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let typ = context.read_u8()?;
        if typ != 0 {
            Err(Error::DeserializationFailure(format!(
                "Failed to deserialize FixedOffset: Invalid type {}",
                typ
            )))?
        } else {
            let offset = context.read_var_i32()?;
            FixedOffset::east_opt(offset).ok_or_else(|| {
                Error::DeserializationFailure(format!(
                    "Failed to deserialize FixedOffset: Invalid offset {}",
                    offset
                ))
            })
        }
    }
}

impl BinarySerializer for Tz {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u8(1);
        self.name().serialize(context)
    }
}

impl BinaryDeserializer for Tz {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let typ = context.read_u8()?;
        if typ != 1 {
            Err(Error::DeserializationFailure(format!(
                "Failed to deserialize Tz: Invalid type {}",
                typ
            )))?
        } else {
            let name = String::deserialize(context)?;
            Tz::from_str(&name).map_err(|err| {
                Error::DeserializationFailure(format!("Failed to deserialize Tz: {}", err))
            })
        }
    }
}

impl BinarySerializer for DateTime<Utc> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_i64(self.timestamp());
        context.write_u32(self.timestamp_subsec_nanos());
        Ok(())
    }
}

impl BinaryDeserializer for DateTime<Utc> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let seconds = context.read_i64()?;
        let nanos = context.read_u32()?;
        DateTime::<Utc>::from_timestamp(seconds, nanos).ok_or_else(|| {
            Error::DeserializationFailure(format!(
                "Failed to deserialize DateTime<Utc>: Invalid timestamp {} {}",
                seconds, nanos
            ))
        })
    }
}

impl BinarySerializer for NaiveDate {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        use chrono::Datelike;

        context.write_var_u32(self.year() as u32);
        context.write_u8(self.month() as u8);
        context.write_u8(self.day() as u8);
        Ok(())
    }
}

impl BinaryDeserializer for NaiveDate {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let year = context.read_var_u32()?;
        let month = context.read_u8()?;
        let day = context.read_u8()?;
        NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32).ok_or_else(|| {
            Error::DeserializationFailure(format!(
                "Failed to deserialize NaiveDate: Invalid date {} {} {}",
                year, month, day
            ))
        })
    }
}

impl BinarySerializer for NaiveTime {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u8(self.hour() as u8);
        context.write_u8(self.minute() as u8);
        context.write_u8(self.second() as u8);
        context.write_var_u32(self.nanosecond());
        Ok(())
    }
}

impl BinaryDeserializer for NaiveTime {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let hour = context.read_u8()?;
        let minute = context.read_u8()?;
        let second = context.read_u8()?;
        let nanosecond = context.read_var_u32()?;
        NaiveTime::from_hms_nano_opt(hour as u32, minute as u32, second as u32, nanosecond)
            .ok_or_else(|| {
                Error::DeserializationFailure(format!(
                    "Failed to deserialize NaiveTime: Invalid time {} {} {} {}",
                    hour, minute, second, nanosecond
                ))
            })
    }
}

impl BinarySerializer for NaiveDateTime {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        self.date().serialize(context)?;
        self.time().serialize(context)?;
        Ok(())
    }
}

impl BinaryDeserializer for NaiveDateTime {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let date = NaiveDate::deserialize(context)?;
        let time = NaiveTime::deserialize(context)?;
        Ok(NaiveDateTime::new(date, time))
    }
}

impl BinarySerializer for DateTime<Local> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        self.date_naive().serialize(context)?;
        self.time().serialize(context)?;
        Ok(())
    }
}

impl BinaryDeserializer for DateTime<Local> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let date = NaiveDate::deserialize(context)?;
        let time = NaiveTime::deserialize(context)?;
        let naive = NaiveDateTime::new(date, time);
        Local.from_local_datetime(&naive).single().ok_or_else(|| {
            Error::DeserializationFailure(format!("Failed to deserialize DateTime<Local>: {naive}"))
        })
    }
}

impl BinarySerializer for DateTime<FixedOffset> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        self.naive_local().serialize(context)?;
        self.offset().serialize(context)?;
        Ok(())
    }
}

impl BinaryDeserializer for DateTime<FixedOffset> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let naive = NaiveDateTime::deserialize(context)?;
        let offset = FixedOffset::deserialize(context)?;
        offset.from_local_datetime(&naive).single().ok_or_else(|| {
            Error::DeserializationFailure(format!(
                "Failed to deserialize DateTime<FixedOffset>: {naive}"
            ))
        })
    }
}

impl BinarySerializer for DateTime<Tz> {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        self.naive_utc().serialize(context)?;
        Tz::from_str(self.offset().tz_id())?.serialize(context)?;
        Ok(())
    }
}

impl BinaryDeserializer for DateTime<Tz> {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let naive = NaiveDateTime::deserialize(context)?;
        let tz = Tz::deserialize(context)?;
        Ok(tz.from_utc_datetime(&naive))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::roundtrip;
    use chrono::{
        DateTime, FixedOffset, Local, Month, NaiveDate, NaiveDateTime, TimeZone, Utc, Weekday,
    };
    use chrono_tz::Tz;
    use proptest::prelude::*;
    use proptest_arbitrary_interop::arb;
    use test_r::test;

    fn datetime_tz_strategy() -> impl Strategy<Value = DateTime<Tz>> {
        (arb::<NaiveDateTime>(), arb::<Tz>())
            .prop_map(|(datetime, tz)| tz.from_utc_datetime(&datetime))
    }

    fn datetime_local_strategy() -> impl Strategy<Value = DateTime<Local>> {
        (arb::<NaiveDateTime>()).prop_filter_map("valid local datetime", |naive| {
            Local.from_local_datetime(&naive).single()
        })
    }

    fn datetime_fixed_offset_strategy() -> impl Strategy<Value = DateTime<FixedOffset>> {
        (arb::<NaiveDateTime>(), (-85_399..86_400)).prop_map(|(naive, offset)| {
            FixedOffset::east_opt(offset)
                .unwrap()
                .from_local_datetime(&naive)
                .single()
                .unwrap()
        })
    }

    proptest! {
        #[test]
        fn roundtrip_weekday(value in arb::<Weekday>()) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_month(value in arb::<Month>()) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_fixed_offset(value in arb::<FixedOffset>()) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_tz(value in arb::<Tz>()) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_datetime_utc(value in arb::<DateTime<Utc>>()) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_datetime_local(value in datetime_local_strategy()) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_datetime_fixed_offset(value in datetime_fixed_offset_strategy()) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_datetime_tz(value in datetime_tz_strategy()) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_naive_date(value in arb::<NaiveDate>()) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_naive_time(value in arb::<NaiveDate>()) {
            roundtrip(value);
        }

        #[test]
        fn roundtrip_naive_date_time(value in arb::<NaiveDateTime>()) {
            roundtrip(value);
        }
    }
}
