use super::*;
use std::fmt::{self, Debug, Write};

impl std::fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(val) => val.fmt(f),
            Value::UInt(val) => write!(f, "{val}u",),
            Value::Str(val) => val.fmt(f),
            Value::Bytes(bytes) => {
                f.write_char('(')?;
                let mut bytes = bytes.iter().peekable();
                while let Some(byte) = bytes.next() {
                    if bytes.peek().is_some() {
                        write!(f, "{byte} ")?;
                    } else {
                        write!(f, "{byte}")?;
                    }
                }
                f.write_char(')')
            }
            Value::F32(val) => write!(f, "{val:#?}f"),
            Value::F64(val) => val.fmt(f),
            Value::Bool(val) => val.fmt(f),
            Value::List(list) => list.fmt(f),
            Value::Struct(items) => items.fmt(f),
        }
    }
}

impl Debug for List<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(val) => Debug::fmt(val, f),
            Self::F32(val) => Debug::fmt(val, f),
            Self::F64(val) => Debug::fmt(val, f),
            Self::Int(val) => Debug::fmt(val, f),
            Self::UInt(val) => Debug::fmt(val, f),
            Self::Str(val) => Debug::fmt(val, f),
            Self::Bytes(val) => Debug::fmt(val, f),
            Self::List(val) => Debug::fmt(val, f),
            Self::Struct(val) => Debug::fmt(val, f),
        }
    }
}

impl<'de> fmt::Debug for Entries<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}
