use std::{rc::Rc, sync::Arc};

use crate::*;

macro_rules! impl_type_id_for {
    [$($ty: ty), *] => {$(
        impl<T: TypeId + ?Sized> TypeId for $ty {
            fn ty(r: &mut TypeRegistry) -> Type {
                T::ty(r)
            }
        }
    )*};
}

impl_type_id_for! { &T, Box<T>, Arc<T>, Rc<T> }