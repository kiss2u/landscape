pub mod repository;

use repository::{LandscapeDBStore, Repository};

use crate::error::LdError;

#[async_trait::async_trait]
pub trait LandscapeDBTrait: Repository {
    async fn set(&self, config: Self::Data) -> Result<Self::Data, LdError> {
        Ok(self.set_or_update_model(config.get_id(), config).await?)
    }

    async fn list(&self) -> Result<Vec<Self::Data>, LdError> {
        Ok(self.list_all().await?)
    }

    async fn delete(&self, id: Self::Id) -> Result<(), LdError> {
        Ok(self.delete_model(id).await?)
    }
}

#[async_trait::async_trait]
pub trait LandscapeServiceDBTrait: LandscapeDBTrait {
    async fn find_by_iface_name(&self, id: Self::Id) -> Result<Option<Self::Data>, LdError> {
        Ok(self.find_by_id(id).await?)
    }
}
