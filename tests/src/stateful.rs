use setu::Context;

pub const MSG_KEY: u16 = 0;

pub async fn store(msg: String) {
    Context::get(|ctx| {
        ctx.as_mut().get_or_insert_with(MSG_KEY, || msg);
    });
}

pub async fn load() -> Box<String> {
    Context::get(|ctx| ctx.as_mut().take::<String>(MSG_KEY))
}

pub async fn what_is_my_ip() -> String {
    Context::get(|ctx| ctx.state.addr.to_string())
}
