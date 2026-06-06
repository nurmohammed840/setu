use crate::{
    Context, Result, Status, Timeout,
    frame::{self, FrameDecoder},
    transport::http::{HttpBody, HttpContext, HttpRequest, HttpResponse},
};
use futures::FutureExt;
use lipi::{DecodeOwned, encoder::OptionalField};
use std::{
    future::poll_fn,
    pin::pin,
    rc::Rc,
    str::FromStr,
    task::{self, Poll},
};
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
            let Ok((context, mut timer, mut body, mut res)) = ctx.parts() else {
                return;
            };

            let args = match decode_args::<Args>(&mut body).await {
                Err(err) => return res.send_error(http::StatusCode::BAD_REQUEST, err),
                Ok(args) => args,
            };

            let mut fut = pin!(func.call_once(args));

            let mut ctx = Some(context);
            let output = poll_fn(
                |cx| match poll_timeout_or_reset(cx, timer.as_mut(), &mut res) {
                    CallStatus::Timeout => Poll::Ready(CallStatus::Timeout),
                    CallStatus::Canceled => Poll::Ready(CallStatus::Canceled),
                    CallStatus::Output(()) => {
                        Context::swap(&mut ctx);
                        let poll = fut.as_mut().poll(cx);
                        Context::swap(&mut ctx);

                        poll.map(encode_data).map(CallStatus::Output)
                    }
                },
            )
            .await;

            send_output(res, output);
        });
    }
}

impl HttpContext {
    fn parts(self) -> Result<(Context, Option<nio::Sleep>, HttpBody, HttpResponse), ()> {
        let HttpContext {
            state,
            mut req,
            res,
        } = self;
        let timeout = match req.get_timeout() {
            Err(err) => {
                res.send_error(http::StatusCode::BAD_REQUEST, err);
                return Err(());
            }
            Ok(timeout) => timeout,
        };
        let HttpRequest { meta, body } = req;
        let context = Context {
            state,
            timeout,
            http_headers: Some(Rc::new(meta.headers)),
        };
        let timer = timeout.map(Timeout::duration).map(nio::sleep);
        Ok((context, timer, body, res))
    }
}

fn encode_data(field: impl OptionalField) -> std::io::Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    field.encode(&mut buf, 0)?;
    buf.push(lipi::DataType::StructEnd.code());
    Ok(buf)
}

fn poll_timeout_or_reset(
    cx: &mut task::Context<'_>,
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

fn send_output(mut res: HttpResponse, output: CallStatus<std::io::Result<Vec<u8>>>) {
    match output {
        CallStatus::Canceled => {}
        CallStatus::Timeout => res.send_reset(h2::Reason::CANCEL),
        CallStatus::Output(data) => match data {
            Err(err) => res.send_error(http::StatusCode::INTERNAL_SERVER_ERROR, err),
            Ok(buf) => res.send_final_message(buf),
        },
    }
}

impl HttpRequest {
    fn get_timeout(&mut self) -> Result<Option<Timeout>, &'static str> {
        let Some(val) = self.meta.headers.remove("rpc-timeout") else {
            return Ok(None);
        };
        let input = val.to_str().map_err(|_| "invalid ascii header")?;
        let timeout = Timeout::from_str(input)?;
        Ok(Some(timeout))
    }
}

impl HttpResponse {
    fn send_error(mut self, code: http::StatusCode, _err: impl ToString) {
        *self.status_mut() = code;
        if cfg!(debug_assertions) {
            let _ = self.write_unbound(_err.to_string());
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
