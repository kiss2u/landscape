use landscape_common::{
    config::iface::NetworkIfaceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

use super::entity::{NetIfaceConfigActiveModel, NetIfaceConfigEntity, NetIfaceConfigModel};

#[derive(Clone)]
pub struct NetIfaceRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for NetIfaceRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for NetIfaceRepository {}

impl NetIfaceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for NetIfaceRepository {
    type Model = NetIfaceConfigModel;
    type Entity = NetIfaceConfigEntity;
    type ActiveModel = NetIfaceConfigActiveModel;
    type Data = NetworkIfaceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
