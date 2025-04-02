use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use crate::net::MacAddr;

#[derive(Serialize, Deserialize, Debug, Clone, Default, TS)]
#[ts(export, export_to = "docker.ts")]
pub struct LandscapeDockerNetwork {
    // Name
    pub name: String,
    pub id: String,
    pub driver: Option<String>,
    pub containers: HashMap<String, LandscapeDockerNetworkContainer>,
    pub iface_name: String,
    pub options: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, TS)]
#[ts(export, export_to = "docker.ts")]
pub struct LandscapeDockerNetworkContainer {
    pub name: String,
    pub mac: Option<MacAddr>,
}
