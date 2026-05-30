mod code_writer;
mod symbol_trie;
mod utils;

pub mod typescript;
pub use setu_type_info as type_info;

pub use code_writer::CodeWriter;

use symbol_trie::SymbolTrie;
use type_info::TypeInfo;

pub struct Context {
    pub info: TypeInfo,
    pub symbol: SymbolTrie,
}

impl Context {
    pub fn new(info: TypeInfo) -> Self {
        let symbol = SymbolTrie::from(&info);
        Self { info, symbol }
    }
}
