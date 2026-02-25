use crate::convert::DataType;
use std::fmt;

macro_rules! error {
    [$name:ty ; $item:item] => {
        impl std::error::Error for $name {}
        impl fmt::Display for $name {
            $item
        }
    };
}

#[derive(Debug)]
pub struct UnexpectedEof {
    /// Number of additional bytes required.
    pub needed: usize,
}
error! {
    UnexpectedEof;
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unexpected end of file: needed {} more bytes", self.needed)
    }
}

#[derive(Debug)]
pub struct VarIntError;

error! {
    VarIntError;
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid variable-length integer")
    }
}

#[derive(Debug, Clone)]
pub struct InvalidType {
    pub found: DataType,
    pub expected: DataType,
}
error! {
    InvalidType;
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    InvalidArrayLen;
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid array size: expected {}, found {}",
            self.expected, self.found
        )
    }
}

#[derive(Debug, Clone)]
pub struct RequiredField {
    pub name: &'static str,
}
error! {
    RequiredField;
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "missing required field `{}`", self.name)
    }
}
