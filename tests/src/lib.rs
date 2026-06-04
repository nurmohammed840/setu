mod greeting;

use greeting::*;

setu::export! {
    as TestSuite;

    fn say_hello(req) = 1;
}
