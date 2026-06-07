mod writer;

use crate::{Result, Status};
use bytes::{Buf, Bytes};
use futures::{Stream, StreamExt};
pub use writer::FrameEncoder;

type StreamData = Result<Bytes, h2::Error>;

#[derive(Debug)]
pub struct MaybeCompressed<T> {
    #[allow(unused)]
    pub is_compressed: bool,
    pub data: T,
}

#[derive(Debug)]
pub enum Frame {
    Message(RawBytes),
    Trailer { status: Status, bytes: RawBytes },
}

impl Frame {
    pub fn message(self) -> Option<RawBytes> {
        match self {
            Frame::Message(data) => Some(data),
            Frame::Trailer { .. } => None,
        }
    }

    pub fn trailer(self) -> Option<(Status, RawBytes)> {
        match self {
            Frame::Trailer { status, bytes } => Some((status, bytes)),
            Frame::Message(_) => None,
        }
    }
}

#[derive(Debug)]
pub enum RawBytes {
    Bytes(Bytes),
    Buf(Vec<u8>),
}

#[derive(Debug, Default)]
pub struct FrameDecoder {
    data: bytes::Bytes,
}

#[derive(Debug, Default)]
pub struct FrameDecoderStream {
    decoder: FrameDecoder,
    pub trailer: Option<(Status, MaybeCompressed<RawBytes>)>,
}

#[allow(unused)]
impl FrameDecoderStream {
    pub fn is_end(&self) -> bool {
        self.trailer.is_some()
    }

    pub async fn next<I>(&mut self, stream: &mut I) -> Result<Option<MaybeCompressed<RawBytes>>>
    where
        I: Stream<Item = StreamData> + Unpin,
    {
        if self.is_end() {
            return Ok(None);
        }

        let frame = self.decoder.parse_frame(stream).await?;

        Ok(match frame.data {
            Frame::Message(data) => Some(MaybeCompressed {
                is_compressed: frame.is_compressed,
                data,
            }),
            Frame::Trailer { status, bytes } => {
                let data = MaybeCompressed {
                    is_compressed: frame.is_compressed,
                    data: bytes,
                };
                self.trailer = Some((status, data));
                None
            }
        })
    }
}

impl FrameDecoder {
    pub async fn parse_frame<I>(&mut self, stream: &mut I) -> Result<MaybeCompressed<Frame>>
    where
        I: Stream<Item = StreamData> + Unpin,
    {
        let header = FrameHeader::parse(self.read_byte(stream).await?);
        let len = self.parse_len_big_endian(stream, header.len_size).await?;

        // TODO: `len` should less then 16MB

        let bytes = self.read_bytes(stream, len).await?;

        Ok(MaybeCompressed {
            is_compressed: header.is_compressed,
            data: if header.is_trailer {
                Frame::Trailer {
                    status: header.code.into(),
                    bytes,
                }
            } else {
                Frame::Message(bytes)
            },
        })
    }

    async fn parse_len_big_endian<I>(&mut self, stream: &mut I, size: u8) -> Result<usize>
    where
        I: Stream<Item = StreamData> + Unpin,
    {
        let mut len = 0usize;
        for _ in 0..size {
            len = (len << 8) | self.read_byte(stream).await? as usize;
        }
        Ok(len)
    }

    async fn read_bytes<I>(&mut self, stream: &mut I, len: usize) -> Result<RawBytes>
    where
        I: Stream<Item = StreamData> + Unpin,
    {
        if len == 0 {
            return Ok(RawBytes::Buf(Vec::new()));
        }

        let data = self.read_data(stream).await?;

        if len <= data.len() {
            return Ok(RawBytes::Bytes(data.split_to(len)));
        }

        let mut buf = Vec::with_capacity(len);

        while buf.len() < len {
            let data = self.read_data(stream).await?;

            let remaining = len - buf.len();
            let take = remaining.min(data.len());
            buf.extend_from_slice(&data.split_to(take));
        }

        Ok(RawBytes::Buf(buf))
    }

    async fn read_byte<I>(&mut self, stream: &mut I) -> Result<u8>
    where
        I: Stream<Item = StreamData> + Unpin,
    {
        loop {
            match self.data.try_get_u8() {
                Ok(byte) => return Ok(byte),
                Err(_) => {
                    self.data = fetch_data(stream).await?;
                }
            }
        }
    }

    async fn read_data<I>(&mut self, stream: &mut I) -> Result<&mut Bytes>
    where
        I: Stream<Item = StreamData> + Unpin,
    {
        while self.data.is_empty() {
            self.data = fetch_data(stream).await?;
        }
        Ok(&mut self.data)
    }
}

pub struct LenBE {
    buf: [u8; 4],
    size: u8,
}

impl std::ops::Deref for LenBE {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buf[(4 - self.size as usize)..]
    }
}

