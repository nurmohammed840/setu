use std::format_args as args;

use type_id::{Attributes, ComplexData, ComplexDataType, Discriminant};
use type_id::{EnumField, StructField};

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
                ComplexDataType::Enum { is_numeric , fields } if *is_numeric => {
                    let Some((ty, repr)) = enum_numeric_repr(fields) else {
                        continue;
                    };
                    c.line(args!("{interface_name}: function {repr}(this: $.lipi.Encode, z: {interface_name}) {{"));
                    c.scope(|c| {
                        c.line(args!("this.{ty}(z)"));
                    });
                    c.line("},");
                },
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
                ComplexDataType::Enum { is_numeric, fields } if *is_numeric => {
                    let Some((ty, repr)) = enum_numeric_repr(fields) else {
                        continue;
                    };
                    c.line(args!("{interface_name}: function {repr}(this: $.lipi.Decode): {interface_name} {{"));
                    c.scope(|c| {
                        c.line(args!("let tag = this.{ty}();"));
                        c.block("switch (tag)", |c| {
                            for (_, EnumField {name, discriminant, ..}) in fields {
                                c.line(args!("case {discriminant}: {interface_name}.{name};"));
                            }
                            c.line("default: throw new Error(`unknown tag: ${tag}`);");
                        });
                    });
                    c.line("},");
                },
                ComplexDataType::Enum { .. } => {

                },
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

fn enum_numeric_repr(fields: &[(Attributes, EnumField)]) -> Option<(&str, &str)> {
    fields
        .iter()
        .find_map(|(_, field)| match field.discriminant {
            Discriminant::U8(_) => Some(("U8", "U8")),
            Discriminant::I8(_) => Some(("I8", "I8")),

            Discriminant::U16(_) => Some(("U16", "UInt")),
            Discriminant::U32(_) => Some(("U32", "UInt")),
            Discriminant::U64(_) => Some(("U64", "UInt")),

            Discriminant::I16(_) => Some(("I16", "Int")),
            Discriminant::I32(_) => Some(("I32", "Int")),
            Discriminant::I64(_) => Some(("I64", "Int")),
            Discriminant::None => None,
        })
}
