use std::fmt;

mod entries;
mod list;
mod table;

pub use entries::{Entries, Entry};
pub use list::List;
pub use table::Table;

#[derive(Clone)]
pub enum Value {
    Bool(bool),

    U8(u8),
    I8(i8),

    F32(f32),
    F64(f64),

    UInt(u64),
    Int(i64),

    Str(Box<str>),

    Struct(Entries),
    Union(Box<Entry>),
    List(List),
    Table(Table),

    // ---------------
    UnknownI(Box<[u8]>),
    UnknownII(Box<[u8]>),
    UnknownIII(Box<[u8]>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(val) => val.fmt(f),
            Value::UInt(val) => write!(f, "{val}u",),
            Value::Str(val) => val.fmt(f),

            Value::I8(val) => val.fmt(f),
            Value::U8(val) => write!(f, "{val}u",),

            Value::F32(val) => write!(f, "{val:#?}f"),
            Value::F64(val) => val.fmt(f),
            Value::Bool(val) => val.fmt(f),
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
