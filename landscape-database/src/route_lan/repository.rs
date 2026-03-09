use landscape_common::route::lan::RouteLanServiceConfig;
use sea_orm::DatabaseConnection;

use super::entity::{
    RouteLanServiceConfigActiveModel, RouteLanServiceConfigEntity, RouteLanServiceConfigModel,
};

#[derive(Clone)]
pub struct RouteLanServiceRepository {
    db: DatabaseConnection,
}

impl RouteLanServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

crate::impl_repository!(
    RouteLanServiceRepository,
    RouteLanServiceConfigModel,
    RouteLanServiceConfigEntity,
    RouteLanServiceConfigActiveModel,
    RouteLanServiceConfig,
    String
);
