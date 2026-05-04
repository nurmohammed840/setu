use std::collections::HashMap;

use lipi::*;

#[derive(Encode, Decode, Debug, PartialEq)]
pub struct Types {
    #[key = 0]
    pub bool: bool,

    #[key = 1]
    pub u8: u8,

    #[key = 2]
    pub i8: i8,

    #[key = 3]
    pub f32: f32,

    #[key = 4]
    pub f64: f64,

    #[key = 5]
    pub u64: u64,

    #[key = 6]
    pub i64: i64,

    #[key = 7]
    pub string: String,

    #[key = 8]
    pub object: User,

    #[key = 9]
    pub union: TagUnion,

    #[key = 10]
    pub list: Vec<TagUnion>,

    #[key = 11]
    pub map: HashMap<u32, TagUnion>,

    // --- extra edge cases ---
    #[key = 12]
    pub optional: Option<TagUnion>,

    #[key = 13]
    pub bools: Vec<bool>,

    #[key = 14]
    pub nested: Vec<Vec<TagUnion>>,

    #[key = 15]
    pub matrix3d: Vec<Vec<Vec<f32>>>,

    pub canvas_2d: Vec<Vec<u8>>,
}

#[repr(u32)]
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum TagUnion {
    Unit = 0,
    Simple(u32) = 1,
    Complex(User) = 2,
    Bytes(Vec<u8>) = 3,
    String(String) = 4,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct User {
    #[key = 0]
    pub id: Option<u32>,

    #[key = 1]
    #[default]
    pub name: String,
}

impl Types {
    pub fn min() -> Types {
        Types {
            bool: false,
            u8: 0,
            i8: 0,
            f32: 0.0,
            f64: 0.0,
            u64: 0,
            i64: 0,
            string: String::new(),
            object: User {
                id: None,
                name: String::new(),
            },
            union: TagUnion::Unit,
            list: Vec::new(),
            map: HashMap::new(),
            optional: None,
            bools: Vec::new(),
            nested: Vec::new(),
            matrix3d: Vec::new(),
            canvas_2d: Vec::new(),
        }
    }

    pub fn mid() -> Types {
        Types {
            bool: true,
            u8: 128,
            i8: -64,
            f32: 3.14,
            f64: 2.71828,
            u64: 1 << 32,
            i64: -(1 << 32),
            string: "Hello, Lipi!".to_string(),
            object: User {
                id: Some(42),
                name: "Alice".to_string(),
            },
            union: TagUnion::Complex(User {
                id: Some(99),
                name: "Bob".to_string(),
            }),
            list: vec![
                TagUnion::Unit,
                TagUnion::Simple(123),
                TagUnion::Bytes(vec![1, 2, 3]),
                TagUnion::String("Test".to_string()),
            ],
            map: HashMap::from([
                (1, TagUnion::Simple(10)),
                (
                    2,
                    TagUnion::Complex(User {
                        id: Some(7),
                        name: "Charlie".to_string(),
                    }),
                ),
            ]),
            optional: Some(TagUnion::String("Optional".to_string())),
            bools: vec![true, false, true],
            nested: vec![
                vec![TagUnion::Unit, TagUnion::Simple(456)],
                vec![TagUnion::Bytes(vec![4, 5, 6])],
            ],
            matrix3d: vec![
                vec![vec![0.0, 1.0], vec![2.0, 3.0]],
                vec![vec![4.0, 5.0], vec![6.0, 7.0]],
            ],
            canvas_2d: vec![
                vec![255, 0, 0], // Red
                vec![0, 255, 0], // Green
                vec![0, 0, 255], // Blue
            ],
        }
    }

    pub fn max() -> Types {
        Types {
            bool: true,
            u8: u8::MAX,
            i8: i8::MAX,
            f32: f32::MAX,
            f64: f64::MAX,
            u64: u64::MAX,
            i64: i64::MAX,
            string: "A".repeat(1000),
            object: User {
                id: Some(u32::MAX),
                name: "Z".repeat(1000),
            },
            union: TagUnion::Bytes(vec![0xFF; 1000]),
            list: vec![TagUnion::String("X".repeat(500)); 10],
            map: HashMap::from([(u32::MAX, TagUnion::Simple(u32::MAX))]),
            optional: Some(TagUnion::Complex(User {
                id: Some(u32::MAX),
                name: "Y".repeat(1000),
            })),
            bools: vec![true; 100],
            nested: vec![vec![TagUnion::Bytes(vec![0xAA; 32])]; 3],
            matrix3d: vec![vec![vec![f32::MAX; 10]; 10]; 10],
            canvas_2d: vec![
                vec![255; 100]; // White
                100
            ],
        }
    }
}
