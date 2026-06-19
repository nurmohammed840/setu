mod common;
mod data;
mod greeting;
mod stateful;
mod stream;
mod utils;

pub use common::*;
pub use data::*;
pub use greeting::*;
pub use stateful::*;
pub use stream::*;

setu::export! {
    as TestSuite;

    fn say_hello(input) = 1;

    fn add_something(a, b) = 2;
    fn find_in_string(input, pat) = 3;
    fn print(msg) = 4;

    fn store(msg) = 5;
    fn load() = 6;
    fn what_is_my_ip() = 7;

    // stream
    fn fetch_user_ids(count) = 8;
    fn process_msg() = 9;

    // ----------
    fn random_data() = 101;
    fn echo_data(input) = 102;
    fn compare_data(left, right) = 103;

    fn random_js_value() = 104;
    fn echo_js_value(input) = 105;
    fn compare_js_value(left, right) = 106;
}
