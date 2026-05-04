use crate::{DataType, Result, decoder::*, errors, utils, varint};

impl<'c, 'de> FieldInfoDecoder<'c, 'de> {
    pub fn skip_field(&mut self, id: u64, ty: DataType) -> Result<()> {
        self.skip_field_value(ty)
            .map_err(|error| errors::SkipFieldError { id, error }.into())
    }

    pub fn skip_field_value(&mut self, ty: DataType) -> Result<()> {
        match ty {
            DataType::True | DataType::False => {}
            DataType::U8 | DataType::I8 => {
                utils::read_byte(self.reader)?;
            }
            DataType::F32 => {
                utils::read_bytes(self.reader, 4)?;
            }
            DataType::F64 => {
                utils::read_bytes(self.reader, 8)?;
            }
            DataType::UInt | DataType::Int => {
                varint::read_u64(self.reader)?;
            }
            DataType::Str | DataType::UnknownI | DataType::UnknownII => {
                decode_bytes(self.reader)?;
            }

            DataType::StructEnd => return Err("unexpected struct end".into()),
            DataType::Struct => self.skip_struct()?,
            DataType::Union => self.skip_union()?,
            DataType::List => self.skip_list()?,
            DataType::Table => self.skip_table()?,
        }
        Ok(())
    }

    fn skip_struct(&mut self) -> Result<()> {
        while let Some((id, ty)) = self.next_field_id_and_ty()? {
            self.skip_field(id, ty)?;
        }
        Ok(())
    }

    fn skip_union(&mut self) -> Result<()> {
        let (id, ty) = decode_field_id_and_ty(self.reader)?;
        self.skip_field(id, ty)
    }

    fn skip_list_values(&mut self, len: usize, ty: DataType) -> Result<()> {
        match ty {
            DataType::False => return Err("unexpected bool packed in list".into()),
            DataType::True => {
                // Ignore packed_bools
                utils::read_bytes(self.reader, utils::bool_packed_len(len))?;
            }
            DataType::U8 | DataType::I8 => {
                utils::read_bytes(self.reader, len)?;
            }
            DataType::F32 => {
                utils::read_bytes(self.reader, len * 4)?;
            }
            DataType::F64 => {
                utils::read_bytes(self.reader, len * 8)?;
            }

            DataType::UInt | DataType::Int => {
                for _ in 0..len {
                    varint::read_u64(self.reader)?;
                }
            }

            DataType::Str | DataType::UnknownI | DataType::UnknownII => {
                for _ in 0..len {
                    decode_bytes(self.reader)?;
                }
            }

            DataType::StructEnd => return Err("unexpected list ty struct end".into()),
            DataType::Struct => {
                for _ in 0..len {
                    self.skip_struct()?;
                }
            }
            DataType::Union => {
                for _ in 0..len {
                    self.skip_union()?;
                }
            }
            DataType::List => {
                for _ in 0..len {
                    self.skip_list()?;
                }
            }
            DataType::Table => {
                for _ in 0..len {
                    self.skip_table()?;
                }
            }
        }
        Ok(())
    }

    fn skip_list(&mut self) -> Result<()> {
        let (len, ty) = decode_list_len_and_ty(self.reader)?;
        self.skip_list_values(len, ty)
    }

    fn skip_table(&mut self) -> Result<()> {
        let cols = decode_len(self.reader)?;
        let rows = decode_len(self.reader)?;

        for _ in 0..cols {
            let (col_id, col_ty) = decode_field_id_and_ty(self.reader)?;
            self.skip_list_values(rows, col_ty)
                .map_err(|error| errors::SkipFieldError { id: col_id, error })?;
        }
        Ok(())
    }
}
