use std::u8;

use crate::{errors, *};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    False = 0,
    True = 1,

    U8 = 2,
    I8 = 3,

    F32 = 4,
    F64 = 5,

    UInt = 6,
    Int = 7,

    Str = 8,

    Struct = 9,
    Union = 10,
    List = 11,
    Table = 12,

    UnknownI = 13,
    UnknownII = 14,
    UnknownIII = 15,
}

impl DataType {
    #[inline]
    fn expected(self, expected: Self) -> Result<()> {
        if self == expected {
            return Ok(());
        }
        Err(Box::new(errors::InvalidType {
            found: self,
            expected,
        }))
    }
}

pub fn parse_length(reader: &mut &[u8]) -> Result<usize> {
    Ok(usize::try_from(varint::read_u64(reader)?)?)
}

pub fn parse_header(reader: &mut &[u8]) -> Result<(u64, DataType)> {
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

pub fn parse_packed_booleans<'de>(reader: &mut &'de [u8], len: usize) -> Result<BitSet<&'de [u8]>> {
    let packed = utils::read_bytes(reader, utils::bool_packed_len(len))?;
    Ok(BitSet::from_parts(len, packed))
}

pub fn parse_bytes<'de>(reader: &mut &'de [u8]) -> Result<&'de [u8]> {
    let len = parse_length(reader)?;
    utils::read_bytes(reader, len)
}

pub fn parse_str<'de>(reader: &mut &'de [u8]) -> Result<&'de str> {
    Ok(str::from_utf8(parse_bytes(reader)?)?)
}

// -----------------------------------------------------------------

pub trait Decode<'de>: Sized {
    fn decode(_: &mut &'de [u8]) -> Result<Self> {
        unimplemented!()
    }

    fn decode_field(ty: DataType, reader: &mut &'de [u8]) -> Result<Self>;
}

impl Decode<'_> for bool {
    fn decode_field(ty: DataType, _: &mut &[u8]) -> Result<Self> {
        match ty {
            DataType::False => Ok(false),
            DataType::True => Ok(true),
            _ => Err(errors::InvalidType::error(ty, DataType::True)),
        }
    }
}

impl Decode<'_> for f32 {
    fn decode(reader: &mut &[u8]) -> Result<Self> {
        utils::read_buf(reader).map(f32::from_le_bytes)
    }

    fn decode_field(ty: DataType, reader: &mut &[u8]) -> Result<Self> {
        match ty {
            DataType::U8 => u8::decode(reader).map(Self::from),
            DataType::I8 => i8::decode(reader).map(Self::from),
            DataType::F32 => Self::decode(reader),
            DataType::UInt => u16::decode(reader).map(Self::from),
            DataType::Int => i16::decode(reader).map(Self::from),
            _ => Err(errors::InvalidType::error(ty, DataType::F32)),
        }
    }
}

impl Decode<'_> for f64 {
    fn decode(reader: &mut &[u8]) -> Result<Self> {
        utils::read_buf(reader).map(f64::from_le_bytes)
    }

    fn decode_field(ty: DataType, reader: &mut &[u8]) -> Result<Self> {
        match ty {
            DataType::U8 => u8::decode(reader).map(Self::from),
            DataType::I8 => i8::decode(reader).map(Self::from),
            DataType::F32 => f32::decode(reader).map(Self::from),
            DataType::F64 => Self::decode(reader),
            DataType::UInt => u32::decode(reader).map(Self::from),
            DataType::Int => i32::decode(reader).map(Self::from),
            _ => Err(errors::InvalidType::error(ty, DataType::F64)),
        }
    }
}

