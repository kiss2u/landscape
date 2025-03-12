use landscape_common::{
    service::{service_manager::ServiceHandler, DefaultServiceStatus, DefaultWatchServiceStatus},
    store::storev2::LandScapeStore,
};
use serde::{Deserialize, Serialize};

use crate::iface::get_iface_by_name;

/// 控制进行路由通告
#[derive(Clone)]
pub struct IPV6RAService;

impl ServiceHandler for IPV6RAService {
    type Status = DefaultServiceStatus;
    type Config = IPV6RAServiceConfig;

    async fn initialize(config: IPV6RAServiceConfig) -> DefaultWatchServiceStatus {
        let service_status = DefaultWatchServiceStatus::new();
        if config.enable {
            let status_clone = service_status.clone();
            if let Some(iface) = get_iface_by_name(&config.iface_name).await {
                if let Some(mac) = iface.mac {
                    tokio::spawn(async move {
                        let _ = crate::icmp::v6::icmp_ra_server(
                            config.config,
                            mac,
                            config.iface_name,
                            status_clone,
                        )
                        .await;
                    });
                }
            }
        }

        service_status
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPV6RAServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    pub config: IPV6RAConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IPV6RAConfig {
    /// 子网前缀长度, 一般是使用 64
    pub subnet_prefix: u8,
    /// 子网索引
    pub subnet_index: u128,
    /// 当前主机的 mac 地址
    pub depend_iface: String,
    /// 通告 IP 时间
    pub ra_preferred_lifetime: u32,
    pub ra_valid_lifetime: u32,
}

impl LandScapeStore for IPV6RAServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}
