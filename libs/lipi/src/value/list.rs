use std::fmt::Write;

use super::*;
use crate::BitSet;

#[derive(Clone)]
pub enum List {
    Bool(BitSet<Box<[u8]>>),

    U8(Box<[u8]>),
    I8(Box<[i8]>),

    F32(Vec<f32>),
    F64(Vec<f64>),

    UInt(Vec<u64>),
    Int(Vec<i64>),

    Str(Vec<Box<str>>),

    Struct(Vec<Entries>),
    Union(Vec<Entry>),
    List(Vec<List>),
    Table(Vec<Table>),

    // ---------------
    UnknownI(Vec<Box<[u8]>>),
    UnknownII(Vec<Box<[u8]>>),
    UnknownIII(Vec<Box<[u8]>>),
}

// impl List {
//     pub fn get(&self, idx: usize) -> Option<Value> {
//         match self {
//             List::Bool(bit_set) => bit_set.get(idx).map(Value::Bool),
//             List::U8(items) => items.get(idx).copied().map(Value::U8),
//             List::I8(items) => items.get(idx).copied().map(Value::I8),
//             List::F32(items) => items.get(idx).copied().map(Value::F32),
//             List::F64(items) => items.get(idx).copied().map(Value::F64),
//             List::UInt(items) => items.get(idx).copied().map(Value::UInt),
//             List::Int(items) => items.get(idx).copied().map(Value::Int),
//             List::Str(items) => items.get(idx).copied().map(Value::Str),
//             // ---
//             List::Struct(items) => items.get(idx).cloned().map(Value::Struct),
//             List::Union(items) => items.get(idx).cloned().map(Box::new).map(Value::Union),
//             List::List(items) => items.get(idx).cloned().map(Value::List),
//             List::Table(items) => items.get(idx).cloned().map(Value::Table),
//             // ---
//             List::UnknownI(items) => items.get(idx).copied().map(Value::UnknownI),
//             List::UnknownII(items) => items.get(idx).copied().map(Value::UnknownII),
//             List::UnknownIII(items) => items.get(idx).copied().map(Value::UnknownII),
//         }
//     }
// }

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            List::Bool(val) => fmt::Debug::fmt(val, f),

            List::I8(val) => fmt::Debug::fmt(val, f),
            List::U8(val) => {
                f.write_char('(')?;
                let mut bytes = val.iter().peekable();
                while let Some(byte) = bytes.next() {
                    if bytes.peek().is_some() {
                        write!(f, "{byte} ")?;
                    } else {
                        write!(f, "{byte}")?;
                    }
                }
                f.write_char(')')
            }

            List::F32(val) => fmt::Debug::fmt(val, f),
            List::F64(val) => fmt::Debug::fmt(val, f),
            List::Int(val) => fmt::Debug::fmt(val, f),
            List::UInt(val) => fmt::Debug::fmt(val, f),
            List::Str(val) => fmt::Debug::fmt(val, f),

            List::Struct(val) => fmt::Debug::fmt(val, f),
            List::Union(val) => fmt::Debug::fmt(val, f),
            List::List(val) => fmt::Debug::fmt(val, f),
            List::Table(val) => fmt::Debug::fmt(val, f),
            // ---
            List::UnknownI(bytes) => bytes.fmt(f),
            List::UnknownII(bytes) => bytes.fmt(f),
            List::UnknownIII(bytes) => bytes.fmt(f),
        }
    }
}
