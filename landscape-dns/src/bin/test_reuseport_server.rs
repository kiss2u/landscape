use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use landscape_common::{
    config::dns::DnsUpstreamType,
    dns::{DNSRuleInitInfo, DnsUpstreamMode},
};
use landscape_dns::reuseport_server::LandscapeReusePortDnsServer;

/// cargo run --package landscape-dns --bin test_reuseport_server
#[tokio::main]
async fn main() -> std::io::Result<()> {
    landscape_common::init_tracing!();

    let listen_port = 54;
    let server = LandscapeReusePortDnsServer::new(listen_port);

    // handler

    let default_rule = vec![DNSRuleInitInfo::default()];
    println!("=============================================");
    server.init_server(default_rule).await;

    println!("=============================================");
    tokio::time::sleep(Duration::from_secs(30)).await;

    println!("=============================================");

    let mut rule2 = DNSRuleInitInfo::default();

    rule2.resolve_mode = DnsUpstreamMode::Upstream {
        upstream: DnsUpstreamType::Plaintext,
        ips: vec![IpAddr::V4(Ipv4Addr::new(223, 5, 5, 5))],
        port: None,
    };
    rule2.mark.set_reuseport(true);

    server.refresh_flow_server(0, vec![rule2]).await;
    println!("=============================================");

    let _ = tokio::signal::ctrl_c().await;

    Ok(())
}