impl LenBE {
    #[inline]
    fn new(len: usize) -> Self {
        let buf = (len as u32).to_be_bytes();
        let size = if len <= 0x_FF {
            1
        } else if len <= 0x_FF_FF {
            2
        } else if len <= 0x_FF_FF_FF {
            3
        } else {
            debug_assert!(len <= 0x_FF_FF_FF_FF);
            4
        };

        Self { buf, size }
    }
}

async fn fetch_data<I>(stream: &mut I) -> Result<bytes::Bytes>
where
    I: Stream<Item = StreamData> + Unpin,
{
    match stream.next().await {
        Some(data) => Ok(data?),
        None => Err("unexpected end of message".into()),
    }
}

pub struct FrameHeader {
    // 1 bit
    pub is_compressed: bool,
    // 1 bit
    pub is_trailer: bool,
    // 2 bit
    pub len_size: u8,
    // 4 bit
    pub code: u8, // Must be `0` if `!is_trailer`
}

impl FrameHeader {
    pub const fn new(trailer_status: Option<Status>, len_size: u8) -> FrameHeader {
        FrameHeader {
            is_compressed: false,
            is_trailer: trailer_status.is_some(),
            len_size: len_size - 1,
            code: match trailer_status {
                Some(status) => status.code(),
                None => 0,
            },
        }
    }

    #[inline]
    pub const fn encode(self) -> u8 {
        (self.code << 4)
            | (self.len_size << 2)
            | ((self.is_trailer as u8) << 1)
            | (self.is_compressed as u8)
    }

    #[inline]
    pub const fn parse(byte: u8) -> FrameHeader {
        FrameHeader {
            is_compressed: (byte & 0b1) == 0b1,
            is_trailer: (byte & 0b10) == 0b10,
            len_size: ((byte >> 2) & 0b11) + 1,
            code: byte >> 4,
        }
    }
}

impl std::ops::Deref for RawBytes {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            RawBytes::Bytes(bytes) => bytes,
            RawBytes::Buf(buf) => buf,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_header() {
        let raw = FrameHeader::new(Some(Status::Cancelled), 4).encode();
        assert_eq!(raw, 0b_1_11_1_0);

        let h = FrameHeader::parse(raw);
        assert!(!h.is_compressed);
        assert!(h.is_trailer);
        assert_eq!(h.len_size, 4);
        assert_eq!(h.code, 1);
    }

    fn create_stream(s: &[&'static [u8]]) -> impl Stream<Item = StreamData> + Unpin {
        futures::stream::iter(s.iter().copied().map(Bytes::from).map(Ok))
    }

    #[nio::test]
    async fn test_read_byte() -> Result<()> {
        let mut stream = create_stream(&[&[], &[4, 2], &[], &[42]]);
        let mut de = FrameDecoder::default();
        assert_eq!(de.read_byte(&mut stream).await?, 4);
        assert_eq!(de.read_byte(&mut stream).await?, 2);
        assert_eq!(de.read_byte(&mut stream).await?, 42);
        Ok(())
    }

    #[nio::test]
    async fn test_read_data() -> Result<()> {
        let mut stream = create_stream(&[&[], &[1, 2, 3], &[], &[4], &[], &[5]]);
        let mut de = FrameDecoder::default();

        let data = de.read_bytes(&mut stream, 2).await?;
        assert!(matches!(data, RawBytes::Bytes(d) if *d == [1, 2]));

        let data = de.read_bytes(&mut stream, 3).await?;
        assert!(matches!(data, RawBytes::Buf(d) if d == [3, 4, 5]));
        Ok(())
    }

    #[nio::test]
    async fn test_read_data_eof() -> Result<()> {
        let mut stream = create_stream(&[&[1], &[2]]);
        let mut de = FrameDecoder::default();
        assert!(de.read_bytes(&mut stream, 3).await.is_err());
        Ok(())
    }

    #[test]
    fn test_len_be() {
        assert_eq!(&*LenBE::new(0x1234), [0x12, 0x34]);
        assert_eq!(&*LenBE::new(0x123456), [0x12, 0x34, 0x56]);
        assert_eq!(&*LenBE::new(0x12345678), [0x12, 0x34, 0x56, 0x78]);
    }

    #[nio::test]
    async fn test_encode_frame() -> Result<()> {
        let mut stream = create_stream(&[&[], &[0, 2], &[54], &[55], &[2], &[0], &[]]);

        let mut de = FrameDecoder::default();

        assert_eq!(
            de.parse_frame(&mut stream)
                .await?
                .data
                .message()
                .unwrap()
                .as_ref(),
            [54, 55]
        );

        let (status, data) = de.parse_frame(&mut stream).await?.data.trailer().unwrap();
        assert_eq!(status, Status::Ok);
        assert!(data.is_empty());
        Ok(())
    }
}
