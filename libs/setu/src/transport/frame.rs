use crate::Result;
use bytes::{Buf, Bytes};
use futures::{Stream, StreamExt};

type StreamData = Result<Bytes, h2::Error>;

#[derive(Debug)]
pub struct Frame<T> {
    pub is_compressed: bool,
    pub data: T,
}

#[derive(Debug)]
pub enum FrameData {
    Message(Data),
    Trailer(Data),
}

#[derive(Debug)]
pub enum Data {
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
    trailer: Option<Frame<Data>>,
}

impl FrameDecoderStream {
    pub fn is_end(&self) -> bool {
        self.trailer.is_some()
    }

    pub async fn next<I>(&mut self, stream: &mut I) -> Result<Option<Frame<Data>>>
    where
        I: Stream<Item = StreamData> + Unpin,
    {
        if self.is_end() {
            return Ok(None);
        }

        let frame = self.decoder.parse_frame(stream).await?;

        Ok(match frame.data {
            FrameData::Message(data) => Some(Frame {
                is_compressed: frame.is_compressed,
                data,
            }),
            FrameData::Trailer(data) => {
                self.trailer = Some(Frame {
                    is_compressed: frame.is_compressed,
                    data,
                });
                None
            }
        })
    }

    pub fn trailer(&self) -> Option<&Frame<Data>> {
        self.trailer.as_ref()
    }
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
    pub async fn parse_frame<I>(&mut self, stream: &mut I) -> Result<Frame<FrameData>>
    where
        I: Stream<Item = StreamData> + Unpin,
    {
        let header = FrameHeader::parse(self.read_byte(stream).await?)?;
        let len = self.parse_len_big_endian(stream, header.len_size).await?;

        let data = self.parse_data(stream, len).await?;

        Ok(Frame {
            is_compressed: header.is_compressed,
            data: if header.is_trailer {
                FrameData::Trailer(data)
            } else {
                FrameData::Message(data)
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

    async fn parse_data<I>(&mut self, stream: &mut I, len: usize) -> Result<Data>
    where
        I: Stream<Item = StreamData> + Unpin,
    {
        let data = self.read_data(stream).await?;

        if len <= data.len() {
            return Ok(Data::Bytes(data.split_to(len)));
        }

        let mut buf = Vec::with_capacity(len);

        while buf.len() < len {
            let data = self.read_data(stream).await?;

            let remaining = len - buf.len();
            let take = remaining.min(data.len());
            buf.extend_from_slice(&data.split_to(take));
        }

        Ok(Data::Buf(buf))
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

async fn fetch_data<I>(stream: &mut I) -> Result<bytes::Bytes>
where
    I: Stream<Item = StreamData> + Unpin,
{
    match stream.next().await {
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

    fn create_stream(s: &[&'static [u8]]) -> impl Stream<Item = StreamData> + Unpin {
        futures::stream::iter(s.iter().copied().map(Bytes::from_static).map(Ok))
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

        let data = de.parse_data(&mut stream, 2).await?;
        assert!(matches!(data, Data::Bytes(d) if &d[..] == [1, 2]));

        let data = de.parse_data(&mut stream, 3).await?;
        assert!(matches!(data, Data::Buf(d) if &d[..] == [3, 4, 5]));
        Ok(())
    }

    #[nio::test]
    async fn test_read_data_eof() -> Result<()> {
        let mut stream = create_stream(&[&[1], &[2]]);
        let mut de = FrameDecoder::default();
        assert!(de.parse_data(&mut stream, 3).await.is_err());
        Ok(())
    }
}
