use crate::dev::{DeviceKind, DeviceType, LandscapeInterface};
use landscape_common::config::iface::{CreateDevType, IfaceZoneType, NetworkIfaceConfig, WifiMode};

use super::dev_wifi::LandscapeWifiInterface;

pub fn from_phy_dev(iface: &LandscapeInterface) -> NetworkIfaceConfig {
    from_phy_dev_with_wifi_info(iface, &None)
}

pub fn from_phy_dev_with_wifi_info(
    iface: &LandscapeInterface,
    wifi_info: &Option<LandscapeWifiInterface>,
) -> NetworkIfaceConfig {
    let zone_type = match iface.dev_type {
        DeviceType::Ppp => IfaceZoneType::Wan,
        _ => IfaceZoneType::Undefined,
    };
    let wifi_mode = if let Some(info) = wifi_info {
        match info.wifi_type {
            super::dev_wifi::WLANType::Station => WifiMode::Client,
            super::dev_wifi::WLANType::Ap => WifiMode::AP,
            _ => WifiMode::Undefined,
        }
    } else {
        WifiMode::default()
    };
    NetworkIfaceConfig {
        name: iface.name.clone(),
        create_dev_type: create_from(iface),
        controller_name: None,
        enable_in_boot: matches!(iface.dev_status, crate::dev::DevState::Up),
        zone_type,
        wifi_mode,
        xps_rps: None,
    }
}

pub fn create_from(iface: &LandscapeInterface) -> CreateDevType {
    if !iface.is_virtual_dev() {
        CreateDevType::default()
    } else {
        match iface.dev_kind {
            DeviceKind::Bridge => CreateDevType::Bridge,
            _ => CreateDevType::default(),
        }
    }
}
