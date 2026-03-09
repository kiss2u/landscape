use serde::{Deserialize, Serialize};

use crate::cert::account::CertAccountConfig;
use crate::cert::order::CertConfig;
use crate::config::settings::LandscapeConfig;
use crate::dhcp::v4_server::config::DHCPv4ServiceConfig;
use crate::dhcp::v6_client::config::IPV6PDServiceConfig;
use crate::dns::config::DnsUpstreamConfig;
use crate::dns::redirect::DNSRedirectRule;
use crate::dns::rule::DNSRuleConfig;
use crate::enrolled_device::EnrolledDevice;
use crate::firewall::blacklist::FirewallBlacklistConfig;
use crate::firewall::service::FirewallServiceConfig;
use crate::firewall::FirewallRuleConfig;
use crate::flow::config::FlowConfig;
use crate::flow::service::FlowWanServiceConfig;
use crate::gateway::HttpUpstreamRuleConfig;
use crate::geo::{GeoIpSourceConfig, GeoSiteSourceConfig};
use crate::iface::config::NetworkIfaceConfig;
use crate::iface::ip_config::IfaceIpServiceConfig;
use crate::iface::mss_clamp::MSSClampServiceConfig;
use crate::iface::nat::StaticNatMappingConfig;
use crate::iface::ppp::PPPDServiceConfig;
use crate::iface::wifi::WifiServiceConfig;
use crate::ip_mark::WanIpRuleConfig;
use crate::ipv6::lan::LanIPv6ServiceConfig;
use crate::ipv6::ra::IPV6RAServiceConfig;
use crate::route::lan::RouteLanServiceConfig;
use crate::route::wan::RouteWanServiceConfig;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct InitConfig {
    pub config: LandscapeConfig,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ifaces: Vec<NetworkIfaceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ipconfigs: Vec<IfaceIpServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nats: Vec<crate::iface::nat::NatServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<FlowWanServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pppds: Vec<PPPDServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub flow_rules: Vec<FlowConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dns_rules: Vec<DNSRuleConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dst_ip_mark: Vec<WanIpRuleConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dhcpv6pds: Vec<IPV6PDServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub icmpras: Vec<IPV6RAServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub lan_ipv6s: Vec<LanIPv6ServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub firewalls: Vec<FirewallServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub firewall_rules: Vec<FirewallRuleConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub firewall_blacklists: Vec<FirewallBlacklistConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub wifi_configs: Vec<WifiServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dhcpv4_services: Vec<DHCPv4ServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mss_clamps: Vec<MSSClampServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub geo_ips: Vec<GeoIpSourceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub geo_sites: Vec<GeoSiteSourceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub route_lans: Vec<RouteLanServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub route_wans: Vec<RouteWanServiceConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub static_nat_mappings: Vec<StaticNatMappingConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dns_redirects: Vec<DNSRedirectRule>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dns_upstream_configs: Vec<DnsUpstreamConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub enrolled_devices: Vec<EnrolledDevice>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cert_accounts: Vec<CertAccountConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub certs: Vec<CertConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gateway_rules: Vec<HttpUpstreamRuleConfig>,
}
