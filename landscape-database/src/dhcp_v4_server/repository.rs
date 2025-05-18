use landscape_common::config::dhcp_v4_server::DHCPv4ServiceConfig;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr};

use crate::repository::Repository;

use super::entity::{
    DHCPv4ServiceConfigActiveModel, DHCPv4ServiceConfigEntity, DHCPv4ServiceConfigModel,
};

pub struct DHCPv4ServerRepository {
    db: DatabaseConnection,
}

impl DHCPv4ServerRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list(&self) -> Result<Vec<DHCPv4ServiceConfig>, DbErr> {
        Ok(self.list_all().await?)
    }

    pub async fn find_by_iface_name(
        &self,
        iface_name: &str,
    ) -> Result<Option<DHCPv4ServiceConfig>, DbErr> {
        Ok(self.find_by_id(iface_name.to_string()).await?)
    }

    pub async fn set(&self, config: DHCPv4ServiceConfig) -> Result<DHCPv4ServiceConfig, DbErr> {
        if let Some(data) = self.find_by_id(config.iface_name.clone()).await? {
            let mut d: DHCPv4ServiceConfigActiveModel = data.into();
            super::entity::update(config, &mut d);
            Ok(d.update(&self.db).await?.into())
        } else {
            Ok(self.set_model(config).await?)
        }
    }

    pub async fn delete(&self, iface_name: &str) -> Result<(), DbErr> {
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
