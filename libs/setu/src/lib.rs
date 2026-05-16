pub(crate) mod frame;

mod utils;
mod status_code;
mod trailer;
mod timeout;

pub mod transport;
pub use status_code::Status;
pub use trailer::Trailer;
pub use timeout::Timeout;

mod output;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

pub use output::Output;

pub trait Application {
    fn execute(id: u32, req: transport::http::HttpRequest, res: transport::http::HttpResponse);
}
