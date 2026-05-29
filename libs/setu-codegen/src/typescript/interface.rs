use crate::utils::fmt;
pub use std::fmt::from_fn as fmt;

use crate::CodeWriter;
use setu_type_info::TypeInfo;
use type_id::Type;

pub fn generate(c: &mut CodeWriter, info: &TypeInfo) {
    for f in &info.fns {
        let name = &*f.meta.ident;

        c.block(format_args!("export interface {name}"), |c| {
            for (arg, ty) in f.meta.args.iter().zip(&f.input_ty) {
                if let Some(ty) = ty.optional() {
                    c.fmt_line(format_args!("{}?: {},", arg.0, data_ty(ty)))
                } else {
                    c.fmt_line(format_args!("{}: {},", arg.0, data_ty(ty)))
                }
            }
        });
        c.newline();
    }
}

fn data_ty(ty: &Type) -> fmt!(type) {
    fmt(|f| match ty {
        Type::U8
        | Type::U16
        | Type::U32
        | Type::F32
        | Type::F64
        | Type::I8
        | Type::I16
        | Type::I32 => f.write_str("number"),

        Type::U64 | Type::I64 => f.write_str("bigint"),

        Type::Bool => f.write_str("boolean"),
        Type::String => f.write_str("string"),

        Type::Complex(name) => f.write_str(name),

        Type::List { ty, .. } | Type::Array { ty, .. } => {
            f.write_fmt(format_args!("Array<{}>", data_ty(ty)))
        }
        Type::Map { ty, .. } => {
            f.write_fmt(format_args!("Map<{}, {}>", data_ty(&ty.0), data_ty(&ty.1)))
        }

        Type::Option(ty) => f.write_fmt(format_args!("{} | undefined", data_ty(&ty))),
        Type::Tuple(_) | Type::Result(_) | Type::Char | Type::U128 | Type::I128 => {
            unimplemented!()
        }
    })
}
