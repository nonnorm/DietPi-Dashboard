use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use client::{BackendClient, BackendContext, SystemComponents};
use config::{
    APP_VERSION,
    backend::{BackendConfig, get_config},
};
use log::{error, info};
use simple_logger::SimpleLogger;
use terminal::Terminal;
use tokio::sync::mpsc;

mod actions;
mod client;
mod getters;
mod terminal;

pub type SharedConfig = Arc<BackendConfig>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = Arc::new(get_config().context("failed to get config")?);

    SimpleLogger::new()
        .with_level(config.log_level)
        .init()
        .unwrap();

    info!("Starting DietPi-Dashboard backend v{APP_VERSION}...");

    info!("Connecting to {}", config.frontend_addr);

    let (term_tx, term_rx) = mpsc::unbounded_channel();
    let (socket_tx, socket_rx) = mpsc::unbounded_channel();

    let terminal = Terminal::new(socket_tx.clone(), term_rx).context("terminal build error")?;
    tokio::spawn(terminal.run());

    let system = Arc::new(Mutex::new(SystemComponents::new()));
    let context = BackendContext {
        config,
        system,
        term_tx,
        socket_tx,
    };

    let client = BackendClient::new(context, socket_rx).await?;

    if let Err(err) = client.run().await {
        error!("{err:#}");
    }

    Ok(())
}
