use landscape_common::{
    config::wifi::WifiServiceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

use super::entity::{
    WifiServiceConfigActiveModel, WifiServiceConfigEntity, WifiServiceConfigModel,
};

#[derive(Clone)]
pub struct WifiServiceRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for WifiServiceRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for WifiServiceRepository {}

impl WifiServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for WifiServiceRepository {
    type Model = WifiServiceConfigModel;
    type Entity = WifiServiceConfigEntity;
    type ActiveModel = WifiServiceConfigActiveModel;
    type Data = WifiServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
