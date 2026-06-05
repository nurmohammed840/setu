use setu::Context;

pub async fn store(msg: String) {
    Context::get(|ctx| {
        ctx.as_mut().init(|| msg);
    });
}

pub async fn load() -> Box<String> {
    Context::get(|ctx| ctx.as_mut().take::<String>().unwrap())
}

pub async fn what_is_my_ip() -> String {
    Context::get(|ctx| ctx.state.addr.to_string())
}
