use std::net::IpAddr;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct WanRouteInfo {
    pub weight: u32,
    pub ifindex: u32,
    pub has_mac: bool,
    pub default_route: bool,
    pub iface_name: String,

    pub iface_ip: IpAddr,
    pub gateway_ip: IpAddr,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct LanRouteInfo {
    pub ifindex: u32,
    pub iface_name: String,

    pub iface_ip: IpAddr,
    pub prefix: u8,
}
