use crate::Error;
use bytes::{BufMut, BytesMut};
use flate2::read::DeflateEncoder;
use flate2::Compression;
use std::io::Read;
use std::ops::Range;

use crate::error::Result;

pub trait BinaryOutput {
    fn write_u8(&mut self, value: u8);
    fn write_bytes(&mut self, bytes: &[u8]);

    fn contiguous_len(&self) -> Option<usize> {
        None
    }

    fn insert_bytes(&mut self, _position: usize, _bytes: &[u8]) -> Result<()> {
        Err(Error::SerializationFailure(
            "output does not support contiguous byte insertion".to_string(),
        ))
    }

    fn reorder_ranges(
        &mut self,
        _start: usize,
        _len: usize,
        _ranges: &[Range<usize>],
        _order: &[usize],
    ) -> Result<()> {
        Err(Error::SerializationFailure(
            "output does not support contiguous byte reordering".to_string(),
        ))
    }

    fn supports_efficient_bulk_bytes(&self) -> bool {
        false
    }

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
        if value < 0x80 {
            self.write_u8(value as u8);
        } else if value < 0x4000 {
            let buf = [((value & 0x7F) | 0x80) as u8, (value >> 7) as u8];
            self.write_bytes(&buf);
        } else if value < 0x200000 {
            let buf = [
                ((value & 0x7F) | 0x80) as u8,
                ((value >> 7) | 0x80) as u8,
                (value >> 14) as u8,
            ];
            self.write_bytes(&buf);
        } else if value < 0x10000000 {
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

impl<Output: BinaryOutput + ?Sized> BinaryOutput for &mut Output {
    fn write_u8(&mut self, value: u8) {
        (**self).write_u8(value);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        (**self).write_bytes(bytes);
    }

    fn supports_efficient_bulk_bytes(&self) -> bool {
        (**self).supports_efficient_bulk_bytes()
    }

    fn contiguous_len(&self) -> Option<usize> {
        (**self).contiguous_len()
    }

    fn insert_bytes(&mut self, position: usize, bytes: &[u8]) -> Result<()> {
        (**self).insert_bytes(position, bytes)
    }

    fn reorder_ranges(
        &mut self,
        start: usize,
        len: usize,
        ranges: &[Range<usize>],
        order: &[usize],
    ) -> Result<()> {
        (**self).reorder_ranges(start, len, ranges, order)
    }
}

impl BinaryOutput for BytesMut {
    fn write_u8(&mut self, value: u8) {
        self.put_u8(value);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.put_slice(bytes);
    }

    fn supports_efficient_bulk_bytes(&self) -> bool {
        true
    }

    fn contiguous_len(&self) -> Option<usize> {
        Some(self.len())
    }

    fn insert_bytes(&mut self, position: usize, bytes: &[u8]) -> Result<()> {
        insert_into_slice_backed_output(self, position, bytes)
    }

    fn reorder_ranges(
        &mut self,
        start: usize,
        len: usize,
        ranges: &[Range<usize>],
        order: &[usize],
    ) -> Result<()> {
        reorder_slice_backed_output(self, start, len, ranges, order)
    }
}

impl BinaryOutput for Vec<u8> {
    #[inline(always)]
    fn write_u8(&mut self, value: u8) {
        self.push(value);
    }

    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    fn supports_efficient_bulk_bytes(&self) -> bool {
        true
    }

    fn contiguous_len(&self) -> Option<usize> {
        Some(self.len())
    }

    fn insert_bytes(&mut self, position: usize, bytes: &[u8]) -> Result<()> {
        insert_into_slice_backed_output(self, position, bytes)
    }

    fn reorder_ranges(
        &mut self,
        start: usize,
        len: usize,
        ranges: &[Range<usize>],
        order: &[usize],
    ) -> Result<()> {
        reorder_slice_backed_output(self, start, len, ranges, order)
    }
}

fn insert_into_slice_backed_output<Output>(
    output: &mut Output,
    position: usize,
    bytes: &[u8],
) -> Result<()>
where
    Output: AsMut<[u8]> + Extend<u8>,
{
    let current_len = output.as_mut().len();
    if position > current_len {
        return Err(Error::SerializationFailure(
            "insert position is past the end of the output".to_string(),
        ));
    }
    output.extend(std::iter::repeat_n(0, bytes.len()));
    let new_len = current_len + bytes.len();
    let slice = output.as_mut();
    slice.copy_within(position..current_len, position + bytes.len());
    slice[position..position + bytes.len()].copy_from_slice(bytes);
    debug_assert_eq!(slice.len(), new_len);
    Ok(())
}

fn reorder_slice_backed_output<Output>(
    output: &mut Output,
    start: usize,
    len: usize,
    ranges: &[Range<usize>],
    order: &[usize],
) -> Result<()>
where
    Output: AsMut<[u8]> + AsRef<[u8]>,
{
    let slice = output.as_ref();
    if start.checked_add(len).is_none_or(|end| end > slice.len()) {
        return Err(Error::SerializationFailure(
            "reordered range is past the end of the output".to_string(),
        ));
    }

    let mut scratch = Vec::with_capacity(len);
    for idx in order {
        let range = ranges.get(*idx).ok_or_else(|| {
            Error::SerializationFailure("field range order is out of bounds".to_string())
        })?;
        if range.end > slice.len() || range.start > range.end {
            return Err(Error::SerializationFailure(
                "field range is outside the output".to_string(),
            ));
        }
        scratch.extend_from_slice(&slice[range.clone()]);
    }

    if scratch.len() != len {
        return Err(Error::SerializationFailure(
            "reordered ranges do not cover the serialized fields".to_string(),
        ));
    }

    output.as_mut()[start..start + len].copy_from_slice(&scratch);
    Ok(())
}

pub struct SizeCalculator {
    size: usize,
}

impl SizeCalculator {
    pub fn new() -> Self {
        SizeCalculator { size: 0 }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl Default for SizeCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryOutput for SizeCalculator {
    fn write_u8(&mut self, _value: u8) {
        self.size += 1;
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.size += bytes.len();
    }
}
