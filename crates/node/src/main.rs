mod config;
mod proxy;
mod rathole_runner;

use std::convert::TryFrom;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use proxy::ReverseProxy;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::config::{ManagerLinkConfig, NodeConfig};
use laval_model::{PortMappingMode, PortMappingSpec};

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
    let port_mapping = load_port_mapping(&config)?;
    let rathole = port_mapping
        .as_ref()
        .map(rathole_runner::spawn_rathole)
        .transpose()?;

    run_proxy_service(&config, proxy)?;

    if let Some(handle) = rathole {
        handle.shutdown();
    }

    Ok(())
}

fn load_port_mapping(config: &NodeConfig) -> Result<Option<PortMappingSpec>> {
    let mut spec = config.port_mapping.clone();

    if let Some(manager) = &config.manager {
        match fetch_port_mapping_from_manager(manager)? {
            Some(remote) => {
                info!(
                    endpoint = %manager.endpoint,
                    node = %manager.node_name,
                    "loaded port mapping from manager",
                );
                spec = Some(remote);
            }
            None => {
                info!(
                    endpoint = %manager.endpoint,
                    node = %manager.node_name,
                    "manager did not provide port mapping configuration",
                );
            }
        }
    }

    Ok(spec)
}

fn fetch_port_mapping_from_manager(manager: &ManagerLinkConfig) -> Result<Option<PortMappingSpec>> {
    use laval_proto::manager::v1::{
        node_manager_client::NodeManagerClient, GetNodeConfigRequest, PortMappingMode as ProtoMode,
    };

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        let mut client = NodeManagerClient::connect(manager.endpoint.clone()).await?;
        let response = client
            .get_node_config(GetNodeConfigRequest {
                name: manager.node_name.clone(),
            })
            .await?
            .into_inner();

        if let Some(port_mapping) = response.port_mapping {
            let mode = ProtoMode::try_from(port_mapping.mode)
                .map_err(|_| anyhow!("unknown port mapping mode from manager"))?;
            let mode = match mode {
                ProtoMode::Server => PortMappingMode::Server,
                ProtoMode::Client => PortMappingMode::Client,
                ProtoMode::Unspecified => {
                    return Err(anyhow!("manager returned unspecified port mapping mode"))
                }
            };

            let config = serde_json::from_str(&port_mapping.config_json)
                .with_context(|| "failed to parse port mapping configuration from manager")?;

            Ok(Some(PortMappingSpec { mode, config }))
        } else {
            Ok(None)
        }
    })
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
