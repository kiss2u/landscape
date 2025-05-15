use landscape_common::{
    config::ra::IPV6RAServiceConfig,
    service::{service_manager::ServiceHandler, DefaultServiceStatus, DefaultWatchServiceStatus},
};

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
