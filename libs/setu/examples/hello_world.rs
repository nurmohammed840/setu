#[nio::main]
async fn main() {
    setu::transport::HttpServer::new().run().await.unwrap();
}
