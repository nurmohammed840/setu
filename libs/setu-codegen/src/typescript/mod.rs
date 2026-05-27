use setu_type_info::TypeInfo;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub output: PathBuf,
    pub preserve_import_extension: bool,
}

impl Config {
    pub fn new(output: PathBuf) -> Self {
        Self {
            output,
            preserve_import_extension: false,
        }
    }

    pub fn preserve_import_extension(mut self) -> Self {
        self.preserve_import_extension = true;
        self
    }
}

impl Config {
    pub fn generate(&self, info: &TypeInfo) {}
}
