use landscape_common::{
    config::iface_ip::IfaceIpServiceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

use super::entity::{
    IfaceIpServiceConfigActiveModel, IfaceIpServiceConfigEntity, IfaceIpServiceConfigModel,
};

#[derive(Clone)]
pub struct IfaceIpServiceRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for IfaceIpServiceRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for IfaceIpServiceRepository {}

impl IfaceIpServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for IfaceIpServiceRepository {
    type Model = IfaceIpServiceConfigModel;
    type Entity = IfaceIpServiceConfigEntity;
    type ActiveModel = IfaceIpServiceConfigActiveModel;
    type Data = IfaceIpServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
