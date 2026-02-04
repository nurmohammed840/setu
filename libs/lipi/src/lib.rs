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

pub use convert::ConvertFrom;
pub use entries::Entries;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

use std::io::{self, Write};

#[doc(hidden)]
pub mod __private {
    pub use crate::encoder::encode_struct_field;
}

pub trait Encoder {
    fn encode(&self, _: &mut (impl Write + ?Sized)) -> io::Result<()>;
}

pub trait Decoder<'de>: Sized {
    fn parse(reader: &mut &'de [u8]) -> Result<Self> {
        Self::decode(&Entries::parse(reader)?)
    }

    fn decode(entries: &Entries<'de>) -> Result<Self>;
}

pub trait IntoValue<'de> {
    fn to_value(&self) -> Value<'de>;
}

#[derive(Clone)]
pub enum Value<'de> {
    Bool(bool),
    F32(f32),
    F64(f64),
    Int(i64),
    UInt(u64),
    Str(&'de str),
    Bytes(&'de [u8]),
    List(List<'de>),
    Struct(Entries<'de>),
}

#[derive(Clone)]
pub enum List<'de> {
    Bool(Vec<bool>),
    F32(Vec<f32>),
    F64(Vec<f64>),
    Int(Vec<i64>),
    UInt(Vec<u64>),
    Str(Vec<&'de str>),
    Bytes(Vec<&'de [u8]>),
    List(Vec<List<'de>>),
    Struct(Vec<Entries<'de>>),
}
