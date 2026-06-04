use setu_type_info::{FnMetaData, FnOutputTy, Func};

use crate::{CodeWriter, Context};
use std::format_args as args;

pub fn generate(c: &mut CodeWriter, ctx: &Context) {
    for Func {
        meta,
        input_ty,
        output_ty,
        ..
    } in &ctx.info.fns
    {
        let FnMetaData {
            index, ident, args, ..
        } = meta;

        c.newline();
        c.block(args!("export interface {ident}"), |c| {
            ctx.write_object_tys(c, ',', args.iter().zip(input_ty));
        });

        c.block(
            args!("export function {ident}(args: {ident}, ctx: $.Context = {{}})"),
            |c| match output_ty {
                FnOutputTy::Return(return_ty) => {
                    c.line(args!("let [i, o] = $.rpc({index}, ctx, function () {{"));
                    c.scope(|c| {
                        let required = return_ty.optional().is_none();
                        let decoder = ctx.serde_ty(return_ty, "decoder");
                        c.line(args!("return $.lipi.OutputDecoder(this, {decoder}, {required});"));
                    });
                    c.line("});");

                    c.line("i.sendAndClose(function (this: $.lipi.Encode) {");
                    c.scope(|c| {
                        let fields = args
                            .iter()
                            .zip(input_ty)
                            .enumerate()
                            .map(|(key, (name, ty))| (name.as_ref(), ty, key as u32));

                        ctx.struct_encoder(c, fields);
                    });
                    c.line("});");
                    c.line("return o;");
                }
                FnOutputTy::Generator { .. } => {}
            },
        );
    }
}
