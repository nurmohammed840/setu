use setu::transport::http::HttpContext;
use setu::{Application, transport::HttpServer};
use test_suite::TestSuite;

#[nio::main]
async fn main() {
    HttpServer::new()
        .run(|mut ctx: HttpContext| {
            if let Some(id) = ctx.req.get_rpc_key() {
                TestSuite::execute(id, ctx);
            } else {
                ctx.res.write_unbound("Hello, World").unwrap();
            }
        })
        .await
        .unwrap();
}
