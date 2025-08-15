use landscape_common::config::nat::StaticNatMappingConfig;
use landscape_common::database::repository::UpdateActiveModel;
use sea_orm::{entity::prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};

use crate::DBId;
use crate::DBTimestamp;

pub type StaticNatMappingConfigModel = Model;
pub type StaticNatMappingConfigEntity = Entity;
pub type StaticNatMappingConfigActiveModel = ActiveModel;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "static_nat_mapping_configs")]
#[cfg_attr(feature = "postgres", sea_orm(schema_name = "public"))]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: DBId,
    /// Whether this mapping is enabled
    pub enable: bool,

    pub remark: String,

    /// External (WAN) port for the NAT rule
    pub wan_port: u16,

    /// Optional name of the WAN interface this rule applies to
    pub wan_iface_name: Option<String>,

    /// Internal (LAN) port to which traffic will be forwarded
    pub lan_port: u16,

    /// Internal IP address to forward traffic to
    /// If set to `UNSPECIFIED` (0.0.0.0 or ::), the mapping targets the router itself
    pub lan_ip: String,

    /// Layer 4 protocol (TCP / UDP)
    #[sea_orm(column_name = "l4_protocol")]
    pub l4_protocol: u8,

    /// Last update timestamp
    pub update_at: DBTimestamp,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for StaticNatMappingConfig {
    fn from(model: Model) -> Self {
        StaticNatMappingConfig {
            id: model.id,
            enable: model.enable,
            remark: model.remark,
            wan_port: model.wan_port,
            wan_iface_name: model.wan_iface_name,
            lan_port: model.lan_port,
            lan_ip: model.lan_ip.parse().unwrap(),
            l4_protocol: model.l4_protocol,
            update_at: model.update_at,
        }
    }
}

impl Into<ActiveModel> for StaticNatMappingConfig {
    fn into(self) -> ActiveModel {
        let mut active = ActiveModel { id: Set(self.id), ..Default::default() };
        self.update(&mut active);
        active
    }
}

impl UpdateActiveModel<ActiveModel> for StaticNatMappingConfig {
    fn update(self, active: &mut ActiveModel) {
        active.enable = Set(self.enable);
        active.remark = Set(self.remark);
        active.wan_port = Set(self.wan_port);
        active.wan_iface_name = Set(self.wan_iface_name);
        active.lan_port = Set(self.lan_port);
        active.lan_ip = Set(self.lan_ip.to_string());
        active.l4_protocol = Set(self.l4_protocol);
        active.update_at = Set(self.update_at);
    }
}
