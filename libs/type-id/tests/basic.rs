#![allow(dead_code)]
use type_id::{Type, TypeId, TypeRegistry};

#[derive(TypeId)]
struct A {
    b: B,
}

#[derive(TypeId)]
struct B {
    b: Box<A>,
}

#[test]
fn test_bad_cycle() {
    let mut reg = TypeRegistry::default();

    let ty = A::ty(&mut reg);
    assert_eq!(reg.len(), 2);
    assert!(matches!(ty, Type::Complex(id) if id.0 == "basic::A"));

    let ty = B::ty(&mut reg);
    assert_eq!(reg.len(), 2);
    assert!(matches!(ty, Type::Complex(id) if id.0 == "basic::B"));
}
