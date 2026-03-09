use landscape_common::iface::nat::StaticNatMappingConfig;
use sea_orm::DatabaseConnection;

use super::entity::{
    StaticNatMappingConfigActiveModel, StaticNatMappingConfigEntity, StaticNatMappingConfigModel,
};
use crate::DBId;

#[derive(Clone)]
pub struct StaticNatMappingConfigRepository {
    db: DatabaseConnection,
}

impl StaticNatMappingConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

crate::impl_repository!(
    StaticNatMappingConfigRepository,
    StaticNatMappingConfigModel,
    StaticNatMappingConfigEntity,
    StaticNatMappingConfigActiveModel,
    StaticNatMappingConfig,
    DBId
);
