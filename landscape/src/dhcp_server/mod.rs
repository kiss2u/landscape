use cidr::Ipv4Inet;
use core::ops::Range;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

use crate::dump::udp_packet::dhcp::options::DhcpOptions;

pub mod dhcp_server;

/// DHCP Server IPv4 Config
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DhcpServerIpv4Config {
    /// dhcp options
    pub options: Vec<DhcpOptions>,
    /// network addr e.g. 192.168.1.0
    pub network_addr: Ipv4Addr,
    /// network mask e.g. 255.255.255.0 = 24
    pub network_mask: u8,
    /// DHCP Server Addr e.g. 192.168.1.1
    pub server_ip_addr: Ipv4Addr,
    /// available host range 1~254 mean 192.168.1.1 ~ 192.168.1.254
    pub host_range: Range<u32>,
}

impl DhcpServerIpv4Config {
    pub fn new(
        server_ip: Ipv4Addr,
        network_mask: u8,
        mut options: Vec<DhcpOptions>,
        host_range: Range<u32>,
    ) -> Self {
        let ipv4 = Ipv4Inet::new(server_ip, network_mask).unwrap();

        let cidr = ipv4.network();
        println!("{:?}", ipv4.network());
        println!("{:?}", ipv4.first());
        println!("{:?}", ipv4.last());
        println!("{:?}", ipv4.is_host_address());
        println!("first: {:?}", ipv4.first().overflowing_add_u32(3).0.address());
        println!("size: {:?}", 1 << (32 - ipv4.network_length()));
        println!("mask: {:?}", ipv4.mask());
        println!("{:?}", cidr.network_length());
        println!("{:?}", cidr.first_address());
        println!("{:?}", cidr.is_host_address());
        println!("{:?}", cidr.last_address());

        options.push(DhcpOptions::SubnetMask(cidr.mask()));
        options.push(DhcpOptions::Router(server_ip));
        options.push(DhcpOptions::ServerIdentifier(server_ip));
        options.push(DhcpOptions::DomainNameServer(vec![server_ip]));

        // TODO: for debug
        options.push(DhcpOptions::AddressLeaseTime(40));

        DhcpServerIpv4Config {
            options,
            network_addr: cidr.mask(),
            network_mask,
            server_ip_addr: server_ip,
            host_range,
        }
    }

    pub fn get_ipv4_inet(&self) -> Option<Ipv4Inet> {
        Some(Ipv4Inet::new(self.server_ip_addr, self.network_mask).unwrap())
    }
}

impl Default for DhcpServerIpv4Config {
    fn default() -> Self {
        DhcpServerIpv4Config::new(Ipv4Addr::new(10, 255, 255, 1), 24, vec![], 1..254)
    }
}

#[cfg(test)]
mod tests {
    use super::DhcpServerIpv4Config;

    #[test]
    pub fn test_addr() {
        let data = DhcpServerIpv4Config::default();
        print!("{:?}", data);
    }
}
