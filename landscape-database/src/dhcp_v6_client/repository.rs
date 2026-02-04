use landscape_common::{
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
    dhcp::v6_client::config::IPV6PDServiceConfig,
};
use sea_orm::DatabaseConnection;

use super::entity::{
    DHCPv6ClientConfigActiveModel, DHCPv6ClientConfigEntity, DHCPv6ClientConfigModel,
};

#[derive(Clone)]
pub struct DHCPv6ClientRepository {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl LandscapeServiceDBTrait for DHCPv6ClientRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for DHCPv6ClientRepository {}

impl DHCPv6ClientRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // pub async fn list(&self) -> Result<Vec<IPV6PDServiceConfig>, DbErr> {
    //     Ok(self.list_all().await?)
    // }

    // pub async fn find_by_iface_name(
    //     &self,
    //     iface_name: &str,
    // ) -> Result<Option<IPV6PDServiceConfig>, DbErr> {
    //     Ok(self.find_by_id(iface_name.to_string()).await?)
    // }

    // pub async fn set(&self, config: IPV6PDServiceConfig) -> Result<IPV6PDServiceConfig, DbErr> {
    //     if let Some(data) = self.find_by_id(config.iface_name.clone()).await? {
    //         let mut d: DHCPv6ClientConfigActiveModel = data.into();
    //         super::entity::update(config, &mut d);
    //         Ok(d.update(&self.db).await?.into())
    //     } else {
    //         Ok(self.set_model(config).await?)
    //     }
    // }

    // pub async fn delete(&self, iface_name: &str) -> Result<(), DbErr> {
    //     Ok(self.delete_model(iface_name.to_string()).await?)
    // }
}

#[async_trait::async_trait]
impl Repository for DHCPv6ClientRepository {
    type Model = DHCPv6ClientConfigModel;
    type Entity = DHCPv6ClientConfigEntity;
    type ActiveModel = DHCPv6ClientConfigActiveModel;
    type Data = IPV6PDServiceConfig;
    type Id = String;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
