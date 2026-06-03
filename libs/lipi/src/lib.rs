mod bit_set;
mod convert;
mod utils;
mod varint;
mod zig_zag;

pub mod errors;

pub use convert::*;

pub use bit_set::BitSet;
pub use lipi_macros::*;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub use convert::{decoder::Decode, encoder::Encode};

pub trait DecodeOwned: for<'de> Decode<'de> {}
impl<T> DecodeOwned for T where T: for<'de> Decode<'de> {}
