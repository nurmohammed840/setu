use bytes::Bytes;
use h2::RecvStream;
use http::Request;
use std::{
    future::poll_fn,
    task::{Context, Poll},
};

pub struct HttpRequest {
    pub meta: http::request::Parts,
    pub body: HttpBody,
}

impl From<Request<RecvStream>> for HttpRequest {
    fn from(req: Request<RecvStream>) -> Self {
        let (header, reader) = req.into_parts();
        Self {
            meta: header,
            body: HttpBody { reader },
        }
    }
}

pub struct HttpBody {
    reader: RecvStream,
}

impl HttpBody {
    #[inline]
    /// Retrieve the next chunk of data from the request body.
    pub fn data(&mut self) -> impl Future<Output = Option<Result<Bytes, h2::Error>>> {
        poll_fn(|cx| self.poll_data(cx))
    }

    #[inline]
    pub fn poll_data(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, h2::Error>>> {
        self.reader.poll_data(cx).map(|out| match out {
            Some(Ok(data)) => {
                let data = self
                    .reader
                    .flow_control()
                    .release_capacity(data.len())
                    .map(|_| data);

                Some(data)
            }
            v => v,
        })
    }
}

impl std::ops::Deref for HttpBody {
    type Target = RecvStream;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl std::ops::DerefMut for HttpBody {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}
