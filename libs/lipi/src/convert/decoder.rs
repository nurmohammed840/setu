use crate::{BitSet, Result, convert::DataType, utils, varint};

pub fn parse_length(reader: &mut &[u8]) -> Result<usize> {
    Ok(usize::try_from(varint::read_u64(reader)?)?)
}

pub fn parse_header(reader: &mut &[u8]) -> Result<(u64, DataType)> {
    let byte = utils::read_byte(reader)?;

    let ty = byte & 0b_1111;
    let id = (byte >> 4) as u64;

    let id = if id == 0b_1111 {
        varint::read_u64(reader)? + 15
    } else {
        id
    };

    Ok((id, unsafe { std::mem::transmute(ty) }))
}

pub fn parse_packed_booleans<'de>(reader: &mut &'de [u8], len: usize) -> Result<&'de [u8]> {
    let packed = utils::read_bytes(reader, utils::bool_packed_len(len))?;
    todo!()
    // Ok(BitSet::from_parts(len, packed))
}

pub fn parse_bytes<'de>(reader: &mut &'de [u8]) -> Result<&'de [u8]> {
    let len = parse_length(reader)?;
    Ok(utils::read_bytes(reader, len)?)
}

pub fn parse_str<'de>(reader: &mut &'de [u8]) -> Result<&'de str> {
    Ok(str::from_utf8(parse_bytes(reader)?)?)
}

// -----------------------------------------------------------------------------

pub trait Decode<'de>: Sized {
    const TY: DataType;
    fn decode(reader: &mut &'de [u8]) -> Result<Self>;

    fn decode_field(ty: DataType, reader: &mut &'de [u8]) -> Result<Self> {
        ty.expected(Self::TY)?;
        Self::decode(reader)
    }

    fn decode_vec(reader: &mut &'de [u8]) -> Result<Vec<Self>> {
        let (len, ty) = parse_header(reader)?;
        ty.expected(Self::TY)?;
        utils::try_collect(usize::try_from(len)?, || Self::decode(reader))
    }
}

// -----------------------------------------------------------------------------
