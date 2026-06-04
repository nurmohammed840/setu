mod greeting;

use greeting::*;

pub async fn add(a: i32, b: i32) -> i32 {
    a + b
}

setu::export! {
    as TestSuite;

    fn say_hello(input) = 1;
    
    fn add(a, b) = 2;
}
