use landscape_common::{
    config::route_wan::RouteWanServiceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

use super::entity::{
    RouteWanServiceConfigActiveModel, RouteWanServiceConfigEntity, RouteWanServiceConfigModel,
};

#[derive(Clone)]
pub struct RouteWanServiceRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for RouteWanServiceRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for RouteWanServiceRepository {}

impl RouteWanServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for RouteWanServiceRepository {
    type Model = RouteWanServiceConfigModel;
    type Entity = RouteWanServiceConfigEntity;
    type ActiveModel = RouteWanServiceConfigActiveModel;
    type Data = RouteWanServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
