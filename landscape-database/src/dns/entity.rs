use landscape_common::config::dns::DNSRuleConfig;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{DBId, DBJson, DBTimestamp};

pub type DNSRuleConfigModel = Model;
pub type DNSRuleConfigEntity = Entity;
pub type DNSRuleConfigActiveModel = ActiveModel;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "dns_rule_configs")]
#[cfg_attr(feature = "postgres", sea_orm(schema_name = "public"))]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    /// 主键 ID
    pub id: DBId,
    pub index: u32,
    pub name: String,
    pub enable: bool,
    pub filter: DBJson,
    pub resolve_mode: DBJson,
    pub mark: u32,
    /// 虽然是 JSON 但是考虑到可能存储较多信息
    #[sea_orm(column_type = "Text")]
    pub source: String,
    pub flow_id: u32,
    pub update_at: DBTimestamp,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert && self.id.is_not_set() {
            self.id = sea_orm::ActiveValue::Set(Uuid::new_v4());
        }
        Ok(self)
    }
}

impl From<Model> for DNSRuleConfig {
    fn from(entity: Model) -> Self {
        DNSRuleConfig {
            id: Some(entity.id),
            name: entity.name,
            index: entity.index,
            enable: entity.enable,
            filter: serde_json::from_value(entity.filter).unwrap(),
            resolve_mode: serde_json::from_value(entity.resolve_mode).unwrap(),
            mark: entity.mark.into(),
            source: serde_json::from_str(&entity.source).unwrap(),
            flow_id: entity.flow_id,
        }
    }
}
