use landscape_common::{
    config::flow::FlowWanServiceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

use super::entity::{
    FlowWanServiceConfigActiveModel, FlowWanServiceConfigEntity, FlowWanServiceConfigModel,
};

#[derive(Clone)]
pub struct FlowWanServiceRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for FlowWanServiceRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for FlowWanServiceRepository {}

impl FlowWanServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for FlowWanServiceRepository {
    type Model = FlowWanServiceConfigModel;
    type Entity = FlowWanServiceConfigEntity;
    type ActiveModel = FlowWanServiceConfigActiveModel;
    type Data = FlowWanServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
