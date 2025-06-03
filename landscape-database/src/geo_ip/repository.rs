use landscape_common::{
    config::geo::GeoIpSourceConfig,
    database::{repository::Repository, LandscapeDBTrait},
    error::LdError,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::DBId;

use super::entity::{
    Column, GeoIpSourceConfigActiveModel, GeoIpSourceConfigEntity, GeoIpSourceConfigModel,
};

#[derive(Clone)]
pub struct GeoIpSourceConfigRepository {
    db: DatabaseConnection,
}

impl GeoIpSourceConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn query_by_name(
        &self,
        name: Option<String>,
    ) -> Result<Vec<GeoIpSourceConfig>, LdError> {
        let result = GeoIpSourceConfigEntity::find()
            .filter(Column::Name.contains(name.unwrap_or("".to_string())))
            .order_by_desc(Column::UpdateAt)
            .all(&self.db)
            .await?;
        Ok(result.into_iter().map(From::from).collect())
    }
}

#[async_trait::async_trait]
impl LandscapeDBTrait for GeoIpSourceConfigRepository {}

#[async_trait::async_trait]
impl Repository for GeoIpSourceConfigRepository {
    type Model = GeoIpSourceConfigModel;
    type Entity = GeoIpSourceConfigEntity;
    type ActiveModel = GeoIpSourceConfigActiveModel;
    type Data = GeoIpSourceConfig;
    type Id = DBId;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
