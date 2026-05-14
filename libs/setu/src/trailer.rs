use crate::Status;

#[derive(lipi::Encode, lipi::Decode, Default, Debug)]
pub struct Trailer {
    #[key = 1]
    pub status: u8,

    #[key = 2]
    pub error: Option<String>,
}

impl Trailer {
    pub(crate) const MIN_ENCODED_LEN: usize = 3;

    pub fn new(status: Status) -> Self {
        Self {
            status: status.code(),
            error: None,
        }
    }
}
