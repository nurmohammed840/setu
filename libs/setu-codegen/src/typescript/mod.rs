pub mod interface;

use setu_type_info::TypeInfo;
use std::{fs, io, path::PathBuf};

use crate::{CodeWriter, utils::copy_dir};

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
    pub fn generate(&self, info: &TypeInfo) -> io::Result<()> {
        fs::create_dir_all(&self.out_dir)?;

        let lib = self.out_dir.join("lib");
        if !lib.exists() {
            let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("clients/typescript");

            copy_dir(&src, &lib, 1, |file| {
                file.extension().is_some_and(|ext| ext == "ts")
            })?;
        }

        let code = generate_code(info);
        fs::write(self.out_dir.join("mod.ts"), code)
    }
}

static TS_PRELUDE: &str = r#"
import * as $ from "./lib/mod.ts";
export const $etu = { RPC: $.RPC };
"#;

pub fn generate_code(info: &TypeInfo) -> String {
    let mut c = CodeWriter::new();
    c.write_line(TS_PRELUDE);
    interface::generate(&mut c, info);
    c.buffer
}
