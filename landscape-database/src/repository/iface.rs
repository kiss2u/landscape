use landscape_common::config::iface::NetworkIfaceConfig;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait};

use crate::entity::iface::{NetIfaceConfigActiveModel, NetIfaceConfigEntity};

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

    pub async fn get_by_name(&self, name: &str) -> Result<Option<NetworkIfaceConfig>, DbErr> {
        Ok(NetIfaceConfigEntity::find_by_id(name)
            .one(&self.db)
            .await?
            .map(|model| NetworkIfaceConfig::from(model)))
    }

    pub async fn set(&self, config: NetworkIfaceConfig) -> Result<NetworkIfaceConfig, DbErr> {
        let active_model: NetIfaceConfigActiveModel = config.into();
        let model = active_model.insert(&self.db).await?;
        Ok(NetworkIfaceConfig::from(model))
    }

    pub async fn list(&self) -> Result<Vec<NetworkIfaceConfig>, DbErr> {
        Ok(NetIfaceConfigEntity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|model| NetworkIfaceConfig::from(model))
            .collect())
    }
}
