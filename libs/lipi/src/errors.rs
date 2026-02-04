use crate::{List, Value};
use std::fmt;

#[derive(Debug)]
pub struct UnexpectedEof;
impl std::error::Error for UnexpectedEof {}
impl fmt::Display for UnexpectedEof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unexpected end of file")
    }
}

#[derive(Debug)]
pub struct VarIntError;
impl std::error::Error for VarIntError {}
impl fmt::Display for VarIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid variable-length integer")
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub message: Box<str>,
}

impl std::error::Error for ParseError {}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl ParseError {
    pub fn new(message: impl Into<Box<str>>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug)]
pub struct UnknownType {
    pub code: u8,
}

impl std::error::Error for UnknownType {}
impl fmt::Display for UnknownType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown type: {}", self.code)
    }
}

impl List<'_> {
    fn type_name(&self) -> &str {
        match self {
            List::Bool(_) => "[boolean]",
            List::F32(_) => "[f32]",
            List::F64(_) => "[f64]",
            List::Int(_) => "[integer]",
            List::UInt(_) => "[unsigned integer]",
            List::Str(_) => "[string]",
            List::Bytes(_) => "[bytes]",
            List::Struct(_) => "[struct]",
            List::List(_) => "[...]",
        }
    }

    pub(crate) fn invalid_type(&self, expected: &str) -> ConvertError {
        ConvertError::new(format!(
            "expected `{expected}`, found `{}`",
            self.type_name()
        ))
    }
}

impl Value<'_> {
    fn type_name(&self) -> &str {
        match self {
            Value::Bool(_) => "boolean",
            Value::F32(_) => "f32",
            Value::F64(_) => "f64",
            Value::Int(_) => "integer",
            Value::UInt(_) => "unsigned integer",
            Value::Str(_) => "string",
            Value::Bytes(_) => "bytes",
            Value::Struct(_) => "struct",
            Value::List(list) => list.type_name(),
        }
    }

    pub(crate) fn invalid_type(&self, expected: &str) -> ConvertError {
        ConvertError::new(format!(
            "expected `{expected}`, found `{}`",
            self.type_name()
        ))
    }
}

pub struct ConvertError {
    pub key: Option<u16>,
    pub error: crate::Error,
}

impl ConvertError {
    pub fn new(message: String) -> Self {
        Self {
            key: None,
            error: message.into(),
        }
    }

    pub fn from(value: impl Into<crate::Error>) -> Self {
        Self {
            key: None,
            error: value.into(),
        }
    }
}

impl std::error::Error for ConvertError {}
impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.key {
            Some(key) => {
                write!(f, "conversion error for key `{key}`: {}", self.error)
            }
            None => f.write_str(&self.error.to_string()),
        }
    }
}
impl fmt::Debug for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut obj = f.debug_struct("ConvertError");
        match self.key {
            Some(key) => obj.field("key", &key).field("message", &self.error),
            None => obj.field("message", &self.error),
        }
        .finish()
    }
}
