use landscape_common::{
    firewall::blacklist::{FirewallBlacklistConfig, FirewallBlacklistSource},
    ip_mark::IpConfig,
};

use crate::geo::ip_service::GeoIpService;

pub async fn resolve_and_sync_blacklist(
    geo_ip_service: &GeoIpService,
    new_configs: Vec<FirewallBlacklistConfig>,
    old_configs: Vec<FirewallBlacklistConfig>,
) {
    let new_ips = resolve_configs(geo_ip_service, &new_configs).await;
    let old_ips = resolve_configs(geo_ip_service, &old_configs).await;

    tracing::info!("sync firewall blacklist: new_ips={}, old_ips={}", new_ips.len(), old_ips.len());

    landscape_ebpf::map_setting::sync_firewall_blacklist(new_ips, old_ips);
}

async fn resolve_configs(
    geo_ip_service: &GeoIpService,
    configs: &[FirewallBlacklistConfig],
) -> Vec<IpConfig> {
    let mut result = vec![];

    for config in configs.iter().filter(|c| c.enable) {
        for source in &config.source {
            match source {
                FirewallBlacklistSource::Config(ip_config) => {
                    result.push(ip_config.clone());
                }
                FirewallBlacklistSource::GeoKey(geo_key) => {
                    let ips = geo_ip_service.resolve_geo_key_to_ips(geo_key).await;
                    result.extend(ips);
                }
            }
        }
    }

    result
}
