use crate::Status;
use lipi::{Decode, Encode};

#[derive(Encode, Decode, Default, Debug)]
pub struct Trailer {
    #[key = 1]
    pub status: u8,

    #[key = 2]
    pub error: Option<String>,
}

impl Trailer {
    pub(crate) const OK_ENCODED: [u8; 3] = [18, 0, 10];

    pub fn new(status: Status) -> Self {
        Self {
            status: status.code(),
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailer_encoded() {
        assert_eq!(Trailer::default().to_bytes().unwrap(), Trailer::OK_ENCODED);
    }
}
