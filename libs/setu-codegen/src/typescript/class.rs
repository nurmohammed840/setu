use std::format_args as args;

use type_id::StructField;
use type_id::{ComplexData, ComplexDataType};

use crate::{CodeWriter, Context};

pub fn generate(c: &mut CodeWriter, ctx: &Context) {
    for (path, ComplexData { ty, .. }) in ctx.info.registry.iter() {
        let class_name = ctx.symbol.class_name(path);

        c.newline();
        c.block(args!("export class {class_name}"), |c| match ty {
            ComplexDataType::Struct { fields } => {
                ctx.write_object_tys(c, ';', fields.iter().map(|(_, s)| (&s.name, &s.ty)));

                c.block(args!("constructor(args: {class_name})"), |c| {
                    for (_, StructField { name, .. }) in fields {
                        c.line(args!("this.{name} = args.{name};"));
                    }
                });

                if ctx.is_decoder_needed(path) {
                    c.block(
                        "static decoder = function Struct(this: $.lipi.Decode)",
                        |c| {
                            c.line(args!(
                                "return new {class_name}(($.lipi.StructDecoder(this, ["
                            ));
                            c.scope(|c| {
                                for (_, StructField { name, ty, key }) in fields {
                                    let required = ty.optional().is_none();
                                    let decoder = ctx.decode_ty(ty);

                                    c.line(args!("[\"{name}\", {key}, {decoder}, {required}],",));
                                }
                            });
                            c.write("] as const)));\n");
                        },
                    );
                    // ...
                }
                if ctx.is_encoder_needed(path) {
                    c.arrow_fn(
                        args!("static encoder = $.Obj<{class_name}>((s, args)"),
                        |c| {
                            for (_, StructField { name, ty, key }) in fields {
                                c.line(args!("{}({key}, args.{name});", ctx.lipi_ty(ty)));
                            }
                        },
                    );
                }
            }
            _ => unimplemented!(),
        });
    }
}
