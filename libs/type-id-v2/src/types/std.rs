use crate::*;

ty! {
    Option<T>: [T: TypeId] (r) {
        Type::Option(Box::new(T::ty(r)))
    }

    Result<T, E>: [T: TypeId, E: TypeId] (r) {
        Type::Result(Box::new((T::ty(r), E::ty(r))))
    }
}
