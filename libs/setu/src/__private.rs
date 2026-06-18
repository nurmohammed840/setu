pub use async_gen;
pub use lipi;
pub use setu_type_info;
pub use setu_type_info::type_id;

use crate::transport::http::HttpContext;

pub fn unknown_rpc(id: u32, mut ctx: HttpContext) {
    *ctx.res.status_mut() = http::StatusCode::NOT_IMPLEMENTED;
    let _ = ctx.res.write_unbound(format!("Unknown call id {id}"));
}

pub const fn __fn_args_count<F, Args>(_: &F) -> u8
where
    F: std_lib::FnOnce<Args>,
    Args: crate::input::Input,
{
    Args::LEN
}
