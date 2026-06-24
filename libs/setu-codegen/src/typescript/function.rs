use setu_type_info::{FnMetaData, FnOutputTy, Func};

use crate::{CodeWriter, Context};
use std::format_args as args;
use type_id::Type;

pub fn generate(c: &mut CodeWriter, ctx: &Context) {
    for Func {
        meta,
        input_ty,
        output_ty,
        stream,
        ..
    } in &ctx.info.fns
    {
        let FnMetaData {
            index, ident, args, ..
        } = meta;

        c.newline();

        let fn_input = match args.len() {
            0 => args!(""),
            1 if let Some(ty) = input_ty.first() => {
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
                let mut fn_call_body =
                    |t, return_tys: &[&Type], stream_encoder: &dyn Fn(&mut CodeWriter)| {
                        c.line(args!("return $.{t}("));
                        c.scope(|c| {
                            c.line(args!("{index}, ctx,"));

                            input_encoder(ctx, c, input_ty, args);
                            stream_encoder(c);

                            for ty in return_tys {
                                if matches!(ty, Type::Tuple(tys) if tys.is_empty()) {
                                    c.line("_ => { }");
                                    continue;
                                }
                                let required = ty.optional().is_none();
                                let decoder = ctx.serde_ty(ty, "$D");

                                c.line(args!("_ => $OD(_, {decoder}, {required}),"));
                            }
                        });
                        c.line(");");
                    };

                match output_ty {
                    FnOutputTy::Return(return_ty) if let Some(ty) = stream => {
                        fn_call_body("uni", &[return_ty], &|c| {
                            
                            // ty.yield_ty;
                            // ty.yield_ty;
                        });
                    }
                    FnOutputTy::Return(return_ty) => {
                        fn_call_body("rpc", &[return_ty], &|_| {});
                    }
                    FnOutputTy::Generator(_) if let Some(_) = stream => {}
                    FnOutputTy::Generator(g) => {
                        fn_call_body("sse", &[&g.yield_ty, &g.return_ty], &|_| {});
                    }
                }
            },
        );
    }
}

fn input_encoder(ctx: &Context, c: &mut CodeWriter, input_ty: &[Type], args: &[Box<str>]) {
    let mut args = args.iter().zip(input_ty);

    if args.len() == 0 {
        return c.line("_ => $SE(_, []),");
    }
    if args.len() == 1 {
        let (arg_name, ty) = args.next().unwrap();
        let decoder = ctx.serde_ty(ty, "$E");
        return c.line(args!("_ => $SE(_, [[0, {arg_name}, {decoder}]]),"));
    }

    let fields = args
        .enumerate()
        .map(|(key, (name, ty))| (name.as_ref(), ty, key as u32));

    c.line("_ => $SE(_, [");
    ctx.struct_encoder(c, fields);
    c.line("]),");
}
