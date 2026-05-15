use std::future::poll_fn;
use std::pin::pin;
use std::str::FromStr;
use std::task::Poll;

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
            let mut timer = match req.get_timeout() {
                Err(err) => return res.send_error(http::StatusCode::BAD_REQUEST, err),
                Ok(timer) => timer,
            };

            let args = match decode_input_args::<Args>(&mut req.body).await {
                Err(err) => return res.send_error(http::StatusCode::BAD_REQUEST, err),
                Ok(args) => args,
            };

            let mut fut = pin!(func.call_once(args));

            let result = poll_fn(|cx| {
                if let Some(timer) = timer.as_mut()
                    && let Poll::Ready(()) = timer.poll_unpin(cx)
                {
                    return Poll::Ready(CallStatus::Timeout);
                }

                if let Poll::Ready(_) = res.poll_reset(cx) {
                    return Poll::Ready(CallStatus::Canceled);
                }

                fut.as_mut().poll(cx).map(CallStatus::Output)
            })
            .await;

            match result {
                CallStatus::Canceled => {}
                CallStatus::Timeout => res.send_reset(h2::Reason::CANCEL),
                CallStatus::Output(data) => match data.to_bytes() {
                    Err(err) => res.send_error(http::StatusCode::INTERNAL_SERVER_ERROR, err),
                    Ok(buf) => res.send_final_message(buf),
                },
            }
        });
    }
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
        self.status = code;
        if cfg!(debug_assertions) {
            let err_msg = _err.to_string();
            // println!("RPC-Error: {err_msg}");
            let _ = self.write_unbound(err_msg);
        } else {
            let _ = self.send_headers();
        }
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
