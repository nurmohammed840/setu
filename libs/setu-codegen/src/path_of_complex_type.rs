use std::{collections, slice::Iter};

use setu_type_info::{FnMetaData, FnOutputTy, Func, TypeInfo};
use type_id::{Ident, Type};

#[derive(Debug)]
pub struct PathsOfComplexType {
    paths: collections::HashSet<Ident>,
}

impl PathsOfComplexType {
    fn new<'a, I>(info: &'a TypeInfo, f: fn(_: Iter<'a, Func<FnMetaData>>) -> I) -> Self
    where
        I: Iterator<Item = &'a Type>,
    {
        let paths = f(info.fns.iter())
            .flat_map(|ty| ty.complex())
            .map(From::from)
            .collect();

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
