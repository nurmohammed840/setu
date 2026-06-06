use setu::Context;

pub async fn store(msg: String) {
    Context::get().as_mut().init(|| msg);
}

pub async fn load() -> Option<Box<String>> {
    Context::get().as_mut().take::<String>()
}

pub async fn what_is_my_ip() -> String {
    Context::get().addr().to_string()
}
