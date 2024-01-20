use crate::error::Result;
use crate::Error;
use bytes::{Buf, Bytes};
use flate2::read::DeflateDecoder;
use std::fs::File;
use std::io::Read;

pub trait BinaryInput {
    fn read_u8(&mut self) -> Result<u8>;
    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>>;

    fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_be_bytes(bytes.as_slice().try_into()?))
    }

    fn read_i16(&mut self) -> Result<i16> {
        let bytes = self.read_bytes(2)?;
        Ok(i16::from_be_bytes(bytes.as_slice().try_into()?))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_be_bytes(bytes.as_slice().try_into()?))
    }

    fn read_i32(&mut self) -> Result<i32> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_be_bytes(bytes.as_slice().try_into()?))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_be_bytes(bytes.as_slice().try_into()?))
    }

    fn read_i64(&mut self) -> Result<i64> {
        let bytes = self.read_bytes(8)?;
        Ok(i64::from_be_bytes(bytes.as_slice().try_into()?))
    }

    fn read_u128(&mut self) -> Result<u128> {
        let bytes = self.read_bytes(16)?;
        Ok(u128::from_be_bytes(bytes.as_slice().try_into()?))
    }

    fn read_i128(&mut self) -> Result<i128> {
        let bytes = self.read_bytes(16)?;
        Ok(i128::from_be_bytes(bytes.as_slice().try_into()?))
    }

    fn read_f32(&mut self) -> Result<f32> {
        let bytes = self.read_bytes(4)?;
        Ok(f32::from_be_bytes(bytes.as_slice().try_into()?))
    }

    fn read_f64(&mut self) -> Result<f64> {
        let bytes = self.read_bytes(8)?;
        Ok(f64::from_be_bytes(bytes.as_slice().try_into()?))
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
        let mut deflater = DeflateDecoder::new(compressed.as_slice());
        let mut result = Vec::with_capacity(uncompressed_len);
        deflater
            .read_to_end(&mut result)
            .map_err(|err| Error::DecompressionFailure(format!("{err}")))?;
        Ok(result)
    }
}

impl BinaryInput for Bytes {
    fn read_u8(&mut self) -> Result<u8> {
        if self.has_remaining() {
            Ok(self.get_u8())
        } else {
            Err(Error::InputEndedUnexpectedly)
        }
    }

    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        if self.remaining() >= count {
            let mut result = vec![0u8; count];
            self.copy_to_slice(&mut result);
            Ok(result)
        } else {
            Err(Error::InputEndedUnexpectedly)
        }
    }
}

impl BinaryInput for File {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)
            .map_err(|_| Error::InputEndedUnexpectedly)?;
        Ok(buf[0])
    }

    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; count];
        self.read_exact(&mut buf)
            .map_err(|_| Error::InputEndedUnexpectedly)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::{BinaryInput, BinaryOutput};
    use bytes::BytesMut;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn roundtrip_var_i32(value: i32) {
            let mut bytes = BytesMut::new();
            bytes.write_var_i32(value);

            let mut bytes = bytes.freeze();
            let result = bytes.read_var_i32().unwrap();
            assert_eq!(value, result);
        }

        #[test]
        fn roundtrip_var_u32(value: u32) {
            let mut bytes = BytesMut::new();
            bytes.write_var_u32(value);

            let mut bytes = bytes.freeze();
            let result = bytes.read_var_u32().unwrap();
            assert_eq!(value, result);
        }

        #[test]
        fn roundtrip_compressed(bytes: Vec<u8>) {
            let mut compressed = BytesMut::new();
            compressed.write_compressed(&bytes, Default::default()).unwrap();

            let mut compressed = compressed.freeze();
            let result = compressed.read_compressed().unwrap();
            assert_eq!(bytes, result);
        }
    }
}
