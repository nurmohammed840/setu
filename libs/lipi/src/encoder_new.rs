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

pub fn encode_length(writer: &mut dyn Write, length: usize) -> Result<()> {
    let len = u32::try_from(length).map_err(io::Error::other)?;

    let mut buf = Leb128Buf::<5>::new();
    buf.write_u32(len);
    writer.write_all(buf.as_bytes())
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

// ------------------------------------------------------------------------

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

// ----------------------------------------------------------------------

// trait Item {
//     const TY: u8;
//     fn encode(&self, writer: &mut dyn Write) -> Result<()>;
// }

// impl<T: Encoder> FieldEncoder for T  {
//     fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
//         todo!()
//     }
// }

// impl<T: Item> FieldEncoder for [T] {
//     fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
//         todo!()
//     }
// }

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
