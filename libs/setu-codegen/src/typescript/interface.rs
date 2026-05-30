use crate::{CodeWriter, Context};
use std::format_args as args;

pub fn generate(c: &mut CodeWriter, ctx: &Context) {
    for f in &ctx.info.fns {
        let name = &*f.meta.ident;

        c.block(args!("export interface {name}"), |c| {
            ctx.write_object_tys(c, ',', f.meta.args.iter().zip(&f.input_ty));
        });
    }
}
