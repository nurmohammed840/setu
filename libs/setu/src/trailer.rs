use lipi::{Decode, Encode};

#[derive(Encode, Decode, Default, Debug)]
pub struct Trailer {
    #[key = 1]
    pub error: Option<String>,
}

impl From<String> for Trailer {
    fn from(error: String) -> Self {
        Self { error: Some(error) }
    }
}

impl Trailer {
    pub fn new() -> Self {
        Self { error: None }
    }
}
