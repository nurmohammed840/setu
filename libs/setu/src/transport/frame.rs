use bytes::{Buf, Bytes};

use crate::{Result, transport::http::HttpRequest};

struct Frame {
    is_compressed: bool,
    frame_data: FrameData,
}

pub(crate) enum FrameData {
    Message(Data),
    Trailer(Data),
}

pub(crate) enum Data {
    Bytes(Bytes),
    Buf(Vec<u8>),
}

struct FrameDecoder {
    curr: bytes::Bytes,
    is_eof: bool,
}

impl FrameDecoder {
    /// write code that decode DataFrame;
    /// Each frame starts with a single header byte containing:
    ///
    /// ```text
    /// 000L_LLFC
    ///
    /// `LLL` = payload length
    /// `F` = frame type (`1` = Trailer, `0` = Message)
    /// `C` = compressed flag
    /// ```
    async fn parse_frame(&mut self, req: &mut HttpRequest) -> Result<Frame> {
        let header = FrameHeader::parse(self.get_byte(req).await?)?;
        let len = self.parse_len_be(header.len_size, req).await?;

        todo!()
    }

    async fn get_byte<'a>(&'a mut self, req: &mut HttpRequest) -> Result<u8> {
        loop {
            match self.curr.try_get_u8() {
                Ok(byte) => return Ok(byte),
                Err(_) => {
                    self.curr = fetch_data(req).await?;
                }
            }
        }
    }

    async fn parse_len_be(&mut self, size: u8, req: &mut HttpRequest) -> Result<usize> {
        let mut len = 0usize;
        for _ in 0..size {
            len = (len << 8) | self.get_byte(req).await? as usize;
        }
        Ok(len)
    }

    async fn parse_data(&mut self, len: usize, req: &mut HttpRequest) -> Result<Data> {
        if len <= self.curr.len() {
            let bytes = self.curr.split_to(len);
            return Ok(Data::Bytes(bytes));
        }

        let buf = Vec::with_capacity(len);

        Ok(Data::Buf(buf))
    }
}

async fn fetch_data(req: &mut HttpRequest) -> Result<bytes::Bytes> {
    match req.body.data().await {
        Some(data) => Ok(data?),
        None => Err("unexpected end of message".into()),
    }
}

struct FrameHeader {
    is_compressed: bool,
    is_trailer: bool,
    len_size: u8,
}

impl FrameHeader {
    fn parse(byte: u8) -> Result<FrameHeader, &'static str> {
        let is_compressed = (byte & 0b1) == 0b1;
        let is_trailer = (byte & 0b10) == 0b10;
        let len_size = byte >> 2;

        if len_size > 4 {
            return Err("invalid length size");
        }

        Ok(FrameHeader {
            is_compressed,
            is_trailer,
            len_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_header() {
        let h = FrameHeader::parse(0b100_11).unwrap();
        assert!(h.is_compressed);
        assert!(h.is_trailer);
        assert_eq!(h.len_size, 4);

        assert!(FrameHeader::parse(0b101_00).is_err());
    }
}
