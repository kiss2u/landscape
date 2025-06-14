use landscape_common::database::LandscapeFlowTrait;
use landscape_common::{
    database::{repository::Repository, LandscapeDBTrait},
    flow::FlowConfig,
};
use sea_orm::DatabaseConnection;

use crate::DBId;

use super::entity::{FlowConfigActiveModel, FlowConfigEntity, FlowConfigModel};

#[derive(Clone)]
pub struct FlowConfigRepository {
    db: DatabaseConnection,
}

impl FlowConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl LandscapeDBTrait for FlowConfigRepository {}

#[async_trait::async_trait]
impl LandscapeFlowTrait for FlowConfigRepository {}

#[async_trait::async_trait]
impl Repository for FlowConfigRepository {
    type Model = FlowConfigModel;
    type Entity = FlowConfigEntity;
    type ActiveModel = FlowConfigActiveModel;
    type Data = FlowConfig;
    type Id = DBId;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
