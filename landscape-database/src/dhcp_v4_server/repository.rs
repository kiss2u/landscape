use landscape_common::{
    dhcp::v4_server::config::DHCPv4ServiceConfig,
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
};
use sea_orm::DatabaseConnection;

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
impl LandscapeServiceDBTrait for DHCPv4ServerRepository {}

#[async_trait::async_trait]
impl LandscapeDBTrait for DHCPv4ServerRepository {}

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

// impl LandscapeDBStore<String> for DHCPv4ServiceConfig {
//     fn get_id(&self) -> String {
//         self.iface_name.clone()
//     }
// }
