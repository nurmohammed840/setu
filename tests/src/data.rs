use rand::random;
use randox::Randox;
use setu::Message;

#[derive(Debug, Message, Randox)]
pub struct Data {
    #[key = 1]
    pub u8: u8,
    #[key = 2]
    pub u16: u16,
    #[key = 3]
    pub u32: u32,
    #[key = 4]
    pub u64: u64,

    #[key = 5]
    pub i8: i8,
    #[key = 6]
    pub i16: i16,
    #[key = 7]
    pub i32: i32,
    #[key = 8]
    pub i64: i64,

    #[key = 9]
    pub f32: f32,
    #[key = 10]
    pub f64: f64,
}

pub async fn load_data() -> Data {
    random()
}

pub async fn echo_data(input: Data) -> Data {
    input
}
