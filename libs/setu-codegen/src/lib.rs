mod code_writer;
mod path_of_complex_type;
mod symbol_trie;
mod utils;

pub mod typescript;
pub use setu_type_info as type_info;

pub use code_writer::CodeWriter;

use symbol_trie::SymbolTrie;
use type_info::TypeInfo;

use crate::path_of_complex_type::PathOfComplexTypes;

pub struct Context {
    pub info: TypeInfo,
    pub symbol: SymbolTrie,

    obj_that_needed_encoder: PathOfComplexTypes,
    obj_that_needed_decoder: PathOfComplexTypes,
}

impl Context {
    pub fn new(info: TypeInfo) -> Self {
        Self {
            obj_that_needed_encoder: PathOfComplexTypes::input_types(&info),
            obj_that_needed_decoder: PathOfComplexTypes::output_types(&info),
            symbol: SymbolTrie::from(&info),
            info,
        }
    }

    pub(crate) fn is_encoder_needed(&self, path: &str) -> bool {
        self.obj_that_needed_encoder.contains(path)
    }

    pub(crate) fn is_decoder_needed(&self, path: &str) -> bool {
        self.obj_that_needed_decoder.contains(path)
    }
}
