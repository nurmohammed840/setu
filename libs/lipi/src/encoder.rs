use super::*;
use std::io::Result;

use varint::*;

pub trait FieldEncoder {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()>;
}

trait Item {
    fn ty() -> u8;
    fn encode(&self, writer: &mut dyn Write) -> Result<()>;
}

fn encode_header(writer: &mut dyn Write, num: u32, ty: u8) -> Result<()> {
    if num < 15 {
        writer.write_all(&[(num as u8) << 4 | ty])
    } else {
        let mut buf = Leb128Buf::<6>::new();
        buf.write_byte((0b1111 << 4) | ty);
        buf.write_u32(num - 15);
        writer.write_all(buf.as_bytes())
    }
}

fn encode_bytes(writer: &mut dyn Write, bytes: &[u8]) -> Result<()> {
    let len = u32::try_from(bytes.len()).map_err(io::Error::other)?;

    let mut buf = Leb128Buf::<5>::new();
    buf.write_u32(len);
    writer.write_all(buf.as_bytes())?;

    writer.write_all(bytes)
}

fn encode_sign(writer: &mut dyn Write, num: i64) -> Result<()> {
    let mut buf = Leb128Buf::<10>::new();
    buf.write_u64(zig_zag::into_u64(num));
    writer.write_all(buf.as_bytes())
}

fn encode_unsign(writer: &mut dyn Write, num: u64) -> Result<()> {
    let mut buf = Leb128Buf::<10>::new();
    buf.write_u64(num);
    writer.write_all(buf.as_bytes())
}

impl Item for bool {
    fn ty() -> u8 {
        1
    }

    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(&[match self {
            false => 0,
            true => 1,
        }])
    }
}

impl Item for f32 {
    fn ty() -> u8 {
        4
    }
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl Item for f64 {
    fn ty() -> u8 {
        5
    }
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

macro_rules! impl_item {
    (@unsign: $($ty: ty),*) => {$(
        impl Item for $ty {
            fn ty() -> u8 { 6 }
            #[inline]
            fn encode(&self, writer: &mut dyn Write) -> Result<()> {
                encode_unsign(writer, (*self).into())
            }
        }
    )*};
    (@sign: $($ty: ty),*) => {$(
        impl Item for $ty {
            fn ty() -> u8 { 7 }
            #[inline]
            fn encode(&self, writer: &mut dyn Write) -> Result<()> {
                encode_sign(writer, (*self).into())
            }
        }
    )*};
}

impl_item! { @unsign: u16, u32, u64 }
impl_item! { @sign: i16, i32, i64 }

impl Item for &str {
    fn ty() -> u8 {
        8
    }
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        encode_bytes(writer, self.as_bytes())
    }
}

impl Item for Entries<'_> {
    fn ty() -> u8 {
        9
    }

    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        for Entry { key, value } in self.iter() {
            FieldEncoder::encode(value, writer, *key)?;
        }
        writer.write_all(&[10])
    }
}

impl<T: Encoder> Item for T {
    fn ty() -> u8 {
        9
    }
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        T::encode(self, writer)
    }
}

impl Item for &[u8] {
    fn ty() -> u8 {
        11
    }
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        let len = u32::try_from(self.len()).map_err(io::Error::other)?;
        encode_header(writer, len, 2)?;
        writer.write_all(self)
    }
}

impl Item for &[i8] {
    fn ty() -> u8 {
        11
    }
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        let len = u32::try_from(self.len()).map_err(io::Error::other)?;
        encode_header(writer, len, 3)?;
        writer.write_all(utils::u8_slice_from(self))
    }
}

impl Item for List<'_> {
    fn ty() -> u8 {
        11
    }
    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        match self {
            List::Bool(items) => Item::encode(items, writer),
            List::U8(items) => Item::encode(items, writer),
            List::I8(items) => Item::encode(items, writer),
            List::F32(items) => Item::encode(items, writer),
            List::F64(items) => Item::encode(items, writer),
            List::UInt(items) => Item::encode(items, writer),
            List::Int(items) => Item::encode(items, writer),
            List::Str(items) => Item::encode(items, writer),
            List::List(items) => Item::encode(items, writer),
            List::Struct(items) => Item::encode(items, writer),
            List::Table(_) => todo!(),
            List::Union(_) => todo!(),
            // ---
            List::UnknownI(_) => todo!(),
            List::UnknownII(_) => todo!(),
            List::UnknownIII(_) => todo!(),
        }
    }
}

impl<T: Item> Item for Vec<T> {
    fn ty() -> u8 {
        11
    }

    fn encode(&self, writer: &mut dyn Write) -> Result<()> {
        // +--------+--------+...+--------+
        // |sssstttt| elements            |
        // +--------+--------+...+--------+
        // Compact protocol list header (2+ bytes, long form) and elements:
        // +--------+--------+...+--------+--------+...+--------+
        // |1111tttt| size                | elements            |
        // +--------+--------+...+--------+--------+...+--------+
        let len = u32::try_from(self.len()).map_err(io::Error::other)?;
        encode_header(writer, len, T::ty())?;
        self.iter().try_for_each(|el| T::encode(el, writer))
    }
}

// ----------------------------------------------------------------------------

