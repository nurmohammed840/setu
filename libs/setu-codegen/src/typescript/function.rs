use setu_type_info::{FnMetaData, FnOutputTy, Func};

use crate::{CodeWriter, Context};
use std::format_args as args;
use type_id::Type;

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

        let fn_input = match args.len() {
            0 => args!(""),
            1 if let Some(ty) = input_ty.get(0) => {
                if let Some(ty) = ty.optional() {
                    args!("{}?: {}, ", args[0], ctx.data_ty(ty))
                } else {
                    args!("{}: {}, ", args[0], ctx.data_ty(ty))
                }
            }
            _ => {
                c.block(args!("export interface {ident}"), |c| {
                    ctx.write_object_tys(c, ',', args.iter().zip(input_ty));
                });
                args!("z: {ident}, ")
            }
        };

        c.block(
            args!("export function {ident}({fn_input}ctx: $.Context = {{}})"),
            |c| {
                let mut fn_call_body = |t: &str, return_tys: &[&Type]| {
                    c.line(args!("return $.{t}("));
                    c.scope(|c| {
                        c.line(args!("{index}, ctx,"));

                        c.line("function(_) {");
                        c.scope(|c| {
                            let mut fields = args.iter().zip(input_ty);
                            if fields.len() == 1 {
                                let (arg_name, ty) = fields.next().unwrap();
                                let decoder = ctx.serde_ty(ty, "$E");
                                return c.line(args!(
                                    "$.lipi.StructEncoder(_, [[0, {arg_name}, {decoder}]]);"
                                ));
                            }

                            let fields = fields
                                .enumerate()
                                .map(|(key, (name, ty))| (name.as_ref(), ty, key as u32));

                            ctx.struct_encoder(c, fields);
                        });
                        c.line("},");

                        for ty in return_tys {
                            if matches!(ty, Type::Tuple(tys) if tys.is_empty()) {
                                c.line("function(_) {}");
                                continue;
                            }
                            c.line("function(_) {");
                            c.scope(|c| {
                                let required = ty.optional().is_none();
                                let decoder = ctx.serde_ty(ty, "$D");
                                c.line(args!(
                                    "return $.lipi.OutputDecoder(_, {decoder}, {required});"
                                ));
                            });
                            c.line("},");
                        }
                    });
                    c.line(");");
                };
                match output_ty {
                    FnOutputTy::Return(return_ty) => {
                        fn_call_body("rpc", &[return_ty]);
                    }
                    FnOutputTy::Generator {
                        return_ty,
                        yield_ty,
                    } => {
                        fn_call_body("sse", &[yield_ty, return_ty]);
                    }
                }
            },
        );
    }
}
