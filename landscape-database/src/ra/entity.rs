use landscape_common::config::ra::IPV6RAServiceConfig;
use landscape_common::database::repository::UpdateActiveModel;
use sea_orm::{entity::prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};

use crate::DBTimestamp;

pub type IPV6RAServiceConfigModel = Model;
pub type IPV6RAServiceConfigEntity = Entity;
pub type IPV6RAServiceConfigActiveModel = ActiveModel;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ipv6_ra_service_configs")]
#[cfg_attr(feature = "postgres", sea_orm(schema_name = "public"))]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub iface_name: String,
    pub enable: bool,

    pub subnet_prefix: u8,
    pub subnet_index: u32,

    pub depend_iface: String,
    pub ra_preferred_lifetime: u32,
    pub ra_valid_lifetime: u32,
    pub ra_flag: u8,

    pub update_at: DBTimestamp,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for IPV6RAServiceConfig {
    fn from(entity: Model) -> Self {
        IPV6RAServiceConfig {
            iface_name: entity.iface_name,
            enable: entity.enable,
            update_at: entity.update_at,

            config: landscape_common::config::ra::IPV6RAConfig {
                subnet_prefix: entity.subnet_prefix,
                subnet_index: entity.subnet_index,
                depend_iface: entity.depend_iface,
                ra_preferred_lifetime: entity.ra_preferred_lifetime,
                ra_valid_lifetime: entity.ra_valid_lifetime,
                ra_flag: (entity.ra_flag).into(),
            },
        }
    }
}

impl Into<ActiveModel> for IPV6RAServiceConfig {
    fn into(self) -> ActiveModel {
        let mut active = ActiveModel {
            iface_name: Set(self.iface_name.clone()),
            ..Default::default()
        };
        self.update(&mut active);
        active
    }
}

impl UpdateActiveModel<ActiveModel> for IPV6RAServiceConfig {
    fn update(self, active: &mut ActiveModel) {
        active.enable = Set(self.enable);
        active.update_at = Set(self.update_at);

        active.subnet_prefix = Set(self.config.subnet_prefix);
        active.subnet_index = Set(self.config.subnet_index);
        active.depend_iface = Set(self.config.depend_iface);
        active.ra_preferred_lifetime = Set(self.config.ra_preferred_lifetime);
        active.ra_valid_lifetime = Set(self.config.ra_valid_lifetime);
        active.ra_flag = Set(self.config.ra_flag.into());
    }
}
