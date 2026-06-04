pub async fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub async fn find_in_string(str: String, pat: String) -> Option<u32> {
    str.find(&pat).map(|idx| idx as u32)
}
