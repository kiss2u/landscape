use landscape_common::config::nat::StaticNatMappingConfig;
use landscape_common::database::repository::UpdateActiveModel;
use sea_orm::{entity::prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};

use crate::DBId;
use crate::DBJson;
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

    /// Optional name of the WAN interface this rule applies to
    pub wan_iface_name: Option<String>,

    /// Port Pair for the NAT rule
    pub mapping_pair_ports: DBJson,

    /// Internal IP address to forward traffic to
    /// If set to `UNSPECIFIED` (0.0.0.0 or ::), the mapping targets the router itself
    #[sea_orm(column_name = "lan_ipv4")]
    pub lan_ipv4: Option<String>,

    #[sea_orm(column_name = "lan_ipv6")]
    pub lan_ipv6: Option<String>,

    /// Ipv4 Layer 4 protocol (TCP / UDP)
    #[sea_orm(column_name = "ipv4_l4_protocol")]
    pub ipv4_l4_protocol: DBJson,

    /// Ipv6 Layer 4 protocol (TCP / UDP)
    #[sea_orm(column_name = "ipv6_l4_protocol")]
    pub ipv6_l4_protocol: DBJson,

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
            mapping_pair_ports: serde_json::from_value(model.mapping_pair_ports).unwrap(),
            wan_iface_name: model.wan_iface_name,
            lan_ipv4: model.lan_ipv4.map(|e| e.parse().ok()).unwrap_or(None),
            lan_ipv6: model.lan_ipv6.map(|e| e.parse().ok()).unwrap_or(None),
            ipv4_l4_protocol: serde_json::from_value(model.ipv4_l4_protocol).unwrap(),
            ipv6_l4_protocol: serde_json::from_value(model.ipv6_l4_protocol).unwrap(),
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
        active.wan_iface_name = Set(self.wan_iface_name);
        active.mapping_pair_ports = Set(serde_json::to_value(&self.mapping_pair_ports).unwrap());

        active.lan_ipv4 = Set(self.lan_ipv4.map(|ip| ip.to_string()));
        active.lan_ipv6 = Set(self.lan_ipv6.map(|ip| ip.to_string()));
        active.ipv4_l4_protocol = Set(serde_json::to_value(&self.ipv4_l4_protocol).unwrap());
        active.ipv6_l4_protocol = Set(serde_json::to_value(&self.ipv6_l4_protocol).unwrap());
        active.update_at = Set(self.update_at);
    }
}
