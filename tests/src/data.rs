use std::collections::HashMap as Map;

use crate::utils::*;
use rand::random;
use randox::Rand;
use setu::Message;

#[derive(Debug, Message, Rand, PartialEq)]
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

    #[key = 11]
    pub bool: bool,

    #[key = 12]
    #[sample(rand_string(0..32))]
    pub string: String,

    #[key = 13]
    pub numeric: Numerical,
}

#[derive(Debug, Message, Rand, PartialEq)]
#[numeric]
#[repr(u8)]
pub enum Numerical {
    A = 1,
    B = 2,
    C = 3,
}

pub async fn random_data() -> Data {
    random()
}

pub async fn echo_data(input: Data) -> Data {
    input
}

pub async fn compare_data(left: Data, right: Data) -> bool {
    left == right
}

#[derive(Debug, Message, Rand, PartialEq)]
#[repr(u8)]
pub enum JsValue {
    Null = 0,
    Bool(bool) = 1,
    Number(f64) = 2,
    String(#[sample(rand_string(0..10))] String) = 3,
    Array(#[sample(sample_list(2, 0..6))] Vec<JsValue>) = 4,
    Object(#[sample(default_sample)] Map<String, JsValue>) = 5,
}

pub async fn random_js_value() -> JsValue {
    random()
}

pub async fn echo_js_value(input: JsValue) -> JsValue {
    input
}

pub async fn compare_js_value(left: JsValue, right: JsValue) -> bool {
    left == right
}
