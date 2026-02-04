use crate::{errors::ConvertError, *};
use std::{
    any::type_name,
    collections::{BTreeMap, HashMap},
    hash::Hash,
};
use utils::{convert_vec, convert_vec_from};

// -------------------------------------------------------------------------

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

impl<'v, 'de, T> ConvertFrom<Option<&'v Value<'de>>> for T
where
    T: ConvertFrom<&'v Value<'de>>,
{
    fn convert_from(value: Option<&'v Value<'de>>) -> Result<Self, ConvertError> {
        match value {
            Some(val) => T::convert_from(val),
            None => Err(ConvertError::new(format!(
                "expected `{}`, found `None`",
                type_name::<T>()
            ))),
        }
    }
}

impl<'v, 'de, T> ConvertFrom<&'v Value<'de>> for T
where
    T: Decoder<'de>,
{
    fn convert_from(value: &'v Value<'de>) -> Result<Self, ConvertError> {
        match value {
            Value::Struct(entries) => T::decode(entries).map_err(ConvertError::from),
            _ => Err(ConvertError::new(format!(
                "expected `{}`, found `None`",
                type_name::<T>()
            ))),
        }
    }
}

// -------------------------------------------------------------------------

macro_rules! convert {
    [$($name:ident($ty:ty))*] => [$(
        impl<'de> ConvertFrom<&Value<'de>> for $ty {
            fn convert_from(val: &Value<'de>) -> Result<Self, ConvertError> {
                match val {
                    Value::$name(val) => Ok(*val),
                    val => Err(val.invalid_type(type_name::<$ty>())),
                }
            }
        }

        impl<'v, 'de> ConvertFrom<&'v Value<'de>> for Vec<$ty> {
            fn convert_from(value: &'v Value<'de>) -> Result<Self, ConvertError> {
                match value {
                    Value::List(List::$name(items)) => Ok(items.clone().into()),
                    _ => Err(value.invalid_type(type_name::<Self>())),
                }
            }
        }
    )*];
    [$($name:ident => $ty:ty)*] => [$(
        impl ConvertFrom<&Value<'_>> for $ty {
            fn convert_from(val: &Value) -> Result<Self, ConvertError> {
                match val {
                    Value::$name(val) => <$ty>::try_from(*val).map_err(ConvertError::from),
                    val => Err(val.invalid_type(type_name::<$ty>())),
                }
            }
        }

        impl<'v, 'de> ConvertFrom<&'v Value<'de>> for Vec<$ty> {
            fn convert_from(value: &'v Value<'de>) -> Result<Self, ConvertError> {
                match value {
                    Value::List(List::$name(items)) => convert_vec(items).map_err(ConvertError::from),
                    _ => Err(value.invalid_type(type_name::<Self>())),
                }
            }
        }
    )*];
    [@zero_copy $($name:ident => $ty:ty)*] => [$(
        impl<'de> ConvertFrom<&Value<'de>> for $ty {
            fn convert_from(val: &Value<'de>) -> Result<Self, ConvertError> {
                match val {
                    Value::List(List::$name(val)) => Ok(*val),
                    val => Err(val.invalid_type(type_name::<$ty>())),
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
    Str(&'de str)
}

convert! {
    UInt => u16
    UInt => u32

    Int => i16
    Int => i32

    Str => String
}

convert! {
    @zero_copy
    U8 => &'de [u8]
    I8 => &'de [i8]
}

// -----------------------------------------------

impl<'v, 'de, T> ConvertFrom<&'v List<'de>> for T
where
    T: Decoder<'de>,
{
    fn convert_from(list: &'v List<'de>) -> Result<Self, ConvertError> {
        todo!()
    }
}

impl<'v, 'de, T> ConvertFrom<&'v Value<'de>> for Vec<T>
where
    T: ConvertFrom<&'v List<'de>>,
{
    fn convert_from(value: &'v Value<'de>) -> Result<Self, ConvertError> {
        match value {
            Value::List(list) => {
                // list;
                todo!()
            }
            _ => Err(value.invalid_type(type_name::<Self>())),
        }
    }
}

// ------------------------------------------------------------------------

macro_rules! tuples {
    [$($name:tt : $idx:tt)*] => {
        impl<'de, $($name,)*> Decoder<'de> for ($($name,)*)
        where
            $($name: for<'v> ConvertFrom<&'v Value<'de>>,)*
        {
            fn decode(entries: &Entries<'de>) -> Result<Self> {
                Ok((
                    $(ConvertFrom::convert_from(entries.get($idx))?,)*
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

// ------------------------------------------------------------------------

macro_rules! impl_map {
    [$map_ty:tt : $($bound:path),*] => {
        impl<'de, K, V> Decoder<'de> for $map_ty<K, V>
        where
            K: $($bound+)*,
            Vec<K>: for<'v> ConvertFrom<&'v Value<'de>>,
            Vec<V>: for<'v> ConvertFrom<&'v Value<'de>>
        {
            fn decode(entries: &Entries<'de>) -> Result<Self> {
                if entries.len() != 2 {
                    return Err(errors::ConvertError::from("expected 2 entries for map").into());
                }
                let keys: Vec<K> = ConvertFrom::convert_from(entries.get(0))?;
                let values: Vec<V> = ConvertFrom::convert_from(entries.get(1))?;

                if keys.len() != values.len() {
                    return Err(errors::ConvertError::from("mismatched key/value lengths").into());
                }
                Ok(Self::from_iter(keys.into_iter().zip(values)))
            }
        }
    };
}

impl_map! {
    BTreeMap: Ord
}
impl_map! {
    HashMap: Eq, Hash
}
