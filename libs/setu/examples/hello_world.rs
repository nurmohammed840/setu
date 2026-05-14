use http::Response;
use setu::{
    Application,
    transport::{
        HttpServer,
        http::{HttpRequest, HttpResponse},
    },
};

#[nio::main]
async fn main() {
    HttpServer::new()
        .run(|req: HttpRequest, mut res: HttpResponse| {
            if let Some(id) = req.get_rpc_key() {
                Example::execute(id, req, res);
            } else {
                let response = Response::new(());
                let mut stream = res.writer.send_response(response, false).unwrap();
                stream.send_data("Hello, World".into(), true).unwrap();
            }
        })
        .await
        .unwrap();
}

async fn add(a: u8, b: u8) -> u8 {
    a + b
}

#[derive(Clone)]
struct Example;

impl setu::Application for Example {
    fn execute(id: u32, req: HttpRequest, res: HttpResponse) {
        match id {
            42 => setu::Output::process(add, req, res),
            _ => {}
        }
    }
}
