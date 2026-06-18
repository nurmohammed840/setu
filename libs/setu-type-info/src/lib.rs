use std::sync::Arc;

pub use type_id;

use type_id::{Ident, Type, TypeId, TypeRegistry};

#[derive(Debug, Clone)]
pub struct Func<T> {
    pub stream: Option<Arc<GeneratorType>>,
    pub input_ty: Vec<Type>,
    pub output_ty: FnOutputTy,
    pub meta: T,
}

#[derive(Debug, Clone)]
pub enum FnOutputTy {
    Return(Type),
    Generator(GeneratorType),
}

#[derive(Debug, Clone)]
pub struct FnMetaData {
    pub docs: String,
    pub index: u16,
    pub ident: Ident,
    pub args: Vec<Ident>,
}

impl<T> Func<T> {
    pub fn new<F, Args>(_: &F, r: &mut TypeRegistry, meta: T) -> Func<T>
    where
        F: std_lib::FnOnce<Args>,
        F::Output: FnOutputType,
        Args: TypeId,
    {
        let Type::Tuple(mut input_ty) = Args::ty(r) else {
            unreachable!()
        };

        let stream = input_ty
            .pop_if(|ty| matches!(ty, Type::Other(ty) if ty.0.is::<GeneratorType>()))
            .map(|ty| match ty {
                Type::Other(ty) => ty.0.downcast().unwrap(),
                _ => unreachable!(),
            });

        Func {
            meta,
            stream,
            input_ty,
            output_ty: <F::Output as FnOutputType>::fn_output_ty(r),
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
        args: &[&str],
    ) -> Func<FnMetaData>
    where
        F: std_lib::FnOnce<Args>,
        F::Output: FnOutputType,
        Args: TypeId,
    {
        let meta = FnMetaData {
            docs: docs.to_string(),
            index,
            ident: Ident::from(ident),
            args: args.iter().copied().map(Ident::from).collect(),
        };
        Func::new(f, r, meta)
    }
}

#[derive(Debug, Clone)]
pub struct GeneratorType {
    pub yield_ty: Type,
    pub return_ty: Type,
}

#[derive(Clone, Debug, Default)]
pub struct TypeInfo {
    pub registry: TypeRegistry,
    pub fns: Vec<Func<FnMetaData>>,
}

impl TypeInfo {
    pub fn from<T: TypeDefinition>() -> TypeInfo {
        let mut registry = TypeRegistry::new();
        let fns = T::type_definition(&mut registry);
        TypeInfo { registry, fns }
    }
}

pub trait TypeDefinition {
    fn type_definition(r: &mut TypeRegistry) -> Vec<Func<FnMetaData>>;
}

pub trait FnOutputType {
    fn fn_output_ty(_: &mut TypeRegistry) -> FnOutputTy;
}

impl<Fut> FnOutputType for Fut
where
    Fut: Future,
    Fut::Output: TypeId,
{
    fn fn_output_ty(c: &mut TypeRegistry) -> FnOutputTy {
        FnOutputTy::Return(<Fut::Output as TypeId>::ty(c))
    }
}
