mod config;
mod rathole_runner;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::config::ClientConfig;

#[derive(Parser, Debug)]
#[command(author, version, about = "Laval port mapping client", long_about = None)]
struct Cli {
    /// Path to the client configuration file (TOML format)
    #[arg(long, default_value = "client.toml")]
    config: PathBuf,
}

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();
    let config = ClientConfig::from_file(&cli.config)?;

    let handle = rathole_runner::spawn_rathole(&config.port_mapping)?;

    // Rathole runs until the process receives a termination signal. We simply
    // wait for the thread to finish. The thread owns the tokio runtime and will
    // exit if the runtime shuts down (for example when Ctrl+C is pressed).
    handle.shutdown();

    Ok(())
}
