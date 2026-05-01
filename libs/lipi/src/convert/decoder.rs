use crate::assert_or_err;
use crate::{Result, bit_set, convert::DataType, errors, utils, varint, zig_zag};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::Hash;

pub fn decode_field_id_and_ty(reader: &mut &[u8]) -> Result<(u64, DataType)> {
    let byte = utils::read_byte(reader)?;

    let ty = byte & 0b_1111;
    let id = (byte >> 4) as u64;

    let id = if id == 0b_1111 {
        varint::read_u64(reader)? + 15
    } else {
        id
    };

    Ok((id, unsafe { std::mem::transmute(ty) }))
}

pub fn decode_packed_bools<'de>(reader: &mut &'de [u8], len: usize) -> Result<Vec<bool>> {
    let packed = utils::read_bytes(reader, utils::bool_packed_len(len))?;
    Ok(bit_set::bitvec_to_bools(len, packed))
}

pub fn decode_len(reader: &mut &[u8]) -> Result<usize> {
    Ok(usize::try_from(varint::read_u64(reader)?)?)
}

pub fn decode_bytes<'de>(reader: &mut &'de [u8]) -> Result<&'de [u8]> {
    let len = decode_len(reader)?;
    Ok(utils::read_bytes(reader, len)?)
}

pub fn decode_str<'de>(reader: &mut &'de [u8]) -> Result<&'de str> {
    Ok(str::from_utf8(decode_bytes(reader)?)?)
}

pub fn decode_list_len_and_ty(reader: &mut &[u8]) -> Result<(usize, DataType)> {
    let (len, ty) = decode_field_id_and_ty(reader)?;
    Ok((usize::try_from(len)?, ty))
}

// fn decode_or_convert<T>(ty: DataType, expected: DataType, reader: &mut &[u8]) -> Result<T>
// where
//     T: TryFrom<u8> + TryFrom<i8> + TryFrom<u64> + TryFrom<i64>,
//     <T as TryFrom<u8>>::Error: std::error::Error + Send + Sync + 'static,
//     <T as TryFrom<i8>>::Error: std::error::Error + Send + Sync + 'static,
//     <T as TryFrom<u64>>::Error: std::error::Error + Send + Sync + 'static,
//     <T as TryFrom<i64>>::Error: std::error::Error + Send + Sync + 'static,
// {
//     match ty {
//         DataType::U8 => Ok(T::try_from(u8::decode(reader)?)?),
//         DataType::I8 => Ok(T::try_from(i8::decode(reader)?)?),
//         DataType::UInt => Ok(T::try_from(u64::decode(reader)?)?),
//         DataType::Int => Ok(T::try_from(i64::decode(reader)?)?),
//         _ => Err(errors::InvalidType::error(ty, expected)),
//     }
// }

// -----------------------------------------------------------------------------

pub trait Decode<'de>: Sized {
    const TY: DataType;
    fn decode(reader: &mut &'de [u8]) -> Result<Self>;

    fn decode_or_convert(ty: DataType, reader: &mut &'de [u8]) -> Result<Self> {
        ty.expected(Self::TY)?;
        Self::decode(reader)
    }

    fn decode_vec(reader: &mut &'de [u8]) -> Result<Vec<Self>> {
        let (len, ty) = decode_list_len_and_ty(reader)?;
        ty.expected(Self::TY)?;

        utils::try_collect(len, || Self::decode(reader))
    }

    fn decode_list<List>(
        reader: &mut &'de [u8],
        new: fn(len: usize) -> List,
        add: fn(list: &mut List, val: Self),
    ) -> Result<List> {
        let (len, ty) = decode_list_len_and_ty(reader)?;
        ty.expected(Self::TY)?;

        let mut list = new(len);
        for _ in 0..len {
            add(&mut list, Self::decode(reader)?);
        }
        Ok(list)
    }
}

// -------------------------------------- Macros  ------------------------------------

macro_rules! decode_types {
    [$($ty:ty = $expected:tt { $($decode:item)* })*] => [$(
        impl Decode<'_> for $ty {
            const TY: DataType = DataType::$expected;
            #[inline] $($decode)*
        }
    )*];
}

