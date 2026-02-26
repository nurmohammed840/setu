use std::fmt;

mod entries;
mod list;
mod table;

pub use entries::{Struct, Entry};
pub use list::List;
pub use table::Table;

type Array<T> = Box<[T]>;
use crate::convert::DataType;

#[derive(Clone)]
pub enum Value {
    False,
    True,

    U8(u8),
    I8(i8),

    F32(f32),
    F64(f64),

    UInt(u64),
    Int(i64),

    Str(Box<str>),

    Struct(Struct),
    Union(Box<Entry>),
    List(List),
    Table(Table),
    // ---------------
    UnknownI(Array<u8>),
    UnknownII(Array<u8>),
    UnknownIII(Array<u8>),
}

impl Value {
    fn data_type(&self) -> DataType {
        match self {
            Value::False => DataType::False,
            Value::True => DataType::True,
            Value::U8(_) => DataType::I8,
            Value::I8(_) => DataType::I8,
            Value::F32(_) => DataType::F32,
            Value::F64(_) => DataType::F64,
            Value::UInt(_) => DataType::UInt,
            Value::Int(_) => DataType::Int,
            Value::Str(_) => DataType::Str,
            Value::Struct(_) => DataType::Struct,
            Value::Union(_) => DataType::Union,
            Value::List(_) => DataType::List,
            Value::Table(_) => DataType::Table,
            Value::UnknownI(_) => DataType::UnknownI,
            Value::UnknownII(_) => DataType::UnknownII,
            Value::UnknownIII(_) => DataType::UnknownIII,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::False => f.write_str("false"),
            Value::True => f.write_str("true"),

            Value::Int(val) => val.fmt(f),
            Value::UInt(val) => write!(f, "{val}u",),
            Value::Str(val) => val.fmt(f),

            Value::I8(val) => val.fmt(f),
            Value::U8(val) => write!(f, "{val}u",),

            Value::F32(val) => write!(f, "{val:#?}f"),
            Value::F64(val) => val.fmt(f),
            Value::Struct(items) => items.fmt(f),
            Value::Union(union) => union.fmt(f),
            Value::List(list) => list.fmt(f),
            Value::Table(table) => table.fmt(f),
            // ---
            Value::UnknownI(bytes) => bytes.fmt(f),
            Value::UnknownII(bytes) => bytes.fmt(f),
            Value::UnknownIII(bytes) => bytes.fmt(f),
        }
    }
}
