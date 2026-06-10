mod request;
mod response;
mod rpc_utils;

pub use request::{HttpBody, HttpRequest};
pub use response::{HttpResponse, HttpWriter};

use crate::{Result, context::State, transport::tls};
use std::{env, net::SocketAddr, rc::Rc, sync::Arc};

use nio::net::{TcpConnection, TcpListener};
use tokio_rustls::{TlsAcceptor, rustls};

pub trait HttpHandler: 'static + Send {
    fn handler(&self, ctx: HttpContext);
}

impl<F> HttpHandler for F
where
    F: std::ops::Fn(HttpContext) + Send + 'static,
{
    #[inline]
    fn handler(&self, ctx: HttpContext) {
        self(ctx)
    }
}

pub struct HttpContext {
    pub state: Rc<State>,
    pub req: HttpRequest,
    pub res: HttpResponse,
}

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

    pub async fn run(self, h: impl HttpHandler + Clone) -> Result<()> {
        let addr = self.addr.unwrap_or_else(|| {
            env::var("SERVER_ADDR")
                .ok()
                .and_then(|a| a.parse::<SocketAddr>().ok())
                .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)))
        });

        let certs = self
            .certs
            .unwrap_or_else(|| env::var("TLS_CERTS").expect(""));

        let private_key = self
            .private_key
            .unwrap_or_else(|| env::var("TLS_KEY").expect(""));

        let mut tls_config = tls::server_config(certs, private_key)?;

        tls_config.alpn_protocols = vec!["h2".into()];
        if env::var("SSLKEYLOGFILE").is_ok() {
            tls_config.key_log = Arc::new(rustls::KeyLogFile::new());
        }

        let tls = TlsAcceptor::from(Arc::new(tls_config));

        HttpServer::_run(addr, tls, h).await
    }

    async fn _run(addr: SocketAddr, tls: TlsAcceptor, h: impl HttpHandler + Clone) -> Result<()> {
        let mut listener = TcpListener::bind(addr).await?;

        println!("Runing HTTP server: https://{}", listener.local_addr()?);

        loop {
            let Ok(tcp) = listener.accept().await else {
                continue;
            };

            let tls = tls.clone();
            let h = h.clone();

            nio::spawn_pinned(|| async move {
                if let Err(_err) = HttpServer::serve(tls, tcp, h).await {
                    // println!("http-error: {_err:?}");
                }
            });
        }
    }

    async fn serve(tls: TlsAcceptor, tcp: TcpConnection, h: impl HttpHandler) -> Result<()> {
        let addr = tcp.peer_addr()?;
        let conn = tls.accept(tcp.connect().await?).await?;
        let mut conn = h2::server::handshake(conn).await?;

        println!("H2 connection: {addr}");

        let session = State::new(addr);

        while let Some(stream) = conn.accept().await {
            let (req, res) = stream?;
            h.handler(HttpContext {
                state: session.clone(),
                req: HttpRequest::from(req),
                res: HttpResponse::from(res),
            });
        }
        Ok(())
    }
}
