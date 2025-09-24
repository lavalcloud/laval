use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use laval_model::PortMappingSpec;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ManagerConfig {
    #[serde(default)]
    pub nodes: HashMap<String, NodeRecord>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NodeRecord {
    pub name: String,
    pub reverse_proxy_bind: Option<String>,
    pub port_mapping_role: Option<String>,
    pub management_url: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub port_mapping: Option<PortMappingSpec>,
}

impl ManagerConfig {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read manager config at {}", path.display()))?;
        let cfg = toml::from_str(&raw).context("invalid manager configuration")?;
        Ok(cfg)
    }

    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, content)
            .await
            .with_context(|| format!("failed to write manager config at {}", path.display()))
    }
}

pub struct ManagerState {
    path: PathBuf,
    nodes: RwLock<HashMap<String, NodeRecord>>,
}

impl ManagerState {
    pub async fn initialize(path: PathBuf) -> Result<Self> {
        let config = ManagerConfig::load(&path).await?;
        Ok(Self {
            path,
            nodes: RwLock::new(config.nodes),
        })
    }

    pub fn list(&self) -> Vec<NodeRecord> {
        self.nodes.read().values().cloned().collect::<Vec<_>>()
    }

    pub fn get(&self, name: &str) -> Option<NodeRecord> {
        self.nodes.read().get(name).cloned()
    }

    pub async fn upsert(&self, node: NodeRecord) -> Result<()> {
        self.nodes.write().insert(node.name.clone(), node);
        self.persist().await
    }

    pub async fn remove(&self, name: &str) -> Result<bool> {
        let removed = self.nodes.write().remove(name).is_some();
        if removed {
            self.persist().await?;
        }
        Ok(removed)
    }

    async fn persist(&self) -> Result<()> {
        let snapshot = self.nodes.read().clone();
        let cfg = ManagerConfig { nodes: snapshot };
        cfg.save(&self.path).await
    }
}
