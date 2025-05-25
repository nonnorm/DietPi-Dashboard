use std::{
    collections::HashMap,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use conn::{BackendConnection, BackendInfo};
use log::{error, info};
use tokio::net::TcpListener;

mod cache;
mod conn;

pub use conn::BackendHandle;

pub type BackendRegistry = HashMap<IpAddr, BackendInfo>;
pub type SharedBackendRegistry = Arc<Mutex<BackendRegistry>>;

pub struct BackendServer {
    listener: TcpListener,
    registry: SharedBackendRegistry,
}

impl BackendServer {
    pub async fn new(port: u16, registry: SharedBackendRegistry) -> Result<Self> {
        info!("Starting backend server on port {port}");

        let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, port));
        let listener = TcpListener::bind(addr)
            .await
            .context("failed to bind backend tcp server")?;

        Ok(Self { listener, registry })
    }

    pub async fn run(self) {
        loop {
            let (stream, peer_ip) = match self.listener.accept().await {
                Ok((stream, peer_addr)) => (stream, peer_addr.ip().to_canonical()),
                Err(err) => {
                    error!("Failed to accept backend connection: {err:#}");
                    continue;
                }
            };

            info!("New backend connection from {}", peer_ip);

            let conn = BackendConnection::new(stream, self.registry.clone(), peer_ip);

            tokio::spawn(conn.handle_connection());
        }
    }
}
