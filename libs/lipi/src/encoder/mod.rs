mod field;
mod optional_field;

use super::DataType;
use crate::bit_set;
use crate::varint::{LEB128, Leb128Buf};
use crate::{utils, zig_zag};
use std::collections::{BTreeMap, HashMap};
use std::io::{self, Result, Write};

pub use field::Field;
pub use optional_field::OptionalField;

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

    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.encode(&mut buf)?;
        Ok(buf)
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
        #[allow(clippy::useless_conversion)]
        encode_uint(writer, (*self).into())
    }

    i16, i32, i64 = Int (self, writer) {
        #[allow(clippy::useless_conversion)]
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
                T::encode(&**self, writer)
            }
        }
    )*]
}

deref_impl! {
    &T, &mut T, Box<T>
}

// --------------------------------- STD ----------------------------------

macro_rules! error_type {
    [$($ty: ty)*] => [$(
        impl Encode for $ty {
            const TY: DataType = DataType::Str;
            fn encode(&self, w: &mut (impl Write + ?Sized)) -> io::Result<()> {
                encode_bytes(w, self.to_string().as_bytes())
            }
        }
    )*];
}

error_type! {
    dyn std::error::Error
    dyn std::error::Error + Send
    dyn std::error::Error + Send + Sync
}

impl<T, E> Encode for std::result::Result<T, E>
where
    T: OptionalField,
    E: Field,
{
    const TY: DataType = DataType::Struct;
    #[inline]
    fn encode(&self, w: &mut (impl Write + ?Sized)) -> io::Result<()> {
        match self {
            Ok(val) => OptionalField::encode(val, w, 0)?,
            Err(err) => Field::encode(err, w, 1)?,
        };
        w.write_all(&[DataType::StructEnd.code()])
    }
}

// --------------------------------- Other ----------------------------------

/// Tuples encoded as Struct.
macro_rules! tuples {
    [$($name:tt : $idx:tt)*] => {
        impl<$($name,)*> Encode for ($($name,)*)
        where
            $($name: OptionalField,)*
        {
            const TY: DataType = DataType::Struct;
            fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
                $($name::encode(&self.$idx, writer, $idx)?;)*
                writer.write_all(&[DataType::StructEnd.code()])
            }
        }
    }
}

tuples! {}
tuples! { T0:0 }
tuples! { T0:0 T1:1 }
tuples! { T0:0 T1:1 T2:2 }
tuples! { T0:0 T1:1 T2:2 T3:3 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 T13:13 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 T13:13 T14:14 }
