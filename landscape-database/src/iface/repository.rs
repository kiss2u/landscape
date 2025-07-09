use landscape_common::{
    config::iface::{IfaceZoneType, NetworkIfaceConfig},
    database::{repository::Repository, LandscapeDBTrait, LandscapeServiceDBTrait},
    error::LdError,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::iface::entity::Column;

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

    pub async fn get_all_wan_iface(&self) -> Result<Vec<NetworkIfaceConfig>, LdError> {
        let result = NetIfaceConfigEntity::find()
            .filter(Column::ZoneType.eq(IfaceZoneType::Wan))
            .all(&self.db)
            .await?;

        Ok(result.into_iter().map(From::from).collect())
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
