use setu::{
    Application, export,
    transport::{HttpServer, http::HttpContext},
};

#[nio::main]
async fn main() {
    HttpServer::new()
        .run(|ctx: HttpContext| {
            if let Some(id) = ctx.req.get_rpc_key() {
                Example::execute(id, ctx);
            } else {
                ctx.res.write_unbound("Hello, World").unwrap();
            }
        })
        .await
        .unwrap();
}

async fn add(a: u8, b: u8) -> u8 {
    a + b
}

export! {
    as Example;

    fn add(a, b) = 15;
}
