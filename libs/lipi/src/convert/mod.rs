use crate::errors;

mod skip;

pub mod decoder;
pub mod encoder;

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
    StructEnd = 10,

    Union = 11,
    List = 12,
    Table = 13,

    UnknownI = 14,
    UnknownII = 15,
}

impl From<bool> for DataType {
    #[inline]
    fn from(value: bool) -> Self {
        match value {
            false => DataType::False,
            true => DataType::True,
        }
    }
}

impl TryFrom<DataType> for bool {
    type Error = errors::InvalidType;
    #[inline]
    fn try_from(value: DataType) -> Result<Self, Self::Error> {
        match value {
            DataType::False => Ok(false),
            DataType::True => Ok(true),
            _ => Err(errors::InvalidType {
                found: value,
                expected: DataType::True,
            }),
        }
    }
}

impl DataType {
    #[inline]
    pub fn code(self) -> u8 {
        self as u8
    }

    #[inline]
    pub fn expected(self, expected: Self) -> Result<(), errors::InvalidType> {
        if self == expected {
            return Ok(());
        }
        Err(errors::InvalidType {
            found: self,
            expected,
        })
    }
}
