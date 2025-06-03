pub mod dhcp_v4_server;
pub mod dhcp_v6_client;
pub mod dns;
pub mod firewall;
pub mod flow;
pub mod geo;
pub mod iface;
pub mod iface_ip;
pub mod mss_clamp;
pub mod nat;
pub mod ppp;
pub mod ra;
pub mod wifi;

use dhcp_v4_server::DHCPv4ServiceConfig;
use dhcp_v6_client::IPV6PDServiceConfig;
use dns::DNSRuleConfig;
use firewall::FirewallServiceConfig;
use flow::FlowWanServiceConfig;
use iface::NetworkIfaceConfig;
use iface_ip::IfaceIpServiceConfig;
use mss_clamp::MSSClampServiceConfig;
use nat::NatServiceConfig;
use ppp::PPPDServiceConfig;
use ra::IPV6RAServiceConfig;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wifi::WifiServiceConfig;

use crate::{firewall::FirewallRuleConfig, flow::FlowConfig, ip_mark::WanIpRuleConfig};

pub type FlowId = u32;
pub type ConfigId = Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LandscapeConfig {}

/// 初始化配置结构体
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct InitConfig {
    pub ifaces: Vec<NetworkIfaceConfig>,
    pub ipconfigs: Vec<IfaceIpServiceConfig>,
    pub nats: Vec<NatServiceConfig>,
    pub marks: Vec<FlowWanServiceConfig>,
    pub pppds: Vec<PPPDServiceConfig>,

    pub flow_rules: Vec<FlowConfig>,
    pub dns_rules: Vec<DNSRuleConfig>,
    pub dst_ip_mark: Vec<WanIpRuleConfig>,

    pub dhcpv6pds: Vec<IPV6PDServiceConfig>,
    pub icmpras: Vec<IPV6RAServiceConfig>,

    pub firewalls: Vec<FirewallServiceConfig>,
    pub firewall_rules: Vec<FirewallRuleConfig>,

    pub wifi_configs: Vec<WifiServiceConfig>,
    pub dhcpv4_services: Vec<DHCPv4ServiceConfig>,

    pub mss_clamps: Vec<MSSClampServiceConfig>,
}
