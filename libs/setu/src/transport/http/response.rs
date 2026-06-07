use std::{
    future::poll_fn,
    ops::{Deref, DerefMut},
    task::{Context, Poll},
};

use bytes::Bytes;
use h2::server::SendResponse;

type Result<T, E = h2::Error> = std::result::Result<T, E>;

/// Represents an HTTP response object.
#[derive(Debug)]
pub struct HttpResponse {
    // /// Represens status code of HTTP response.
    // pub status: http::StatusCode,
    // /// Represens headers of HTTP response.
    // pub headers: Option<http::HeaderMap>,
    response: http::Response<()>,

    /// Responsible for sending the HTTP response body
    pub writer: SendResponse<Bytes>,
}

impl From<SendResponse<Bytes>> for HttpResponse {
    fn from(writer: SendResponse<Bytes>) -> Self {
        Self {
            writer,
            response: http::Response::new(()),
        }
    }
}

impl HttpResponse {
    /// Returns the stream ID of the response stream.
    ///
    /// # Panics
    ///
    /// If the lock on the stream store has been poisoned.
    #[inline]
    pub fn stream_id(&self) -> h2::StreamId {
        self.writer.stream_id()
    }

    /// Send the response headers.
    #[inline]
    pub fn send_headers(mut self) -> Result<()> {
        self.writer.send_response(self.response, true)?;
        Ok(())
    }

    /// This method is used to obtain a [Responder] that can be used to send the response body.
    #[inline]
    pub fn create_stream(mut self) -> Result<HttpWriter> {
        Ok(HttpWriter {
            stream: self.writer.send_response(self.response, false)?,
        })
    }

    /// Sends response data to the remote peer.
    #[inline]
    pub async fn write(self, bytes: impl Into<Bytes>) -> Result<()> {
        self.create_stream()?.end_write(bytes).await
    }

    #[inline]
    pub fn write_unbound(self, bytes: impl Into<Bytes>) -> Result<()> {
        self.create_stream()?.end_write_unbound(bytes)
    }

    #[inline]
    pub fn poll_reset(&mut self, cx: &mut Context<'_>) -> Poll<Result<h2::Reason, h2::Error>> {
        self.writer.poll_reset(cx)
    }

    #[inline]
    pub fn send_reset(&mut self, reason: h2::Reason) {
        self.writer.send_reset(reason)
    }
}

impl Deref for HttpResponse {
    type Target = http::Response<()>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl DerefMut for HttpResponse {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.response
    }
}

/// The [Responder] struct created from `Response::send_stream`
///
/// It is responsible for sending the HTTP response body.
pub struct HttpWriter {
    pub stream: h2::SendStream<Bytes>,
}

impl HttpWriter {
    #[inline]
    pub fn poll_reset(&mut self, cx: &mut Context<'_>) -> Poll<Result<h2::Reason, h2::Error>> {
        self.stream.poll_reset(cx)
    }

    #[inline]
    pub fn send_reset(&mut self, reason: h2::Reason) {
        self.stream.send_reset(reason)
    }

    pub async fn write_bytes(&mut self, mut bytes: Bytes, end: bool) -> Result<()> {
        loop {
            self.stream.reserve_capacity(bytes.len());
            match poll_fn(|cx| self.stream.poll_capacity(cx)).await {
                None => return Err(h2::Error::from(h2::Reason::CANCEL)),
                Some(cap) => {
                    let cap = cap?;
                    if bytes.len() <= cap {
                        return self.stream.send_data(bytes, end);
                    }
                    self.stream.send_data(bytes.split_to(cap), false)?;
                }
            };
        }
    }

    /// Sends a single data frame to the remote peer.
    pub async fn write(&mut self, bytes: impl Into<Bytes>) -> Result<()> {
        let bytes = bytes.into();
        if bytes.is_empty() {
            return Ok(());
        }
        self.write_bytes(bytes, false).await
    }

    /// Sends final chunk of data to the remote peer.
    pub async fn end_write(mut self, bytes: impl Into<Bytes>) -> Result<()> {
        let bytes = bytes.into();
        if bytes.is_empty() {
            return self.end();
        }
        self.write_bytes(bytes, true).await
    }

    /// The data is buffered and the capacity is implicitly requested. Once the
    /// capacity becomes available, the data is flushed to the connection.
    ///
    /// However, this buffering is unbounded. As such, sending large amounts of
    /// data without reserving capacity before hand could result in large
    /// amounts of data being buffered in memory.
    pub fn write_unbound(&mut self, bytes: impl Into<Bytes>) -> Result<()> {
        self.stream.send_data(bytes.into(), false)
    }

    /// Sends final chunk of data to the remote peer.
    pub fn end_write_unbound(mut self, bytes: impl Into<Bytes>) -> Result<()> {
        self.stream.send_data(bytes.into(), true)
    }

    /// Signals the end of writing the response body.
    #[inline]
    pub fn end(mut self) -> Result<()> {
        self.stream.send_data(Bytes::new(), true)
    }
}
