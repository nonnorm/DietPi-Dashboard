use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use backend::{BackendRegistry, BackendServer};
use config::{
    APP_VERSION,
    frontend::{FrontendConfig, get_config},
};
use http::HttpServer;
use log::info;
use simple_logger::SimpleLogger;

mod backend;
mod http;
mod pages;

pub type SharedConfig = Arc<FrontendConfig>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = Arc::new(get_config().context("failed to get config")?);

    SimpleLogger::new()
        .with_level(config.log_level)
        .init()
        .unwrap();

    info!("Starting DietPi-Dashboard frontend v{APP_VERSION}...");

    let backends = Arc::new(Mutex::new(BackendRegistry::new()));

    let backend_server = BackendServer::new(config.backend_port, backends.clone()).await?;

    let http_server = HttpServer::new(config, backends.clone()).await?;

    tokio::join!(http_server.run(), backend_server.run());

    Ok(())
}
