pub mod dhcp_v4_server;
pub mod dhcp_v6_client;
pub mod dns;
pub mod firewall;
pub mod flow;
pub mod iface;
pub mod mss_clamp;
pub mod nat;
pub mod ppp;
pub mod ra;
pub mod wanip;
pub mod wifi;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LandscapeConfig {}
