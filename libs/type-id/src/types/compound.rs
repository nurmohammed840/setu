use crate::*;
use std::collections::*;

ty! {
    [T; N]: [T: TypeId, const N: usize] (r) {
        Type::Array { ty: Box::new(T::ty(r)), len: N }
    }

    Vec<T>: [T: TypeId] (r) {
        Type::List { variant: ListVariant::Vec, ty: Box::new(T::ty(r)) }
    }

    VecDeque<T>: [T: TypeId] (r) {
        Type::List { variant: ListVariant::VecDeque, ty: Box::new(T::ty(r)) }
    }

    LinkedList<T>: [T: TypeId] (r) {
        Type::List { variant: ListVariant::LinkedList, ty: Box::new(T::ty(r)) }
    }

    BTreeSet<T>: [T: TypeId] (r) {
        Type::List { variant: ListVariant::BTreeSet, ty: Box::new(T::ty(r)) }
    }

    BinaryHeap<T>: [T: TypeId] (r) {
        Type::List { variant: ListVariant::BinaryHeap, ty: Box::new(T::ty(r)) }
    }

    HashSet<T>: [T: TypeId] (r) {
        Type::List { variant: ListVariant::HashSet, ty: Box::new(T::ty(r)) }
    }

    BTreeMap<K, V>: [K: TypeId, V: TypeId] (r) {
        Type::Map { variant: MapVariant::BTreeMap, ty: Box::new((K::ty(r), V::ty(r))) }
    }

    HashMap<K, V>: [K: TypeId, V: TypeId] (r) {
        Type::Map { variant: MapVariant::HashMap, ty: Box::new((K::ty(r), V::ty(r))) }
    }
}


macro_rules! impl_for_typles {
    [$(($($ty: ident)*))*]  => ($(
        impl<$($ty),*> TypeId for ($($ty,)*)
        where
            $($ty: TypeId),*
        {
            fn ty(_c: &mut TypeRegistry) -> Type {
                Type::Tuple(vec![$($ty::ty(_c)),*])
            }
        }
    )*);
}

impl_for_typles!(
    ()
    (T1)
    (T1 T2)
    (T1 T2 T3)
    (T1 T2 T3 T4)
    (T1 T2 T3 T4 T5)
    (T1 T2 T3 T4 T5 T6)
    (T1 T2 T3 T4 T5 T6 T7)
    (T1 T2 T3 T4 T5 T6 T7 T8)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15)
    (T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16)
);
