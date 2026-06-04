mod greeting;
mod common;

use greeting::*;
use common::*;

setu::export! {
    as TestSuite;

    fn say_hello(input) = 1;
    
    fn add(a, b) = 2;
    fn find_in_string(input, pat) = 3;
    fn print(msg) = 4;
}
