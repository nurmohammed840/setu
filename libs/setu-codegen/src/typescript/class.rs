use std::format_args as args;

use type_id::StructField;
use type_id::{ComplexData, ComplexDataType};

use crate::{CodeWriter, Context};

pub fn generate(c: &mut CodeWriter, ctx: &Context) {
    for (path, ComplexData { ty, .. }) in ctx.info.registry.iter() {
        let class_name = ctx.symbol.class_name(path);

        match ty {
            ComplexDataType::Struct { fields } => {
                c.newline();
                c.block(args!("export interface {class_name}"), |c| {
                    ctx.write_object_tys(c, ';', fields.iter().map(|(_, s)| (&s.name, &s.ty)));
                });
                c.block(args!("namespace {class_name}"), |c| {
                    if ctx.is_encoder_needed(path) {
                        c.block(args!("export const encoder = function Struct(this: $.lipi.Encode, args: {class_name})"), |c| {
                            ctx.struct_encoder(c, fields.iter().map(|(_, s)| (s.name.as_ref(), &s.ty, s.key)));
                        });
                    }
                    if ctx.is_decoder_needed(path) {
                        c.block(
                            args!("export const decoder = function Struct(this: $.lipi.Decode): {class_name}"),
                            |c| {
                                c.line(args!(
                                    "return $.lipi.StructDecoder(this, ["
                                ));
                                c.scope(|c| {
                                    for (_, StructField { name, ty, key }) in fields {
                                        let required = ty.optional().is_none();
                                        let decoder = ctx.serde_ty(ty, "decoder");
                                        c.line(args!("[\"{name}\", {key}, {decoder}, {required}],",));
                                    }
                                });
                                c.line("]);");
                            },
                        );
                    }
                });
            }
            _ => unimplemented!(),
        }
    }
}
