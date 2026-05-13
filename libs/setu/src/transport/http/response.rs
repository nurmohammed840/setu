use bytes::Bytes;
use h2::server::SendResponse;

/// Represents an HTTP response object.
#[derive(Debug)]
pub struct HttpResponse {
    /// Responsible for sending the HTTP response body
    pub writer: SendResponse<Bytes>,
}

impl From<SendResponse<Bytes>> for HttpResponse {
    fn from(writer: SendResponse<Bytes>) -> Self {
        Self { writer }
    }
}
