use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    #[serde(default)]
    pub reverse_proxy: ReverseProxyConfig,
    #[serde(default)]
    pub port_mapping: Option<PortMappingConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReverseProxyConfig {
    #[serde(default = "default_bind")]
    pub bind: String,
    #[serde(default)]
    pub tls: Option<TlsConfig>,
    #[serde(default)]
    pub routes: HashMap<String, String>,
    #[serde(default)]
    pub default_upstream: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PortMappingConfig {
    /// Path to a Rathole compatible configuration file.
    pub config_path: PathBuf,
    /// When set to true, the node service will treat Rathole as a server
    /// endpoint. Otherwise it will run in client mode.
    #[serde(default = "PortMappingConfig::default_is_server")]
    pub server: bool,
}

impl PortMappingConfig {
    const fn default_is_server() -> bool {
        true
    }
}

impl Default for ReverseProxyConfig {
    fn default() -> Self {
        Self {
            bind: default_bind(),
            tls: None,
            routes: HashMap::new(),
            default_upstream: None,
        }
    }
}

fn default_bind() -> String {
    "0.0.0.0:8443".to_string()
}

impl NodeConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let raw = fs::read_to_string(&path).with_context(|| {
            format!("failed to read node config at {}", path.as_ref().display())
        })?;
        toml::from_str(&raw).context("invalid node configuration")
    }
}
