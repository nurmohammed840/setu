use crate::bit_set::BitSet;

use super::*;
use std::{io::Result, u32};
use varint::{LEB128, Leb128Buf};

fn encode_header(writer: &mut dyn Write, num: u32, ty: u8) -> Result<()> {
    if num < 15 {
        writer.write_all(&[(num as u8) << 4 | ty])
    } else {
        let mut buf = Leb128Buf::<6>::new();
        buf.write_byte((0b_1111 << 4) | ty);
        buf.write_u32(num - 15);
        writer.write_all(buf.as_bytes())
    }
}

fn encode_bytes(writer: &mut dyn Write, bytes: &[u8]) -> Result<()> {
    encode_length(writer, bytes.len())?;
    writer.write_all(bytes)
}

fn encode_uint(writer: &mut dyn Write, num: u64) -> Result<()> {
    let mut buf = Leb128Buf::<10>::new();
    buf.write_u64(num);
    writer.write_all(buf.as_bytes())
}

fn encode_int(writer: &mut dyn Write, num: i64) -> Result<()> {
    encode_uint(writer, zig_zag::into_u64(num))
}

fn encode_header_list(writer: &mut dyn Write, id: u16, len: usize, ty: u8) -> Result<()> {
    let len = u32::try_from(len).map_err(io::Error::other)?;
    encode_header(writer, id.into(), 11)?;
    encode_header(writer, len, ty)
}

// ------------------------------------------------------------------------

pub fn encode_length(writer: &mut dyn Write, length: usize) -> Result<()> {
    let len = u32::try_from(length).map_err(io::Error::other)?;

    let mut buf = Leb128Buf::<5>::new();
    buf.write_u32(len);
    writer.write_all(buf.as_bytes())
}

pub fn encode_struct<T: Encoder>(this: &T, writer: &mut dyn Write, id: u16) -> Result<()> {
    encode_header(writer, id.into(), 9)?;
    T::encode(this, writer)
}

// ------------------------------------------------------------------------

pub trait FieldEncoder {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()>;
}

impl FieldEncoder for bool {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        let ty = match self {
            false => 0,
            true => 1,
        };
        encode_header(writer, id.into(), ty)
    }
}

impl FieldEncoder for u8 {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        encode_header(writer, id.into(), 2)?;
        writer.write_all(&[*self])
    }
}

impl FieldEncoder for i8 {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        encode_header(writer, id.into(), 3)?;
        writer.write_all(&[self.cast_unsigned()])
    }
}

impl FieldEncoder for f32 {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        encode_header(writer, id.into(), 4)?;
        writer.write_all(&self.to_le_bytes())
    }
}

impl FieldEncoder for f64 {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        encode_header(writer, id.into(), 5)?;
        writer.write_all(&self.to_le_bytes())
    }
}

macro_rules! encode_for {
    [@uint: $($ty: ty),*] => [$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
                encode_header(writer, id.into(), 6)?;
                encode_uint(writer, (*self).into())
            }
        }
    )*];
    [@int: $($ty: ty),*] => [$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
                encode_header(writer, id.into(), 7)?;
                encode_int(writer, (*self).into())
            }
        }
    )*];
    [@string: $($ty: ty),*] => [$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
                encode_header(writer, id.into(), 8)?;
                encode_bytes(writer, self.as_bytes())
            }
        }
    )*];
}

encode_for! {
    @uint: u16, u32, u64
}
encode_for! {
    @int: i16, i32, i64
}
encode_for! {
    @string: str, String
}

// ---------------------------------------------------------------------

impl<Bytes: AsRef<[u8]>> FieldEncoder for BitSet<Bytes> {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        encode_header_list(writer, id, self.len(), 1)?;
        writer.write_all(self.as_bytes())
    }
}

macro_rules! encode_list {
    [@bools: $($ty: ty),*] => [$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
                let bs = BitSet::<Vec<u8>>::from(self);
                bs.encode(writer, id)
            }
        }
    )*];
    [@bytes: $($ty: ty),*] => [$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
                encode_header_list(writer, id.into(), self.len(), 2)?;
                writer.write_all(self)
            }
        }
    )*];
    [@i8_bytes: $($ty: ty),*] => [$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
                encode_header_list(writer, id.into(), self.len(), 3)?;
                writer.write_all(utils::u8_slice_from(self))
            }
        }
    )*];
}

encode_list! {
    @bools: [bool], Vec<bool>
}

encode_list! {
    @bytes: [u8], Vec<u8>
}

encode_list! {
    @i8_bytes: [i8], Vec<i8>
}

// ----------------------------------------------------------------------

trait Item {
    const TY: u8;
    fn encode(&self, _writer: &mut dyn Write) -> Result<()>;
}

impl Item for f32 {
    const TY: u8 = 4;
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        todo!()
    }
}

impl Item for f64 {
    const TY: u8 = 5;
    fn encode(&self, _writer: &mut dyn Write) -> Result<()> {
        todo!()
    }
}
impl Item for u16 {
    const TY: u8 = 6;

    fn encode(&self, _writer: &mut dyn Write) -> Result<()> {
        todo!()
    }
}
impl Item for u32 {
    const TY: u8 = 6;
    fn encode(&self, _writer: &mut dyn Write) -> Result<()> {
        todo!()
    }
}
impl Item for u64 {
    const TY: u8 = 6;
    fn encode(&self, _writer: &mut dyn Write) -> Result<()> {
        todo!()
    }
}

impl Item for i16 {
    const TY: u8 = 7;

    fn encode(&self, _writer: &mut dyn Write) -> Result<()> {
        todo!()
    }
}
impl Item for i32 {
    const TY: u8 = 7;

    fn encode(&self, _writer: &mut dyn Write) -> Result<()> {
        todo!()
    }
}

impl Item for i64 {
    const TY: u8 = 7;

    fn encode(&self, _writer: &mut dyn Write) -> Result<()> {
        todo!()
    }
}

impl<T: Item> Item for [T] {
    const TY: u8 = 0;
    fn encode(&self, _writer: &mut dyn Write) -> Result<()> {
        todo!()
    }
}

impl<T: Item> Item for Vec<T> {
    const TY: u8 = 0;
    fn encode(&self, _writer: &mut dyn Write) -> Result<()> {
        todo!()
    }
}

impl<T: Item> Item for &T {
    const TY: u8 = T::TY;
    fn encode(&self, _writer: &mut dyn Write) -> Result<()> {
        todo!()
    }
}

// ----------------------------------------------------------------------

impl<T: Item> FieldEncoder for [T] {
    fn encode(&self, _: &mut dyn Write, _: u16) -> Result<()> {
        todo!()
    }
}

impl<T: Item> FieldEncoder for Vec<T> {
    fn encode(&self, _: &mut dyn Write, _: u16) -> Result<()> {
        todo!()
    }
}

// ----------------------------------------------------------------------

impl<T: FieldEncoder> FieldEncoder for Option<T> {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        match self {
            Some(val) => FieldEncoder::encode(val, writer, id),
            None => Ok(()),
        }
    }
}

// impl<T: FieldEncoder, const N: usize> FieldEncoder for [T; N]  {
//     fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
//         todo!()
//     }
// }

macro_rules! deref_impl {
    [$($ty: ty),*] => [$(
        impl<T: ?Sized + FieldEncoder> FieldEncoder for $ty {
            #[inline]
            fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
                FieldEncoder::encode(*self, writer, id)
            }
        }
    )*]
}

deref_impl! {
    &T, &mut T
}
