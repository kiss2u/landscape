use landscape_common::{
    config::ra::IPV6RAServiceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

use super::entity::{
    IPV6RAServiceConfigActiveModel, IPV6RAServiceConfigEntity, IPV6RAServiceConfigModel,
};

#[derive(Clone)]
pub struct IPV6RAServiceRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for IPV6RAServiceRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for IPV6RAServiceRepository {}

impl IPV6RAServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for IPV6RAServiceRepository {
    type Model = IPV6RAServiceConfigModel;
    type Entity = IPV6RAServiceConfigEntity;
    type ActiveModel = IPV6RAServiceConfigActiveModel;
    type Data = IPV6RAServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
