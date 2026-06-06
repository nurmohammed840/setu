use setu::Context;

pub async fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub async fn find_in_string(str: String, pat: String) -> Option<u32> {
    str.find(&pat).map(|idx| idx as u32)
}

pub async fn print(msg: String) {
    let headers = Context::http_headers().unwrap();
    let addr = Context::addr();

    println!("headers: {headers:#?}");
    println!("addr: {addr}");
    println!("data: {msg}");
}
