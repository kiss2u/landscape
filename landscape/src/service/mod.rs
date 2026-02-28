pub mod dhcp_v4;
pub mod ipconfig;
pub mod ipv6pd;
pub mod lan_ipv6;
pub mod mss_clamp;
pub mod nat_service;
pub mod pppd_service;
// Old RA service module — kept for reference but no longer compiled.
// The new LAN IPv6 service (lan_ipv6.rs) replaces it.
// pub mod ra;

pub mod route_lan;
pub mod route_wan;
