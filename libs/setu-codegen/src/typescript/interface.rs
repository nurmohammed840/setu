use crate::{CodeWriter, Context};

pub fn generate(c: &mut CodeWriter, ctx: &Context) {
    for f in &ctx.info.fns {
        let name = &*f.meta.ident;

        c.block(format_args!("export interface {name}"), |c| {
            for (arg, ty) in f.meta.args.iter().zip(&f.input_ty) {
                if let Some(ty) = ty.optional() {
                    c.line(format_args!("{arg}?: {},", ctx.data_ty(ty)))
                } else {
                    c.line(format_args!("{arg}: {},", ctx.data_ty(ty)))
                }
            }
        });
    }
}
