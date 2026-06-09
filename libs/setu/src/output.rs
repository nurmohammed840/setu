use crate::{
    Context, Result, SSE, Status, Timeout,
    frame::{FrameDecoder, FrameEncoder, RawBytes},
    transport::http::{HttpBody, HttpContext, HttpRequest, HttpResponse, HttpWriter},
};
use async_gen::{AsyncGenerator, GeneratorState};
use futures::FutureExt;
use lipi::{DecodeOwned, encoder::OptionalField};
use nio::Sleep;
use setu_type_info::FnOutputType;
use std::{
    future::poll_fn,
    io,
    pin::pin,
    str::FromStr,
    task::{self, Poll},
};
use type_id::TypeId;

pub trait Output: FnOutputType {
    fn process<Args, F>(func: F, ctx: HttpContext)
    where
        Args: DecodeOwned,
        F: std_lib::FnOnce<Args, Output = Self> + 'static;
}

impl<T> Output for T
where
    T: Future,
    T::Output: OptionalField + TypeId,
{
    fn process<Args, F>(func: F, ctx: HttpContext)
    where
        Args: DecodeOwned,
        F: std_lib::FnOnce<Args, Output = Self> + 'static,
    {
        nio::spawn_local(async move {
            let Ok((context, mut timer, input, output)) = ctx.parts() else {
                return;
            };

            let args = match decode_args(input).await {
                Err(err) => return output.send_error(http::StatusCode::BAD_REQUEST, err),
                Ok(args) => args,
            };

            let Ok(mut output) = output.create_setu_stream() else {
                return;
            };

            let mut fut = pin!(func.call_once(args));
            let mut ctx = context.boxed();

            let result = poll_fn(|cx| {
                if timeout_or_cancellation(cx, timer.as_mut(), &mut output.stream).is_ready() {
                    return Poll::Ready(None);
                }

                Context::swap(&mut ctx);
                let poll = fut.as_mut().poll(cx);
                Context::swap(&mut ctx);

                poll.map(encode_data).map(Some)
            })
            .await;

            send_output(output, result)
        });
    }
}

impl<S> Output for SSE<S>
where
    S: AsyncGenerator,
    S::Yield: OptionalField + TypeId,
    S::Return: OptionalField + TypeId,
{
    fn process<Args, F>(func: F, ctx: HttpContext)
    where
        Args: DecodeOwned,
        F: std_lib::FnOnce<Args, Output = Self> + 'static,
    {
        nio::spawn_local(async {
            let Ok((context, mut timer, input, output)) = ctx.parts() else {
                return;
            };

            let args = match decode_args(input).await {
                Err(err) => return output.send_error(http::StatusCode::BAD_REQUEST, err),
                Ok(args) => args,
            };

            let Ok(mut output) = output.create_setu_stream() else {
                return;
            };

            let mut stream = pin!(func.call_once(args).0);
            let mut ctx = context.boxed();
            loop {
                let resume = poll_fn(|cx| {
                    if timeout_or_cancellation(cx, timer.as_mut(), &mut output.stream).is_ready() {
                        return Poll::Ready(None);
                    }

                    Context::swap(&mut ctx);
                    let poll = stream.as_mut().poll_resume(cx);
                    Context::swap(&mut ctx);

                    poll.map(|state| match state {
                        GeneratorState::Yielded(data) => {
                            (encode_data(data), GeneratorState::Yielded(()))
                        }
                        GeneratorState::Complete(data) => {
                            (encode_data(data), GeneratorState::Complete(()))
                        }
                    })
                    .map(Some)
                })
                .await;

                let Some(o) = send_stream(output, resume).await else {
                    break;
                };

                output = o
            }
        });
    }
}

type MaybeResumed = Option<(io::Result<Vec<u8>>, GeneratorState<(), ()>)>;
async fn send_stream(mut output: FrameEncoder, resume: MaybeResumed) -> Option<FrameEncoder> {
    let (result, state) = resume?;

    let data = match result {
        Ok(data) => data,
        Err(err) => {
            let _ = output.send_error(Status::Internal, err.to_string());
            return None;
        }
    };

    match state {
        GeneratorState::Yielded(()) => match output.send(data).await {
            Ok(()) => Some(output), // continue
            Err(_) => None,
        },
        GeneratorState::Complete(()) => {
            let _ = output.end(data);
            None
        }
    }
}

impl HttpContext {
    fn parts(self) -> Result<(Context, Option<Sleep>, HttpBody, HttpResponse), ()> {
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
            http_headers: meta.headers,
        };
        let timer = timeout.map(Timeout::duration).map(nio::sleep);
        Ok((context, timer, body, res))
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

fn timeout_or_cancellation(
    cx: &mut task::Context,
    timer: Option<&mut Sleep>,
    output: &mut HttpWriter,
) -> Poll<()> {
    if let Some(timer) = timer
        && timer.poll_unpin(cx).is_ready()
    {
        output.send_reset(h2::Reason::CANCEL);
        return Poll::Ready(());
    }
    if output.poll_reset(cx).is_ready() {
        return Poll::Ready(());
    }

    Poll::Pending
}

fn encode_data(data: impl OptionalField) -> io::Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    data.encode(&mut buf, 0)?;
    buf.push(lipi::DataType::StructEnd.code());
    Ok(buf)
}

fn send_output(output: FrameEncoder, result: Option<io::Result<Vec<u8>>>) {
    let Some(result) = result else {
        return;
    };
    match result {
        Err(err) => {
            let _ = output.send_error(Status::Internal, err.to_string());
        }
        Ok(buf) => {
            let _ = output.end(buf);
        }
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

async fn decode_args<Args: DecodeOwned>(mut input: HttpBody) -> Result<Args> {
    let bytes = decode_last_msg(&mut input).await?;
    Args::decode(&mut &*bytes)
}

async fn decode_last_msg(stream: &mut HttpBody) -> Result<RawBytes> {
    let mut de = FrameDecoder::default();

    let (status, bytes) = de
        .parse_frame(stream)
        .await?
        .data
        .trailer()
        .ok_or("expected trailer frame")?;

    if status != Status::Ok {
        return Err(format!("unexpected status: {status:?}").into());
    }
    Ok(bytes)
}
