use landscape_common::{
    config::mss_clamp::MSSClampServiceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

use super::entity::{
    MSSClampServiceConfigActiveModel, MSSClampServiceConfigEntity, MSSClampServiceConfigModel,
};

#[derive(Clone)]
pub struct MssClampServiceRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for MssClampServiceRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for MssClampServiceRepository {}

impl MssClampServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for MssClampServiceRepository {
    type Model = MSSClampServiceConfigModel;
    type Entity = MSSClampServiceConfigEntity;
    type ActiveModel = MSSClampServiceConfigActiveModel;
    type Data = MSSClampServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
