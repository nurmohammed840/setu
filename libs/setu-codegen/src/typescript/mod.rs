pub mod function;
pub mod interface;

use std::format_args as args;
use std::{fs, io, path::PathBuf};
use type_id::{Ident, PathIdent, Type};

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

static TS_PRELUDE: &str = r#"// AUTO-GENERATED FILE. DO NOT EDIT.
import * as $ from "./lib/mod.ts";
export const $etu = { RPC: $.RPC };
"#;

impl Context {
    pub fn generate_typescript_code(&self) -> String {
        let mut c = CodeWriter::new();
        c.buffer.push_str(TS_PRELUDE);
        interface::generate(&mut c, self);
        function::generate(&mut c, self);
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

            Type::Complex(path) => f.write_str(&self.symbol.interface_name(path)),

            Type::List { ty, .. } | Type::Array { ty, .. } => match ty.as_ref() {
                Type::U8 => f.write_str("Uint8Array"),
                Type::U16 => f.write_str("Uint16Array"),
                Type::U32 => f.write_str("Uint32Array"),
                Type::U64 => f.write_str("BigUint64Array"),

                Type::I8 => f.write_str("Int8Array"),
                Type::I16 => f.write_str("Int16Array"),
                Type::I32 => f.write_str("Int32Array"),
                Type::I64 => f.write_str("BigInt64Array"),

                Type::F32 => f.write_str("Float32Array"),
                Type::F64 => f.write_str("Float64Array"),

                _ => f.write_fmt(args!("Array<{}>", self.data_ty(ty))),
            },
            Type::Map { ty, .. } => f.write_fmt(args!(
                "Map<{}, {}>",
                self.data_ty(&ty.0),
                self.data_ty(&ty.1)
            )),

            Type::Tuple(tys) => {
                f.write_str("[")?;
                for ty in tys {
                    f.write_fmt(args!("{}, ", self.data_ty(ty)))?;
                }
                f.write_str("]")
            }

            Type::Option(ty) => f.write_fmt(args!("{} | undefined", self.data_ty(ty))),
            Type::Result(_) | Type::Char | Type::U128 | Type::I128 => {
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

    fn serde_ty(&self, ty: &Type, codex: &str) -> fmt!(type) {
        fmt(move |f| match ty {
            Type::U8 => f.write_str("_.U8"),
            Type::I8 => f.write_str("_.I8"),

            Type::F32 => f.write_str("_.F32"),
            Type::F64 => f.write_str("_.F64"),

            Type::U16 => f.write_str("_.U16"),
            Type::U32 => f.write_str("_.U32"),
            Type::U64 => f.write_str("_.U64"),

            Type::I16 => f.write_str("_.I16"),
            Type::I32 => f.write_str("_.I32"),
            Type::I64 => f.write_str("_.I64"),

            Type::Bool => f.write_str("_.Bool"),
            Type::String => f.write_str("_.Str"),

            Type::Array { ty, .. } | Type::List { ty, .. } => match ty.as_ref() {
                Type::U8 => f.write_str("_.ListU8"),
                Type::I8 => f.write_str("_.ListI8"),

                Type::F32 => f.write_str("_.ListF32"),
                Type::F64 => f.write_str("_.ListF64"),

                Type::U16 => f.write_str("_.ListU16"),
                Type::U32 => f.write_str("_.ListU32"),
                Type::U64 => f.write_str("_.ListU64"),

                Type::I16 => f.write_str("_.ListI16"),
                Type::I32 => f.write_str("_.ListI32"),
                Type::I64 => f.write_str("_.ListI64"),

                Type::Bool => f.write_str("_.ListBool"),
                ty => f.write_fmt(args!("_.List({})", self.serde_ty(ty, codex))),
            },
            Type::Complex(path) => {
                f.write_fmt(args!("{codex}.{}", self.symbol.interface_name(path)))
            }
            Type::Option(ty) => f.write_fmt(args!("{}", self.serde_ty(ty, codex))),
            Type::Map { ty, .. } => f.write_fmt(args!(
                "_.Table({}, {})",
                self.serde_ty(&ty.0, codex),
                self.serde_ty(&ty.1, codex)
            )),
            ty => unimplemented!("{ty:?}"),
        })
    }

    fn struct_encoder<'a, I>(&'a self, c: &mut CodeWriter, fields: I)
    where
        I: Iterator<Item = (&'a str, &'a Type, u32)>,
        I: ExactSizeIterator,
    {
        if fields.len() == 0 {
            return c.line("$.lipi.StructEncoder(_, []);");
        }
        c.line("$.lipi.StructEncoder(_, [");
        c.scope(|c| {
            for (name, ty, key) in fields {
                let decoder = self.serde_ty(ty, "$E");
                c.line(args!("[{key}, z.{name}, {decoder}],"));
            }
        });
        c.line("]);");
    }
}

mod cached {
    use crate::utils::LocalCachedTable;
    use std::rc::Rc;
    use type_id::PathIdent;

    thread_local! {
       pub static SYMBOL: LocalCachedTable<PathIdent, str> = LocalCachedTable::new();
    }

    pub fn get_symbol(path: &PathIdent, init: impl FnOnce() -> String) -> Rc<str> {
        SYMBOL.with(|cached| cached.get_or_insert_with(path.clone(), init))
    }
}

impl SymbolTrie {
    fn interface_name(&self, path: &PathIdent) -> std::rc::Rc<str> {
        cached::get_symbol(path, || {
            self.shortest_symbol(path)
                .flat_map(|part| {
                    let mut chars = part.chars();

                    let ch = chars.next()?.to_ascii_uppercase();
                    let rest = chars.as_str();

                    Some(format!("{ch}{rest}"))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        })
    }
}