macro_rules! impl_map {
    [$($name:ty)*] => [$(
        impl<K: Item, V: Item> Item for $name {
            fn ty() -> u8 { 9 }
            fn encode(&self, writer: &mut dyn Write) -> Result<()> {
                let len = u32::try_from(self.len()).map_err(io::Error::other)?;

                encode_header(writer, 0, 11)?;
                encode_header(writer, len, K::ty())?;
                for key in self.keys() {
                    K::encode(key, writer)?;
                }

                encode_header(writer, 1, 11)?;
                encode_header(writer, len, V::ty())?;
                for value in self.values() {
                    V::encode(value, writer)?;
                }
                writer.write_all(&[10])
            }
        }
        impl<K: Item, V: Item> FieldEncoder for $name {
            fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
                encode_header(writer, id.into(), 9)?;
                Item::encode(self, writer)
            }
        }
    )*];
}

impl_map! {
    std::collections::HashMap<K, V>
    std::collections::BTreeMap<K, V>
}

// ----------------------------------------------------------------------------

impl<T: FieldEncoder> FieldEncoder for Option<T> {
    #[inline]
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        match self {
            None => Ok(()),
            Some(val) => FieldEncoder::encode(val, writer, id),
        }
    }
}

impl FieldEncoder for bool {
    #[inline]
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        let ty = match self {
            false => 0,
            true => 1,
        };
        encode_header(writer, id.into(), ty)
    }
}

impl FieldEncoder for u8 {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        encode_header(writer, id.into(), 2)?;
        writer.write_all(&[*self])
    }
}
impl FieldEncoder for i8 {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        encode_header(writer, id.into(), 3)?;
        writer.write_all(&[self.cast_unsigned()])
    }
}

impl<T: Item> FieldEncoder for Vec<T> {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        encode_header(writer, id.into(), 11)?;
        Item::encode(self, writer)
    }
}

// -------------------------------------------------------------------------------

macro_rules! impl_field_encoder {
    [$($ty:ty)*] => [$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
                encode_header(writer, id.into(), <Self as Item>::ty())?;
                <Self as Item>::encode(self, writer)
            }
        }
    )*];
    [$($ty:ty : $target:ty)*] => [$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
                encode_header(writer, id.into(), <$target as Item>::ty())?;
                <$target as Item>::encode(&self, writer)
            }
        }
    )*];
}

impl_field_encoder! {
    f32
    f64
    u16 u32 u64
    i16 i32 i64
    Entries<'_>
    List<'_>
}

impl_field_encoder! {
    str: &str
    [u8]: &[u8]
    [i8]: &[i8]
}

// ---------------------------------------------------------------

impl FieldEncoder for Vec<u8> {
    fn encode(&self, w: &mut dyn Write, id: u16) -> Result<()> {
        <[u8]>::encode(self, w, id)
    }
}

impl FieldEncoder for Vec<i8> {
    fn encode(&self, w: &mut dyn Write, id: u16) -> Result<()> {
        <[i8]>::encode(self, w, id)
    }
}

impl FieldEncoder for String {
    fn encode(&self, w: &mut dyn Write, id: u16) -> Result<()> {
        <str>::encode(self, w, id)
    }
}

// ---------------------------------------------------------------

// ---------------------------------------------------------------

impl FieldEncoder for Value<'_> {
    fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
        match self {
            Value::Bool(val) => FieldEncoder::encode(val, writer, id),
            Value::U8(val) => FieldEncoder::encode(val, writer, id),
            Value::I8(val) => FieldEncoder::encode(val, writer, id),
            Value::F32(val) => FieldEncoder::encode(val, writer, id),
            Value::F64(val) => FieldEncoder::encode(val, writer, id),
            Value::UInt(val) => FieldEncoder::encode(val, writer, id),
            Value::Int(val) => FieldEncoder::encode(val, writer, id),
            Value::Str(val) => FieldEncoder::encode(*val, writer, id),
            Value::Struct(entries) => FieldEncoder::encode(entries, writer, id),
            Value::List(list) => FieldEncoder::encode(list, writer, id),
            Value::Table(_) => todo!(),
            Value::Union(_) => todo!(),

            Value::UnknownI(_) => todo!(),
            Value::UnknownII(_) => todo!(),
            Value::UnknownIII(_) => todo!(),
        }
    }
}

pub fn field_encoder<T: Encoder>(this: &T, writer: &mut dyn Write, id: u16) -> Result<()> {
    encode_header(writer, id.into(), 9)?;
    T::encode(this, writer)
}

// ------------------------------------------------------------------------------------------

macro_rules! tuples {
    [$($name:tt : $idx:tt)*] => {
        impl<$($name,)*> Item for ($($name,)*)
        where
            $($name: FieldEncoder,)*
        {
            fn ty() -> u8 { 9 }
            fn encode(&self, writer: &mut dyn Write) -> Result<()> {
                $($name::encode(&self.$idx, writer, $idx)?;)*
                writer.write_all(&[10])
            }
        }

        impl<$($name,)*> FieldEncoder for ($($name,)*)
        where
            Self: Item,
        {
            fn encode(&self, writer: &mut dyn Write, id: u16) -> Result<()> {
                encode_header(writer, id.into(), Self::ty())?;
                Item::encode(self, writer)
            }
        }
    }
}

tuples! { T0:0 }
tuples! { T0:0 T1:1 }
tuples! { T0:0 T1:1 T2:2 }
tuples! { T0:0 T1:1 T2:2 T3:3 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 T13:13 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 T13:13 T14:14 }
