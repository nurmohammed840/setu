pub(crate) mod frame;

mod context;
mod status_code;
mod timeout;
mod trailer;
mod utils;

#[doc(hidden)]
pub mod __private;
pub mod transport;
pub use context::Context;
pub use status_code::Status;
pub use timeout::Timeout;
pub use trailer::Trailer;

mod output;
pub use setu_macros::*;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

pub use output::Output;

pub trait Application {
    fn execute(id: u32, ctx: transport::http::HttpContext);
}

pub struct SSE<S>(pub S);