fn decode_field_num<T>(ty: DataType, expected: DataType, reader: &mut &[u8]) -> Result<T>
where
    T: TryFrom<u8> + TryFrom<i8> + TryFrom<u64> + TryFrom<i64>,
    <T as TryFrom<u8>>::Error: std::error::Error + Send + Sync + 'static,
    <T as TryFrom<i8>>::Error: std::error::Error + Send + Sync + 'static,
    <T as TryFrom<u64>>::Error: std::error::Error + Send + Sync + 'static,
    <T as TryFrom<i64>>::Error: std::error::Error + Send + Sync + 'static,
{
    match ty {
        DataType::U8 => Ok(T::try_from(u8::decode(reader)?)?),
        DataType::I8 => Ok(T::try_from(i8::decode(reader)?)?),
        DataType::UInt => Ok(T::try_from(u64::decode(reader)?)?),
        DataType::Int => Ok(T::try_from(i64::decode(reader)?)?),
        _ => Err(errors::InvalidType::error(ty, expected)),
    }
}

macro_rules! decode_num {
    [$($ty:ty: $expected:path { $decode:item })*] => [$(
        impl Decode<'_> for $ty {
            $decode

            fn decode_field(ty: DataType, reader: &mut &[u8]) -> Result<Self> {
                decode_field_num(ty, $expected, reader)
            }
        }
    )*];
}

decode_num! {
    u8: DataType::U8 {
        fn decode(reader: &mut &[u8]) -> Result<Self> {
            utils::read_byte(reader)
        }
    }
    i8: DataType::U8 {
        fn decode(reader: &mut &[u8]) -> Result<Self> {
            utils::read_byte(reader).map(u8::cast_signed)
        }
    }
    u64: DataType::UInt {
        fn decode(reader: &mut &[u8]) -> Result<Self> {
            varint::read_u64(reader)
        }
    }
    i64: DataType::Int {
        fn decode(reader: &mut &[u8]) -> Result<Self> {
            varint::read_u64(reader).map(zig_zag::from_u64)
        }
    }

    u16: DataType::UInt {
        fn decode(reader: &mut &[u8]) -> Result<Self> {
            Ok(Self::try_from(u64::decode(reader)?)?)
        }
    }
    u32: DataType::UInt {
        fn decode(reader: &mut &[u8]) -> Result<Self> {
            Ok(Self::try_from(u64::decode(reader)?)?)
        }
    }

    i16: DataType::Int {
        fn decode(reader: &mut &[u8]) -> Result<Self> {
            Ok(Self::try_from(i64::decode(reader)?)?)
        }
    }
    i32: DataType::Int {
        fn decode(reader: &mut &[u8]) -> Result<Self> {
            Ok(i32::try_from(i64::decode(reader)?)?)
        }
    }
}

impl<'de> Decode<'de> for &'de str {
    fn decode(reader: &mut &'de [u8]) -> Result<Self> {
        parse_str(reader)
    }

    fn decode_field(ty: DataType, reader: &mut &'de [u8]) -> Result<Self> {
        ty.expected(DataType::Str)?;
        Self::decode(reader)
    }
}

impl Decode<'_> for String {
    fn decode(reader: &mut &[u8]) -> Result<Self> {
        parse_str(reader).map(String::from)
    }

    fn decode_field(ty: DataType, reader: &mut &[u8]) -> Result<Self> {
        ty.expected(DataType::Str)?;
        Self::decode(reader)
    }
}

// -----------------------------------------------------------------

pub struct FieldDecoder<'c, 'de> {
    pub reader: &'c mut &'de [u8],
    len: usize,
}

impl<'c, 'de> FieldDecoder<'c, 'de> {
    #[inline]
    pub fn new(reader: &'c mut &'de [u8]) -> Result<Self> {
        let len = parse_length(reader)?;
        Ok(Self { reader, len })
    }

    #[inline]
    pub fn header(&mut self) -> Result<Option<(u64, DataType)>> {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        parse_header(self.reader).map(Some)
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
        fn decode(reader: &mut &'de [u8]) -> Result<Self> {
            let mut name: Option<String> = None;
            let mut id = None;
            let mut is_admin = None;

            let mut fd = FieldDecoder::new(reader)?;

            while let Some((key, ty)) = fd.header()? {
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

        fn decode_field(ty: DataType, reader: &mut &[u8]) -> Result<Self> {
            ty.expected(DataType::Struct)?;
            Self::decode(reader)
        }
    }

    #[test]
    fn test_name() {}
}
