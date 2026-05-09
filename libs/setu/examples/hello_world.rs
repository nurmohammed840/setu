#[nio::main]
async fn main() {
    setu::transport::HttpServer::new().build().await.unwrap();
}
