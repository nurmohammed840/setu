mod convert;
mod decoder;
mod encoder;
mod entries;
mod print;
mod utils;
mod varint;
mod zig_zag;

pub mod errors;

pub use lipi_macros::*;
#[doc(hidden)]
pub use encoder::FieldEncoder;

pub use convert::ConvertFrom;
pub use entries::Entries;

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
    List(List<'de>),
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
    List(Vec<List<'de>>),
}
