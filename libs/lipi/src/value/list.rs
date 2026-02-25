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
