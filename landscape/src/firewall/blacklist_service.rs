use landscape_common::{
    event::dns::DstIpEvent, firewall::blacklist::FirewallBlacklistConfig,
    service::controller::ConfigController,
};
use landscape_database::{
    firewall_blacklist::repository::FirewallBlacklistRepository,
    provider::LandscapeDBServiceProvider,
};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::firewall::blacklist::resolve_and_sync_blacklist;

use crate::geo::ip_service::GeoIpService;

#[derive(Clone)]
pub struct FirewallBlacklistService {
    store: FirewallBlacklistRepository,
    geo_ip_service: GeoIpService,
}

impl FirewallBlacklistService {
    pub async fn new(
        store: LandscapeDBServiceProvider,
        geo_ip_service: GeoIpService,
        mut receiver: broadcast::Receiver<DstIpEvent>,
    ) -> Self {
        let store = store.firewall_blacklist_store();
        let service = Self { store, geo_ip_service };

        // Initial full sync
        let configs = service.list().await;
        resolve_and_sync_blacklist(&service.geo_ip_service, configs, vec![]).await;

        // Listen for GeoIP update events
        let service_clone = service.clone();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                match event {
                    DstIpEvent::GeoIpUpdated => {
                        tracing::info!("refresh firewall blacklist due to GeoIP update");
                        let configs = service_clone.list().await;
                        resolve_and_sync_blacklist(&service_clone.geo_ip_service, configs, vec![])
                            .await;
                    }
                }
            }
        });

        service
    }
}

#[async_trait::async_trait]
impl ConfigController for FirewallBlacklistService {
    type Id = Uuid;
    type Config = FirewallBlacklistConfig;
    type DatabseAction = FirewallBlacklistRepository;

    fn get_repository(&self) -> &Self::DatabseAction {
        &self.store
    }

    async fn after_update_config(
        &self,
        new_configs: Vec<Self::Config>,
        old_configs: Vec<Self::Config>,
    ) {
        resolve_and_sync_blacklist(&self.geo_ip_service, new_configs, old_configs).await;
    }
}
