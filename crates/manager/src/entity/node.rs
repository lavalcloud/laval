use sea_orm::entity::prelude::*;
use sea_orm::JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "nodes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    pub reverse_proxy_bind: Option<String>,
    pub port_mapping_role: Option<String>,
    pub management_url: Option<String>,
    pub description: Option<String>,
    pub tags: Option<JsonValue>,
    pub port_mapping: Option<JsonValue>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
