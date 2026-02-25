use crate::errors;

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
    Union = 10,
    List = 11,
    Table = 12,

    UnknownI = 13,
    UnknownII = 14,
    UnknownIII = 15,
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

impl DataType {
    #[inline]
    pub fn code(self) -> u8 {
        self as u8
    }

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
