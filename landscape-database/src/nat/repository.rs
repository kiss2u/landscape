use landscape_common::{
    config::nat::NatServiceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

use super::entity::{NatServiceConfigActiveModel, NatServiceConfigEntity, NatServiceConfigModel};

#[derive(Clone)]
pub struct NatServiceRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for NatServiceRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for NatServiceRepository {}

impl NatServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for NatServiceRepository {
    type Model = NatServiceConfigModel;
    type Entity = NatServiceConfigEntity;
    type ActiveModel = NatServiceConfigActiveModel;
    type Data = NatServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
