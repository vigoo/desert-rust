use std::io::Read;

use flate2::read::DeflateDecoder;

use crate::error::Result;
use crate::Error;

pub trait BinaryInput {
    fn read_u8(&mut self) -> Result<u8>;
    fn read_bytes(&mut self, count: usize) -> Result<&[u8]>;
    fn skip(&mut self, count: usize) -> Result<()>;

    fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_be_bytes(bytes.try_into()?))
    }

    fn read_i16(&mut self) -> Result<i16> {
        let bytes = self.read_bytes(2)?;
        Ok(i16::from_be_bytes(bytes.try_into()?))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_be_bytes(bytes.try_into()?))
    }

    fn read_i32(&mut self) -> Result<i32> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_be_bytes(bytes.try_into()?))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_be_bytes(bytes.try_into()?))
    }

    fn read_i64(&mut self) -> Result<i64> {
        let bytes = self.read_bytes(8)?;
        Ok(i64::from_be_bytes(bytes.try_into()?))
    }

    fn read_u128(&mut self) -> Result<u128> {
        let bytes = self.read_bytes(16)?;
        Ok(u128::from_be_bytes(bytes.try_into()?))
    }

    fn read_i128(&mut self) -> Result<i128> {
        let bytes = self.read_bytes(16)?;
        Ok(i128::from_be_bytes(bytes.try_into()?))
    }

    fn read_f32(&mut self) -> Result<f32> {
        let bytes = self.read_bytes(4)?;
        Ok(f32::from_be_bytes(bytes.try_into()?))
    }

    fn read_f64(&mut self) -> Result<f64> {
        let bytes = self.read_bytes(8)?;
        Ok(f64::from_be_bytes(bytes.try_into()?))
    }

    fn read_var_u32(&mut self) -> Result<u32> {
        let b = self.read_u8()?;
        let r = (b & 0x7F) as u32;
        if b & 0x80 == 0 {
            return Ok(r);
        }

        let b = self.read_u8()?;
        let r = r | (((b & 0x7F) as u32) << 7);
        if b & 0x80 == 0 {
            return Ok(r);
        }

        let b = self.read_u8()?;
        let r = r | (((b & 0x7F) as u32) << 14);
        if b & 0x80 == 0 {
            return Ok(r);
        }

        let b = self.read_u8()?;
        let r = r | (((b & 0x7F) as u32) << 21);
        if b & 0x80 == 0 {
            return Ok(r);
        }

        let b = self.read_u8()?;
        let r = r | (((b & 0x7F) as u32) << 28);
        Ok(r)
    }

    fn read_var_i32(&mut self) -> Result<i32> {
        let r = self.read_var_u32()?;
        Ok(((r >> 1) ^ (-((r & 1) as i32) as u32)) as i32)
    }

    fn read_compressed(&mut self) -> Result<Vec<u8>> {
        let uncompressed_len = self.read_var_u32()? as usize;
        let compressed_len = self.read_var_u32()? as usize;
        let compressed = self.read_bytes(compressed_len)?;
        let mut deflater = DeflateDecoder::new(compressed);
        let mut result = Vec::with_capacity(uncompressed_len);
        deflater
            .read_to_end(&mut result)
            .map_err(|err| Error::DecompressionFailure(format!("{err}")))?;
        Ok(result)
    }
}

pub struct SliceInput<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}

impl<'a> SliceInput<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub const EMPTY: Self = Self { data: &[], pos: 0 };
}

impl<'a> BinaryInput for SliceInput<'a> {
    fn read_u8(&mut self) -> Result<u8> {
        if self.pos == self.data.len() {
            Err(Error::InputEndedUnexpectedly)
        } else {
            let result = self.data[self.pos];
            self.pos += 1;
            Ok(result)
        }
    }

    fn read_bytes(&mut self, count: usize) -> Result<&[u8]> {
        if self.pos + count > self.data.len() {
            Err(Error::InputEndedUnexpectedly)
        } else {
            let result = &self.data[self.pos..self.pos + count];
            self.pos += count;
            Ok(result)
        }
    }

    fn skip(&mut self, count: usize) -> Result<()> {
        if self.pos + count > self.data.len() {
            Err(Error::InputEndedUnexpectedly)
        } else {
            self.pos += count;
            Ok(())
        }
    }
}

pub struct OwnedInput {
    data: Vec<u8>,
    pos: usize,
}

impl OwnedInput {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }
}

impl BinaryInput for OwnedInput {
    fn read_u8(&mut self) -> Result<u8> {
        if self.pos == self.data.len() {
            Err(Error::InputEndedUnexpectedly)
        } else {
            let result = self.data[self.pos];
            self.pos += 1;
            Ok(result)
        }
    }

    fn read_bytes(&mut self, count: usize) -> Result<&[u8]> {
        if self.pos + count > self.data.len() {
            Err(Error::InputEndedUnexpectedly)
        } else {
            let result = &self.data[self.pos..self.pos + count];
            self.pos += count;
            Ok(result)
        }
    }

    fn skip(&mut self, count: usize) -> Result<()> {
        if self.pos + count > self.data.len() {
            Err(Error::InputEndedUnexpectedly)
        } else {
            self.pos += count;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use proptest::prelude::*;
    use test_r::test;

    use crate::binary_input::OwnedInput;
    use crate::{BinaryInput, BinaryOutput};

    proptest! {
        #[test]
        fn roundtrip_var_i32(value: i32) {
            let mut bytes = BytesMut::new();
            bytes.write_var_i32(value);

            let mut bytes = OwnedInput::new(bytes.freeze().to_vec());
            let result = bytes.read_var_i32().unwrap();
            assert_eq!(value, result);
        }

        #[test]
        fn roundtrip_var_u32(value: u32) {
            let mut bytes = BytesMut::new();
            bytes.write_var_u32(value);

            let mut bytes = OwnedInput::new(bytes.freeze().to_vec());
            let result = bytes.read_var_u32().unwrap();
            assert_eq!(value, result);
        }

        #[test]
        fn roundtrip_compressed(bytes: Vec<u8>) {
            let mut compressed = BytesMut::new();
            compressed.write_compressed(&bytes, Default::default()).unwrap();

            let mut compressed = OwnedInput::new(compressed.freeze().to_vec());
            let result = compressed.read_compressed().unwrap();
            assert_eq!(bytes, result);
        }
    }
}
