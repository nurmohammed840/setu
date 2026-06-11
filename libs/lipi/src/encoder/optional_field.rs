use super::*;

pub trait OptionalField {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()>;
}

impl OptionalField for bool {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        encode_field_id_and_ty(writer, id.into(), DataType::from(*self))
    }
}

impl<T: OptionalField> OptionalField for Option<T> {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        match self {
            Some(val) => OptionalField::encode(val, writer, id),
            None => Ok(()),
        }
    }
}

impl<T> OptionalField for T
where
    T: Encode + ?Sized,
{
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u16) -> Result<()> {
        encode_field_id_and_ty(writer, id.into(), T::TY)?;
        T::encode(self, writer)
    }
}
