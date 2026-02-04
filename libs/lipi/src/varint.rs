use crate::{errors, utils};

use super::Result;

pub trait LEB128 {
    fn write_byte(&mut self, byte: u8);

    fn write_u32(&mut self, mut num: u32) {
        while num > 0b0111_1111 {
            self.write_byte((num as u8) | 0b1000_0000); // Set continuation bit
            num >>= 7; // Shift right by 7 bits
        }
        self.write_byte(num as u8); // Push last byte without continuation bit
    }

    fn write_u64(&mut self, mut num: u64) {
        while num > 0b0111_1111 {
            self.write_byte((num as u8) | 0b1000_0000); // Set continuation bit
            num >>= 7; // Shift right by 7 bits
        }
        self.write_byte(num as u8); // Push last byte without continuation bit
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
        if self.pos as usize >= N {
            panic!("Buffer overflow");
        }
        self.buf[self.pos as usize] = byte; // Write the byte to the current position
        self.pos += 1; // Move the position forward
    }
}

pub fn read_unsigned(reader: &mut &[u8]) -> Result<u64> {
    let mut result = 0;
    let mut shift = 0;

    loop {
        let byte = utils::read_byte(reader)?;
        result |= ((byte & 0x7F) as u64) << shift; // low-order 7 bits of value

        if (byte & 0x80) == 0 {
            break Ok(result); // No continuation bit, end of LEB128
        }
        if shift >= 64 {
            return Err(errors::VarIntError.into());
        }
        shift += 7;
    }
}
