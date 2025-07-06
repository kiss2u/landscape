use std::net::IpAddr;

use crate::net::MacAddr;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct RouteTargetInfo {
    pub weight: u32,
    pub ifindex: u32,
    pub has_mac: bool,
    pub default_route: bool,
    pub is_docker: bool,

    pub iface_name: String,

    pub iface_ip: IpAddr,
    pub gateway_ip: IpAddr,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct LanRouteInfo {
    pub ifindex: u32,
    pub iface_name: String,

    pub iface_ip: IpAddr,
    pub mac: Option<MacAddr>,
    pub prefix: u8,
}
