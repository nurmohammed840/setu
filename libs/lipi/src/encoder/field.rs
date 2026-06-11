use super::*;

pub trait Field {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()>;
}

impl Field for bool {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_id_and_ty(writer, id, DataType::from(*self))
    }
}

impl<T> Field for T
where
    T: Encode + ?Sized,
{
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_id_and_ty(writer, id, T::TY)?;
        T::encode(self, writer)
    }
}
