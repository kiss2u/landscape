use std::net::{Ipv4Addr, Ipv6Addr};

use landscape_common::{
    args::LAND_HOSTNAME,
    global_const::default_router::{RouteInfo, RouteType, LD_ALL_ROUTERS},
    iface::IfaceZoneType,
    service::{
        service_manager::ServiceHandler, DefaultServiceStatus, DefaultWatchServiceStatus,
        ServiceStatus,
    },
    store::storev2::LandScapeStore,
};
use serde::{Deserialize, Serialize};

use crate::{
    dev::LandScapeInterface,
    iface::{config::NetworkIfaceConfig, get_iface_by_name},
};

#[derive(Clone)]
pub struct IPConfigService;

impl ServiceHandler for IPConfigService {
    type Status = DefaultServiceStatus;

    type Config = IfaceIpServiceConfig;

    async fn initialize(config: IfaceIpServiceConfig) -> DefaultWatchServiceStatus {
        let service_status = DefaultWatchServiceStatus::new();

        if config.enable {
            if let Some(iface) = get_iface_by_name(&config.iface_name).await {
                let status_clone = service_status.clone();
                tokio::spawn(async move {
                    init_service_from_config(iface, config.ip_model, status_clone).await
                });
            } else {
                tracing::error!("Interface {} not found", config.iface_name);
            }
        }

        service_status
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfaceIpServiceConfig {
    pub iface_name: String,
    pub enable: bool,
    pub ip_model: IfaceIpModelConfig,
}

impl LandScapeStore for IfaceIpServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(tag = "t")]
#[serde(rename_all = "lowercase")]
pub enum IfaceIpModelConfig {
    #[default]
    Nothing,
    Static {
        #[serde(default)]
        default_router_ip: Option<Ipv4Addr>,
        #[serde(default)]
        default_router: bool,
        #[serde(default)]
        ipv4: Option<Ipv4Addr>,
        #[serde(default)]
        ipv4_mask: u8,
        #[serde(default)]
        ipv6: Option<Ipv6Addr>,
    },
    PPPoE {
        #[serde(default)]
        default_router: bool,
        username: String,
        password: String,
        mtu: u32,
    },
    DhcpClient {
        #[serde(default)]
        default_router: bool,
        hostname: Option<String>,
    },
}

impl IfaceIpModelConfig {
    /// 检查当前的 zone 设置是否满足 IP 配置的要求
    pub fn check_iface_status(&self, iface_config: &NetworkIfaceConfig) -> bool {
        match self {
            IfaceIpModelConfig::PPPoE { .. } => {
                matches!(iface_config.zone_type, IfaceZoneType::Wan)
            }
            IfaceIpModelConfig::DhcpClient { .. } => {
                matches!(iface_config.zone_type, IfaceZoneType::Wan)
            }
            _ => true,
        }
    }
}

async fn init_service_from_config(
    iface: LandScapeInterface,
    service_config: IfaceIpModelConfig,
    service_status: DefaultWatchServiceStatus,
) {
    match service_config {
        IfaceIpModelConfig::Nothing => {}
        IfaceIpModelConfig::Static {
            default_router, default_router_ip, ipv4, ipv4_mask, ..
        } => {
            // TODO: IPV6 的设置
            if let Some(ipv4) = ipv4 {
                service_status.just_change_status(ServiceStatus::Staring);
                let iface_name = iface.name;
                tracing::info!("set ipv4 is: {}", ipv4);
                let _ = std::process::Command::new("ip")
                    .args(&["addr", "add", &format!("{}/{}", ipv4, ipv4_mask), "dev", &iface_name])
                    .output();
                tracing::debug!("start setting");
                landscape_ebpf::map_setting::add_wan_ip(iface.index, ipv4);
                if default_router {
                    if let Some(default_router_ip) = default_router_ip {
                        if !default_router_ip.is_broadcast()
                            && !default_router_ip.is_unspecified()
                            && !default_router_ip.is_loopback()
                        {
                            tracing::info!("setting default route: {:?}", default_router_ip);
                            LD_ALL_ROUTERS
                                .add_route(RouteInfo {
                                    iface_name: iface_name.clone(),
                                    weight: 1,
                                    route: RouteType::Ipv4(default_router_ip),
                                })
                                .await;
                        }
                    }
                } else {
                    LD_ALL_ROUTERS.del_route_by_iface(&iface_name).await;
                }

                service_status.just_change_status(ServiceStatus::Running);
                service_status.wait_to_stopping().await;
                let _ = std::process::Command::new("ip")
                    .args(&["addr", "del", &format!("{}/{}", ipv4, ipv4_mask), "dev", &iface_name])
                    .output();

                if default_router {
                    LD_ALL_ROUTERS.del_route_by_iface(&iface_name).await;
                }
                landscape_ebpf::map_setting::del_wan_ip(iface.index);
                service_status.just_change_status(ServiceStatus::Stop);
            }
        }
        IfaceIpModelConfig::PPPoE { username: _, password: _, mtu: _, .. } => {
            // TODO： 重构 PPPoE ebpf 版本
            // if let Some(mac_addr) = iface.mac {
            //     let iface_name = iface.name.clone();
            //     let service_status = ip_config.clone();
            //     crate::pppoe_client::pppoe_client_v2::create_pppoe_client(
            //         iface.index,
            //         iface_name,
            //         mac_addr,
            //         username,
            //         password,
            //         service_status,
            //     )
            //     .await;
            // } else {
            //     ip_config.send_replace(ServiceStatus::Stop {
            //         message: Some("mac addr is empty".into()),
            //     });
            // }
        }
        IfaceIpModelConfig::DhcpClient { default_router, hostname } => {
            if let Some(mac_addr) = iface.mac {
                let hostname =
                    hostname.filter(|h| !h.is_empty()).unwrap_or_else(|| LAND_HOSTNAME.clone());
                crate::dhcp_client::v4::dhcp_v4_client(
                    iface.index,
                    iface.name,
                    mac_addr,
                    68,
                    service_status,
                    hostname,
                    default_router,
                )
                .await;
            } else {
                service_status.just_change_status(ServiceStatus::Stop);
            }
        }
    };
}
