#![allow(dead_code)]

mod bit_set;
mod utils;
mod varint;
mod zig_zag;

pub mod convert;
pub mod errors;
// pub mod value;

pub use bit_set::BitSet;
pub use lipi_macros::*;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub use convert::decoder;
pub use convert::encoder::Encode;
