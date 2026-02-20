mod bit_set;
mod convert;
pub mod decode;
mod decoder;
mod encoder;
mod entries;
mod print;
mod utils;
mod varint;
mod zig_zag;

pub mod errors;

pub use lipi_macros::*;

pub use bit_set::BitSet;
pub use convert::ConvertFrom;
pub use entries::{Entries, Entry};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

use std::io::{self, Write};

pub use encoder::Encode;

#[doc(hidden)]
pub mod __private {
    pub use crate::encoder::*;
}

pub trait Decode<'de>: Sized {
    fn parse(reader: &mut &'de [u8]) -> Result<Self> {
        Self::decode(&Entries::parse(reader)?)
    }

    fn decode(entries: &Entries<'de>) -> Result<Self>;
}

#[derive(Clone)]
pub enum Value<'de> {
    Bool(bool),

    U8(u8),
    I8(i8),

    F32(f32),
    F64(f64),

    UInt(u64),
    Int(i64),

    Str(&'de str),

    Struct(Entries<'de>),
    Union(Box<Entry<'de>>),
    List(List<'de>),
    Table(Table<'de>),

    // ---------------
    UnknownI(&'de [u8]),
    UnknownII(&'de [u8]),
    UnknownIII(&'de [u8]),
}

#[derive(Clone)]
pub enum List<'de> {
    Bool(BitSet<&'de [u8]>),

    U8(&'de [u8]),
    I8(&'de [i8]),

    F32(Vec<f32>),
    F64(Vec<f64>),

    UInt(Vec<u64>),
    Int(Vec<i64>),

    Str(Vec<&'de str>),

    Struct(Vec<Entries<'de>>),
    Union(Vec<Entry<'de>>),
    List(Vec<List<'de>>),
    Table(Vec<Table<'de>>),

    // ---------------
    UnknownI(Vec<&'de [u8]>),
    UnknownII(Vec<&'de [u8]>),
    UnknownIII(Vec<&'de [u8]>),
}

impl List<'_> {
    pub fn get(&self, idx: usize) -> Option<Value<'_>> {
        match self {
            List::Bool(bit_set) => bit_set.get(idx).map(Value::Bool),
            List::U8(items) => items.get(idx).copied().map(Value::U8),
            List::I8(items) => items.get(idx).copied().map(Value::I8),
            List::F32(items) => items.get(idx).copied().map(Value::F32),
            List::F64(items) => items.get(idx).copied().map(Value::F64),
            List::UInt(items) => items.get(idx).copied().map(Value::UInt),
            List::Int(items) => items.get(idx).copied().map(Value::Int),
            List::Str(items) => items.get(idx).copied().map(Value::Str),
            // ---
            List::Struct(items) => items.get(idx).cloned().map(Value::Struct),
            List::Union(items) => items.get(idx).cloned().map(Box::new).map(Value::Union),
            List::List(items) => items.get(idx).cloned().map(Value::List),
            List::Table(items) => items.get(idx).cloned().map(Value::Table),
            // ---
            List::UnknownI(items) => items.get(idx).copied().map(Value::UnknownI),
            List::UnknownII(items) => items.get(idx).copied().map(Value::UnknownII),
            List::UnknownIII(items) => items.get(idx).copied().map(Value::UnknownII),
        }
    }
}

#[derive(Clone)]
pub struct Table<'de>(pub(crate) Vec<(u16, List<'de>)>);

impl<'de> Default for Table<'de> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'de> Table<'de> {
    pub fn new() -> Self {
        Table(Vec::with_capacity(8))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Table(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn insert(&mut self, key: u16, value: List<'de>) {
        self.0.push((key, value));
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, (u16, List<'de>)> {
        self.0.iter()
    }
}
