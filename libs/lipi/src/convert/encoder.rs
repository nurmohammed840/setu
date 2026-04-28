use super::DataType;
use crate::bit_set::BitSet;
use crate::varint::{LEB128, Leb128Buf};
use crate::{utils, zig_zag};
use std::io::{self, Result, Write};

pub fn encode_header(writer: &mut (impl Write + ?Sized), num: u32, ty: DataType) -> Result<()> {
    if num < 15 {
        writer.write_all(&[(num as u8) << 4 | ty.code()])
    } else {
        let mut buf = unsafe { Leb128Buf::<6>::new() };
        buf.write_byte((0b_1111 << 4) | ty.code());
        buf.write_u32(num - 15);
        writer.write_all(buf.as_bytes())
    }
}

pub fn encode_bytes(writer: &mut (impl Write + ?Sized), bytes: &[u8]) -> Result<()> {
    encode_length(writer, bytes.len())?;
    writer.write_all(bytes)
}

pub fn encode_uint(writer: &mut (impl Write + ?Sized), num: u64) -> Result<()> {
    let mut buf = unsafe { Leb128Buf::<10>::new() };
    buf.write_u64(num);
    writer.write_all(buf.as_bytes())
}

#[inline]
pub fn encode_int(writer: &mut (impl Write + ?Sized), num: i64) -> Result<()> {
    encode_uint(writer, zig_zag::zigzag_encode(num))
}

pub fn encode_list_type(
    writer: &mut (impl Write + ?Sized),
    len: usize,
    ty: DataType,
) -> Result<()> {
    let len = u32::try_from(len).map_err(io::Error::other)?;
    encode_header(writer, len, ty)
}

pub fn encode_length(writer: &mut (impl Write + ?Sized), length: usize) -> Result<()> {
    let mut buf = unsafe { Leb128Buf::<10>::new() };
    buf.write_u64(length as u64);
    writer.write_all(buf.as_bytes())
}

// ------------------------------------------------------------------------

pub trait Encode {
    const TY: DataType;
    fn encode(&self, _: &mut (impl Write + ?Sized)) -> io::Result<()>;

    fn encode_slice(writer: &mut (impl Write + ?Sized), this: &[Self]) -> io::Result<()>
    where
        Self: Sized,
    {
        Self::encode_iter(writer, this.len(), this.iter())
    }

    fn encode_iter<'a, I>(writer: &mut (impl Write + ?Sized), len: usize, iter: I) -> io::Result<()>
    where
        I: Iterator<Item = &'a Self> + Clone,
        Self: Sized + 'a,
    {
        encode_list_type(writer, len, Self::TY)?;

        for val in iter {
            Self::encode(val, writer)?;
        }
        Ok(())
    }
}

// ------------------------------------------------------------------------

pub trait EnumEncoder {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()>;
}

impl EnumEncoder for bool {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        encode_header(writer, id.into(), DataType::from(*self))
    }
}

impl<T: Encode + ?Sized> EnumEncoder for T {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        encode_header(writer, id.into(), T::TY)?;
        Encode::encode(self, writer)
    }
}

// ------------------------------------------------------------------------

pub trait FieldEncoder {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()>;
}

impl FieldEncoder for bool {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        encode_header(writer, id.into(), DataType::from(*self))
    }
}

impl<T: FieldEncoder> FieldEncoder for Option<T> {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        match self {
            Some(val) => FieldEncoder::encode(val, writer, id),
            None => Ok(()),
        }
    }
}

impl<T: Encode + ?Sized> FieldEncoder for T {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        encode_header(writer, id.into(), T::TY)?;
        Encode::encode(self, writer)
    }
}

// ------------------------------- Number ------------------------------------

impl Encode for u8 {
    const TY: DataType = DataType::U8;
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> io::Result<()> {
        writer.write_all(&[*self])
    }

    fn encode_slice(writer: &mut (impl Write + ?Sized), this: &[Self]) -> io::Result<()> {
        encode_list_type(writer, this.len(), Self::TY)?;
        writer.write_all(this)
    }
}

impl Encode for i8 {
    const TY: DataType = DataType::I8;
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> io::Result<()> {
        writer.write_all(&[self.cast_unsigned()])
    }

    fn encode_slice(writer: &mut (impl Write + ?Sized), this: &[Self]) -> io::Result<()> {
        encode_list_type(writer, this.len(), Self::TY)?;
        writer.write_all(utils::u8_slice_from(this))
    }
}

