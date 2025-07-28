use std::net::{Ipv6Addr, SocketAddr};

use anyhow::{Context, Result};
use auth::SharedLoginMap;
use flexible_hyper_server_tls::{HttpOrHttpsAcceptor, rustls_helpers};
use hyper::service::service_fn;
use log::{error, info};
use request::ServerRequest;
use router::router;
use tokio::net::TcpListener;

use crate::{SharedConfig, backend::SharedBackendRegistry};

pub mod auth;
pub mod query_array;
pub mod request;
pub mod response;
mod router;
mod statics;

#[derive(Clone)]
pub struct FrontendContext {
    backends: SharedBackendRegistry,
    config: SharedConfig,
    logins: SharedLoginMap,
}

pub struct HttpServer {
    acceptor: HttpOrHttpsAcceptor,
    context: FrontendContext,
}

impl HttpServer {
    pub async fn new(config: SharedConfig, backends: SharedBackendRegistry) -> Result<Self> {
        info!("Starting web server on port {}", config.http_port);

        let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, config.http_port));
        let listener = TcpListener::bind(addr)
            .await
            .context("failed to bind http server")?;

        let mut acceptor = HttpOrHttpsAcceptor::new(listener);

        if config.enable_tls {
            let tls =
                rustls_helpers::get_tlsacceptor_from_files(&config.cert_path, &config.key_path)
                    .await
                    .context("failed to build TlsAcceptor")?;

            acceptor = acceptor.with_tls(tls)
        }

        let logins = SharedLoginMap::new();

        Ok(Self {
            acceptor,
            context: FrontendContext {
                config,
                logins,
                backends,
            },
        })
    }

    pub async fn run(self) {
        loop {
            let ctx = self.context.clone();

            let service = service_fn(move |req| {
                let req = ServerRequest::new(req, ctx.clone());
                async move { router(req).await }
            });

            if let Ok((_, conn_fut)) = self.acceptor.accept(service).await {
                tokio::spawn(async move {
                    if let Err(err) = conn_fut.await {
                        error!("Error serving HTTP connection: {err}");
                    }
                });
            }
        }
    }
}
