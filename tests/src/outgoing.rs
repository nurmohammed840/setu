use setu::{Output, sse};
use std::time::{Duration, Instant};

#[derive(Output)]
pub struct Event {
    pub elapsed: u64,
}

pub fn events(count: u8) -> impl Output {
    sse! {
        if count > 10 {
            return Err(format!("count: {count} should be <= 10"));
        }
        let time = Instant::now();
        for _ in 0..count {
            nio::sleep(Duration::from_secs(1)).await;
            yield Event { elapsed: time.elapsed().as_secs() }
        }
        Ok(())
    }
}
