#![allow(warnings)]
use std::any::type_name;

use crate::utils::try_convert_into_vec;
use crate::{errors::ConvertError, *};

pub trait ConvertFrom<T>: Sized {
    fn convert_from(value: T) -> Result<Self, ConvertError>;
}

impl<'v, 'de, T> ConvertFrom<Option<&'v Value<'de>>> for Option<T>
where
    T: ConvertFrom<&'v Value<'de>>,
{
    #[inline]
    fn convert_from(val: Option<&'v Value<'de>>) -> Result<Self, ConvertError> {
        val.map(T::convert_from).transpose()
    }
}

macro_rules! convert {
    [$($name:ident($ty:ty))*] => [$(
        impl ConvertFrom<&Value<'_>> for $ty {
            fn convert_from(value: &Value) -> Result<Self, ConvertError> {
                match value {
                    Value::$name(val) => Ok(*val),
                    _ => Err(value.invalid_type(type_name::<Self>())),
                }
            }
        }
    )*];
    [$($name:ident => $ty:ty)*] => [$(
        impl ConvertFrom<&Value<'_>> for $ty {
            fn convert_from(value: &Value) -> Result<Self, ConvertError> {
                match value {
                    Value::$name(val) => Self::try_from(*val).map_err(ConvertError::from),
                    _ => Err(value.invalid_type(type_name::<Self>())),
                }
            }
        }
    )*];
}

convert! {
    Bool(bool)
    I8(i8)
    U8(u8)
    F32(f32)
    F64(f64)
    Int(i64)
    UInt(u64)
}

convert! {
    UInt => u16
    UInt => u32

    Int => i16
    Int => i32

    Str => String
}

impl ConvertFrom<&Value<'_>> for char {
    fn convert_from(value: &Value) -> Result<Self, ConvertError> {
        match value {
            Value::UInt(val) => {
                let code = u32::try_from(*val).map_err(ConvertError::from)?;
                char::try_from(code).map_err(ConvertError::from)
            }
            _ => Err(value.invalid_type(type_name::<Self>())),
        }
    }
}

// -------------------------------------------------------------------------------

impl<'v, 'de, T> ConvertFrom<&'v List<'de>> for T
where
    T: Decode<'de>,
{
    fn convert_from(_list: &'v List<'de>) -> Result<Self, ConvertError> {
        todo!()
    }
}

// -------------------------------------------------------------------------------

impl<'v, 'de, T> ConvertFrom<&'v Value<'de>> for Vec<T>
where
    Self: ConvertFrom<&'v List<'de>>,
{
    fn convert_from(value: &'v Value<'de>) -> Result<Self, ConvertError> {
        match value {
            Value::List(list) => Self::convert_from(list),
            _ => Err(value.invalid_type(type_name::<Self>())),
        }
    }
}
