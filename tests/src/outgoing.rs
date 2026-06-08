use setu::{Output, sse};
use std::time::Duration;

pub fn fetch_user_ids(count: u8) -> impl Output {
    sse! {
        for id in 1..=count {
            nio::sleep(Duration::from_secs(1)).await;
            yield id;
        }
    }
}
