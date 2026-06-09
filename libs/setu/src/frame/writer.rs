use bytes::Bytes;
use lipi::Encode;

use crate::frame::{FrameHeader, LenBE};
use crate::transport::http::{HttpResponse, HttpWriter};
use crate::{Status, Trailer};

impl HttpResponse {
    pub fn create_setu_stream(mut self) -> Result<FrameEncoder, h2::Error> {
        self.add_setu_content_type_header();
        Ok(FrameEncoder {
            stream: self.create_stream()?,
        })
    }
}

pub struct FrameEncoder {
    pub stream: HttpWriter,
}

impl FrameEncoder {
    pub fn send_error(mut self, status: Status, reason: String) -> Result<(), h2::Error> {
        debug_assert!(status != Status::Ok);

        let Ok(msg) = Trailer::from(reason).to_bytes() else {
            return Ok(());
        };

        self.stream
            .write_unbound(encode_header(Some(status), &msg))?;

        self.stream.end_write_unbound(msg)
    }

    pub async fn send(&mut self, msg: Vec<u8>) -> Result<(), h2::Error> {
        self.stream.write_unbound(encode_header(None, &msg))?;
        self.stream.write(msg).await
    }

    pub fn end(mut self, msg: Vec<u8>) -> Result<(), h2::Error> {
        self.stream
            .write_unbound(encode_header(Some(Status::Ok), &msg))?;

        self.stream.end_write_unbound(msg)
    }
}

pub fn encode_header(status: Option<Status>, msg: &[u8]) -> Bytes {
    let len = LenBE::new(msg.len());

    let mut frame = Vec::with_capacity(1 + len.size as usize);
    frame.push(FrameHeader::new(status, len.size).encode());
    frame.extend_from_slice(&len);

    Bytes::from(frame.into_boxed_slice())
}
