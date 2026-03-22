use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use clap::Parser;
use landscape::iface::get_iface_by_name;
use landscape_common::{
    firewall::{FirewallRuleItem, FirewallRuleMark},
    flow::mark::FlowMark,
    network::LandscapeIpProtocolCode,
};
use landscape_ebpf::map_setting::add_firewall_rule;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[arg(short, long, default_value = "veth0")]
    pub iface_name: String,
}

// cargo run --package landscape --bin firewall_test
#[tokio::main]
pub async fn main() {
    landscape_common::init_tracing!();
    landscape_ebpf::setting_libbpf_log();

    let args = Args::parse();
    tracing::info!("using args is: {:#?}", args);

    add_firewall_rule(get_allow_icmp_echo());
    let firewall = if let Some(iface) = get_iface_by_name(&args.iface_name).await {
        println!("Starting firewall on ifindex: {:?}", iface.index);
        match landscape_ebpf::firewall::new_firewall(iface.index as i32, iface.mac.is_some()) {
            Ok(handle) => Some(handle),
            Err(err) => {
                tracing::debug!("error: {err:?}");
                None
            }
        }
    } else {
        None
    };

    let _ = tokio::signal::ctrl_c().await;

    drop(firewall);
}

fn get_allow_icmp_echo() -> Vec<FirewallRuleMark> {
    vec![
        FirewallRuleMark {
            item: FirewallRuleItem {
                ip_protocol: Some(LandscapeIpProtocolCode::ICMPv6),
                local_port: Some(128),
                address: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                ip_prefixlen: 0,
            },
            mark: FlowMark::default(),
        },
        FirewallRuleMark {
            item: FirewallRuleItem {
                ip_protocol: Some(LandscapeIpProtocolCode::ICMPv6),
                local_port: Some(129),
                address: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                ip_prefixlen: 0,
            },
            mark: FlowMark::default(),
        },
        FirewallRuleMark {
            item: FirewallRuleItem {
                ip_protocol: Some(LandscapeIpProtocolCode::ICMP),
                local_port: Some(0),
                address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                ip_prefixlen: 0,
            },
            mark: FlowMark::default(),
        },
        FirewallRuleMark {
            item: FirewallRuleItem {
                ip_protocol: Some(LandscapeIpProtocolCode::ICMP),
                local_port: Some(8),
                address: IpAddr::V4(Ipv4Addr::BROADCAST),
                ip_prefixlen: 0,
            },
            mark: FlowMark::default(),
        },
    ]
}
