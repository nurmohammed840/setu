use std::collections;

use setu_type_info::FnOutputTy;
use type_id::{ComplexData, Type};

use crate::{CodeWriter, Context};

pub fn generate(c: &mut CodeWriter, ctx: &Context) {
    let encoder = ComplexTypePaths::input_types(ctx);
    let decoder = ComplexTypePaths::output_types(ctx);

    for ComplexData { path, .. } in ctx.info.registry.values() {
        let name = ctx.symbol.class_name(path);
        c.block(format_args!("export class {name}"), |_c| {
            if encoder.needed(path) {}
            if decoder.needed(path) {}
        });
    }
}

#[derive(Debug)]
struct ComplexTypePaths<'a> {
    paths: collections::HashSet<&'a str>,
}

impl<'a> ComplexTypePaths<'a> {
    fn needed(&self, path: &str) -> bool {
        self.paths.contains(path)
    }

    fn input_types(ctx: &'a Context) -> Self {
        let fns = ctx.info.fns.iter();
        let paths = fns
            .flat_map(|f| &f.input_ty)
            .flat_map(|ty| ty.complex())
            .collect();

        Self { paths }
    }

    fn output_types(ctx: &'a Context) -> Self {
        let fns = ctx.info.fns.iter();
        let paths = fns
            .flat_map(|f| OutputTypeIter::new(&f.output_ty))
            .flat_map(|ty| ty.complex())
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
