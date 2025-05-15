use crate::{
    iface::{IfaceCpuSoftBalance, IfaceZoneType},
    store::storev2::LandscapeStore,
    LANDSCAPE_DEFAULT_LAN_NAME,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 用于存储网卡信息的结构体
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "common/iface.d.ts")]
pub struct NetworkIfaceConfig {
    // 名称 关联的网卡名称 相当于网卡的唯一 id
    #[ts(rename = "iface_name")]
    pub name: String,

    #[serde(default)]
    pub create_dev_type: CreateDevType,

    // 是否有 master 使用 name 因为 Linux 中名称是唯一的
    pub controller_name: Option<String>,

    #[serde(default)]
    pub zone_type: IfaceZoneType,

    #[serde(default = "yes")]
    pub enable_in_boot: bool,

    #[serde(default)]
    pub wifi_mode: WifiMode,

    /// NIC XPS / RPS Config
    #[serde(default)]
    pub xps_rps: Option<IfaceCpuSoftBalance>,
}

impl LandscapeStore for NetworkIfaceConfig {
    fn get_store_key(&self) -> String {
        self.name.clone()
    }
}

fn yes() -> bool {
    true
}

impl NetworkIfaceConfig {
    pub fn get_iface_name(&self) -> String {
        self.name.clone()
    }

    pub fn crate_default_br_lan() -> NetworkIfaceConfig {
        NetworkIfaceConfig::crate_bridge(
            LANDSCAPE_DEFAULT_LAN_NAME.into(),
            Some(IfaceZoneType::Lan),
        )
    }

    pub fn crate_bridge(name: String, zone_type: Option<IfaceZoneType>) -> NetworkIfaceConfig {
        NetworkIfaceConfig {
            name,
            create_dev_type: CreateDevType::Bridge,
            controller_name: None,
            enable_in_boot: true,
            zone_type: zone_type.unwrap_or_default(),
            wifi_mode: WifiMode::default(),
            xps_rps: None,
        }
    }
}

/// 需要创建的设备类型
#[derive(Debug, Serialize, Deserialize, Default, Clone, TS)]
#[ts(export, export_to = "common/iface.d.ts")]
#[serde(rename_all = "lowercase")]
pub enum CreateDevType {
    #[default]
    NoNeedToCreate,
    Bridge,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, TS)]
#[ts(export, export_to = "common/iface.d.ts")]
#[serde(rename_all = "lowercase")]
pub enum WifiMode {
    #[default]
    Undefined,
    Client,
    AP,
}
