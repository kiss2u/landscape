use std::collections::HashMap;

use crate::database::LandscapeDBTrait;
use crate::database::LandscapeServiceDBTrait;

use super::{
    service_code::WatchService,
    service_manager::{ServiceHandler, ServiceManager},
};

#[async_trait::async_trait]
pub trait ControllerService {
    type ID: ToString + Clone + Send;
    type Config: Send + Sync + Clone;
    type DatabseAction: LandscapeServiceDBTrait<Data = Self::Config, ID = Self::ID> + Send;
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
        iface_name: Self::ID,
    ) -> Option<WatchService<<Self::H as ServiceHandler>::Status>> {
        self.get_repository().delete(iface_name.clone()).await.unwrap();
        self.get_service().stop_service(iface_name.to_string()).await
    }

    async fn get_config_by_name(&self, iface_name: Self::ID) -> Option<Self::Config> {
        self.get_repository().find_by_iface_name(iface_name).await.unwrap()
    }
}
