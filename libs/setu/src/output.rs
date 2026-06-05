use crate::{
    Result, Status, Timeout,
    frame::{self, FrameDecoder},
    transport::http::{HttpBody, HttpContext, HttpRequest, HttpResponse},
};
use futures::FutureExt;
use lipi::{DecodeOwned, encoder::OptionalField};
use std::{future::poll_fn, pin::pin, str::FromStr, task::Poll};
use type_id::TypeId;

pub trait Output {
    fn process<Args>(func: impl std_lib::FnOnce<Args, Output = Self> + 'static, ctx: HttpContext)
    where
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
    F::Output: OptionalField + TypeId,
{
    fn process<Args>(func: impl std_lib::FnOnce<Args, Output = Self> + 'static, ctx: HttpContext)
    where
        Args: DecodeOwned,
    {
        nio::spawn_local(async move {
            let HttpContext {
                state: _,
                mut req,
                mut res,
            } = ctx;

            let mut timer = match req.get_timeout() {
                Err(err) => return res.send_error(http::StatusCode::BAD_REQUEST, err),
                Ok(timer) => timer,
            };

            let args = match decode_args::<Args>(&mut req.body).await {
                Err(err) => return res.send_error(http::StatusCode::BAD_REQUEST, err),
                Ok(args) => args,
            };

            let mut fut = pin!(func.call_once(args));

            let result = poll_fn(
                |cx| match poll_timeout_or_reset(cx, timer.as_mut(), &mut res) {
                    CallStatus::Timeout => Poll::Ready(CallStatus::Timeout),
                    CallStatus::Canceled => Poll::Ready(CallStatus::Canceled),
                    CallStatus::Output(()) => fut
                        .as_mut()
                        .poll(cx)
                        .map(encode_data)
                        .map(CallStatus::Output),
                },
            )
            .await;

            send_output(res, result);
        });
    }
}

fn encode_data(field: impl OptionalField) -> std::io::Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    field.encode(&mut buf, 0)?;
    buf.push(lipi::DataType::StructEnd.code());
    Ok(buf)
}

fn poll_timeout_or_reset(
    cx: &mut std::task::Context<'_>,
    timer: Option<&mut nio::Sleep>,
    res: &mut HttpResponse,
) -> CallStatus<()> {
    if let Some(timer) = timer
        && timer.poll_unpin(cx).is_ready()
    {
        return CallStatus::Timeout;
    }

    if res.poll_reset(cx).is_ready() {
        return CallStatus::Canceled;
    }

    CallStatus::Output(())
}

fn send_output(mut res: HttpResponse, result: CallStatus<std::io::Result<Vec<u8>>>) {
    match result {
        CallStatus::Canceled => {}
        CallStatus::Timeout => res.send_reset(h2::Reason::CANCEL),
        CallStatus::Output(data) => match data {
            Err(err) => res.send_error(http::StatusCode::INTERNAL_SERVER_ERROR, err),
            Ok(buf) => res.send_final_message(buf),
        },
    }
}

impl HttpRequest {
    fn get_timeout(&self) -> Result<Option<nio::Sleep>, &'static str> {
        let Some(val) = self.meta.headers.get("rpc-timeout") else {
            return Ok(None);
        };
        let input = val.to_str().map_err(|_| "invalid ascii header")?;
        let timeout: Timeout = Timeout::from_str(input)?;
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
