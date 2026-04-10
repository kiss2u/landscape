use landscape_common::ipv6::lan::LanIPv6ServiceConfigV2;
use sea_orm::DatabaseConnection;

use super::entity::{
    LanIPv6ServiceConfigV2ActiveModel, LanIPv6ServiceConfigV2Entity, LanIPv6ServiceConfigV2Model,
};

#[derive(Clone)]
pub struct LanIPv6V2ServiceRepository {
    db: DatabaseConnection,
}

impl LanIPv6V2ServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

crate::impl_repository!(
    LanIPv6V2ServiceRepository,
    LanIPv6ServiceConfigV2Model,
    LanIPv6ServiceConfigV2Entity,
    LanIPv6ServiceConfigV2ActiveModel,
    LanIPv6ServiceConfigV2,
    String
);
