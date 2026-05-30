use type_id::ComplexData;

use crate::{CodeWriter, Context};

pub fn generate(c: &mut CodeWriter, ctx: &Context) {
    for ComplexData { path, .. } in ctx.info.registry.values() {
        let name = ctx.symbol.class_name(path);
        c.block(format_args!("export class {name}"), |_c| {});
    }
}
