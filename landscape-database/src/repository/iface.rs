use landscape_common::config::iface::NetworkIfaceConfig;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::entity::iface::NetIfaceConfigEntity;

pub struct NetIfaceRepository {
    db: DatabaseConnection,
}

impl NetIfaceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn truncate(&self) -> Result<(), DbErr> {
        NetIfaceConfigEntity::delete_many().exec(&self.db).await?;
        Ok(())
    }

    pub async fn find_by_name(&self, name: String) -> Result<Option<NetworkIfaceConfig>, DbErr> {
        Ok(NetIfaceConfigEntity::find_by_id(name)
            .one(&self.db)
            .await?
            .map(|model| NetworkIfaceConfig::from(model)))
    }

    pub async fn set(&self, _config: NetworkIfaceConfig) -> Result<NetworkIfaceConfig, DbErr> {
        todo!()
    }
}
