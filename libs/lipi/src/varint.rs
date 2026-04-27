#![allow(clippy::unusual_byte_groupings)]
use std::mem::MaybeUninit;

use super::Result;
use crate::{errors, utils};

/// See: https://en.wikipedia.org/wiki/LEB128
pub trait LEB128 {
    fn write_byte(&mut self, byte: u8);

    #[inline]
    fn write_u32(&mut self, mut num: u32) {
        while num > 0b_111_1111 {
            self.write_byte((num as u8) | 0b_1000_0000); // Set continuation bit
            num >>= 7; // Shift right by 7 bits
        }
        self.write_byte(num as u8); // Push last byte without continuation bit
    }

    #[inline]
    fn write_u64(&mut self, mut num: u64) {
        while num > 0b_111_1111 {
            self.write_byte((num as u8) | 0b_1000_0000);
            num >>= 7;
        }
        self.write_byte(num as u8);
    }
}

pub fn read_u64(reader: &mut &[u8]) -> Result<u64> {
    let mut result = 0;
    let mut shift = 0;

    loop {
        let byte = utils::read_byte(reader)?;

        if shift == 63 && byte >= 2 {
            return Err(errors::VarIntError.into());
        }

        if (byte & 0b_1000_0000) == 0 {
            break Ok(result | ((byte as u64) << shift));
        }

        result |= ((byte & 0b_111_1111) as u64) << shift; // low-order 7 bits of value
        shift += 7;
    }
}

#[derive(Debug)]
pub struct Leb128Buf<const N: usize> {
    buf: [MaybeUninit<u8>; N],
    len: u8,
}

impl<const N: usize> Leb128Buf<N> {
    /// # Safety
    ///
    /// The caller must ensure that no more than `N` bytes are written to the buffer.
    #[inline]
    pub unsafe fn new() -> Self {
        Self {
            buf: [MaybeUninit::uninit(); N],
            len: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.buf.as_ptr() as *const u8, self.len()) }
    }
}

impl<const N: usize> LEB128 for Leb128Buf<N> {
    #[inline]
    fn write_byte(&mut self, byte: u8) {
        debug_assert!(
            self.len < N as u8,
            "Attempting to write more than {N} bytes to Leb128Buf",
        );
        unsafe { *self.buf.get_unchecked_mut(self.len as usize) = MaybeUninit::new(byte) };
        self.len += 1;
    }
}

impl LEB128 for Vec<u8> {
    #[inline]
    fn write_byte(&mut self, byte: u8) {
        self.push(byte);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leb128_buf_bound_check() {
        let mut buf = unsafe { Leb128Buf::<4>::new() };
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.as_bytes(), &[]);

        for i in 0..4 {
            buf.write_byte(i);
        }
        assert_eq!(buf.len(), 4);
        assert_eq!(buf.as_bytes(), &[0, 1, 2, 3]);
    }

    fn encoded_varint64_len(value: u64) -> usize {
        // Bitwise-or'ing by 1 allows the `value = zero` case to work without affecting other cases.
        let significant_bits = 64 - (value | 1).leading_zeros();
        (significant_bits + 6) as usize / 7
    }

    fn decode_varint64(mut buf: &[u8]) -> Result<u64> {
        read_u64(&mut buf)
    }

    fn encode_varint64(num: u64) -> Vec<u8> {
        let mut buf = Vec::with_capacity(10);
        buf.write_u64(num);
        buf
    }

    #[test]
    fn test_varint_encoding() {
        fn check(num: u64) {
            let encoded = encode_varint64(num);

            assert_eq!(encoded.len(), encoded_varint64_len(num));
            assert_eq!(num, decode_varint64(&encoded).unwrap());
        }

        for n in 0..1000 {
            check(n);
        }

        for n in (0..).map_while(|exp| 2u64.checked_pow(exp)) {
            check(n);
        }

        for n in (0..).map_while(|exp| 3u64.checked_pow(exp)) {
            check(n);
        }

        check(u64::MAX);
        check(u64::MAX - 1);

        check(i64::MAX as u64);
        check((i64::MAX as u64) + 1);
        check((i64::MAX as u64) - 1);

        check(u32::MAX as u64);
        check((u32::MAX as u64) + 1);
        check((u32::MAX as u64) - 1);

        check(i32::MAX as u64);
        check((i32::MAX as u64) + 1);
        check((i32::MAX as u64) - 1);
    }

    #[test]
    fn test_varint_decoding() {
        fn check(num: u64, encoded: &[u8]) {
            assert_eq!(encode_varint64(num), encoded);

            assert_eq!(encoded.len(), encoded_varint64_len(num));
            assert_eq!(num, decode_varint64(encoded).unwrap());
        }

        check(1, &[0x01]);
        check(127, &[0x7F]);
        check(128, &[0x80, 0x01]);
        check(300, &[0xAC, 0x02]);

        check(2u64.pow(14) - 1, &[0xFF, 0x7F]);
        check(2u64.pow(14), &[0x80, 0x80, 0x01]);

        check(2u64.pow(21) - 1, &[0xFF, 0xFF, 0x7F]);
        check(2u64.pow(21), &[0x80, 0x80, 0x80, 0x01]);

        check(2u64.pow(28) - 1, &[0xFF, 0xFF, 0xFF, 0x7F]);
        check(2u64.pow(28), &[0x80, 0x80, 0x80, 0x80, 0x01]);

        check(2u64.pow(35) - 1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
        check(2u64.pow(35), &[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);

        check(2u64.pow(42) - 1, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
        check(2u64.pow(42), &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);

        check(
            2u64.pow(49) - 1,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        check(
            2u64.pow(49),
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );

        check(
            2u64.pow(56) - 1,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        check(
            2u64.pow(56),
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );

        check(
            2u64.pow(63) - 1,
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F],
        );
        check(
            2u64.pow(63),
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );

        // ==========================================================================

        check(0, &[0]);
        check(
            u64::MAX,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        );
        check(u32::MAX as u64, &[0xff, 0xff, 0xff, 0xff, 0x0f]);

        assert!(
            decode_varint64(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02]).is_err()
        );
    }
}
