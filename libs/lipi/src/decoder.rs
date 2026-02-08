use crate::{utils::try_collect, *};

fn parse_header(reader: &mut &[u8]) -> Result<(u64, u8)> {
    let byte = utils::read_byte(reader)?;

    let ty = byte & 0b_1111;
    let id = (byte >> 4) as u64;

    let id = if id == 0b1111 {
        varint::read_u64(reader)? + 15
    } else {
        id
    };
    Ok((id, ty))
}

fn parse_bytes<'de>(reader: &mut &'de [u8]) -> Result<&'de [u8]> {
    let len = varint::read_u64(reader)?;
    utils::read_bytes(reader, usize::try_from(len)?)
}

fn parse_str<'de>(reader: &mut &'de [u8]) -> Result<&'de str> {
    Ok(str::from_utf8(parse_bytes(reader)?)?)
}

fn parse_table<'de>(reader: &mut &'de [u8]) -> Result<Table<'de>> {
    let len = usize::try_from(varint::read_u64(reader)?)?;
    let mut table = Table::with_capacity(len);

    for _ in 0..len {
        let (key, ty) = parse_header(reader)?;
        table.insert(key.try_into()?, parse_list_values(reader, len, ty)?);
    }
    Ok(table)
}

fn parse_list<'de>(reader: &mut &'de [u8]) -> Result<List<'de>> {
    let (len, ty) = parse_header(reader)?;
    parse_list_values(reader, usize::try_from(len)?, ty)
}

fn parse_list_values<'de>(reader: &mut &'de [u8], len: usize, ty: u8) -> Result<List<'de>> {
    let value = match ty {
        1 => {
            try_collect(len, || utils::read_byte(reader).and_then(utils::bool_from)).map(List::Bool)
        }

        2 => utils::read_bytes(reader, len).map(List::U8),
        3 => utils::read_bytes(reader, len)
            .map(utils::i8_slice_from)
            .map(List::I8),

        4 => try_collect(len, || utils::read_buf(reader).map(f32::from_le_bytes)).map(List::F32),
        5 => try_collect(len, || utils::read_buf(reader).map(f64::from_le_bytes)).map(List::F64),

        6 => try_collect(len, || varint::read_u64(reader)).map(List::UInt),
        7 => try_collect(len, || varint::read_u64(reader).map(zig_zag::from_u64)).map(List::Int),

        8 => try_collect(len, || parse_str(reader)).map(List::Str),
        9 => try_collect(len, || Entries::parse(reader)).map(List::Struct),

        10 => try_collect(len, || parse_list(reader)).map(List::List),
        11 => try_collect(len, || parse_table(reader)).map(List::Table),
        _ => {
            for _ in 0..len {
                let _bytes = parse_bytes(reader)?;
            }
            // return Ok(None);
            todo!()
        }
    };
    value
}

impl<'de> Entries<'de> {
    pub fn parse(reader: &mut &'de [u8]) -> Result<Self> {
        let len = usize::try_from(varint::read_u64(reader)?)?;

        let mut entries = Entries::with_capacity(len.try_into()?);
        for _ in 0..len {
            let (key, ty) = parse_header(reader)?;
            let value = match ty {
                0 => Ok(Value::Bool(false)),
                1 => Ok(Value::Bool(true)),

                2 => utils::read_byte(reader).map(Value::U8),
                3 => utils::read_byte(reader).map(u8::cast_signed).map(Value::I8),

                4 => utils::read_buf(reader)
                    .map(f32::from_le_bytes)
                    .map(Value::F32),

                5 => utils::read_buf(reader)
                    .map(f64::from_le_bytes)
                    .map(Value::F64),

                6 => varint::read_u64(reader).map(Value::UInt),
                7 => varint::read_u64(reader)
                    .map(zig_zag::from_u64)
                    .map(Value::Int),

                8 => parse_str(reader).map(Value::Str),
                9 => Entries::parse(reader).map(Value::Struct),
                10 => Entries::parse(reader).map(Value::Struct),
                11 => parse_list(reader).map(Value::List),
                12 => parse_table(reader).map(Value::Table),
                _ => {
                    let _bytes = parse_bytes(reader)?;
                    continue;
                }
            };
            entries.insert(key.try_into()?, value?);
        }
        Ok(entries)
    }
}

fn parse_value<'de>(reader: &mut &'de [u8]) -> Result<Value<'de>> {
    todo!()
}

fn parse_union<'de>(reader: &mut &'de [u8]) -> Result<(u16, Value<'de>)> {
    let (len, ty) = parse_header(reader)?;
    todo!()
}
