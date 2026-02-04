#![feature(test)]

extern crate test;
use test::Bencher;

use lipi::{Decoder, Encoder, Entries};

#[derive(Encoder, Decoder, Clone, Debug, PartialEq)]
struct Types<'a> {
    #[key = 1]
    bool_true: bool,
    #[key = 2]
    bool_false: bool,

    #[key = 3]
    u16_min: u16,
    #[key = 4]
    u16_max: u16,

    #[key = 5]
    i16_min: i16,
    #[key = 6]
    i16_max: i16,

    #[key = 7]
    u32_min: u32,
    #[key = 8]
    u32_max: u32,

    #[key = 9]
    i32_min: i32,
    #[key = 10]
    i32_max: i32,

    #[key = 11]
    u64_min: u64,
    #[key = 12]
    u64_max: u64,

    #[key = 13]
    i64_min: i64,
    #[key = 14]
    i64_max: i64,

    #[key = 15]
    f32_min: f32,
    #[key = 16]
    f32_max: f32,

    #[key = 17]
    f64_min: f64,
    #[key = 18]
    f64_max: f64,

    #[key = 19]
    string: &'a str,
    #[key = 20]
    bytes: &'a [u8],

    #[key = 21]
    user: User,

    // ---------------
    #[key = 22]
    arr_bool: Vec<bool>,

    #[key = 23]
    arr_i8: Vec<i8>,
    #[key = 24]
    arr_i16: Vec<i16>,
    #[key = 25]
    arr_i32: Vec<i32>,
    #[key = 26]
    arr_i64: Vec<i64>,

    #[key = 27]
    arr_u8: Vec<u8>,
    #[key = 28]
    arr_u16: Vec<u16>,
    #[key = 29]
    arr_u32: Vec<u32>,
    #[key = 30]
    arr_u64: Vec<u64>,

    #[key = 31]
    arr_f32: Vec<f32>,

    #[key = 32]
    arr_f64: Vec<f64>,

    #[key = 33]
    matrix_2d: Vec<Vec<f64>>,

    #[key = 34]
    matrixs: Vec<Vec<Vec<f32>>>,
}

#[derive(Encoder, Debug, Decoder, Clone, PartialEq)]
struct User {
    #[key = 0]
    id: Vec<u8>,
    #[key = 1]
    name: String,
    #[key = 2]
    email: Option<String>,
}

impl<'a> Types<'a> {
    fn new() -> Self {
        Types {
            bool_true: true,
            bool_false: false,

            u16_min: u16::MIN,
            u16_max: u16::MAX,

            i16_min: i16::MIN,
            i16_max: i16::MAX,

            u32_min: u32::MIN,
            u32_max: u32::MAX,

            i32_min: i32::MIN,
            i32_max: i32::MAX,

            u64_min: u64::MIN,
            u64_max: u64::MAX,

            i64_min: i64::MIN,
            i64_max: i64::MAX,

            f32_min: f32::MIN,
            f32_max: f32::MAX,

            f64_min: f64::MIN,
            f64_max: f64::MAX,

            string: "Hello World",
            bytes: "Hello, World".as_bytes(),
            user: User {
                id: vec![1, 2, 3, 4, 5],
                name: "Alex".into(),
                email: None,
            },

            arr_bool: vec![true, false],
            arr_i8: vec![i8::MIN, 0, i8::MAX],
            arr_i16: vec![i16::MIN, 0, i16::MAX],
            arr_i32: vec![i32::MIN, 0, i32::MAX],
            arr_i64: vec![i64::MIN, 0, i64::MAX],

            arr_u8: vec![u8::MIN, u8::MAX / 2, u8::MAX],
            arr_u16: vec![u16::MIN, u16::MAX / 2, u16::MAX],
            arr_u32: vec![u32::MIN, u32::MAX / 2, u32::MAX],
            arr_u64: vec![u64::MIN, u64::MAX / 2, u64::MAX],

            arr_f32: vec![f32::MIN, 0., f32::MAX],
            arr_f64: vec![f64::MIN, 0., f64::MAX],

            matrix_2d: vec![vec![0., 1.], vec![1., 0.]],
            matrixs: vec![
                vec![vec![0., 1.], vec![1., 0.]],
                vec![vec![1., 0.], vec![0., 1.]],
            ],
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode(&mut buf).unwrap();
        buf
    }

    fn from_bytes(mut buf: &'a [u8]) -> lipi::Result<Self> {
        Self::decode(&Entries::parse(&mut buf)?)
    }
}

#[test]
fn test_all_types() {
    let all_types = Types::new();

    let buf = all_types.to_bytes();

    let mut reader = &buf[..];
    let entries = Entries::parse(&mut reader).unwrap();
    let new_all_types = Types::decode(&entries);

    // println!("{:#?}", new_all_types);
    // println!("{:#?}", lipi::Value::Struct(entries));
    assert_eq!(all_types, new_all_types.unwrap());
}

// ---------------------------------------------------------------------------------------

#[bench]
fn bench_encode(b: &mut Bencher) {
    let all_types = Types::new();
    b.iter(|| {
        let buf = all_types.to_bytes();
        assert!(!buf.is_empty());
    });
}

#[bench]
fn bench_parse_and_decode(b: &mut Bencher) {
    let all_types = Types::new();
    let buf = all_types.to_bytes();

    b.iter(|| {
        let res = Types::from_bytes(&buf[..]);
        assert!(res.is_ok());
    });
}

#[bench]
fn bench_parse(b: &mut Bencher) {
    let all_types = Types::new();
    let buf = all_types.to_bytes();

    b.iter(|| {
        let mut reader = &buf[..];
        let entries = Entries::parse(&mut reader).unwrap();
        assert!(!entries.is_empty());
    });
}

#[bench]
fn bench_decode(b: &mut Bencher) {
    let all_types = Types::new();
    let buf = all_types.to_bytes();

    let mut reader = &buf[..];
    let entries = Entries::parse(&mut reader).unwrap();

    b.iter(|| {
        let all_types = Types::decode(&entries);
        assert!(all_types.is_ok());
    });
}
