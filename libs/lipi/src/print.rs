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

            Value::I8(val) => val.fmt(f),
            Value::U8(val) => write!(f, "{val}u",),

            Value::F32(val) => write!(f, "{val:#?}f"),
            Value::F64(val) => val.fmt(f),
            Value::Bool(val) => val.fmt(f),
            Value::List(list) => list.fmt(f),
            Value::Struct(items) => items.fmt(f),
            Value::Table(table) => table.fmt(f),
            Value::Union(union) => union.fmt(f),
            // ---
            Value::UnknownI(bytes) => bytes.fmt(f),
            Value::UnknownII(bytes) => bytes.fmt(f),
            Value::UnknownIII(bytes) => bytes.fmt(f),
        }
    }
}

impl Debug for List<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            List::Bool(val) => Debug::fmt(val, f),

            List::I8(val) => Debug::fmt(val, f),
            List::U8(val) => {
                f.write_char('(')?;
                let mut bytes = val.iter().peekable();
                while let Some(byte) = bytes.next() {
                    if bytes.peek().is_some() {
                        write!(f, "{byte} ")?;
                    } else {
                        write!(f, "{byte}")?;
                    }
                }
                f.write_char(')')
            }

            List::F32(val) => Debug::fmt(val, f),
            List::F64(val) => Debug::fmt(val, f),
            List::Int(val) => Debug::fmt(val, f),
            List::UInt(val) => Debug::fmt(val, f),
            List::Str(val) => Debug::fmt(val, f),

            List::Struct(val) => Debug::fmt(val, f),
            List::Union(val) => Debug::fmt(val, f),
            List::List(val) => Debug::fmt(val, f),
            List::Table(val) => Debug::fmt(val, f),
            // ---
            List::UnknownI(bytes) => bytes.fmt(f),
            List::UnknownII(bytes) => bytes.fmt(f),
            List::UnknownIII(bytes) => bytes.fmt(f),
        }
    }
}

impl<'de> fmt::Display for Entries<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for Entry { key, value } in self.iter() {
            writeln!(f, "{key}: {value:#?}")?;
        }
        Ok(())
    }
}

impl<'de> fmt::Debug for Entries<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|Entry { key, value }| (key, value)))
            .finish()
    }
}

impl<'de> fmt::Debug for Table<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}
