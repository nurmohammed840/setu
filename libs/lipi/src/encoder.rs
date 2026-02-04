use super::*;
use std::io::Result;
use varint::*;

pub trait FieldEncoder {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()>;
}

trait Item {
    fn ty() -> u8;
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()>;
}

fn encode_header(writer: &mut (impl Write + ?Sized), id: u32, ty: u8) -> Result<()> {
    if id < 15 {
        let header = (id as u8) << 4;
        writer.write_all(&[header | ty])
    } else {
        let header = 0b1111 << 4;
        let mut buf: Leb128Buf<8> = Leb128Buf::<8>::new();
        buf.write_byte(header | ty);
        buf.write_u32(id - 15);
        writer.write_all(buf.as_bytes())
    }
}

fn encode_len_u32(writer: &mut (impl Write + ?Sized), len: usize) -> Result<()> {
    let len: u32 = u32_list_len(len)?;
    let mut buf = Leb128Buf::<8>::new();
    buf.write_u32(len);
    writer.write_all(buf.as_bytes())
}

fn u32_list_len(len: usize) -> Result<u32> {
    u32::try_from(len).map_err(io::Error::other)
}

fn encode_sign(writer: &mut (impl Write + ?Sized), num: i64) -> Result<()> {
    let mut buf = Leb128Buf::<10>::new();
    buf.write_u64(zig_zag::into(num));
    writer.write_all(buf.as_bytes())
}

fn encode_unsign(writer: &mut (impl Write + ?Sized), num: u64) -> Result<()> {
    let mut buf = Leb128Buf::<10>::new();
    buf.write_u64(num);
    writer.write_all(buf.as_bytes())
}

impl Item for bool {
    fn ty() -> u8 {
        1
    }

    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        writer.write_all(&[match self {
            false => 0,
            true => 1,
        }])
    }
}

impl Item for f32 {
    fn ty() -> u8 {
        2
    }

    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl Item for f64 {
    fn ty() -> u8 {
        3
    }
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

macro_rules! impl_item {
    (@sign: $($ty: ty),*) => {$(
        impl Item for $ty {
            fn ty() -> u8 { 4 }
            #[inline]
            fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
                encode_sign(writer, (*self).into())
            }
        }
    )*};
    (@unsign: $($ty: ty),*) => {$(
        impl Item for $ty {
            fn ty() -> u8 { 5 }
            #[inline]
            fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
                encode_unsign(writer, (*self).into())
            }
        }
    )*};
}

impl_item! { @sign: i8, i16, i32, i64 }
impl_item! { @unsign: u16, u32, u64 }

impl Item for &str {
    fn ty() -> u8 {
        6
    }

    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        let bytes = self.as_bytes();
        encode_len_u32(writer, bytes.len())?;
        writer.write_all(bytes)
    }
}

impl Item for &[u8] {
    fn ty() -> u8 {
        7
    }
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        encode_len_u32(writer, self.len())?;
        writer.write_all(self)
    }
}

impl Item for List<'_> {
    fn ty() -> u8 {
        8
    }

    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        match self {
            List::Bool(items) => Item::encode(items, writer),
            List::F32(items) => Item::encode(items, writer),
            List::F64(items) => Item::encode(items, writer),
            List::Int(items) => Item::encode(items, writer),
            List::UInt(items) => Item::encode(items, writer),
            List::Str(items) => Item::encode(items, writer),
            List::Bytes(items) => Item::encode(items, writer),
            List::List(items) => Item::encode(items, writer),
            List::Struct(items) => Item::encode(items, writer),
        }
    }
}

impl<T: Item> Item for Vec<T> {
    fn ty() -> u8 {
        8
    }

    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        // +--------+--------+...+--------+
        // |sssstttt| elements            |
        // +--------+--------+...+--------+
        // Compact protocol list header (2+ bytes, long form) and elements:
        // +--------+--------+...+--------+--------+...+--------+
        // |1111tttt| size                | elements            |
        // +--------+--------+...+--------+--------+...+--------+
        encode_header(writer, u32_list_len(self.len())?, T::ty())?;
        self.iter().try_for_each(|el| T::encode(el, writer))
    }
}

impl Item for Entries<'_> {
    fn ty() -> u8 {
        9
    }

    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        for (key, value) in self.iter() {
            Value::encode(value, writer, *key)?;
        }
        writer.write_all(&[10])
    }
}

impl<T: Encoder> Item for T {
    fn ty() -> u8 {
        9
    }
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        T::encode(self, writer)
    }
}

// ----------------------------------------------------------------------------

impl<T: FieldEncoder> FieldEncoder for Option<T> {
    #[inline]
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        match self {
            None => Ok(()),
            Some(val) => FieldEncoder::encode(val, writer, id),
        }
    }
}

impl FieldEncoder for bool {
    #[inline]
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        let ty = match self {
            false => 0,
            true => 1,
        };
        encode_header(writer, id.into(), ty)
    }
}

macro_rules! impl_field_encoder {
    [$($ty:ty)*] => [$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
                encode_header(writer, id.into(), <Self as Item>::ty())?;
                <Self as Item>::encode(self, writer)
            }
        }
    )*];

    [$($ty:ty : $target:ty)*] => [$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
                encode_header(writer, id.into(), <$target as Item>::ty())?;
                <$target as Item>::encode(&self, writer)
            }
        }
    )*];
}

impl_field_encoder! {
    f32
    f64
    Entries<'_>
    List<'_>
}

impl_field_encoder! {
    str: &str
    [u8]: &[u8]
}

impl FieldEncoder for String {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        FieldEncoder::encode(self.as_str(), writer, id)
    }
}

impl FieldEncoder for Vec<u8> {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        FieldEncoder::encode(self.as_slice(), writer, id)
    }
}

macro_rules! impl_for {
    (unsign: $($ty: ty)*) => {$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
                encode_header(writer, id.into(), 5)?;
                encode_unsign(writer, (*self).into())
            }
        }
    )*};
    (sign: $($ty: ty)*) => {$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
                encode_header(writer, id.into(), 4)?;
                encode_sign(writer, (*self).into())
            }
        }
    )*};
}

impl_for! {
    unsign: u8 u16 u32 u64
}

impl_for! {
    sign: i8 i16 i32 i64
}

impl<'de, T> FieldEncoder for T
where
    T: IntoValue<'de>,
{
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        Value::encode(&self.to_value(), writer, id)
    }
}

impl FieldEncoder for Value<'_> {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        match self {
            Value::Bool(val) => FieldEncoder::encode(val, writer, id),
            Value::F32(val) => FieldEncoder::encode(val, writer, id),
            Value::F64(val) => FieldEncoder::encode(val, writer, id),
            Value::Int(val) => FieldEncoder::encode(val, writer, id),
            Value::UInt(val) => FieldEncoder::encode(val, writer, id),
            Value::Str(val) => FieldEncoder::encode(*val, writer, id),
            Value::Bytes(val) => FieldEncoder::encode(*val, writer, id),
            Value::List(list) => FieldEncoder::encode(list, writer, id),
            Value::Struct(entries) => FieldEncoder::encode(entries, writer, id),
        }
    }
}

pub fn encode_struct_field<T: Encoder>(
    this: &T,
    writer: &mut (impl Write + ?Sized),
    id: u16,
) -> Result<()> {
    encode_header(writer, id.into(), 9)?;
    T::encode(this, writer)
}

impl<T: Item> FieldEncoder for Vec<T> {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        encode_header(writer, id.into(), 8)?;
        Item::encode(self, writer)
    }
}
