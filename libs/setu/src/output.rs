use std::future::poll_fn;
use std::pin::pin;
use std::str::FromStr;
use std::task::Poll;

use crate::frame::{self, FrameDecoder};
use crate::transport::http::{HttpBody, HttpRequest, HttpResponse};
use crate::{Result, Status};

use futures::FutureExt;
use lipi::DecodeOwned;
use type_id::TypeId;

pub trait Output {
    fn process<Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + 'static,
        req: HttpRequest,
        res: HttpResponse,
    ) where
        Args: DecodeOwned;
}

enum CallStatus<T> {
    Canceled,
    Timeout,
    Output(T),
}

impl<F> Output for F
where
    F: Future,
    F::Output: lipi::encoder::OptionalField + TypeId,
{
    fn process<Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + 'static,
        mut req: HttpRequest,
        mut res: HttpResponse,
    ) where
        Args: DecodeOwned,
    {
        nio::spawn_local(async move {
            let mut timer = match req.get_timeout() {
                Err(err) => return res.send_error(http::StatusCode::BAD_REQUEST, err),
                Ok(timer) => timer,
            };

            let args = match decode_args::<Args>(&mut req.body).await {
                Err(err) => return res.send_error(http::StatusCode::BAD_REQUEST, err),
                Ok(args) => args,
            };

            let mut fut = pin!(func.call_once(args));

            let result = poll_fn(|cx| {
                if let Some(timer) = timer.as_mut()
                    && timer.poll_unpin(cx).is_ready()
                {
                    return Poll::Ready(CallStatus::Timeout);
                }

                if res.poll_reset(cx).is_ready() {
                    return Poll::Ready(CallStatus::Canceled);
                }

                fut.as_mut().poll(cx).map(CallStatus::Output)
            })
            .await;

            match result {
                CallStatus::Canceled => {}
                CallStatus::Timeout => res.send_reset(h2::Reason::CANCEL),
                CallStatus::Output(data) => match encode_data(data) {
                    Err(err) => res.send_error(http::StatusCode::INTERNAL_SERVER_ERROR, err),
                    Ok(buf) => res.send_final_message(buf),
                },
            }
        });
    }
}

fn encode_data(field: impl lipi::encoder::OptionalField) -> std::io::Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    field.encode(&mut buf, 0)?;
    buf.push(lipi::DataType::StructEnd.code());
    Ok(buf)
}

impl HttpRequest {
    fn get_timeout(&self) -> Result<Option<nio::Sleep>, &'static str> {
        let Some(val) = self.meta.headers.get("rpc-timeout") else {
            return Ok(None);
        };
        let input = val.to_str().map_err(|_| "invalid ascii header")?;
        let timeout = crate::Timeout::from_str(input)?;
        Ok(Some(nio::sleep(timeout.duration())))
    }
}

impl HttpResponse {
    fn send_error(mut self, code: http::StatusCode, _err: impl ToString) {
        *self.status_mut() = code;
        if cfg!(debug_assertions) {
            let err_msg = _err.to_string();
            // println!("RPC-Error: {err_msg}");
            let _ = self.write_unbound(err_msg);
        } else {
            let _ = self.send_headers();
        }
    }
}

async fn decode_args<Args: DecodeOwned>(stream: &mut HttpBody) -> Result<Args> {
    let bytes = decode_last_msg(stream).await?;
    Args::decode(&mut &*bytes)
}

async fn decode_last_msg(stream: &mut HttpBody) -> Result<frame::RawBytes> {
    let mut de = FrameDecoder::default();
    let bytes = de
        .parse_frame(stream)
        .await?
        .data
        .message()
        .ok_or("expected message frame")?;

    let (status, _) = de
        .parse_frame(stream)
        .await?
        .data
        .trailer()
        .ok_or("expected trailer frame")?;

    if status != Status::Ok {
        return Err("unexpected status code".into());
    }
    Ok(bytes)
}
