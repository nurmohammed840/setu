use std::format_args as args;
use type_id::{
    Attributes, ComplexData, ComplexDataType, Discriminant, EnumField, EnumFieldType, PathIdent,
    StructField, Type,
};

use crate::{CodeWriter, Context};

pub fn generate(c: &mut CodeWriter, ctx: &Context) {
    c.block("const $E =", |c| {
        ctx.info
            .registry
            .iter()
            .filter(|(path, _)| ctx.is_encoder_needed(path))
            .for_each(|(path, data)| generate_encoder(c, ctx, path, data));
    });
    c.block("const $D =", |c| {
        ctx.info
            .registry
            .iter()
            .filter(|(path, _)| ctx.is_decoder_needed(path))
            .for_each(|(path, data)| generate_decoder(c, ctx, path, data));
    });

    for (path, ComplexData { ty, .. }) in ctx.info.registry.iter() {
        let interface = ctx.symbol.interface_name(path);
        match ty {
            ComplexDataType::Struct { fields } => {
                c.block(args!("export interface {interface}"), |c| {
                    ctx.write_object_tys(c, ';', fields.iter().map(|(_, s)| (&s.name, &s.ty)));
                });
            }
            ComplexDataType::Enum { is_numeric, fields } if *is_numeric => {
                c.block(args!("export enum {interface}"), |c| {
                    for (_, field) in fields {
                        c.line(args!("{} = {},", field.name, field.discriminant));
                    }
                });
            }
            ComplexDataType::Enum { fields, .. } => {
                c.line(args!("export type {interface} ="));
                c.scope(|c| {
                    for (_, EnumField { name, ty, .. }) in fields {
                        let Some(kind) = enum_field_kind(ty) else {
                            continue;
                        };
                        let value = match kind {
                            EnumKind::Unit => args!(" "),
                            EnumKind::Field(ty) => args!("; value: {} ", ctx.data_ty(ty)),
                        };
                        c.line(args!("| {{ type: {name:?}{value}}}"));
                    }
                });
                c.newline();
            }
            ComplexDataType::Tuple { .. } => {}
        }
    }
}

fn generate_encoder(c: &mut CodeWriter, ctx: &Context, path: &PathIdent, data: &ComplexData) {
    let interface_name = ctx.symbol.interface_name(path);
    match &data.ty {
        ComplexDataType::Struct { fields } => {
            c.line(args!(
                "{interface_name}: function Struct(this: $.lipi.Encode, z: {interface_name}) {{"
            ));
            c.scope(|c| {
                ctx.struct_encoder(
                    c,
                    fields.iter().map(|(_, s)| (s.name.as_ref(), &s.ty, s.key)),
                );
            });
            c.line("},");
        }
        ComplexDataType::Enum { is_numeric, fields } if *is_numeric => {
            let Some((ty, repr)) = enum_numeric_repr(fields) else {
                return;
            };
            c.line(args!(
                "{interface_name}: function {repr}(this: $.lipi.Encode, z: {interface_name}) {{"
            ));
            c.scope(|c| {
                c.line(args!("this.{ty}(z)"));
            });
            c.line("},");
        }
        ComplexDataType::Enum { fields, .. } => {
            c.line(args!(
                "{interface_name}: function Union(this: $.lipi.Encode, z: {interface_name}) {{"
            ));
            c.scope(|c| {
                c.block("switch (z.type)", |c| {
                    for (_, field) in fields {
                        field_encoder(ctx, c, field)
                    }
                });
            });
            c.line("},");
        }
        ComplexDataType::Tuple { .. } => {}
    }
}

fn field_encoder(ctx: &Context, c: &mut CodeWriter, field: &EnumField) {
    let EnumField {
        name,
        ty,
        discriminant: key,
    } = field;

    let Some(kind) = enum_field_kind(ty) else {
        return;
    };

    match kind {
        EnumKind::Unit => {
            c.line(args!(
                "case {name:?}: return $.lipi.FieldEncoder(this, [{key}, false, this.Bool]);"
            ));
        }
        EnumKind::Field(ty) => {
            let decoder = ctx.serde_ty(ty, "$E");
            c.line(args!(
                "case {name:?}: return $.lipi.FieldEncoder(this, [{key}, z.value, {decoder}]);"
            ));
        }
    }
}

fn generate_decoder(c: &mut CodeWriter, ctx: &Context, path: &PathIdent, data: &ComplexData) {
    let interface = ctx.symbol.interface_name(path);
    match &data.ty {
        ComplexDataType::Struct { fields } => {
            c.line(args!(
                "{interface}: function Struct(this: $.lipi.Decode): {interface} {{"
            ));
            c.scope(|c| {
                c.line(args!("return $.lipi.StructDecoder(this, ["));
                c.scope(|c| {
                    for (_, StructField { name, ty, key }) in fields {
                        let required = ty.optional().is_none() as u8;
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
                return;
            };
            c.line(args!(
                "{interface}: function {repr}(this: $.lipi.Decode): {interface} {{"
            ));
            c.scope(|c| {
                c.line(args!("let tag = this.{ty}();"));
                c.block("switch (tag)", |c| {
                    for (_, field) in fields {
                        let key = &field.discriminant;
                        let name = &field.name;
                        c.line(args!("case {key}: {interface}.{name};"));
                    }
                    c.line("default: throw new Error(`unknown tag: ${tag}`);");
                });
            });
            c.line("},");
        }
        ComplexDataType::Enum { fields, .. } => {
            c.line(args!(
                "{interface}: function Union(this: $.lipi.Decode): {interface} {{"
            ));
            c.scope(|c| {
                c.line("return $.lipi.EnumDecoder(this, [");
                c.scope(|c| {
                    for (_, field) in fields {
                        let EnumField {
                            name,
                            ty,
                            discriminant: key,
                        } = field;
                        let Some(kind) = enum_field_kind(ty) else {
                            continue;
                        };
                        match kind {
                            EnumKind::Unit => c.line(args!("[{key}, {name:?}, this.Bool, 1],")),
                            EnumKind::Field(ty) => {
                                let de = ctx.serde_ty(ty, "$D");
                                c.line(args!("[{key}, {name:?}, {de}, 0],"))
                            }
                        }
                    }
                });
                c.line("]);");
            });
            c.line("},");
        }
        ComplexDataType::Tuple { .. } => {}
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

enum EnumKind<'a> {
    Unit,
    Field(&'a Type),
}

fn enum_field_kind(ty: &EnumFieldType) -> Option<EnumKind<'_>> {
    match ty {
        EnumFieldType::Tuple(items) if items.len() == 1 => {
            let (_, ty) = &items[0];
            Some(EnumKind::Field(ty))
        }
        EnumFieldType::Unit => Some(EnumKind::Unit),
        _ => None,
    }
}
