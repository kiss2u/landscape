use landscape_common::{
    service::{service_manager::ServiceHandler, DefaultServiceStatus, DefaultWatchServiceStatus},
    store::storev2::LandScapeStore,
};
use serde::{Deserialize, Serialize};

use crate::{iface::get_iface_by_name, pppd_client::PPPDConfig};

#[derive(Clone)]
pub struct PPPDService;

impl ServiceHandler for PPPDService {
    type Status = DefaultServiceStatus;
    type Config = PPPDServiceConfig;

    async fn initialize(config: PPPDServiceConfig) -> DefaultWatchServiceStatus {
        let service_status = DefaultWatchServiceStatus::new();
        if config.enable {
            if let Some(_) = get_iface_by_name(&config.attach_iface_name).await {
                let status_clone = service_status.clone();

                tokio::spawn(async move {
                    crate::pppd_client::pppd::create_pppd_thread(
                        config.attach_iface_name,
                        config.iface_name,
                        config.pppd_config,
                        status_clone,
                    )
                    .await
                });
            } else {
                tracing::error!("Interface {} not found", config.iface_name);
            }
        }

        service_status
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PPPDServiceConfig {
    pub attach_iface_name: String,
    pub iface_name: String,
    pub enable: bool,
    pub pppd_config: PPPDConfig,
}

impl LandScapeStore for PPPDServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}
