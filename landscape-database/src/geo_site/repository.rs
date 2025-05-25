use landscape_common::{
    config::geo::GeoSiteConfig,
    database::{repository::Repository, LandscapeDBTrait},
};
use sea_orm::DatabaseConnection;

use crate::DBId;

use super::entity::{GeoSiteConfigActiveModel, GeoSiteConfigEntity, GeoSiteConfigModel};

#[derive(Clone)]
pub struct GeoSiteConfigRepository {
    db: DatabaseConnection,
}

impl GeoSiteConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl LandscapeDBTrait for GeoSiteConfigRepository {}

#[async_trait::async_trait]
impl Repository for GeoSiteConfigRepository {
    type Model = GeoSiteConfigModel;
    type Entity = GeoSiteConfigEntity;
    type ActiveModel = GeoSiteConfigActiveModel;
    type Data = GeoSiteConfig;
    type Id = DBId;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