macro_rules! decode {
    [$($ty:ty = $expected:tt $([$($param:tt)*])? ($r:tt) $decode:block)*] => [$(
        impl<'de, $($($param)*)?> Decode<'de> for $ty {
            const TY: DataType = DataType::$expected;
            #[inline]
            fn decode($r: &mut &'de [u8]) -> Result<Self> $decode
        }
    )*];
}

// -----------------------------------------------------------------------------------

decode_types! {
    u8 = U8 {
        fn decode(reader: &mut &[u8]) -> Result<Self> {
            Ok(utils::read_byte(reader)?)
        }

        fn decode_vec(reader: &mut &[u8]) -> Result<Vec<Self>> {
            <&[u8] as Decode>::decode(reader).map(|bytes| bytes.to_vec())
        }
    }

    i8 = I8 {
        fn decode(reader: &mut &[u8]) -> Result<Self> {
            Ok(utils::read_byte(reader).map(u8::cast_signed)?)
        }

        fn decode_vec(reader: &mut &[u8]) -> Result<Vec<Self>> {
            <&[i8] as Decode>::decode(reader).map(|bytes| bytes.to_vec())
        }
    }
}

decode! {
    f32 = F32 (reader) {
        Ok(utils::read_buf(reader).map(Self::from_le_bytes)?)
    }

    f64 = F64 (reader) {
        Ok(utils::read_buf(reader).map(Self::from_le_bytes)?)
    }

    u64 = UInt (reader) {
        varint::read_u64(reader)
    }
    i64 = Int (reader) {
        varint::read_u64(reader).map(zig_zag::zigzag_decode)
    }

    u16 = UInt (reader) {
        Ok(Self::try_from(u64::decode(reader)?)?)
    }
    u32 = UInt (reader) {
        Ok(Self::try_from(u64::decode(reader)?)?)
    }
    char = UInt (reader) {
        Ok(Self::try_from(u32::decode(reader)?)?)
    }

    i16 = Int (reader) {
        Ok(Self::try_from(i64::decode(reader)?)?)
    }
    i32 = Int (reader) {
        Ok(Self::try_from(i64::decode(reader)?)?)
    }

    &'de str = Str (reader) {
        decode_str(reader)
    }

    String = Str (reader) {
        decode_str(reader).map(Self::from)
    }

    // ==================== List ==========================

    &'de [u8] = List (reader) {
        let (len, ty) = decode_list_len_and_ty(reader)?;
        ty.expected(DataType::U8)?;
        Ok(utils::read_bytes(reader, len)?)
    }

    &'de [i8] = List (reader) {
       let (len, ty) = decode_list_len_and_ty(reader)?;
       ty.expected(DataType::I8)?;
       Ok(utils::read_bytes(reader, len).map(utils::i8_slice_from)?)
    }

    [T; N] = List [T: Decode<'de>, const N: usize] (reader) {
        let list = T::decode_vec(reader)?; // todo: remove unnecessary allocation
        let array = Self::try_from(list).map_err(|list| errors::InvalidArrayLen {
            expected: N,
            found: list.len(),
        });
        Ok(array?)
    }

    Vec<T> = List [T: Decode<'de>] (reader) {
        T::decode_vec(reader)
    }

    VecDeque<T> = List [T: Decode<'de>] (reader) {
        T::decode_vec(reader).map(Self::from)
    }

    HashSet<T> = List [T: Eq + Hash + Decode<'de>] (reader) {
        T::decode_list(reader, Self::with_capacity, |set, val| {
            Self::insert(set, val);
        })
    }

    BTreeSet<T> = List [T: Ord + Decode<'de>] (reader) {
        T::decode_list(reader, |_| Self::new(), |set, val| {
            Self::insert(set, val);
        })
    }

    BinaryHeap<T> = List [T: Ord + Decode<'de>] (reader) {
        T::decode_list(reader, Self::with_capacity, Self::push)
    }

    LinkedList<T> = List [T: Decode<'de>] (reader) {
        T::decode_list(reader, |_| Self::new(), Self::push_back)
    }

    // ==================== Map ===========================

    HashMap<K, V> = Table [K: Eq + Hash + Decode<'de>, V: Decode<'de>] (reader) {
        decode_map(reader, Self::with_capacity, |map, k, v| {
            Self::insert(map, k, v);
        })
    }

    BTreeMap<K, V> = Table [K: Ord + Decode<'de>, V: Decode<'de>] (reader) {
        decode_map(reader, |_| Self::new(), |map, k, v| {
            Self::insert(map, k, v);
        })
    }
}

