use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ClientConfig {
    pub port_mapping: PortMappingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PortMappingConfig {
    pub config_path: PathBuf,
    #[serde(default = "PortMappingConfig::default_is_server")]
    pub server: bool,
}

impl PortMappingConfig {
    const fn default_is_server() -> bool {
        false
    }
}

impl ClientConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let raw = fs::read_to_string(&path).with_context(|| {
            format!(
                "failed to read client config at {}",
                path.as_ref().display()
            )
        })?;
        toml::from_str(&raw).context("invalid client configuration")
    }
}
