use landscape_common::{
    config::dhcp_v4_server::DHCPv4ServiceConfig,
    service::{
        dhcp::{DHCPv4ServiceStatus, DHCPv4ServiceWatchStatus},
        service_manager::ServiceHandler,
    },
};

use crate::iface::get_iface_by_name;

#[derive(Clone)]
pub struct DHCPv4Service;

impl ServiceHandler for DHCPv4Service {
    type Status = DHCPv4ServiceStatus;
    type Config = DHCPv4ServiceConfig;

    async fn initialize(config: DHCPv4ServiceConfig) -> DHCPv4ServiceWatchStatus {
        let service_status = DHCPv4ServiceWatchStatus::new();

        if config.enable {
            if let Some(_) = get_iface_by_name(&config.iface_name).await {
                let status = service_status.clone();
                tokio::spawn(async move {
                    crate::dhcp_server::dhcp_server_new::dhcp_v4_server(
                        config.iface_name,
                        config.config,
                        status,
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
