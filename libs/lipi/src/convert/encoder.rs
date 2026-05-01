use super::DataType;
use crate::bit_set;
use crate::varint::{LEB128, Leb128Buf};
use crate::{utils, zig_zag};
use std::collections::{BTreeMap, HashMap};
use std::io::{self, Result, Write};

pub fn encode_field_id_and_ty(
    writer: &mut (impl Write + ?Sized),
    num: u32,
    ty: DataType,
) -> Result<()> {
    if num < 15 {
        writer.write_all(&[(num as u8) << 4 | ty.code()])
    } else {
        let mut buf = unsafe { Leb128Buf::<6>::new() };
        buf.write_byte((0b_1111 << 4) | ty.code());
        buf.write_u32(num - 15);
        writer.write_all(buf.as_bytes())
    }
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

#[inline]
pub fn encode_len(writer: &mut (impl Write + ?Sized), len: usize) -> Result<()> {
    encode_uint(writer, len as u64)
}

pub fn encode_bytes(writer: &mut (impl Write + ?Sized), bytes: &[u8]) -> Result<()> {
    encode_len(writer, bytes.len())?;
    writer.write_all(bytes)
}

pub fn encode_list_len_and_ty(
    writer: &mut (impl Write + ?Sized),
    len: usize,
    ty: DataType,
) -> Result<()> {
    let len = u32::try_from(len).map_err(io::Error::other)?;
    encode_field_id_and_ty(writer, len, ty)
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
        encode_list_len_and_ty(writer, len, Self::TY)?;

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
        encode_field_id_and_ty(writer, id.into(), DataType::from(*self))
    }
}

impl<T: Encode + ?Sized> EnumEncoder for T {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        encode_field_id_and_ty(writer, id.into(), T::TY)?;
        Encode::encode(self, writer)
    }
}

// ------------------------------------------------------------------------

pub trait FieldEncoder {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()>;
}

impl FieldEncoder for bool {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        encode_field_id_and_ty(writer, id.into(), DataType::from(*self))
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
        encode_field_id_and_ty(writer, id.into(), T::TY)?;
        Encode::encode(self, writer)
    }
}

// ------------------------------- Macros ------------------------------------

macro_rules! encode {
    [$( $( $ty:ty ),* = $dt:tt ($self:tt, $w:tt) $encode:block)*] => [$(
        $(
            impl Encode for $ty {
                const TY: DataType = DataType::$dt;
                #[inline] fn encode(&$self, $w: &mut (impl Write + ?Sized)) -> Result<()> $encode
            }
        )*
    )*];
}

macro_rules! encode_types {
    [$( $ty:ty = $dt:tt $([ $( $param:tt )* ])? $(where {$( $where:tt )*} )? ($self:tt, $w:tt) $encode:block )*] => [$(
        impl <$($($param)*)?> Encode for $ty
        $(where $( $where )* )? {
            const TY: DataType = DataType::$dt;
            #[inline] fn encode(&$self, $w: &mut (impl Write + ?Sized)) -> Result<()> $encode
        }
    )*];
}

// ------------------------------- Number ------------------------------------

impl Encode for u8 {
    const TY: DataType = DataType::U8;
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> io::Result<()> {
        writer.write_all(&[*self])
    }

    fn encode_slice(writer: &mut (impl Write + ?Sized), this: &[Self]) -> io::Result<()> {
        encode_list_len_and_ty(writer, this.len(), Self::TY)?;
        writer.write_all(this)
    }
}

impl Encode for i8 {
    const TY: DataType = DataType::I8;
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> io::Result<()> {
        writer.write_all(&[self.cast_unsigned()])
    }

    fn encode_slice(writer: &mut (impl Write + ?Sized), this: &[Self]) -> io::Result<()> {
        encode_list_len_and_ty(writer, this.len(), Self::TY)?;
        writer.write_all(utils::u8_slice_from(this))
    }
}

encode! {
    f32 = F32 (self, writer) {
        writer.write_all(&self.to_le_bytes())
    }

    f64 = F64 (self, writer) {
        writer.write_all(&self.to_le_bytes())
    }

    u16, u32, u64 = UInt (self, writer) {
        encode_uint(writer, (*self).into())
    }

    i16, i32, i64 = Int (self, writer) {
        encode_int(writer, (*self).into())
    }

    char = UInt (self, writer) {
        encode_uint(writer, u32::from(*self).into())
    }

    str, String = Str (self, writer) {
        encode_bytes(writer, self.as_bytes())
    }

    // --------------------------------- List ----------------------------------

    [bool] = List (self, writer) {
        encode_list_len_and_ty(writer, self.len(), DataType::True)?;
        writer.write_all(&bit_set::bitvec_from(self))
    }
}

// --------------------------------- List ----------------------------------

macro_rules! encode_list {
    [$($ty: ty),*] => [$(
        impl<T: Encode> Encode for $ty {
            const TY: DataType = DataType::List;
            #[inline] fn encode(&self, writer: & mut (impl Write + ?Sized)) -> Result<()> {
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

encode_types! {
    [T] = List [T: Encode] (self, writer) {
        T::encode_slice(writer, self)
    }

    Vec<T> = List [T] where { [T]: Encode } (self, writer) {
        <[T] as Encode>::encode(self, writer)
    }

    [T; N] = List [T, const N: usize] where { [T]: Encode } (self, writer) {
        <[T] as Encode>::encode(self, writer)
    }

    // --------------------------------- Table ----------------------------------

    HashMap<K, V> = Table [K: Encode, V: Encode] (self, writer) {
        encode_map(writer, self.len(), || self.keys(), || self.values())
    }

    BTreeMap<K, V> = Table [K: Encode, V: Encode] (self, writer) {
        encode_map(writer, self.len(), || self.keys(), || self.values())
    }
}

// --------------------------------- Table ----------------------------------

fn encode_map<'a, K, V, Keys, Vals>(
    writer: &mut (impl Write + ?Sized),
    len: usize,
    keys: impl FnOnce() -> Keys,
    vals: impl FnOnce() -> Vals,
) -> Result<()>
where
    K: Encode + 'a,
    V: Encode + 'a,
    Keys: Iterator<Item = &'a K>,
    Vals: Iterator<Item = &'a V>,
{
    encode_len(writer, 2)?; // Column count
    encode_len(writer, len)?; // row count

    encode_field_id_and_ty(writer, 0, K::TY)?; // column id & ty
    for key in keys() {
        K::encode(key, writer)?; // values for first column
    }

    // second column
    encode_field_id_and_ty(writer, 1, V::TY)?;
    for val in vals() {
        V::encode(val, writer)?;
    }
    Ok(())
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

/// Tuples encoded as Struct.
macro_rules! tuples {
    [Len: $len:tt $($name:tt : $idx:tt)*] => {
        impl<$($name,)*> Encode for ($($name,)*)
        where
            $($name: FieldEncoder,)*
        {
            const TY: DataType = DataType::Struct;
            fn encode(&self, writer: & mut (impl Write + ?Sized)) -> Result<()> {
                encode_len(writer, $len)?; // field count
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