// --------------------------------------- Table ----------------------------------------

fn decode_map<'de, Map, K, V>(
    reader: &mut &'de [u8],
    new: fn(len: usize) -> Map,
    add: fn(&mut Map, K, V),
) -> Result<Map>
where
    K: Decode<'de>,
    V: Decode<'de>,
{
    let col_count = decode_len(reader)?;
    assert_or_err!(
        col_count == 2,
        format!("invalid column count: expected `2`, found {col_count}")
    );

    let row_count = decode_len(reader)?;

    let (col_id, col_ty) = decode_field_id_and_ty(reader)?;

    if col_id == 0 {
        col_ty.expected(K::TY)?;
        let keys = utils::try_collect(row_count, || K::decode(reader))?;

        let (val_id, val_ty) = decode_field_id_and_ty(reader)?;
        assert_or_err!(
            val_id == 1,
            format!("invalid column (value) id: expected `1`, found {val_id}")
        );

        val_ty.expected(V::TY)?;

        let mut map = new(row_count);
        for key in keys {
            add(&mut map, key, V::decode(reader)?);
        }
        return Ok(map);
    }
    if col_id == 1 {
        col_ty.expected(V::TY)?;
        let vals = utils::try_collect(row_count, || V::decode(reader))?;

        let (key_id, key_ty) = decode_field_id_and_ty(reader)?;
        assert_or_err!(
            key_id == 0,
            format!("invalid column (key) id: expected `0`, found {key_id}")
        );
        key_ty.expected(K::TY)?;

        let mut map = new(row_count);
        for val in vals {
            add(&mut map, K::decode(reader)?, val);
        }
        return Ok(map);
    }

    Err(format!("invalid column id: expected `0` or `1`, found {col_id}").into())
}

// ---------------------------------------------------------------------------------

pub struct FieldDecoder<'c, 'de> {
    pub reader: &'c mut &'de [u8],
    len: usize,
}

impl<'c, 'de> FieldDecoder<'c, 'de> {
    #[inline]
    pub fn new(reader: &'c mut &'de [u8]) -> Result<Self> {
        let len = decode_len(reader)?;
        Ok(Self { reader, len })
    }

    #[inline]
    pub fn next_field_id_and_ty(&mut self) -> Result<Option<(u64, DataType)>> {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        decode_field_id_and_ty(self.reader).map(Some)
    }

    #[inline]
    pub fn decode<T: Decode<'de>>(&mut self) -> Result<Option<T>> {
        T::decode(self.reader).map(Some)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

pub trait Optional<T>: Sized {
    type Error;
    fn convert(val: Option<T>, _: &'static str) -> Result<Self, Self::Error>;
}

impl<T> Optional<T> for Option<T> {
    type Error = std::convert::Infallible;
    #[inline]
    fn convert(val: Option<T>, _: &'static str) -> Result<Self, Self::Error> {
        Ok(val)
    }
}

impl<T> Optional<T> for T {
    type Error = errors::RequiredField;
    fn convert(val: Option<T>, name: &'static str) -> Result<Self, Self::Error> {
        match val {
            Some(val) => Ok(val),
            None => Err(errors::RequiredField { name }),
        }
    }
}

impl Decode<'_> for bool {
    const TY: DataType = DataType::True;

    fn decode(_: &mut &'_ [u8]) -> Result<Self> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #![allow(warnings)]

    use super::*;
    use crate::errors::RequiredField;

    struct User {
        // 1
        name: Option<String>,
        // 2
        id: u64,
        // 3
        is_admin: bool,
    }

    impl<'de> Decode<'de> for User {
        const TY: DataType = DataType::Struct;

        fn decode(reader: &mut &'de [u8]) -> Result<Self> {
            let mut name: Option<String> = None;
            let mut id = None;
            let mut is_admin = None;

            let mut fd = FieldDecoder::new(reader)?;

            while let Some((key, ty)) = fd.next_field_id_and_ty()? {
                match key {
                    1 => name = fd.decode()?,
                    2 => id = fd.decode()?,
                    3 => is_admin = fd.decode()?,
                    // Unknown Field
                    _ => {}
                }
            }

            Ok(Self {
                name: Optional::convert(name, "name")?,
                id: Optional::convert(id, "id")?,
                is_admin: Optional::convert(is_admin, "is_admin")?,
            })
        }
    }
}
