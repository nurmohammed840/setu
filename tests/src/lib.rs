mod common;
mod greeting;
mod stateful;

use common::*;
use greeting::*;
use stateful::*;

setu::export! {
    as TestSuite;

    fn say_hello(input) = 1;

    fn add(a, b) = 2;
    fn find_in_string(input, pat) = 3;
    fn print(msg) = 4;

    fn store(msg) = 5;
    fn load() = 6;
    fn what_is_my_ip() = 7;
}
