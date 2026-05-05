#[macro_export]
macro_rules! ty {
    [ $($ty:ty = $id:expr)* ] => [$(
        impl TypeId for $ty {
            fn ty(_: &mut TypeRegistry) -> Type {
                $id
            }
        }
    )*];

    [ $($ty:ty: $([ $($generic:tt)* ])? ($r:tt) $code:block)* ] => [$(
        impl< $( $($generic)* )? > TypeId for $ty {
            fn ty($r: &mut TypeRegistry) -> Type $code
        }
    )*];
}
