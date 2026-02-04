#![allow(clippy::unusual_byte_groupings)]
use super::Result;
use crate::{errors, utils};

/// See: https://en.wikipedia.org/wiki/LEB128
pub trait LEB128 {
    fn write_byte(&mut self, byte: u8);

    fn write_u32(&mut self, mut num: u32) {
        while num > 0b_111_1111 {
            self.write_byte((num as u8) | 0b_1000_0000); // Set continuation bit
            num >>= 7; // Shift right by 7 bits
        }
        self.write_byte(num as u8); // Push last byte without continuation bit
    }

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

        result |= ((byte & 0b_111_1111) as u64) << shift; // low-order 7 bits of value

        if (byte & 0b_1000_0000) == 0 {
            break Ok(result); // No continuation bit, end of LEB128
        }

        shift += 7;
    }
}

#[derive(Debug)]
pub struct Leb128Buf<const N: usize> {
    buf: [u8; N],
    pos: u8,
}

impl<const N: usize> Leb128Buf<N> {
    pub fn new() -> Self {
        Self {
            buf: [0; N],
            pos: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.pos as usize]
    }
}

impl<const N: usize> LEB128 for Leb128Buf<N> {
    fn write_byte(&mut self, byte: u8) {
        self.buf[self.pos as usize] = byte;
        self.pos += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded_varint64_len(value: u64) -> usize {
        // Bitwise-or'ing by 1 allows the `value = zero` case to work without affecting other cases.
        let significant_bits = 64 - (value | 1).leading_zeros();
        (significant_bits + 6) as usize / 7
    }

    fn check(num: u64) {
        let mut buf = Leb128Buf::<10>::new();
        buf.write_u64(num);
        assert_eq!(buf.as_bytes().len(), encoded_varint64_len(num));
    }

    #[test]
    fn test_varint_encoding() {
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

    fn decode_varint64(mut buf: &[u8]) -> Result<u64> {
        read_u64(&mut buf)
    }

    #[test]
    fn test_varint_decoding() {
        assert_eq!(0, decode_varint64(&[0]).unwrap());
        assert_eq!(
            u64::MAX,
            decode_varint64(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]).unwrap()
        );
        assert!(
            decode_varint64(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02]).is_err()
        );
    }
}
