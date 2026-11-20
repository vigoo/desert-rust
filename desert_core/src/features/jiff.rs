use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::{
    BinaryDeserializer, BinarySerializer, DeserializationContext, Error, Result,
    SerializationContext,
};
use jiff::civil::{Date, DateTime, Time, Weekday};
use jiff::tz::{Offset, TimeZone};
use jiff::{Timestamp, ZonedDateTime};

impl BinarySerializer for Weekday {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        self.to_monday_one_offset().serialize(context)
    }
}

impl BinaryDeserializer for Weekday {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        Self::from_monday_one_offset(i8::deserialize(context)?)
            .map_err(|err| Error::DeserializationFailure(err.to_string()))
    }
}

impl BinarySerializer for Offset {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u8(0);
        context.write_var_i32(self.seconds());
        Ok(())
    }
}

impl BinaryDeserializer for Offset {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let tag = context.read_u8()?;
        if tag != 0 {
            return Err(Error::InvalidTimeZone(format!("Invalid offset tag {tag}")));
        }
        Self::from_seconds(context.read_var_i32()?)
            .map_err(|err| Error::InvalidTimeZone(err.to_string()))
    }
}

impl BinarySerializer for TimeZone {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        if self.is_unknown() {
            return Err(Error::InvalidTimeZone("Unknown time zone".to_string()));
        }
        if let Some(name) = self.iana_name() {
            context.write_u8(1);
            name.serialize(context)
        } else {
            // POSIX rules and unnamed TZif data cannot be represented by an IANA
            // name or fixed offset. Do not silently discard their transitions.
            self.to_fixed_offset()
                .map_err(|err| Error::InvalidTimeZone(err.to_string()))?
                .serialize(context)
        }
    }
}

impl BinaryDeserializer for TimeZone {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        match context.read_u8()? {
            0 => Offset::from_seconds(context.read_var_i32()?)
                .map(Self::fixed)
                .map_err(|err| Error::InvalidTimeZone(err.to_string())),
            1 => {
                let zone = Self::get(&String::deserialize(context)?)
                    .map_err(|err| Error::InvalidTimeZone(err.to_string()))?;
                if zone.is_unknown() {
                    return Err(Error::InvalidTimeZone("Unknown time zone".to_string()));
                }
                Ok(zone)
            }
            tag => Err(Error::InvalidTimeZone(format!(
                "Invalid time zone tag {tag}"
            ))),
        }
    }
}

impl BinarySerializer for Timestamp {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        // Chrono's seconds are floored, whereas Jiff truncates toward zero.
        let nanos = self.as_nanosecond();
        context.write_i64(nanos.div_euclid(1_000_000_000) as i64);
        context.write_u32(nanos.rem_euclid(1_000_000_000) as u32);
        Ok(())
    }
}

impl BinaryDeserializer for Timestamp {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let seconds = context.read_i64()?;
        let nanos = context.read_u32()?;
        if nanos >= 1_000_000_000 {
            return Err(Error::DeserializationFailure(format!(
                "Invalid timestamp nanoseconds {nanos}"
            )));
        }
        Self::from_nanosecond(i128::from(seconds) * 1_000_000_000 + i128::from(nanos))
            .map_err(|err| Error::DeserializationFailure(err.to_string()))
    }
}

impl BinarySerializer for Date {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        // Preserve the former chrono codec's unsigned representation of signed years.
        context.write_var_u32(self.year() as i32 as u32);
        context.write_u8(self.month() as u8);
        context.write_u8(self.day() as u8);
        Ok(())
    }
}

impl BinaryDeserializer for Date {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let year = i16::try_from(context.read_var_u32()? as i32)
            .map_err(|err| Error::DeserializationFailure(err.to_string()))?;
        let month = context.read_u8()?;
        let day = context.read_u8()?;
        Self::new(year, month as i8, day as i8)
            .map_err(|err| Error::DeserializationFailure(err.to_string()))
    }
}

impl BinarySerializer for Time {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        context.write_u8(self.hour() as u8);
        context.write_u8(self.minute() as u8);
        context.write_u8(self.second() as u8);
        context.write_var_u32(self.subsec_nanosecond() as u32);
        Ok(())
    }
}

