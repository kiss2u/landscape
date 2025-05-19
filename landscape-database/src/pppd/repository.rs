use landscape_common::{
    config::ppp::PPPDServiceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
    error::LdError,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use super::entity::{
    Column, PPPDServiceConfigActiveModel, PPPDServiceConfigEntity, PPPDServiceConfigModel,
};

#[derive(Clone)]
pub struct PPPDServiceRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for PPPDServiceRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for PPPDServiceRepository {}

impl PPPDServiceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_pppd_configs_by_attach_iface_name(
        &self,
        attach_name: String,
    ) -> Result<Vec<PPPDServiceConfig>, LdError> {
        let all = PPPDServiceConfigEntity::find()
            .filter(Column::AttachIfaceName.eq(attach_name))
            .all(self.db())
            .await?;
        Ok(all.into_iter().map(PPPDServiceConfig::from).collect())
    }
}

#[async_trait::async_trait]
impl Repository for PPPDServiceRepository {
    type Model = PPPDServiceConfigModel;
    type Entity = PPPDServiceConfigEntity;
    type ActiveModel = PPPDServiceConfigActiveModel;
    type Data = PPPDServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
