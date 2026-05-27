pub use type_id;

use type_id::{Ident, Type, TypeId, TypeRegistry};

#[derive(Debug, Clone)]
pub struct Func<T> {
    pub input_ty: Vec<Type>,
    pub output_ty: FnOutputTy,
    pub meta: T,
}

#[derive(Debug, Clone)]
pub enum FnOutputTy {
    Return(Type),
    Generator { yield_ty: Type, return_ty: Type },
}

#[derive(Debug, Clone)]
pub struct FnMetaData {
    pub docs: String,
    pub index: u16,
    pub ident: Ident,
}

impl<T> Func<T> {
    pub fn new<F, Args>(_: &F, r: &mut TypeRegistry, meta: T) -> Func<T>
    where
        F: std_lib::FnOnce<Args>,
        Args: TypeId,
        F::Output: AsyncFnOutputType,
    {
        let Type::Tuple(input_ty) = Args::ty(r) else {
            unreachable!()
        };
        Func {
            meta,
            input_ty,
            output_ty: <F::Output as AsyncFnOutputType>::async_fn_output_ty(r),
        }
    }
}

impl Func<FnMetaData> {
    pub fn with_meta<F, Args>(
        r: &mut TypeRegistry,
        docs: &str,
        f: &F,
        index: u16,
        ident: &str,
    ) -> Func<FnMetaData>
    where
        F: std_lib::FnOnce<Args>,
        Args: TypeId,
        F::Output: AsyncFnOutputType,
    {
        let meta = FnMetaData {
            docs: docs.to_string(),
            index,
            ident: Ident(ident.into()),
        };
        Func::new(f, r, meta)
    }
}

pub trait TypeDefinition {
    fn type_definition(r: &mut TypeRegistry) -> Vec<Func<FnMetaData>>;
}

pub trait AsyncFnOutputType {
    fn async_fn_output_ty(_: &mut TypeRegistry) -> FnOutputTy;
}

impl<Fut> AsyncFnOutputType for Fut
where
    Fut: Future,
    Fut::Output: TypeId,
{
    fn async_fn_output_ty(c: &mut TypeRegistry) -> FnOutputTy {
        FnOutputTy::Return(<Fut::Output as TypeId>::ty(c))
    }
}
