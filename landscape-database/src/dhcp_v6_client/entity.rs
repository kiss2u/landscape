use landscape_common::{
    config::dhcp_v6_client::{IPV6PDConfig, IPV6PDServiceConfig},
    database::repository::UpdateActiveModel,
};
use sea_orm::{entity::prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};

use crate::DBTimestamp;

pub type DHCPv6ClientConfigModel = Model;
pub type DHCPv6ClientConfigEntity = Entity;
pub type DHCPv6ClientConfigActiveModel = ActiveModel;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "dhcp_v6_client_configs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub iface_name: String,
    pub enable: bool,

    pub mac: String,
    pub update_at: DBTimestamp,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for IPV6PDServiceConfig {
    fn from(entity: Model) -> Self {
        let config = IPV6PDConfig { mac: serde_json::from_str(&entity.mac).unwrap() };
        IPV6PDServiceConfig {
            iface_name: entity.iface_name,
            enable: entity.enable,
            update_at: entity.update_at,
            config,
        }
    }
}

impl Into<ActiveModel> for IPV6PDServiceConfig {
    fn into(self) -> ActiveModel {
        let mut active = ActiveModel {
            iface_name: Set(self.iface_name.clone()),
            ..Default::default()
        };
        self.update(&mut active);
        active
    }
}

impl UpdateActiveModel<ActiveModel> for IPV6PDServiceConfig {
    fn update(self, active: &mut ActiveModel) {
        active.enable = Set(self.enable);
        active.mac = Set(self.config.mac.to_string());
        active.update_at = Set(self.update_at);
    }
}

// pub(crate) fn update(config: IPV6PDServiceConfig, active: &mut ActiveModel) {
//     active.enable = Set(config.enable);
//     active.mac = Set(config.config.mac.to_string());
//     active.update_at = Set(config.update_at);
// }
