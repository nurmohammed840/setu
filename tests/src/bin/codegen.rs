use setu_codegen::{Context, type_info::TypeInfo, typescript};
use std::path::PathBuf;
use test_suite::TestSuite;

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let info = Context::new(TypeInfo::from::<TestSuite>());

    typescript::Config::out_dir(dir.join("build/typescript"))
        .generate(&info)
        .unwrap();
}
