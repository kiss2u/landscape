use landscape_common::{
    config::firewall::FirewallServiceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

use super::entity::{
    FirewallServiceConfigActiveModel, FirewallServiceConfigEntity, FirewallServiceConfigModel,
};

#[derive(Clone)]
pub struct FirewallServiceRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for FirewallServiceRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for FirewallServiceRepository {}

impl FirewallServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for FirewallServiceRepository {
    type Model = FirewallServiceConfigModel;
    type Entity = FirewallServiceConfigEntity;
    type ActiveModel = FirewallServiceConfigActiveModel;
    type Data = FirewallServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
