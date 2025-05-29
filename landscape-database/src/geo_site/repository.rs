use landscape_common::{
    config::geo::GeoSiteConfig,
    database::{repository::Repository, LandscapeDBTrait},
    error::LdError,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::DBId;

use super::entity::{Column, GeoSiteConfigActiveModel, GeoSiteConfigEntity, GeoSiteConfigModel};

#[derive(Clone)]
pub struct GeoSiteConfigRepository {
    db: DatabaseConnection,
}

impl GeoSiteConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn query_by_name(&self, name: Option<String>) -> Result<Vec<GeoSiteConfig>, LdError> {
        let result = GeoSiteConfigEntity::find()
            .filter(Column::Name.contains(name.unwrap_or("".to_string())))
            .order_by_desc(Column::UpdateAt)
            .all(&self.db)
            .await?;
        Ok(result.into_iter().map(From::from).collect())
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
