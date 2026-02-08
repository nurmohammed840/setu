// mod bit_set;
mod convert;
mod decoder;
mod encoder;
mod entries;
mod print;
mod utils;
mod varint;
mod zig_zag;

pub mod errors;

#[doc(hidden)]
pub use encoder::FieldEncoder;
pub use lipi_macros::*;

// pub use bit_set::BitSet;
pub use convert::ConvertFrom;
pub use entries::{Entries, Entry};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

use std::io::{self, Write};

#[doc(hidden)]
pub mod __private {
    pub use crate::encoder::field_encoder;
}

pub trait Encoder {
    fn encode(&self, _: &mut dyn Write) -> io::Result<()>;
}

pub trait Decoder<'de>: Sized {
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
    Bool(Vec<bool>),

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

#[derive(Clone)]
pub struct Table<'de>(pub(crate) Vec<(u16, List<'de>)>);

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
