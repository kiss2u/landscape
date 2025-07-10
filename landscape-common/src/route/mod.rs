use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::{flow::FlowTarget, net::MacAddr};

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

impl RouteTargetInfo {
    pub fn docker_new(ifindex: u32, iface_name: &str) -> (Self, Self) {
        (
            RouteTargetInfo {
                weight: 0,
                ifindex,
                has_mac: true,
                default_route: false,
                is_docker: true,
                iface_name: iface_name.to_string(),
                iface_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                gateway_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            },
            RouteTargetInfo {
                weight: 0,
                ifindex,
                has_mac: true,
                default_route: false,
                is_docker: true,
                iface_name: iface_name.to_string(),
                iface_ip: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                gateway_ip: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            },
        )
    }

    pub fn get_flow_target(&self) -> FlowTarget {
        if self.is_docker {
            FlowTarget::Netns { container_name: self.iface_name.clone() }
        } else {
            FlowTarget::Interface { name: self.iface_name.clone() }
        }
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct LanRouteInfo {
    pub ifindex: u32,
    pub iface_name: String,

    pub iface_ip: IpAddr,
    pub mac: Option<MacAddr>,
    pub prefix: u8,
}
