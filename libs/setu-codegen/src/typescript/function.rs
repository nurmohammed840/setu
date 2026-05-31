use setu_type_info::{FnMetaData, Func};

use crate::{CodeWriter, Context};
use std::format_args as args;

pub fn generate(c: &mut CodeWriter, ctx: &Context) {
    for Func { meta, input_ty, .. } in &ctx.info.fns {
        let FnMetaData {
            index, ident, args, ..
        } = meta;

        c.block(args!("\nexport interface {ident}"), |c| {
            ctx.write_object_tys(c, ',', args.iter().zip(input_ty));
        });

        c.block(
            args!("export function {ident}(args: {ident}, ctx: $.Context = {{}})"),
            |c| {
                c.line(args!("let [i, o] = $.rpc({index}, ctx);"));

                c.inline_arrow_fn("i.sendAndClose(s", |c| {
                    for (key, (name, ty)) in args.iter().zip(input_ty).enumerate() {
                        c.line(args!("{}({key}, args.{name});", ctx.lipi_ty(ty)));
                    }
                });
            },
        );
    }
}
