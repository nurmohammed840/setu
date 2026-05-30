pub mod class;
pub mod interface;

use std::format_args as args;
use std::{fs, io, path::PathBuf};
use type_id::{Ident, Type};

use crate::symbol_trie::SymbolTrie;
use crate::{CodeWriter, utils::copy_dir};
use crate::{Context, utils::fmt};
pub use std::fmt::from_fn as fmt;

#[derive(Debug, Clone)]
pub struct Config {
    pub out_dir: PathBuf,
    pub preserve_import_extension: bool,
}

impl Config {
    pub fn out_dir(path: PathBuf) -> Self {
        Self {
            out_dir: path,
            preserve_import_extension: false,
        }
    }

    pub fn preserve_import_extension(mut self) -> Self {
        self.preserve_import_extension = true;
        self
    }
}

impl Config {
    pub fn generate(&self, ctx: &Context) -> io::Result<()> {
        fs::create_dir_all(&self.out_dir)?;

        let lib = self.out_dir.join("lib");
        if !lib.exists() {
            let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("clients/typescript");

            copy_dir(&src, &lib, 1, |file| {
                file.extension().is_some_and(|ext| ext == "ts")
            })?;
        }

        let code = ctx.generate_typescript_code();
        fs::write(self.out_dir.join("mod.ts"), code)
    }
}

static TS_PRELUDE: &str = r#"
import * as $ from "./lib/mod.ts";
export const $etu = { RPC: $.RPC };
"#;

impl Context {
    pub fn generate_typescript_code(&self) -> String {
        let mut c = CodeWriter::new();
        c.write_line(TS_PRELUDE);
        interface::generate(&mut c, self);
        class::generate(&mut c, self);
        c.buffer
    }

    fn data_ty(&self, ty: &Type) -> fmt!(type) {
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

            Type::Complex(path) => f.write_str(&self.symbol.class_name(path)),

            Type::List { ty, .. } | Type::Array { ty, .. } => {
                f.write_fmt(args!("Array<{}>", self.data_ty(ty)))
            }
            Type::Map { ty, .. } => f.write_fmt(args!(
                "Map<{}, {}>",
                self.data_ty(&ty.0),
                self.data_ty(&ty.1)
            )),

            Type::Option(ty) => f.write_fmt(args!("{} | undefined", self.data_ty(ty))),
            Type::Tuple(_) | Type::Result(_) | Type::Char | Type::U128 | Type::I128 => {
                unimplemented!()
            }
        })
    }

    fn write_object_tys<'a, I>(&'a self, c: &mut CodeWriter, sep: char, fields: I)
    where
        I: Iterator<Item = (&'a Ident, &'a Type)>,
    {
        for (name, ty) in fields {
            if let Some(ty) = ty.optional() {
                c.line(args!("{name}?: {}{sep}", self.data_ty(ty)))
            } else {
                c.line(args!("{name}: {}{sep}", self.data_ty(ty)))
            }
        }
    }

    fn lipi_ty(&self, ty: &Type) -> fmt!(type) {
        fmt(|f| match ty {
            Type::U8 => f.write_str("s.U8"),
            Type::I8 => f.write_str("s.I8"),
            Type::F32 => f.write_str("s.F32"),
            Type::F64 => f.write_str("s.F64"),

            Type::U16 | Type::U32 | Type::U64 => f.write_str("s.UInt"),
            Type::I16 | Type::I32 | Type::I64 => f.write_str("s.Int"),

            Type::Bool => f.write_str("s.Bool"),
            Type::String => f.write_str("s.Str"),

            Type::Option(ty) => f.write_fmt(args!("s.Option({})", self.lipi_ty(ty))),

            Type::Complex(path) => {
                f.write_fmt(args!("s.Field({}.encoder)", self.symbol.class_name(path)))
            }

            Type::Tuple(_) | Type::Result(_) | Type::Char | Type::U128 | Type::I128 => {
                unimplemented!()
            }
            _ => unimplemented!(),
        })
    }
}

impl SymbolTrie {
    fn class_name(&self, path: &str) -> String {
        let mut w: Vec<_> = self
            .shortest_symbol(path)
            .flat_map(|part| {
                let mut chars = part.chars();
                let ch = chars.next()?.to_ascii_uppercase();
                let rest = chars.as_str();

                let mut s = String::with_capacity(1 + rest.len());
                s.push(ch);
                s.push_str(rest);
                Some(s)
            })
            .collect();

        w.reverse();
        w.join("")
    }
}
