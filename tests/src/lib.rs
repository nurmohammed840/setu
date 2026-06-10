mod common;
mod greeting;
mod stream;
mod stateful;

pub use common::*;
pub use greeting::*;
pub use stream::*;
pub use stateful::*;

setu::export! {
    as TestSuite;

    fn say_hello(input) = 1;

    fn add(a, b) = 2;
    fn find_in_string(input, pat) = 3;
    fn print(msg) = 4;

    fn store(msg) = 5;
    fn load() = 6;
    fn what_is_my_ip() = 7;

    // stream
    fn fetch_user_ids(count) = 8;
}
