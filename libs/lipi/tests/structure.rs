use lipi::{Decode, Encode, Entries};

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct Types<'a> {
    #[key = 1]
    boolen: bool,

    #[key = 2]
    u16: u16,

    #[key = 3]
    i16: i16,

    #[key = 4]
    u32: u32,

    #[key = 5]
    i32: i32,

    #[key = 6]
    u64: u64,

    #[key = 7]
    i64: i64,

    #[key = 8]
    f32: f32,

    #[key = 9]
    f64: f64,

    #[key = 10]
    string: &'a str,
    #[key = 11]
    bytes: &'a [u8],

    #[key = 12]
    user: User,

    // ---------------
    #[key = 13]
    arr_bool: Vec<bool>,

    #[key = 14]
    arr_i8: Vec<i8>,
    #[key = 15]
    arr_i16: Vec<i16>,
    #[key = 16]
    arr_i32: Vec<i32>,
    #[key = 17]
    arr_i64: Vec<i64>,

    #[key = 18]
    arr_u8: Vec<u8>,
    #[key = 19]
    arr_u16: Vec<u16>,
    #[key = 20]
    arr_u32: Vec<u32>,
    #[key = 21]
    arr_u64: Vec<u64>,

    #[key = 22]
    arr_f32: Vec<f32>,

    #[key = 23]
    arr_f64: Vec<f64>,
    // #[key = 24]
    // matrix_2d: Vec<Vec<f64>>,

    // #[key = 25]
    // matrixs: Vec<Vec<Vec<f32>>>,
    // #[key = 26]
    // users: Vec<User>,
    // #[key = 27]
    // map: HashMap<u32, User>,
}

#[derive(Encode, Debug, Decode, Clone, PartialEq)]
struct User {
    // #[key = 0]
    // id: Vec<u8>,
    #[key = 1]
    name: String,
    #[key = 2]
    email: Option<String>,
}

impl<'a> Types<'a> {
    pub fn new() -> Self {
        Types {
            boolen: true,
            u16: u16::MAX,
            i16: i16::MAX,
            u32: u32::MAX,
            i32: i32::MAX,
            u64: u64::MAX,
            i64: i64::MAX,
            f32: f32::MAX,
            f64: f64::MAX,
            string: "Hello World",
            bytes: "Hello, World".as_bytes(),
            user: User {
                // id: vec![1, 2, 3, 4, 5],
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
            // matrix_2d: vec![vec![0., 1.], vec![1., 0.]],
            // matrixs: vec![
            //     vec![vec![0., 1.], vec![1., 0.]],
            //     vec![vec![1., 0.], vec![0., 1.]],
            // ],
            // map: {
            //     let mut map = HashMap::new();
            //     map.insert(
            //         1,
            //         User {
            //             id: vec![1],
            //             name: "Nur".into(),
            //             email: Some("nur@email.com".into()),
            //         },
            //     );
            //     map.insert(
            //         2,
            //         User {
            //             id: vec![2],
            //             name: "Mohammed".into(),
            //             email: Some("mohammed@email.com".into()),
            //         },
            //     );
            //     map
            // },
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode(&mut buf).unwrap();
        buf
    }

    #[allow(dead_code)]
    pub fn from_bytes(mut buf: &'a [u8]) -> lipi::Result<Self> {
        Self::decode(&Entries::parse(&mut buf)?)
    }
}
