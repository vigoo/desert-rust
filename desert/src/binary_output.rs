use crate::Error;
use bytes::{BufMut, BytesMut};
use flate2::read::DeflateEncoder;
use flate2::Compression;
use std::io::Read;

use crate::error::Result;

pub trait BinaryOutput {
    fn write_u8(&mut self, value: u8);
    fn write_bytes(&mut self, bytes: &[u8]);

    fn write_i8(&mut self, value: i8) {
        self.write_u8(value as u8);
    }

    fn write_u16(&mut self, value: u16) {
        self.write_bytes(&value.to_be_bytes())
    }

    fn write_i16(&mut self, value: i16) {
        self.write_bytes(&value.to_be_bytes())
    }

    fn write_u32(&mut self, value: u32) {
        self.write_bytes(&value.to_be_bytes())
    }

    fn write_i32(&mut self, value: i32) {
        self.write_bytes(&value.to_be_bytes())
    }

    fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_be_bytes())
    }

    fn write_i64(&mut self, value: i64) {
        self.write_bytes(&value.to_be_bytes())
    }

    fn write_u128(&mut self, value: u128) {
        self.write_bytes(&value.to_be_bytes())
    }

    fn write_i128(&mut self, value: i128) {
        self.write_bytes(&value.to_be_bytes())
    }

    fn write_f32(&mut self, value: f32) {
        self.write_bytes(&value.to_be_bytes())
    }

    fn write_f64(&mut self, value: f64) {
        self.write_bytes(&value.to_be_bytes())
    }

    fn write_var_u32(&mut self, value: u32) {
        if value >> 7 == 0 {
            self.write_u8(value as u8);
        } else if value >> 14 == 0 {
            let buf = [((value & 0x7F) | 0x80) as u8, (value >> 7) as u8];
            self.write_bytes(&buf);
        } else if value >> 21 == 0 {
            let buf = [
                ((value & 0x7F) | 0x80) as u8,
                ((value >> 7) | 0x80) as u8,
                (value >> 14) as u8,
            ];
            self.write_bytes(&buf);
        } else if value >> 28 == 0 {
            let buf = [
                ((value & 0x7F) | 0x80) as u8,
                ((value >> 7) | 0x80) as u8,
                ((value >> 14) | 0x80) as u8,
                (value >> 21) as u8,
            ];
            self.write_bytes(&buf);
        } else {
            let buf = [
                ((value & 0x7F) | 0x80) as u8,
                ((value >> 7) | 0x80) as u8,
                ((value >> 14) | 0x80) as u8,
                ((value >> 21) | 0x80) as u8,
                (value >> 28) as u8,
            ];
            self.write_bytes(&buf);
        }
    }

    fn write_var_i32(&mut self, value: i32) {
        let adjusted = ((value << 1) ^ (value >> 31)) as u32;
        self.write_var_u32(adjusted);
    }

    fn write_compressed(&mut self, bytes: &[u8], opts: Compression) -> Result<()> {
        let mut deflater = DeflateEncoder::new(bytes, opts);
        let mut compressed = Vec::new();
        deflater
            .read_to_end(&mut compressed)
            .map_err(|err| Error::CompressionFailure(format!("{err}")))?;
        self.write_var_u32(bytes.len() as u32);
        self.write_var_u32(compressed.len() as u32);
        self.write_bytes(&compressed);
        Ok(())
    }
}

impl BinaryOutput for BytesMut {
    fn write_u8(&mut self, value: u8) {
        self.put_u8(value);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.put_slice(bytes);
    }
}
