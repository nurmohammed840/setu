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

// -----------------------------------------------------------------------------

pub trait Decode<'de>: Sized {
    const TY: DataType;
    fn decode(reader: &mut &'de [u8]) -> Result<Self>;

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

    Vec<bool> = List (reader) {
        let (len, ty) = decode_list_len_and_ty(reader)?;
        ty.expected(DataType::True)?;
        decode_packed_bools(reader, len)
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

// ------------------------------------- tuples ---------------------------------------

macro_rules! tuples {
    [$( $name:tt : $idx:tt) *] => {
        impl <'de, $($name,)*> Decode<'de> for ($($name,)*)
        where
            $($name: FieldDecoder<'de>,)*
        {
            const TY: DataType = DataType::Struct;
            #[allow(non_snake_case)]
            fn decode(reader: &mut &'de [u8]) -> Result<Self> {
                $(let mut $name: Option<_> = None;)*

                let mut fd = FieldInfoDecoder::new(reader)?;
                while let Some((key, ty)) = fd.next_field_id_and_ty()? {
                    match key {
                        $($idx => $name = fd.decode(ty, concat!("tuple ", $idx))?,)*
                        _ => {}
                    }
                }

                Ok((
                    $(Optional::convert($name, concat!("tuple ", $idx))?,)*
                ))
            }
        }
    }
}

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

// ---------------------------------------------------------------------------------

pub trait FieldDecoder<'de>: Sized {
    fn decode_field(reader: &mut &'de [u8], ty: DataType) -> Result<Self>;
}

impl FieldDecoder<'_> for bool {
    #[inline]
    fn decode_field(_: &mut &'_ [u8], ty: DataType) -> Result<Self> {
        Ok(bool::try_from(ty)?)
    }
}

impl<'de, T> FieldDecoder<'de> for T
where
    T: Decode<'de>,
{
    fn decode_field(reader: &mut &'de [u8], ty: DataType) -> Result<Self> {
        ty.expected(T::TY)?;
        T::decode(reader)
    }
}

pub struct FieldInfoDecoder<'c, 'de> {
    pub reader: &'c mut &'de [u8],
}

impl<'c, 'de> FieldInfoDecoder<'c, 'de> {
    #[inline]
    pub fn new(reader: &'c mut &'de [u8]) -> Result<Self> {
        Ok(Self { reader })
    }

    pub fn next_field_id_and_ty(&mut self) -> Result<Option<(u64, DataType)>> {
        let (id, ty) = decode_field_id_and_ty(self.reader)?;

        if ty == DataType::StructEnd {
            assert_or_err!(
                id == 0,
                format!("invalid struct end field id ({id}), expected `0`")
            );
            return Ok(None);
        }

        Ok(Some((id, ty)))
    }

    #[inline]
    pub fn decode_field<T>(
        &mut self,
        ty: DataType,
        name: &'static str,
    ) -> Result<T, errors::FieldError>
    where
        T: FieldDecoder<'de>,
    {
        T::decode_field(self.reader, ty).map_err(|error| errors::FieldError { ty, name, error })
    }

    #[inline]
    pub fn decode<T>(
        &mut self,
        ty: DataType,
        name: &'static str,
    ) -> Result<Option<T>, errors::FieldError>
    where
        T: FieldDecoder<'de>,
    {
        self.decode_field(ty, name).map(Some)
    }
}

pub trait Optional<T>: Sized {
    type Error;
    fn convert(val: Option<T>, name: &'static str) -> Result<Self, Self::Error>;
}

impl<T> Optional<T> for Option<T> {
    type Error = std::convert::Infallible;
    #[inline]
    fn convert(val: Option<T>, _: &'static str) -> Result<Self, Self::Error> {
        Ok(val)
    }
}

impl<'de, T> Optional<T> for T
where
    T: FieldDecoder<'de>,
{
    type Error = errors::RequiredField;
    #[inline]
    fn convert(val: Option<T>, name: &'static str) -> Result<Self, Self::Error> {
        match val {
            Some(val) => Ok(val),
            None => Err(errors::RequiredField { name }),
        }
    }
}
