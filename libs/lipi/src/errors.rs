use crate::{List, Value};
use std::fmt;

#[derive(Debug)]
pub struct UnexpectedEof {
    /// Number of additional bytes required.
    pub size: usize,
}
impl std::error::Error for UnexpectedEof {}
impl fmt::Display for UnexpectedEof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unexpected end of file: needed {} more bytes", self.size)
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
pub struct ParseError;
impl std::error::Error for ParseError {}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid type tag: `0`")
    }
}

impl List<'_> {
    fn type_name(&self) -> &str {
        match self {
            List::Bool(_) => "[boolean]",
            List::I8(_) => "[i8]",
            List::U8(_) => "[u8]",
            List::F32(_) => "[f32]",
            List::F64(_) => "[f64]",
            List::Int(_) => "[int]",
            List::UInt(_) => "[uint]",
            List::Str(_) => "[string]",
            List::Struct(_) => "[struct]",
            List::List(_) => "[...]",
            List::Table(_) => "[table]",
            List::Union(_) => "[union]",
            // ---
            List::UnknownI(_) => "[unknown I]",
            List::UnknownII(_) => "[unknown II]",
            List::UnknownIII(_) => "[unknown III]",
        }
    }

    pub(crate) fn _invalid_type(&self, expected: &str) -> ConvertError {
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
            Value::I8(_) => "i8",
            Value::U8(_) => "u8",
            Value::F32(_) => "f32",
            Value::F64(_) => "f64",
            Value::Int(_) => "int",
            Value::UInt(_) => "uint",
            Value::Str(_) => "string",
            Value::Struct(_) => "struct",
            Value::List(list) => list.type_name(),
            Value::Table(_) => "table",
            Value::Union(_) => "union",
            // ---
            Value::UnknownI(_) => "unknown I",
            Value::UnknownII(_) => "unknown II",
            Value::UnknownIII(_) => "unknown III",
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
            Some(key) => write!(f, "conversion error for key `{key}`: {}", self.error),
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
