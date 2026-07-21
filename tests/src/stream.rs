use setu::{Output, Stream, sse};
use std::{ops::ControlFlow, time::Duration};

pub fn fetch_user_ids(count: u8) -> impl Output {
    sse! {
        for id in 1..=count {
            nio::sleep(Duration::from_secs(1)).await;
            yield id;
        }
        return "Bye!";
    }
}

pub async fn process_msg(mut stream: Stream<String, u8>) {
    while let Ok(msg) = stream.next().await {
        match msg {
            ControlFlow::Continue(msg) => {
                println!("msg: {msg}",);
            }
            ControlFlow::Break(status) => {
                println!("status: {status}");
                break;
            }
        }
    }
}
