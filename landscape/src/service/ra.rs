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
    /// RA 通告标识
    #[serde(default = "ra_flag_default")]
    pub ra_flag: RouterFlags,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RouterFlags {
    pub managed_address_config: bool, // 0b1000_0000
    pub other_config: bool,           // 0b0100_0000
    pub home_agent: bool,             // 0b0010_0000
    pub prf: u8,                      // 0b0001_1000 (Default Router Preference)
    pub nd_proxy: bool,               // 0b0000_0100
    pub reserved: u8,                 // 0b0000_0011
}

// 实现 From<u8>，用于从字节转换为结构体
impl From<u8> for RouterFlags {
    fn from(byte: u8) -> Self {
        Self {
            managed_address_config: (byte & 0b1000_0000) != 0,
            other_config: (byte & 0b0100_0000) != 0,
            home_agent: (byte & 0b0010_0000) != 0,
            prf: (byte & 0b0001_1000) >> 3,
            nd_proxy: (byte & 0b0000_0100) != 0,
            reserved: byte & 0b0000_0011,
        }
    }
}

// 实现 Into<u8>，用于将结构体转换回字节
impl Into<u8> for RouterFlags {
    fn into(self) -> u8 {
        (self.managed_address_config as u8) << 7
            | (self.other_config as u8) << 6
            | (self.home_agent as u8) << 5
            | (self.prf << 3)
            | (self.nd_proxy as u8) << 2
            | self.reserved
    }
}

fn ra_flag_default() -> RouterFlags {
    0xc0.into()
}

impl IPV6RAConfig {
    pub fn new(depend_iface: String) -> Self {
        Self {
            subnet_prefix: 64,
            subnet_index: 1,
            depend_iface,
            ra_preferred_lifetime: 300,
            ra_valid_lifetime: 300,
            ra_flag: ra_flag_default(),
        }
    }
}

impl LandScapeStore for IPV6RAServiceConfig {
    fn get_store_key(&self) -> String {
        self.iface_name.clone()
    }
}
