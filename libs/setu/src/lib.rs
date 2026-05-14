pub mod transport;

mod output;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

pub use output::Output;

pub trait Application {
    fn execute(id: u32, req: transport::http::HttpRequest, res: transport::http::HttpResponse);
}
