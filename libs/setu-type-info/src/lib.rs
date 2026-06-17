pub use type_id;

use type_id::{ControlFlowType, Ident, Type, TypeId, TypeRegistry};

#[derive(Debug, Clone)]
pub struct Func<T> {
    pub stream: Option<Box<ControlFlowType>>,
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
            .pop_if(|ty| matches!(ty, Type::ControlFlow(_)))
            .map(|ty| match ty {
                Type::ControlFlow(inner) => inner,
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

pub const fn fn_args_count<F, Args>(_: &F) -> usize
where
    F: std_lib::FnOnce<Args>,
    Args: TupleArgs,
{
    Args::LEN
}

pub trait TupleArgs {
    const LEN: usize;
}

macro_rules! tuple_args {
    [Len: $len:tt $($name:tt)*] => {
        impl<$($name,)*> TupleArgs for ($($name,)*) {
            const LEN: usize = $len;
        }
    }
}

tuple_args! { Len: 0 }
tuple_args! { Len: 1 T0 }
tuple_args! { Len: 2 T0 T1 }
tuple_args! { Len: 3 T0 T1 T2 }
tuple_args! { Len: 4 T0 T1 T2 T3 }
tuple_args! { Len: 5 T0 T1 T2 T3 T4 }
tuple_args! { Len: 6 T0 T1 T2 T3 T4 T5 }
tuple_args! { Len: 7 T0 T1 T2 T3 T4 T5 T6 }
tuple_args! { Len: 8 T0 T1 T2 T3 T4 T5 T6 T7 }
tuple_args! { Len: 9 T0 T1 T2 T3 T4 T5 T6 T7 T8 }
tuple_args! { Len:10 T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 }
tuple_args! { Len:11 T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 }
tuple_args! { Len:12 T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 }
tuple_args! { Len:13 T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 }
tuple_args! { Len:14 T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 }
tuple_args! { Len:15 T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 }
