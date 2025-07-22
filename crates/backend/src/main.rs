use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

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
use tryhard::RetryFutureConfig;

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

    let mut client = BackendClient::new(context, socket_rx).await?;

    let mut errors = 0;
    let mut last_attempt = Instant::now();

    loop {
        if let Err(err) = client.run().await {
            error!("{err:#}");

            if Instant::now().duration_since(last_attempt) < Duration::from_secs(30) {
                errors += 1;
            } else {
                errors = 0;
            }

            let capped_errors = errors.min(9);
            let timeout = Duration::from_secs(2_u64.pow(capped_errors));

            info!(
                "retrying in {} secs, errored {errors} times",
                timeout.as_secs()
            );

            tokio::time::sleep(timeout).await;

            last_attempt = Instant::now();
        }
    }

    Ok(())
}
