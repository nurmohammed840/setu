use crate::convert::DataType;
use std::fmt;

#[macro_export]
macro_rules! assert_or_err {
    ($cond: expr, $err: expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
}

macro_rules! error {
    [$name:ty = ($self: tt, $f:tt) $item:block] => {
        impl std::error::Error for $name {}
        impl fmt::Display for $name {
            fn fmt(&$self, $f: &mut fmt::Formatter<'_>) -> fmt::Result $item
        }
    };
}

#[derive(Debug)]
pub struct UnexpectedEof {
    /// Number of additional bytes required.
    pub needed: usize,
}

error! {
    UnexpectedEof = (self, f) {
        write!(f, "unexpected end of file: needed {} more bytes", self.needed)
    }
}

#[derive(Debug)]
pub struct VarIntError;

error! {
    VarIntError = (self, f) {
        f.write_str("invalid variable-length integer")
    }
}

#[derive(Debug, Clone)]
pub struct InvalidType {
    pub found: DataType,
    pub expected: DataType,
}

error! {
    InvalidType = (self, f) {
        write!(f, "invalid type: found {:?}, expected {:?}", self.found, self.expected)
    }
}

impl InvalidType {
    pub fn error(found: DataType, expected: DataType) -> crate::Error {
        Box::new(Self { found, expected })
    }
}

#[derive(Debug, Clone)]
pub struct InvalidArrayLen {
    pub expected: usize,
    pub found: usize,
}

error! {
    InvalidArrayLen = (self, f) {
        write!(f, "invalid array size: expected {}, found {}", self.expected, self.found)
    }
}

#[derive(Debug, Clone)]
pub struct RequiredField {
    pub name: &'static str,
}
error! {
    RequiredField = (self, f) {
        write!(f, "missing required field `{}`", self.name)
    }
}
