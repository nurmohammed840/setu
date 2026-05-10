#![allow(dead_code)]

use setu_message::{FnOutputTy, Func, type_id};
use type_id::{Ident, Type, TypeId};

async fn func() {}

async fn add(a: u8, b: u32) -> u32 {
    a as u32 + b
}

#[derive(TypeId)]
struct User {
    id: u32,
    name: String,
    email: Option<String>,
}

async fn func_with_arg(_: User) {}

#[test]
fn test_async_fn() {
    let mut r = type_id::TypeRegistry::default();
    
    let f = Func::new(&func, &mut r, ());
    assert!(f.input_ty.is_empty());
    assert_eq!(output_ty(f), Type::Tuple(vec![]));

    let f = Func::new(&add, &mut r, ());
    assert_eq!(f.input_ty, [Type::U8, Type::U32]);
    assert_eq!(output_ty(f), Type::U32);

    assert!(r.is_empty());
    let f = Func::new(&func_with_arg, &mut r, ());
    assert_eq!(f.input_ty, [Type::Complex(Ident::from("basic::User"))]);
    assert_eq!(r.len(), 1);
}

fn output_ty(f: Func<()>) -> Type {
    match f.output_ty {
        FnOutputTy::Return(ty) => ty,
        _ => unimplemented!(),
    }
}
