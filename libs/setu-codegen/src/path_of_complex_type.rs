use std::{collections, slice::Iter};

use setu_type_info::{FnMetaData, FnOutputTy, Func, TypeInfo};
use type_id::{PathIdent, Type};

#[derive(Debug)]
pub struct PathsOfComplexType {
    paths: collections::HashSet<PathIdent>,
}

impl PathsOfComplexType {
    fn new<'a, I>(info: &'a TypeInfo, f: fn(_: Iter<'a, Func<FnMetaData>>) -> I) -> Self
    where
        I: Iterator<Item = &'a Type>,
    {
        let mut paths = collections::HashSet::new();

        for ty in f(info.fns.iter()) {
            visit_complex_type(ty, &mut |path| {
                paths.insert(path.clone());
            });
        }

        Self { paths }
    }

    pub fn from_fn_input(info: &TypeInfo) -> Self {
        Self::new(info, |fns| fns.flat_map(|f| &f.input_ty))
    }

    pub fn from_fn_output(info: &TypeInfo) -> Self {
        Self::new(info, |fns| {
            fns.flat_map(|f| OutputTypeIter::new(&f.output_ty))
        })
    }

    pub fn contains(&self, path: &str) -> bool {
        self.paths.contains(path)
    }
}

struct OutputTypeIter<'a> {
    return_ty: Option<&'a Type>,
    yield_ty: Option<&'a Type>,
}

impl<'a> OutputTypeIter<'a> {
    fn new(ty: &'a FnOutputTy) -> Self {
        match ty {
            FnOutputTy::Return(ty) => Self {
                yield_ty: None,
                return_ty: Some(ty),
            },
            FnOutputTy::Generator {
                yield_ty,
                return_ty,
            } => Self {
                yield_ty: Some(yield_ty),
                return_ty: Some(return_ty),
            },
        }
    }
}

impl<'a> Iterator for OutputTypeIter<'a> {
    type Item = &'a Type;
    fn next(&mut self) -> Option<Self::Item> {
        self.yield_ty.take().or_else(|| self.return_ty.take())
    }
}

fn visit_complex_type<'a>(ty: &'a Type, f: &mut impl FnMut(&'a PathIdent)) {
    match ty {
        Type::List { ty, .. } | Type::Array { ty, .. } | Type::Option(ty) => {
            visit_complex_type(ty, f)
        }
        Type::Map { ty, .. } | Type::Result(ty) => {
            visit_complex_type(&ty.0, f);
            visit_complex_type(&ty.1, f);
        }
        Type::Tuple(items) => {
            for ty in items.iter() {
                visit_complex_type(ty, f)
            }
        }
        Type::Complex(ty) => f(ty),
        _ => {}
    }
}
