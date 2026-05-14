#[derive(lipi::Encode, lipi::Decode)]
pub struct Trailer {
    #[key = 1]
    pub status: u8,

    #[key = 2]
    pub error: Option<String>,
}
