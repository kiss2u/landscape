use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use landscape_common::{
    config::dns::{DnsUpstreamType, FilterResult},
    dns::{DnsResolverConfig, DnsServerInitInfo, DnsUpstreamMode, RuleHandlerInfo},
    flow::{mark::FlowMark, DnsRuntimeMarkInfo},
};
use landscape_dns::reuseport_server::LandscapeReusePortDnsServer;
use uuid::Uuid;

/// cargo run --package landscape-dns --bin test_reuseport_server
#[tokio::main]
async fn main() -> std::io::Result<()> {
    landscape_common::init_tracing!();

    let listen_port = 54;
    let server = LandscapeReusePortDnsServer::new(listen_port);

    // handler
    let resolver_id = Uuid::new_v4();
    let mut default_resolver = RuleHandlerInfo {
        rule_id: Some(Uuid::new_v4()),
        flow_id: 0,
        resolver_id,
        mark: DnsRuntimeMarkInfo { mark: FlowMark::default(), priority: 1 },
        filter: FilterResult::Unfilter,
    };
    let mut info = DnsServerInitInfo::default();
    info.default_resolver = Some(default_resolver.clone());

    info.resolver_configs = vec![DnsResolverConfig {
        id: resolver_id,
        resolve_mode: DnsUpstreamMode::default(),
        mark: default_resolver.mark.mark,
        flow_id: default_resolver.flow_id,
    }];

    println!("=============================================");
    server.refresh_flow_server(0, info.clone()).await;

    println!("=============================================");
    tokio::time::sleep(Duration::from_secs(30)).await;

    println!("=============================================");

    default_resolver.mark.mark.set_reuseport(true);

    info.resolver_configs = vec![DnsResolverConfig {
        id: resolver_id,
        resolve_mode: DnsUpstreamMode::Upstream {
            upstream: DnsUpstreamType::Plaintext,
            ips: vec![IpAddr::V4(Ipv4Addr::new(223, 5, 5, 5))],
            port: None,
        },
        mark: default_resolver.mark.mark,
        flow_id: default_resolver.flow_id,
    }];
    info.default_resolver = Some(default_resolver);

    server.refresh_flow_server(0, info).await;
    println!("=============================================");

    let _ = tokio::signal::ctrl_c().await;

    Ok(())
}
