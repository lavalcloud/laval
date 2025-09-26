use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use laval_model::PortMappingSpec;
use sea_orm::sea_query::TableCreateStatement;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, Schema, Set,
};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::entity::node;

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
}

pub struct ManagerState {
    db: DatabaseConnection,
}

impl ManagerState {
    pub async fn initialize(path: PathBuf, database_url: String) -> Result<Self> {
        let config = ManagerConfig::load(&path).await?;
        let db = Database::connect(&database_url)
            .await
            .with_context(|| format!("failed to connect to database at {database_url}"))?;
        Self::run_migrations(&db).await?;

        let state = Self { db };
        for node in config.nodes.into_values() {
            state.upsert(node).await?;
        }
        Ok(state)
    }

    async fn run_migrations(db: &DatabaseConnection) -> Result<()> {
        let backend = db.get_database_backend();
        let schema = Schema::new(backend);
        let mut table: TableCreateStatement = schema.create_table_from_entity(node::Entity);
        db.execute(backend.build(table.if_not_exists()))
            .await
            .context("failed to run manager migrations")?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<NodeRecord>> {
        let models = node::Entity::find().all(&self.db).await?;
        models
            .into_iter()
            .map(model_to_record)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get(&self, name: &str) -> Result<Option<NodeRecord>> {
        let model = node::Entity::find()
            .filter(node::Column::Name.eq(name))
            .one(&self.db)
            .await?;
        model.map(model_to_record).transpose()
    }

    pub async fn upsert(&self, node: NodeRecord) -> Result<()> {
        let tags_value = if node.tags.is_empty() {
            None
        } else {
            Some(serde_json::to_value(&node.tags)?)
        };
        let port_mapping_value = match &node.port_mapping {
            Some(spec) => Some(serde_json::to_value(spec)?),
            None => None,
        };

        if let Some(existing) = node::Entity::find()
            .filter(node::Column::Name.eq(node.name.clone()))
            .one(&self.db)
            .await?
        {
            let mut active: node::ActiveModel = existing.into_active_model();
            active.reverse_proxy_bind = Set(node.reverse_proxy_bind.clone());
            active.port_mapping_role = Set(node.port_mapping_role.clone());
            active.management_url = Set(node.management_url.clone());
            active.description = Set(node.description.clone());
            active.tags = Set(tags_value.clone());
            active.port_mapping = Set(port_mapping_value.clone());
            active.update(&self.db).await?;
        } else {
            let active = node::ActiveModel {
                name: Set(node.name.clone()),
                reverse_proxy_bind: Set(node.reverse_proxy_bind.clone()),
                port_mapping_role: Set(node.port_mapping_role.clone()),
                management_url: Set(node.management_url.clone()),
                description: Set(node.description.clone()),
                tags: Set(tags_value.clone()),
                port_mapping: Set(port_mapping_value.clone()),
                ..Default::default()
            };
            active.insert(&self.db).await?;
        }

        Ok(())
    }

    pub async fn remove(&self, name: &str) -> Result<bool> {
        let result = node::Entity::delete_many()
            .filter(node::Column::Name.eq(name))
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected > 0)
    }
}

fn model_to_record(model: node::Model) -> Result<NodeRecord> {
    let tags = match model.tags {
        Some(value) => serde_json::from_value(value)?,
        None => Vec::new(),
    };
    let port_mapping = match model.port_mapping {
        Some(value) => Some(serde_json::from_value(value)?),
        None => None,
    };

    Ok(NodeRecord {
        name: model.name,
        reverse_proxy_bind: model.reverse_proxy_bind,
        port_mapping_role: model.port_mapping_role,
        management_url: model.management_url,
        description: model.description,
        tags,
        port_mapping,
    })
}
