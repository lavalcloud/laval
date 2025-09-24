mod config;
mod proxy;
mod rathole_runner;

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;
use proxy::ReverseProxy;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::config::NodeConfig;

#[derive(Parser, Debug)]
#[command(author, version, about = "Laval edge node service", long_about = None)]
struct Cli {
    /// Path to the node configuration file (TOML format)
    #[arg(long, default_value = "node.toml")]
    config: PathBuf,
}

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();
    let config = NodeConfig::from_file(&cli.config)?;

    let proxy = ReverseProxy::from_config(&config.reverse_proxy)?;
    let rathole = config
        .port_mapping
        .as_ref()
        .map(rathole_runner::spawn_rathole)
        .transpose()?;

    run_proxy_service(&config, proxy)?;

    if let Some(handle) = rathole {
        handle.shutdown();
    }

    Ok(())
}

#[allow(unreachable_code)]
fn run_proxy_service(config: &NodeConfig, proxy: ReverseProxy) -> Result<()> {
    use pingora_core::server::configuration::Opt;
    use pingora_core::server::Server;

    info!("bootstrapping Pingora reverse proxy");

    let mut server = Server::new(Some(Opt::default()))?;
    server.bootstrap();

    let mut service = pingora_proxy::http_proxy_service(&server.configuration, proxy);
    if let Some(tls) = &config.reverse_proxy.tls {
        let cert = tls
            .cert
            .to_str()
            .ok_or_else(|| anyhow!("certificate path contains invalid UTF-8"))?;
        let key = tls
            .key
            .to_str()
            .ok_or_else(|| anyhow!("key path contains invalid UTF-8"))?;
        service.add_tls(&config.reverse_proxy.bind, cert, key)?;
    } else {
        service.add_tcp(&config.reverse_proxy.bind);
    }

    server.add_service(service);
    info!(bind = %config.reverse_proxy.bind, "reverse proxy listening");
    server.run_forever();
    Ok(())
}