impl BinaryDeserializer for Time {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let hour = context.read_u8()?;
        let minute = context.read_u8()?;
        let second = context.read_u8()?;
        let nanos = i32::try_from(context.read_var_u32()?)
            .map_err(|err| Error::DeserializationFailure(err.to_string()))?;
        Self::new(hour as i8, minute as i8, second as i8, nanos)
            .map_err(|err| Error::DeserializationFailure(err.to_string()))
    }
}

impl BinarySerializer for DateTime {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        self.date().serialize(context)?;
        self.time().serialize(context)
    }
}

impl BinaryDeserializer for DateTime {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        Ok(Self::from_parts(
            Date::deserialize(context)?,
            Time::deserialize(context)?,
        ))
    }
}

impl BinarySerializer for ZonedDateTime {
    fn serialize<Output: BinaryOutput>(
        &self,
        context: &mut SerializationContext<Output>,
    ) -> Result<()> {
        // As with chrono::DateTime<Tz>, store UTC civil time, not ambiguous local time.
        TimeZone::UTC
            .to_datetime(self.timestamp())
            .serialize(context)?;
        self.time_zone().serialize(context)
    }
}

impl BinaryDeserializer for ZonedDateTime {
    fn deserialize(context: &mut DeserializationContext<'_>) -> Result<Self> {
        let utc = DateTime::deserialize(context)?;
        let zone = TimeZone::deserialize(context)?;
        let timestamp = TimeZone::UTC
            .to_timestamp(utc)
            .map_err(|err| Error::DeserializationFailure(err.to_string()))?;
        Ok(Self::new(timestamp, zone))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::roundtrip;
    use crate::{deserialize, serialize_to_byte_vec};
    use proptest::prelude::*;
    use test_r::test;

    fn date_strategy() -> impl Strategy<Value = Date> {
        (-9999i16..=9999, 1i8..=12, 1i8..=31)
            .prop_filter_map("valid date", |(y, m, d)| Date::new(y, m, d).ok())
    }

    fn time_strategy() -> impl Strategy<Value = Time> {
        (0i8..24, 0i8..60, 0i8..60, 0i32..1_000_000_000)
            .prop_map(|(h, m, s, ns)| Time::new(h, m, s, ns).unwrap())
    }

    proptest! {
        #[test]
        fn roundtrip_civil(date in date_strategy(), time in time_strategy(), weekday in 1i8..=7) {
            roundtrip(date);
            roundtrip(time);
            roundtrip(DateTime::from_parts(date, time));
            roundtrip(Weekday::from_monday_one_offset(weekday).unwrap());
        }

        #[test]
        fn roundtrip_timestamp_and_offset(
            nanos in Timestamp::MIN.as_nanosecond()..=Timestamp::MAX.as_nanosecond(),
            offset in -93_599i32..=93_599,
        ) {
            roundtrip(Timestamp::from_nanosecond(nanos).unwrap());
            roundtrip(Offset::from_seconds(offset).unwrap());
        }
    }

    fn fixture<T: BinarySerializer + BinaryDeserializer + std::fmt::Debug + PartialEq>(
        value: T,
        bytes: &[u8],
    ) {
        assert_eq!(serialize_to_byte_vec(&value).unwrap(), bytes);
        assert_eq!(deserialize::<T>(bytes).unwrap(), value);
    }

    #[test]
    fn reads_and_writes_chrono_fixtures() {
        // Captured from the chrono codecs before their replacement.
        fixture(Weekday::Sunday, &[7]);
        fixture(Date::new(2024, 6, 22).unwrap(), &[232, 15, 6, 22]);
        fixture(
            Date::new(-44, 3, 15).unwrap(),
            &[212, 255, 255, 255, 15, 3, 15],
        );
        fixture(
            Time::new(9, 30, 5, 123_456_789).unwrap(),
            &[9, 30, 5, 149, 154, 239, 58],
        );
        fixture(
            Timestamp::from_nanosecond(-1).unwrap(),
            &[255, 255, 255, 255, 255, 255, 255, 255, 59, 154, 201, 255],
        );
        fixture(Offset::from_seconds(-19_800).unwrap(), &[0, 175, 181, 2]);
        let bytes = [
            232, 15, 11, 3, 1, 30, 0, 149, 154, 239, 58, 1, 30, 69, 117, 114, 111, 112, 101, 47,
            66, 117, 100, 97, 112, 101, 115, 116,
        ];
        let value = ZonedDateTime::new(
            Timestamp::new(1_730_597_400, 123_456_789).unwrap(),
            TimeZone::get("Europe/Budapest").unwrap(),
        );
        fixture(value, &bytes);
        assert_eq!(
            deserialize::<ZonedDateTime>(&bytes)
                .unwrap()
                .time_zone()
                .iana_name(),
            Some("Europe/Budapest")
        );
    }

    #[test]
    fn zones_preserve_identity_and_dst_overlap() {
        for name in ["Europe/Budapest", "America/New_York", "UTC"] {
            let zone = TimeZone::get(name).unwrap();
            let bytes = serialize_to_byte_vec(&zone).unwrap();
            assert_eq!(
                deserialize::<TimeZone>(&bytes).unwrap().iana_name(),
                Some(name)
            );
            // Both instants in New York's repeated 01:30 on 2024-11-03.
            for second in [1_730_611_800, 1_730_615_400] {
                let value = ZonedDateTime::new(Timestamp::new(second, 321).unwrap(), zone.clone());
                let bytes = serialize_to_byte_vec(&value).unwrap();
                let decoded = deserialize::<ZonedDateTime>(&bytes).unwrap();
                assert_eq!(decoded.timestamp(), value.timestamp());
                assert_eq!(decoded.time_zone().iana_name(), Some(name));
            }
        }
        for seconds in [-19_800, 0, 20_700] {
            let zone = TimeZone::fixed(Offset::from_seconds(seconds).unwrap());
            let bytes = serialize_to_byte_vec(&zone).unwrap();
            assert_eq!(
                deserialize::<TimeZone>(&bytes)
                    .unwrap()
                    .to_fixed_offset()
                    .unwrap()
                    .seconds(),
                seconds
            );
            let value = ZonedDateTime::new(Timestamp::new(-1234, -567).unwrap(), zone);
            let bytes = serialize_to_byte_vec(&value).unwrap();
            let decoded = deserialize::<ZonedDateTime>(&bytes).unwrap();
            assert_eq!(decoded.timestamp(), value.timestamp());
            assert_eq!(
                decoded.time_zone().to_fixed_offset().unwrap().seconds(),
                seconds
            );
        }
    }

    #[test]
    fn rejects_invalid_values_and_unrepresentable_zones() {
        assert!(deserialize::<Weekday>(&[0]).is_err());
        assert!(deserialize::<Weekday>(&[8]).is_err());
        assert!(deserialize::<Date>(&[232, 15, 2, 30]).is_err());
        assert!(deserialize::<Time>(&[24, 0, 0, 0]).is_err());
        assert!(deserialize::<Time>(&[23, 59, 59, 128, 148, 235, 220, 3]).is_err());
        assert!(deserialize::<Offset>(&[1, 0]).is_err());
        assert!(deserialize::<TimeZone>(&[2]).is_err());
        assert!(deserialize::<TimeZone>(&[1, 2, b'?']).is_err());
        let mut unknown_zone = vec![1];
        unknown_zone.extend(serialize_to_byte_vec(&"Etc/Unknown").unwrap());
        assert!(matches!(
            deserialize::<TimeZone>(&unknown_zone),
            Err(Error::InvalidTimeZone(_))
        ));
        let mut unknown_zoned =
            serialize_to_byte_vec(&Date::new(2024, 11, 3).unwrap().at(1, 30, 0, 0)).unwrap();
        unknown_zoned.extend(unknown_zone);
        assert!(matches!(
            deserialize::<ZonedDateTime>(&unknown_zoned),
            Err(Error::InvalidTimeZone(_))
        ));
        let mut timestamp = vec![0; 8];
        timestamp.extend_from_slice(&1_000_000_000u32.to_be_bytes());
        assert!(deserialize::<Timestamp>(&timestamp).is_err());
        assert!(serialize_to_byte_vec(&TimeZone::unknown()).is_err());
        assert!(
            serialize_to_byte_vec(&TimeZone::posix("EST5EDT,M3.2.0,M11.1.0").unwrap()).is_err()
        );
    }
}
