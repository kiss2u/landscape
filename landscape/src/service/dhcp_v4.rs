use landscape_common::{
    dhcp::DHCPv4ServerConfig,
    service::{
        dhcp::{DHCPv4ServiceStatus, DHCPv4ServiceWatchStatus},
        service_manager::ServiceHandler,
    },
    store::storev2::LandScapeStore,
    LANDSCAPE_DEFAULT_LAN_NAME,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DHCPv4Service;

impl ServiceHandler for DHCPv4Service {
    type Status = DHCPv4ServiceStatus;
    type Config = DHCPv4ServiceConfig;

    async fn initialize(config: DHCPv4ServiceConfig) -> DHCPv4ServiceWatchStatus {
        let service_status = DHCPv4ServiceWatchStatus::new();

        if config.enable {
            let status = service_status.clone();
            tokio::spawn(async move {
                crate::dhcp_server::dhcp_server_new::dhcp_v4_server(
                    config.iface_name,
                    config.config,
                    status,
                )
                .await;
            });
        }

        service_status
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DHCPv4ServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    #[serde(default)]
    pub config: DHCPv4ServerConfig,
}

impl Default for DHCPv4ServiceConfig {
    fn default() -> Self {
        Self {
            iface_name: LANDSCAPE_DEFAULT_LAN_NAME.into(),
            enable: true,
            config: DHCPv4ServerConfig::default(),
        }
    }
}

impl LandScapeStore for DHCPv4ServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}
