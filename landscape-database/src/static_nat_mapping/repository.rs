use landscape_common::{
    config::nat::StaticNatMappingConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

use crate::DBId;

use super::entity::{
    StaticNatMappingConfigActiveModel, StaticNatMappingConfigEntity, StaticNatMappingConfigModel,
};

#[derive(Clone)]
pub struct StaticNatMappingConfigRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for StaticNatMappingConfigRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for StaticNatMappingConfigRepository {}

impl StaticNatMappingConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for StaticNatMappingConfigRepository {
    type Model = StaticNatMappingConfigModel;
    type Entity = StaticNatMappingConfigEntity;
    type ActiveModel = StaticNatMappingConfigActiveModel;
    type Data = StaticNatMappingConfig;
    type Id = DBId;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
