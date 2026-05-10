use crate::{Result, transport};
use std::{env, net::SocketAddr, sync::Arc};

use nio::net::{TcpConnection, TcpListener};
use tokio_rustls::{TlsAcceptor, rustls};

#[derive(Default)]
pub struct HttpServer {
    addr: Option<SocketAddr>,
    certs: Option<String>,
    private_key: Option<String>,
}

impl HttpServer {
    pub fn new() -> HttpServer {
        HttpServer::default()
    }

    pub fn addr(mut self, addr: SocketAddr) -> Self {
        self.addr = Some(addr);
        self
    }

    pub fn certs(mut self, certs: impl Into<String>) -> Self {
        self.certs = Some(certs.into());
        self
    }

    pub fn private_key(mut self, private_key: impl Into<String>) -> Self {
        self.private_key = Some(private_key.into());
        self
    }

    pub async fn run(self) -> Result<()> {
        let addr = self.addr.unwrap_or_else(|| {
            env::var("SERVER_ADDR")
                .ok()
                .map(|a| a.parse::<SocketAddr>().ok())
                .flatten()
                .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)))
        });

        let certs = self
            .certs
            .unwrap_or_else(|| env::var("TLS_CERTS").expect(""));

        let private_key = self
            .private_key
            .unwrap_or_else(|| env::var("TLS_KEY").expect(""));

        let mut tls_config = transport::tls::server_config(certs, private_key)?;

        tls_config.alpn_protocols = vec!["h2".into()];
        if env::var("SSLKEYLOGFILE").is_ok() {
            tls_config.key_log = Arc::new(rustls::KeyLogFile::new());
        }

        HttpServer::_run(addr, TlsAcceptor::from(Arc::new(tls_config))).await
    }

    async fn _run(addr: SocketAddr, tls: TlsAcceptor) -> Result<()> {
        let mut listener = TcpListener::bind(addr).await?;

        println!("HTTP server runing: {}", listener.local_addr()?);

        loop {
            let Ok(tcp) = listener.accept().await else {
                continue;
            };

            let tls = tls.clone();

            nio::spawn_local(async {
                if let Err(err) = HttpServer::serve(tls, tcp).await {
                    println!("http-error: {err:?}");
                }
            });
        }
    }

    async fn serve(tls: TlsAcceptor, tcp: TcpConnection) -> Result<()> {
        let addr = tcp.peer_addr()?;
        let conn = tls.accept(tcp.connect().await?).await?;
        let mut conn = h2::server::handshake(conn).await?;

        println!("H2 connection: {addr}");

        while let Some(stream) = conn.accept().await {
            let (request, respond) = stream?;
            nio::spawn_local(async move {
                if let Err(e) = http_handler::handle_request(request, respond).await {
                    println!("error while handling request: {}", e);
                }
            });
        }

        Ok(())
    }
}

mod http_handler {
    use bytes::Bytes;
    use h2::{RecvStream, server::SendResponse};
    use http::Request;

    pub async fn handle_request(
        mut request: Request<RecvStream>,
        mut respond: SendResponse<Bytes>,
    ) -> crate::Result<()> {
        println!("GOT request: {:#?}", request.uri());

        let body = request.body_mut();
        while let Some(data) = body.data().await {
            let data = data?;
            let _ = body.flow_control().release_capacity(data.len());
        }

        let response = http::Response::new(());
        let mut send = respond.send_response(response, false)?;

        send.send_data(Bytes::from_static(b"hello "), false)?;
        send.send_data(Bytes::from_static(b"world\n"), true)?;

        Ok(())
    }
}
