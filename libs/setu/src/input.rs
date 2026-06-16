#![allow(unused)]
use crate::{
    Result, Status,
    frame::{Frame, FrameDecoder, RawBytes},
    transport::http::HttpBody,
};
use lipi::{Decode, DecodeOwned, decoder::FieldDecoderOwned};
use std::marker::PhantomData;

struct Stream<T, R = ()> {
    pub body: HttpBody,
    end: Option<R>,
    frame_decoder: FrameDecoder,
    data: PhantomData<T>,
}

impl<T, R> Stream<T, R>
where
    T: DecodeOwned,
    R: DecodeOwned,
{
    fn new(frame_decoder: FrameDecoder, body: HttpBody) -> Self {
        Self {
            body,
            end: None,
            frame_decoder,
            data: PhantomData,
        }
    }

    pub async fn next(&mut self) -> Result<Option<T>> {
        if self.end.is_some() {
            return Ok(None);
        }
        match self.frame_decoder.parse(&mut self.body).await?.data {
            Frame::Message(bytes) => T::decode(&mut &*bytes).map(Some),
            Frame::Trailer { status, bytes } => {
                if status != Status::Ok {
                    return Err(format!("unexpected status: {status:?}").into());
                }
                self.end = R::decode(&mut &*bytes).map(Some)?;
                return Ok(None);
            }
        }
    }

    pub fn end(&mut self) -> Option<R> {
        self.end.take()
    }
}

// =======================================================================

pub trait Input: Sized {
    async fn unmarshal(body: HttpBody) -> Result<Self>;
}

impl Input for () {
    async fn unmarshal(_: HttpBody) -> Result<Self> {
        Ok(())
    }
}

// =======================================================================

macro_rules! tuples {
    [$($name:tt : $idx:tt)*] => {
        impl<$($name,)*> Input for ($($name,)*)
        where
            $($name: FieldDecoderOwned,)*
        {
            async fn unmarshal(body: HttpBody) -> Result<Self> {
                let mut bytes = decode_last_msg(body).await?;
                Self::decode(&mut &*bytes)
            }
        }

        impl<$($name,)* T, R> Input for ($($name,)* Stream<T, R>)
        where
            $($name: FieldDecoderOwned,)*
            T: DecodeOwned,
            R: DecodeOwned,
        {
            async fn unmarshal(mut body: HttpBody) -> Result<Self> {
                let mut frame_decoder = FrameDecoder::default();
        
                let mut bytes = decode_first_msg(&mut frame_decoder, &mut body).await?;
                let args = <($($name,)*)>::decode(&mut &*bytes)?;
        
                Ok(( $(args.$idx,)* Stream::new(frame_decoder, body)))
            }
        }
    }
}

tuples! { T0:0 }
tuples! { T0:0 T1:1 }
tuples! { T0:0 T1:1 T2:2 }
tuples! { T0:0 T1:1 T2:2 T3:3 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 T13:13 }
tuples! { T0:0 T1:1 T2:2 T3:3 T4:4 T5:5 T6:6 T7:7 T8:8 T9:9 T10:10 T11:11 T12:12 T13:13 T14:14 }

async fn decode_first_msg(
    frame_decoder: &mut FrameDecoder,
    stream: &mut HttpBody,
) -> Result<RawBytes> {
    let bytes = frame_decoder
        .parse(stream)
        .await?
        .data
        .message()
        .ok_or("expected message frame")?;

    Ok(bytes)
}

async fn decode_last_msg(mut stream: HttpBody) -> Result<RawBytes> {
    let mut frame_decoder = FrameDecoder::default();

    let (status, bytes) = frame_decoder
        .parse(&mut stream)
        .await?
        .data
        .trailer()
        .ok_or("expected trailer frame")?;

    if status != Status::Ok {
        return Err(format!("unexpected status: {status:?}").into());
    }
    Ok(bytes)
}
