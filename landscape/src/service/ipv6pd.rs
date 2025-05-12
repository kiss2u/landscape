use landscape_common::{
    config::dhcp_v6_client::IPV6PDServiceConfig,
    service::{service_manager::ServiceHandler, DefaultServiceStatus, DefaultWatchServiceStatus},
    LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT,
};

use crate::iface::get_iface_by_name;

#[derive(Clone)]
pub struct IPV6PDService;

impl ServiceHandler for IPV6PDService {
    type Status = DefaultServiceStatus;
    type Config = IPV6PDServiceConfig;

    async fn initialize(config: IPV6PDServiceConfig) -> DefaultWatchServiceStatus {
        let service_status = DefaultWatchServiceStatus::new();
        // service_status.just_change_status(ServiceStatus::Staring);
        if config.enable {
            if let Some(_) = get_iface_by_name(&config.iface_name).await {
                let status_clone = service_status.clone();
                tokio::spawn(async move {
                    crate::dhcp_client::v6::dhcp_v6_pd_client(
                        config.iface_name,
                        config.config.mac,
                        LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT,
                        status_clone,
                    )
                    .await;
                });
            } else {
                tracing::error!("Interface {} not found", config.iface_name);
            }
        }

        service_status
    }
}
