#![allow(warnings)]
use crate::{
    Result, Status,
    frame::{Frame, FrameDecoder, RawBytes},
    transport::http::HttpBody,
};
use lipi::{
    Decode,
    decoder::{FieldDecoder, FieldDecoderOwned, FieldInfoDecoder, Optional},
};
use std::marker::PhantomData;
use std::ops::ControlFlow;
use type_id::{Type, TypeId};

struct Stream<T, R = ()> {
    pub body: HttpBody,
    frame_decoder: FrameDecoder,
    data: PhantomData<(T, R)>,
}

impl<T: TypeId, R: TypeId> TypeId for Stream<T, R> {
    fn ty(r: &mut type_id::TypeRegistry) -> Type {
        Type::ControlFlow(Box::new(type_id::ControlFlowType {
            yield_ty: T::ty(r),
            return_ty: R::ty(r),
        }))
    }
}

impl<T, R> Stream<T, R>
where
    T: Optional,
    R: Optional,
    T::Value: FieldDecoderOwned,
    R::Value: FieldDecoderOwned,
{
    fn new(frame_decoder: FrameDecoder, body: HttpBody) -> Self {
        Self {
            body,
            frame_decoder,
            data: PhantomData,
        }
    }

    pub async fn next(&mut self) -> Result<ControlFlow<R, T>> {
        match self.frame_decoder.parse(&mut self.body).await?.data {
            Frame::Message(bytes) => decode_optional_field(&mut &*bytes).map(ControlFlow::Continue),
            Frame::Trailer { status, bytes } => {
                if status != Status::Ok {
                    return Err(format!("unexpected status: {status:?}").into());
                }
                decode_optional_field(&mut &*bytes).map(ControlFlow::Break)
            }
        }
    }
}

fn decode_optional_field<'de, T>(reader: &mut &'de [u8]) -> Result<T>
where
    T: Optional,
    T::Value: FieldDecoder<'de>,
{
    let mut val = None;
    let mut fd = FieldInfoDecoder::new(reader);
    if let Some((key, ty)) = fd.next_field_id_and_ty()? {
        assert_eq!(key, 0);
        val = fd.decode(ty, "tuple 0")?;
    }
    Ok(Optional::convert(val, "tuple 0")?)
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

impl<T, R> Input for (Stream<T, R>,)
where
    T: Optional,
    R: Optional,
    T::Value: FieldDecoderOwned,
    R::Value: FieldDecoderOwned,
{
    async fn unmarshal(mut body: HttpBody) -> Result<Self> {
        let mut frame_decoder = FrameDecoder::default();
        Ok((Stream::new(frame_decoder, body),))
    }
}

// =======================================================================

macro_rules! tuples {
    [$($name:tt : $idx:tt)*] => {
        impl<$($name,)*> Input for ($($name,)*)
        where
            $($name: Optional,)*
            $($name::Value: FieldDecoderOwned,)*
        {
            async fn unmarshal(body: HttpBody) -> Result<Self> {
                let bytes = decode_last_msg(body).await?;
                Self::decode(&mut &*bytes)
            }
        }

        impl<$($name,)* T, R> Input for ($($name,)* Stream<T, R>)
        where
            $($name: Optional,)*
            $($name::Value: FieldDecoderOwned,)*
            T: Optional,
            R: Optional,
            T::Value: FieldDecoderOwned,
            R::Value: FieldDecoderOwned,
        {
            async fn unmarshal(mut body: HttpBody) -> Result<Self> {
                let mut frame_decoder = FrameDecoder::default();

                let bytes = decode_first_msg(&mut frame_decoder, &mut body).await?;
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
