use setu::{
    Application, export,
    transport::{
        HttpServer,
        http::{HttpRequest, HttpResponse},
    },
};

#[nio::main]
async fn main() {
    HttpServer::new()
        .run(|req: HttpRequest, res: HttpResponse| {
            if let Some(id) = req.get_rpc_key() {
                Example::execute(id, req, res);
            } else {
                res.write_unbound("Hello, World").unwrap();
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
