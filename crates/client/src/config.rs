use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use laval_model::PortMappingSpec;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ClientConfig {
    pub port_mapping: PortMappingSpec,
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
