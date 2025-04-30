use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common_iface.ts")]
pub struct BridgeCreate {
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common_iface.ts")]
pub struct AddController {
    pub link_name: String,
    pub link_ifindex: u32,
    #[serde(default)]
    pub master_name: Option<String>,
    #[serde(default)]
    pub master_ifindex: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "common_iface.ts")]
pub struct ChangeZone {
    pub iface_name: String,
    pub zone: IfaceZoneType,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, TS)]
#[ts(export, export_to = "common_iface.ts")]
#[serde(rename_all = "lowercase")]
pub enum IfaceZoneType {
    // 未定义类型
    #[default]
    Undefined,
    Wan,
    Lan,
}
