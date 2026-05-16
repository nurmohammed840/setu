use crate::Status;
use crate::frame::FrameHeader;
use lipi::{Decode, Encode};

#[derive(Encode, Decode, Default, Debug)]
pub struct Trailer {
    #[key = 1]
    pub error: Option<String>,
}

impl Trailer {
    pub(crate) const OK_ENCODED: [u8; 2] = [FrameHeader::new(Some(Status::Ok), 1).encode(), 0];

    pub fn new() -> Self {
        Self { error: None }
    }
}
