use crate::{Result, errors};

pub fn read_byte(buf: &mut &[u8]) -> Result<u8> {
    if !buf.is_empty() {
        unsafe {
            let byte = *buf.get_unchecked(0);
            *buf = buf.get_unchecked(1..);
            Ok(byte)
        }
    } else {
        Err(errors::UnexpectedEof.into())
    }
}

pub fn read_buf<const N: usize>(reader: &mut &[u8]) -> Result<[u8; N]> {
    read_bytes(reader, N).map(|bytes| bytes.try_into().unwrap())
}

pub fn read_bytes<'de>(reader: &mut &'de [u8], len: usize) -> Result<&'de [u8]> {
    if len <= reader.len() {
        unsafe {
            let slice = reader.get_unchecked(..len);
            *reader = reader.get_unchecked(len..);
            Ok(slice)
        }
    } else {
        Err(errors::UnexpectedEof.into())
    }
}
