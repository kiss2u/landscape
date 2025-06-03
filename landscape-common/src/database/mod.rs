pub mod repository;

use repository::{LandscapeDBStore, Repository};
use sea_orm::sea_query::SimpleExpr;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;

use crate::{config::FlowId, error::LdError};

/// 基础 Trait
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

/// 基于 Iface 的服务 Trait
#[async_trait::async_trait]
pub trait LandscapeServiceDBTrait: LandscapeDBTrait {
    async fn find_by_iface_name(&self, id: Self::Id) -> Result<Option<Self::Data>, LdError> {
        Ok(self.find_by_id(id).await?)
    }
}
/// 关于 Flow 的 DB Trait
#[async_trait::async_trait]
pub trait LandscapeFlowTrait: LandscapeDBTrait
where
    Self::Model: LandscapeDBFlowFilterExpr,
{
    async fn find_by_flow_id(&self, id: FlowId) -> Result<Vec<Self::Data>, LdError> {
        let models: Vec<Self::Model> = <Self::Entity as EntityTrait>::find()
            .filter(<Self::Model as LandscapeDBFlowFilterExpr>::get_flow_filter(id))
            .all(self.db())
            .await?;
        Ok(models.into_iter().map(From::from).collect())
    }
}

pub trait LandscapeDBFlowFilterExpr {
    fn get_flow_filter(id: FlowId) -> SimpleExpr;
}
