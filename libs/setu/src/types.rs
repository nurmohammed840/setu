use async_gen::AsyncGenerator;
use lipi::encoder::OptionalField;
use setu_type_info::{FnOutputTy, FnOutputType};
use type_id::{TypeId, TypeRegistry};

use crate::SSE;

impl<S> FnOutputType for SSE<S>
where
    S: AsyncGenerator,
    S::Yield: OptionalField + TypeId,
    S::Return: OptionalField + TypeId,
{
    fn fn_output_ty(c: &mut TypeRegistry) -> FnOutputTy {
        FnOutputTy::Generator {
            yield_ty: <S::Yield as TypeId>::ty(c),
            return_ty: <S::Return as TypeId>::ty(c),
        }
    }
}
