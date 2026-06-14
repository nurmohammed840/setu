use std::format_args as args;

use type_id::StructField;
use type_id::{ComplexData, ComplexDataType};

use crate::{CodeWriter, Context};

pub fn generate(c: &mut CodeWriter, ctx: &Context) {
    c.block("const $E =", |c| {
        for (path, ComplexData { ty, .. }) in ctx.info.registry.iter() {
            if !ctx.is_encoder_needed(path) {
                continue;
            }
            let interface_name = ctx.symbol.interface_name(path);
            match ty {
                ComplexDataType::Struct { fields } => {
                    c.line(args!("{interface_name}: function Struct(this: $.lipi.Encode, z: {interface_name}) {{"));
                    c.scope(|c| {
                        ctx.struct_encoder(c, fields.iter().map(|(_, s)| (s.name.as_ref(), &s.ty, s.key)));
                    });
                    c.line("},");
                }
                ComplexDataType::Enum { .. } => {},
                ComplexDataType::Tuple { .. } => {},
            }
        }
    });

    c.block("const $D =", |c| {
        for (path, ComplexData { ty, .. }) in ctx.info.registry.iter() {
            if !ctx.is_decoder_needed(path) {
                continue;
            }
            let interface_name = ctx.symbol.interface_name(path);
            match ty {
                ComplexDataType::Struct { fields } => {
                    c.line(args!("{interface_name}: function Struct(this: $.lipi.Decode): {interface_name} {{"));
                    c.scope(|c| {
                        c.line(args!("return $.lipi.StructDecoder(this, ["));
                        c.scope(|c| {
                            for (_, StructField { name, ty, key }) in fields {
                                let required = ty.optional().is_none();
                                let decoder = ctx.serde_ty(ty, "$D");
                                c.line(args!("[{key}, \"{name}\", {decoder}, {required}],",));
                            }
                        });
                        c.line("]);");
                    });
                    c.line("},");
                }
                ComplexDataType::Enum { .. } => {},
                ComplexDataType::Tuple { .. } => {},
            }
        }
    });

    for (path, ComplexData { ty, .. }) in ctx.info.registry.iter() {
        let interface_name = ctx.symbol.interface_name(path);
        match ty {
            ComplexDataType::Struct { fields } => {
                c.block(args!("export interface {interface_name}"), |c| {
                    ctx.write_object_tys(c, ';', fields.iter().map(|(_, s)| (&s.name, &s.ty)));
                });
            }
            ComplexDataType::Enum { is_numeric, fields } if *is_numeric => {
                c.block(args!("export enum {interface_name}"), |c| {
                    for (_, field) in fields {
                        c.line(args!("{} = {},", field.name, field.discriminant));
                    }
                });
            }
            ComplexDataType::Enum { .. } => {}
            ComplexDataType::Tuple { .. } => {}
        }
    }
}
