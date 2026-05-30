use std::collections;

use setu_type_info::{FnOutputTy, TypeInfo};
use type_id::{Ident, Type};

#[derive(Debug)]
pub struct PathOfComplexTypes {
    paths: collections::HashSet<Ident>,
}

impl PathOfComplexTypes {
    pub fn contains(&self, path: &str) -> bool {
        self.paths.contains(path)
    }

    pub fn input_types(info: &TypeInfo) -> Self {
        let fns = info.fns.iter();
        let paths = fns
            .flat_map(|f| &f.input_ty)
            .flat_map(|ty| ty.complex())
            .map(From::from)
            .collect();

        Self { paths }
    }

    pub fn output_types(info: &TypeInfo) -> Self {
        let fns = info.fns.iter();
        let paths = fns
            .flat_map(|f| OutputTypeIter::new(&f.output_ty))
            .flat_map(|ty| ty.complex())
            .map(From::from)
            .collect();

        Self { paths }
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
