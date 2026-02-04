use crate::*;

use super::Result;

fn parse_header(reader: &mut &[u8]) -> Result<(u64, u8)> {
    let byte = utils::read_byte(reader)?;

    let ty = byte & 0b00001111;
    let id = (byte >> 4) as u64;

    let id = if id == 0b1111 {
        varint::read_unsigned(reader)? + 15
    } else {
        id
    };
    Ok((id, ty))
}

fn parse_str<'de>(reader: &mut &'de [u8]) -> Result<&'de str> {
    Ok(parse_bytes(reader).map(str::from_utf8)??)
}

fn parse_bytes<'de>(reader: &mut &'de [u8]) -> Result<&'de [u8]> {
    let len = varint::read_unsigned(reader).map(u32::try_from)??;
    utils::read_bytes(reader, len.try_into().unwrap())
}

fn collect<T>(len: u32, mut f: impl FnMut() -> Result<T>) -> Result<Vec<T>> {
    let mut arr = Vec::with_capacity(len.try_into().unwrap());
    for _ in 0..len {
        arr.push(f()?);
    }
    Ok(arr)
}

fn parse_list<'de>(reader: &mut &'de [u8]) -> Result<List<'de>> {
    let (len, ty) = parse_header(reader)?;
    let len = u32::try_from(len)?;

    match ty {
        0 | 1 => collect(len, || match utils::read_byte(reader)? {
            0 => Ok(false),
            1 => Ok(true),
            v => Err(errors::ParseError::new(format!("invalid boolean value: `{v}`")).into()),
        })
        .map(List::Bool),
        2 => collect(len, || utils::read_buf(reader).map(f32::from_le_bytes)).map(List::F32),
        3 => collect(len, || utils::read_buf(reader).map(f64::from_le_bytes)).map(List::F64),
        4 => collect(len, || varint::read_unsigned(reader).map(zig_zag::from)).map(List::Int),
        5 => collect(len, || varint::read_unsigned(reader)).map(List::UInt),
        6 => collect(len, || parse_str(reader)).map(List::Str),
        7 => collect(len, || parse_bytes(reader)).map(List::Bytes),
        8 => collect(len, || parse_list(reader)).map(List::List),
        9 => collect(len, || Entries::parse(reader)).map(List::Struct),
        code => Err(errors::UnknownType { code }.into()),
    }
}

impl<'de> Entries<'de> {
    pub fn parse(reader: &mut &'de [u8]) -> Result<Self> {
        let mut entries = Entries::new();
        loop {
            let (key, ty) = parse_header(reader)?;
            let value = match ty {
                0 => Ok(Value::Bool(false)),
                1 => Ok(Value::Bool(true)),

                2 => utils::read_buf(reader)
                    .map(f32::from_le_bytes)
                    .map(Value::F32),

                3 => utils::read_buf(reader)
                    .map(f64::from_le_bytes)
                    .map(Value::F64),

                4 => varint::read_unsigned(reader)
                    .map(zig_zag::from)
                    .map(Value::Int),

                5 => varint::read_unsigned(reader).map(Value::UInt),
                6 => parse_str(reader).map(Value::Str),
                7 => parse_bytes(reader).map(Value::Bytes),
                8 => parse_list(reader).map(Value::List),
                9 => Entries::parse(reader).map(Value::Struct),
                10 => {
                    debug_assert!(key == 0);
                    break; // End of struct
                }
                code => Err(errors::UnknownType { code }.into()),
            }?;
            entries.insert(key.try_into()?, value);
        }
        Ok(entries)
    }
}
