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

fn encode_list_type(writer: &mut dyn Write, len: usize, ty: u8) -> Result<()> {
    let len = u32::try_from(len).map_err(io::Error::other)?;
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

impl<T: FieldEncoder> FieldEncoder for Option<T> {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        match self {
            Some(val) => FieldEncoder::encode(val, writer, id),
            None => Ok(()),
        }
    }
}

impl<T: Item + ?Sized> FieldEncoder for T {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        encode_header(writer, id.into(), T::TY)?;
        Item::encode(self, writer)
    }
}

// ------------------------------------------------------------------------

pub trait Item {
    const TY: u8;
    fn encode(&self, writer: &mut dyn Write) -> Result<()>;
}

impl Item for f32 {
    const TY: u8 = 4;
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl Item for f64 {
    const TY: u8 = 5;
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl Item for char {
    const TY: u8 = 6;
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(&u32::from(*self).to_le_bytes())
    }
}

macro_rules! encode_for {
    [@uint: $($ty: ty),*] => [$(
        impl Item for $ty {
            const TY: u8 = 6;
            fn encode(&self, writer: &mut dyn Write) -> Result<()> {
                encode_uint(writer, (*self).into())
            }
        }
    )*];
    [@int: $($ty: ty),*] => [$(
        impl Item for $ty {
            const TY: u8 = 7;
            fn encode(&self, writer: &mut dyn Write) -> Result<()> {
                encode_int(writer, (*self).into())
            }
        }
    )*];
    [@string: $($ty: ty),*] => [$(
        impl Item for $ty {
            const TY: u8 = 8;
            fn encode(&self, writer: &mut dyn Write) -> Result<()> {
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

// --------------------------------- List ----------------------------------

impl<Bytes: AsRef<[u8]>> Item for BitSet<Bytes> {
    const TY: u8 = 11;
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        encode_list_type(writer, self.len(), 1)?;
        writer.write_all(self.as_bytes())
    }
}

impl Item for [bool] {
    const TY: u8 = 11;
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        let bs: BitSet<Vec<u8>> = BitSet::from(self);
        Item::encode(&bs, writer)
    }
}

impl Item for [u8] {
    const TY: u8 = 11;
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        encode_list_type(writer, self.len(), 2)?;
        writer.write_all(self)
    }
}

impl Item for [i8] {
    const TY: u8 = 11;
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        encode_list_type(writer, self.len(), 3)?;
        writer.write_all(utils::u8_slice_from(self))
    }
}

macro_rules! encode_list {
    [$($ty: ty),*] => [$(
        impl<T: Item> Item for $ty {
            const TY: u8 = 11;
            fn encode(&self, writer: &mut dyn Write) -> Result<()> {
                encode_list_type(writer, self.len(), T::TY)?;
                for val in self {
                    Item::encode(val, writer)?;
                }
                Ok(())
            }
        }
    )*];
}

encode_list! {
    [T],
    std::collections::HashSet<T>,
    std::collections::BTreeSet<T>,
    std::collections::LinkedList<T>,
    std::collections::VecDeque<T>
}

impl<T> Item for Vec<T>
where
    [T]: Item,
{
    const TY: u8 = 11;
    #[inline]
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        <[T] as Item>::encode(self, writer)
    }
}

impl<T, const N: usize> Item for [T; N]
where
    [T]: Item,
{
    const TY: u8 = 11;
    #[inline]
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        <[T] as Item>::encode(self, writer)
    }
}

// --------------------------------- Table ----------------------------------

macro_rules! encode_map {
    [$($ty: ty),*] => [$(
        impl<K: Item, V: Item> Item for $ty {
            const TY: u8 = 12;
            fn encode(&self, writer: &mut dyn Write) -> Result<()> {
                encode_length(writer, 2)?; // Columns len
                encode_length(writer, self.len())?; // Values len

                encode_header(writer, 0, K::TY)?;
                for key in self.keys() {
                    K::encode(key, writer)?;
                }

                encode_header(writer, 1, V::TY)?;
                for value in self.values() {
                    V::encode(value, writer)?;
                }
                Ok(())
            }
        }
    )*];
}

encode_map! {
    std::collections::HashMap<K, V>,
    std::collections::BTreeMap<K, V>
}

// --------------------------------------------------------------------------

macro_rules! deref_impl {
    [$($ty: ty),*] => [$(
        impl<T: ?Sized + Item> Item for $ty {
            const TY: u8 = T::TY;
            #[inline]
            fn encode(&self, writer: &mut dyn Write) -> Result<()> {
                Item::encode(&**self, writer)
            }
        }
    )*]
}

deref_impl! {
    &T, &mut T, Box<T>
}
