use landscape_common::{config::geo::GeoIpSourceConfig, database::repository::UpdateActiveModel};
use sea_orm::{entity::prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};

use crate::{DBId, DBTimestamp};

pub type GeoIpSourceConfigModel = Model;
pub type GeoIpSourceConfigEntity = Entity;
pub type GeoIpSourceConfigActiveModel = ActiveModel;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "geo_ip_configs")]
#[cfg_attr(feature = "postgres", sea_orm(schema_name = "public"))]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: DBId,
    pub update_at: DBTimestamp,
    pub url: String,
    pub name: String,
    pub enable: bool,
    pub next_update_at: DBTimestamp,
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
            self.id = Set(Uuid::new_v4());
        }
        Ok(self)
    }
}

impl From<Model> for GeoIpSourceConfig {
    fn from(entity: Model) -> Self {
        GeoIpSourceConfig {
            id: Some(entity.id),
            update_at: entity.update_at,
            url: entity.url,
            name: entity.name,
            enable: entity.enable,
            next_update_at: entity.next_update_at,
        }
    }
}

impl Into<ActiveModel> for GeoIpSourceConfig {
    fn into(self) -> ActiveModel {
        let mut active = ActiveModel {
            id: Set(self.id.unwrap_or_else(Uuid::new_v4)),
            ..Default::default()
        };
        self.update(&mut active);
        active
    }
}

impl UpdateActiveModel<ActiveModel> for GeoIpSourceConfig {
    fn update(self, active: &mut ActiveModel) {
        active.update_at = Set(self.update_at);
        active.url = Set(self.url);
        active.name = Set(self.name);
        active.enable = Set(self.enable);
        active.next_update_at = Set(self.next_update_at);
    }
}
