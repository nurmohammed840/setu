mod cached_table;
mod fs_utils;

pub use cached_table::LocalCachedTable;
pub use fs_utils::*;

#[macro_export]
macro_rules! fmt {
    (type $lt: lifetime) => { std::fmt::FromFn<impl Fn(&mut core::fmt::Formatter<'_>) -> core::fmt::Result + $lt> };
    (type) => { std::fmt::FromFn<impl Fn(&mut core::fmt::Formatter<'_>) -> core::fmt::Result> };
}

pub use fmt;
