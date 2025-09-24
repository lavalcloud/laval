use std::thread::{self, JoinHandle};

use anyhow::Result;
use tokio::runtime::Builder;
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::config::PortMappingConfig;

pub struct RatholeHandle {
    shutdown: broadcast::Sender<bool>,
    join: Option<JoinHandle<()>>,
}

impl RatholeHandle {
    pub fn shutdown(mut self) {
        if let Err(err) = self.shutdown.send(true) {
            error!(?err, "failed to signal Rathole shutdown");
        }
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

pub fn spawn_rathole(config: &PortMappingConfig) -> Result<RatholeHandle> {
    let mut cli = rathole::Cli::default();
    cli.config_path = Some(config.config_path.clone());
    cli.server = config.server;
    cli.client = !config.server;

    let (shutdown_tx, shutdown_rx) = broadcast::channel(4);

    info!(
        path = %config.config_path.display(),
        mode = if config.server { "server" } else { "client" },
        "starting Rathole instance"
    );

    let handle = thread::Builder::new()
        .name("rathole-runner".into())
        .spawn(move || {
            let runtime = Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to create Rathole runtime");
            runtime.block_on(async move {
                if let Err(err) = rathole::run(cli, shutdown_rx).await {
                    error!(?err, "Rathole terminated with error");
                }
            });
        })?;

    Ok(RatholeHandle {
        shutdown: shutdown_tx,
        join: Some(handle),
    })
}
