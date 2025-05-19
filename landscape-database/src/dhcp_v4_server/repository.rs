use landscape_common::{
    config::dhcp_v4_server::DHCPv4ServiceConfig,
    database::{LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr};

use crate::repository::Repository;

use super::entity::{
    DHCPv4ServiceConfigActiveModel, DHCPv4ServiceConfigEntity, DHCPv4ServiceConfigModel,
};

#[derive(Clone)]
pub struct DHCPv4ServerRepository {
    db: DatabaseConnection,
}

impl DHCPv4ServerRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
#[async_trait::async_trait]
impl LandscapeServiceDBTrait for DHCPv4ServerRepository {
    async fn find_by_iface_name(
        &self,
        iface_name: String,
    ) -> Result<Option<DHCPv4ServiceConfig>, DbErr> {
        Ok(self.find_by_id(iface_name.to_string()).await?)
    }
}

#[async_trait::async_trait]
impl LandscapeDBTrait for DHCPv4ServerRepository {
    type Data = DHCPv4ServiceConfig;
    type DBErr = DbErr;
    type ID = String;

    async fn list(&self) -> Result<Vec<DHCPv4ServiceConfig>, DbErr> {
        Ok(self.list_all().await?)
    }

    async fn set(&self, config: DHCPv4ServiceConfig) -> Result<DHCPv4ServiceConfig, DbErr> {
        if let Some(data) = self.find_by_id(config.iface_name.clone()).await? {
            let mut d: DHCPv4ServiceConfigActiveModel = data.into();
            super::entity::update(config, &mut d);
            Ok(d.update(&self.db).await?.into())
        } else {
            Ok(self.set_model(config).await?)
        }
    }

    async fn delete(&self, iface_name: String) -> Result<(), DbErr> {
        Ok(self.delete_model(iface_name.to_string()).await?)
    }
}

#[async_trait::async_trait]
impl Repository for DHCPv4ServerRepository {
    type Model = DHCPv4ServiceConfigModel;
    type Entity = DHCPv4ServiceConfigEntity;
    type ActiveModel = DHCPv4ServiceConfigActiveModel;
    type Data = DHCPv4ServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