impl Encode for f32 {
    const TY: DataType = DataType::F32;
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl Encode for f64 {
    const TY: DataType = DataType::F64;
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl Encode for char {
    const TY: DataType = DataType::UInt;
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        writer.write_all(&u32::from(*self).to_le_bytes())
    }
}

macro_rules! encode_for {
    [@uint: $($ty: ty),*] => [$(
        impl Encode for $ty {
            const TY: DataType = DataType::UInt;
            #[inline]
            fn encode(&self, writer: & mut (impl Write + ?Sized)) -> Result<()> {
                encode_uint(writer, (*self).into())
            }
        }
    )*];
    [@int: $($ty: ty),*] => [$(
        impl Encode for $ty {
            const TY: DataType = DataType::Int;
            #[inline]
            fn encode(&self, writer: & mut (impl Write + ?Sized)) -> Result<()> {
                encode_int(writer, (*self).into())
            }
        }
    )*];
    [@string: $($ty: ty),*] => [$(
        impl Encode for $ty {
            const TY: DataType = DataType::Str;
            #[inline]
            fn encode(&self, writer: & mut (impl Write + ?Sized)) -> Result<()> {
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

// impl<Bytes: AsRef<[u8]>> Encode for BitSet<Bytes> {
//     const TY: DataType = DataType::List;
//     fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
//         encode_list_type(writer, self.len(), DataType::True)?;
//         writer.write_all(self.as_bytes())
//     }
// }

impl Encode for [bool] {
    const TY: DataType = DataType::List;
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        // let bs: BitSet<Vec<u8>> = BitSet::from(self);
        // Encode::encode(&bs, writer)
        todo!()
    }
}

macro_rules! encode_list {
    [$($ty: ty),*] => [$(
        impl<T: Encode> Encode for $ty {
            const TY: DataType = DataType::List;
            fn encode(&self, writer: & mut (impl Write + ?Sized)) -> Result<()> {
                T::encode_iter(writer, self.len(), self.iter())
            }
        }
    )*];
}

encode_list! {
    std::collections::HashSet<T>,
    std::collections::BTreeSet<T>,
    std::collections::LinkedList<T>,
    std::collections::VecDeque<T>,
    std::collections::BinaryHeap<T>
}

impl<T: Encode> Encode for [T] {
    const TY: DataType = DataType::List;
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        T::encode_slice(writer, self)
    }
}

impl<T> Encode for Vec<T>
where
    [T]: Encode,
{
    const TY: DataType = DataType::List;
    #[inline]
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        <[T] as Encode>::encode(self, writer)
    }
}

impl<T, const N: usize> Encode for [T; N]
where
    [T]: Encode,
{
    const TY: DataType = DataType::List;
    #[inline]
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        <[T] as Encode>::encode(self, writer)
    }
}

// --------------------------------- Table ----------------------------------

macro_rules! encode_map {
    [$($ty: ty),*] => [$(
        impl<K: Encode, V: Encode> Encode for $ty {
            const TY: DataType = DataType::Table;
            fn encode(&self, writer: & mut (impl Write + ?Sized)) -> Result<()> {
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

// --------------------------------- Deref ----------------------------------

macro_rules! deref_impl {
    [$($ty: ty),*] => [$(
        impl<T: ?Sized + Encode> Encode for $ty {
            const TY: DataType = T::TY;
            #[inline]
            fn encode(&self, writer: & mut (impl Write + ?Sized)) -> Result<()> {
                Encode::encode(&**self, writer)
            }
        }
    )*]
}

deref_impl! {
    &T, &mut T, Box<T>
}

// --------------------------------- Other ----------------------------------

macro_rules! tuples {
    [Len: $len:tt $($name:tt : $idx:tt)*] => {
        impl<$($name,)*> Encode for ($($name,)*)
        where
            $($name: FieldEncoder,)*
        {
            const TY: DataType = DataType::Struct;
            fn encode(&self, writer: & mut (impl Write + ?Sized)) -> Result<()> {
                encode_length(writer, $len)?;
                $($name::encode(&self.$idx, writer, $idx)?;)*
                Ok(())
            }
        }
    }
}

tuples! { Len: 1 T0:0 }
tuples! { Len: 2 T0:0 T1:1 }
tuples! { Len: 3 T0:0 T1:1 T2:2 }
tuples! { Len: 4 T0:0 T1:1 T2:2 T3:3 }
tuples! { Len: 5 T0:0 T1:1 T2:2 T3:3 T4:4 }
tuples! { Len: 6 T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 }
tuples! { Len: 7 T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 }
tuples! { Len: 8 T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 }
tuples! { Len: 9 T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 }
tuples! { Len:10 T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 }
tuples! { Len:11 T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 }
tuples! { Len:12 T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 }
tuples! { Len:13 T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 }
tuples! { Len:14 T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 T13:13 }
tuples! { Len:15 T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 T13:13 T14:14 }
