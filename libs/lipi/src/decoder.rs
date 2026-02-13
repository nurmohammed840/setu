use crate::{utils::try_collect, *};

fn parse_header(reader: &mut &[u8]) -> Result<(u64, u8)> {
    let byte = utils::read_byte(reader)?;

    let ty = byte & 0b_1111;
    let id = (byte >> 4) as u64;

    let id = if id == 0b_1111 {
        varint::read_u64(reader)? + 15
    } else {
        id
    };
    Ok((id, ty))
}

fn parse_length(reader: &mut &[u8]) -> Result<usize> {
    Ok(usize::try_from(varint::read_u64(reader)?)?)
}

fn parse_packed_booleans<'de>(reader: &mut &'de [u8], len: usize) -> Result<BitSet<&'de [u8]>> {
    let packed_bytes_len = len.div_ceil(8);
    let packed = utils::read_bytes(reader, packed_bytes_len)?;
    Ok(BitSet::from_parts(len, packed))
}

fn parse_bytes<'de>(reader: &mut &'de [u8]) -> Result<&'de [u8]> {
    let len = parse_length(reader)?;
    utils::read_bytes(reader, len)
}

fn parse_str<'de>(reader: &mut &'de [u8]) -> Result<&'de str> {
    Ok(str::from_utf8(parse_bytes(reader)?)?)
}

fn parse_table<'de>(reader: &mut &'de [u8]) -> Result<Table<'de>> {
    let cols_len = parse_length(reader)?;
    let vals_len = parse_length(reader)?;

    fn parse_column<'de>(reader: &mut &'de [u8], len: usize) -> Result<(u16, List<'de>)> {
        let (key, ty) = parse_header(reader)?;
        Ok((u16::try_from(key)?, parse_list_values(reader, len, ty)?))
    }

    try_collect(cols_len, || parse_column(reader, vals_len)).map(Table)
}

fn parse_list<'de>(reader: &mut &'de [u8]) -> Result<List<'de>> {
    let (len, ty) = parse_header(reader)?;
    parse_list_values(reader, usize::try_from(len)?, ty)
}

fn parse_list_values<'de>(reader: &mut &'de [u8], len: usize, ty: u8) -> Result<List<'de>> {
    match ty {
        0 => Err(errors::ParseError.into()),
        1 => parse_packed_booleans(reader, len).map(List::Bool),

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
        10 => try_collect(len, || parse_entry(reader)).map(List::Union),

        11 => try_collect(len, || parse_list(reader)).map(List::List),
        12 => try_collect(len, || parse_table(reader)).map(List::Table),
        code => {
            let bytes = try_collect(len, || parse_bytes(reader));
            match code {
                13 => bytes.map(List::UnknownI),
                14 => bytes.map(List::UnknownII),
                15 => bytes.map(List::UnknownIII),
                _ => unreachable!(),
            }
        }
    }
}

impl<'de> Entries<'de> {
    pub fn parse(reader: &mut &'de [u8]) -> Result<Self> {
        let len = parse_length(reader)?;
        try_collect(len, || parse_entry(reader)).map(Entries::from)
    }
}

fn parse_entry<'de>(reader: &mut &'de [u8]) -> Result<Entry<'de>> {
    let (key, ty) = parse_header(reader)?;

    let key = u16::try_from(key)?;
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
        10 => parse_entry(reader).map(Box::new).map(Value::Union),
        11 => parse_list(reader).map(Value::List),
        12 => parse_table(reader).map(Value::Table),
        code => {
            let bytes = parse_bytes(reader);
            match code {
                13 => bytes.map(Value::UnknownI),
                14 => bytes.map(Value::UnknownII),
                15 => bytes.map(Value::UnknownIII),
                _ => unreachable!(),
            }
        }
    }?;

    Ok(Entry { key, value })
}
