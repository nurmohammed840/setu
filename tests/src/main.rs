use setu::{
    Application,
    transport::{
        HttpServer,
        http::{HttpRequest, HttpResponse},
    },
};

use test_suite::TestSuite;

#[nio::main]
async fn main() {
    HttpServer::new()
        .run(|req: HttpRequest, res: HttpResponse| {
            if let Some(id) = req.get_rpc_key() {
                TestSuite::execute(id, req, res);
            } else {
                res.write_unbound("Hello, World").unwrap();
            }
        })
        .await
        .unwrap();
}
