use anyhow::{anyhow, Result};
use rathole::{config::Config as RatholeConfig, InstanceMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PortMappingMode {
    Server,
    Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMappingSpec {
    pub mode: PortMappingMode,
    pub config: RatholeConfig,
}

impl PortMappingSpec {
    pub fn into_rathole(self) -> Result<(RatholeConfig, InstanceMode)> {
        let mut config = self.config;
        let mode = match self.mode {
            PortMappingMode::Server => {
                if config.server.is_none() {
                    return Err(anyhow!("missing server configuration for port mapping"));
                }
                config.client = None;
                InstanceMode::Server
            }
            PortMappingMode::Client => {
                if config.client.is_none() {
                    return Err(anyhow!("missing client configuration for port mapping"));
                }
                config.server = None;
                InstanceMode::Client
            }
        };

        let config = rathole::sanitize_config(config, mode)?;
        Ok((config, mode))
    }
}
