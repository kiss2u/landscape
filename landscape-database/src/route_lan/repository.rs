use landscape_common::{
    config::route_lan::RouteLanServiceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

use super::entity::{
    RouteLanServiceConfigActiveModel, RouteLanServiceConfigEntity, RouteLanServiceConfigModel,
};

#[derive(Clone)]
pub struct RouteLanServiceRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for RouteLanServiceRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for RouteLanServiceRepository {}

impl RouteLanServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for RouteLanServiceRepository {
    type Model = RouteLanServiceConfigModel;
    type Entity = RouteLanServiceConfigEntity;
    type ActiveModel = RouteLanServiceConfigActiveModel;
    type Data = RouteLanServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
