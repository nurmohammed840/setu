use std::future::poll_fn;
use std::pin::pin;
use std::task::Poll;
use std::time::Duration;

use crate::Result;
use crate::frame::FrameDecoder;
use crate::transport::http::{HttpBody, HttpRequest, HttpResponse};

use futures::FutureExt;
use lipi::Encode;
use type_id::TypeId;

pub trait Output {
    fn process<Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + 'static,
        req: HttpRequest,
        res: HttpResponse,
    ) where
        Args: for<'a> lipi::Decode<'a>;
}

enum CallStatus<T> {
    Canceled,
    Timeout,
    Output(T),
}

impl<F> Output for F
where
    F: Future,
    F::Output: lipi::Encode + TypeId,
{
    fn process<Args>(
        func: impl std_lib::FnOnce<Args, Output = Self> + 'static,
        mut req: HttpRequest,
        mut res: HttpResponse,
    ) where
        Args: for<'a> lipi::Decode<'a>,
    {
        nio::spawn_local(async move {
            match decode_input_args::<Args>(&mut req.body).await {
                Err(err) => send_error(res, http::StatusCode::BAD_REQUEST, err),
                Ok(args) => {
                    let mut timer = Some(nio::sleep(Duration::from_secs(60)));
                    let mut fut = pin!(func.call_once(args));

                    let result = poll_fn(|cx| {
                        if let Some(timer) = timer.as_mut()
                            && let Poll::Ready(()) = timer.poll_unpin(cx)
                        {
                            return Poll::Ready(CallStatus::Timeout);
                        }

                        if let Poll::Ready(_) = res.writer.poll_reset(cx) {
                            return Poll::Ready(CallStatus::Canceled);
                        }

                        fut.as_mut().poll(cx).map(CallStatus::Output)
                    })
                    .await;

                    match result {
                        CallStatus::Canceled => {}
                        CallStatus::Timeout => {
                            res.writer.send_reset(h2::Reason::CANCEL);
                        }
                        CallStatus::Output(data) => match data.to_bytes() {
                            Ok(buf) => {
                                let _ = res.send_final_message(buf);
                            }
                            Err(err) => {
                                send_error(res, http::StatusCode::INTERNAL_SERVER_ERROR, err);
                            }
                        },
                    }
                }
            }
        });
    }
}

#[inline]
fn send_error(mut res: HttpResponse, code: http::StatusCode, _err: impl ToString) {
    res.status = code;
    if cfg!(debug_assertions) {
        let err_msg = _err.to_string();
        // println!("RPC-Error: {err_msg}");
        let _ = res.write_unbound(err_msg);
    } else {
        let _ = res.send_headers();
    }
}

async fn decode_input_args<Args>(body: &mut HttpBody) -> Result<Args>
where
    Args: for<'a> lipi::Decode<'a>,
{
    let mut de = FrameDecoder::default();
    let frame = de.parse_frame(body).await?;
    let input = frame.data.message()?;
    Args::decode(&mut &*input)
}
