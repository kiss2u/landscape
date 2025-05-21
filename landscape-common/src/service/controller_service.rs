use std::collections::HashMap;

use crate::config::FlowId;
use crate::database::repository::Repository;
use crate::database::LandscapeDBFlowFilterExpr;
use crate::database::LandscapeDBTrait;
use crate::database::LandscapeFlowTrait;
use crate::database::LandscapeServiceDBTrait;

use super::{
    service_code::WatchService,
    service_manager::{ServiceHandler, ServiceManager},
};

#[async_trait::async_trait]
pub trait ControllerService {
    type Id: ToString + Clone + Send;
    type Config: Send + Sync + Clone;
    type DatabseAction: LandscapeServiceDBTrait<Data = Self::Config, Id = Self::Id> + Send;
    type H: ServiceHandler<Config = Self::Config>;

    fn get_service(&self) -> &ServiceManager<Self::H>;
    fn get_repository(&self) -> &Self::DatabseAction;

    /// 获得所有服务状态
    async fn get_all_status(
        &self,
    ) -> HashMap<String, WatchService<<Self::H as ServiceHandler>::Status>> {
        self.get_service().get_all_status().await
    }

    async fn handle_service_config(&self, config: Self::Config) {
        if let Ok(()) = self.get_service().update_service(config.clone()).await {
            self.get_repository().set(config).await.unwrap();
        }
    }

    async fn delete_and_stop_iface_service(
        &self,
        iface_name: Self::Id,
    ) -> Option<WatchService<<Self::H as ServiceHandler>::Status>> {
        self.get_repository().delete(iface_name.clone()).await.unwrap();
        self.get_service().stop_service(iface_name.to_string()).await
    }

    async fn get_config_by_name(&self, iface_name: Self::Id) -> Option<Self::Config> {
        self.get_repository().find_by_iface_name(iface_name).await.unwrap()
    }
}

#[async_trait::async_trait]
pub trait ConfigController {
    type Id: Clone + Send;
    type Config: Send + Sync + Clone;
    type DatabseAction: LandscapeDBTrait<Data = Self::Config, Id = Self::Id> + Send;

    fn get_repository(&self) -> &Self::DatabseAction;

    async fn set(&self, config: Self::Config) -> Self::Config {
        self.get_repository().set(config).await.unwrap()
    }

    async fn list(&self) -> Vec<Self::Config> {
        self.get_repository().list().await.unwrap()
    }

    async fn find_by_id(&self, id: Self::Id) -> Option<Self::Config> {
        self.get_repository().find_by_id(id).await.ok()?
    }

    async fn delete(&self, id: Self::Id) {
        self.get_repository().delete(id).await.unwrap()
    }
}

#[async_trait::async_trait]
pub trait FlowConfigController: ConfigController
where
    Self::DatabseAction: LandscapeFlowTrait,
    <<Self as ConfigController>::DatabseAction as Repository>::Model: LandscapeDBFlowFilterExpr,
{
    async fn list_flow_configs(&self, id: FlowId) -> Vec<Self::Config> {
        self.get_repository().find_by_flow_id(id).await.unwrap()
    }
}

// /// 与 Flow 相关的配置
// pub trait LandscapeFlowConfig {
//     fn get_flow_id(&self) -> FlowId;
// }
