use arc_swap::ArcSwap;
use landscape_common::config::ra::{IPV6RaConfigSource, IPv6RaPdConfig};
use landscape_common::ipv6_pd::{IAPrefixMap, LDIAPrefix};
use landscape_common::route::{LanIPv6RouteKey, LanRouteInfo};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::route::IpRouteService;

#[derive(Clone)]
pub struct ICMPv6ConfigInfo {
    pub rt_prefix: Ipv6Addr,
    pub rt_prefix_len: u8,

    pub sub_router: Ipv6Addr,
    pub sub_prefix: Ipv6Addr,
    pub sub_prefix_len: u8,

    pub ra_preferred_lifetime: u32,
    pub ra_valid_lifetime: u32,
}

/// Pure data: prefix runtime information shared by RA and DHCPv6
pub struct IPv6PrefixRuntime {
    pub static_info: Vec<ICMPv6ConfigInfo>,
    pub pd_info: HashMap<String, Arc<ArcSwap<Option<ICMPv6ConfigInfo>>>>,
    pub relative_boot_time: Instant,
}

/// Return value of `setup_prefix_sources()`
pub struct PrefixSetupResult {
    pub runtime: IPv6PrefixRuntime,
    pub ra_token: CancellationToken,
    pub dhcpv6_token: CancellationToken,
    pub change_notify: watch::Receiver<()>,
    pub cleanup_ips: Vec<(Ipv6Addr, u8, String)>,
}

/// Set up prefix sources (static + PD) and return runtime data, tokens, and change notifications.
///
/// - Static sources: allocate subnet, set interface IP, register route
/// - PD sources: watch for prefix updates, notify on change
/// - Returns `PrefixSetupResult` with tokens for RA and DHCPv6 lifecycle management
pub async fn setup_prefix_sources(
    source: Vec<IPV6RaConfigSource>,
    iface_name: &str,
    lan_info: &LanRouteInfo,
    route_service: &IpRouteService,
    prefix_map: &IAPrefixMap,
) -> PrefixSetupResult {
    let ra_token = CancellationToken::new();
    let dhcpv6_token = CancellationToken::new();
    let (change_tx, change_rx) = watch::channel(());
    let change_tx = Arc::new(change_tx);

    let mut runtime = IPv6PrefixRuntime {
        static_info: vec![],
        pd_info: HashMap::new(),
        relative_boot_time: Instant::now(),
    };
    let mut cleanup_ips = vec![];

    for src in source {
        match src {
            IPV6RaConfigSource::Static(static_config) => {
                let rt_prefix_len = 56;
                let (sub_prefix, sub_router) = allocate_subnet(
                    static_config.base_prefix,
                    rt_prefix_len,
                    static_config.sub_prefix_len,
                    static_config.sub_index as u128,
                );
                set_iface_ip(sub_router, static_config.sub_prefix_len, iface_name, None, None);
                cleanup_ips.push((
                    sub_router,
                    static_config.sub_prefix_len,
                    iface_name.to_string(),
                ));
                let mut li = lan_info.clone();
                li.iface_ip = IpAddr::V6(sub_router);
                li.prefix = static_config.sub_prefix_len;
                let lan_info_key = LanIPv6RouteKey {
                    iface_name: iface_name.to_string(),
                    subnet_index: static_config.sub_index,
                };
                route_service.insert_ipv6_lan_route(lan_info_key, li).await;
                runtime.static_info.push(ICMPv6ConfigInfo {
                    rt_prefix: static_config.base_prefix,
                    rt_prefix_len,
                    sub_router,
                    sub_prefix,
                    sub_prefix_len: static_config.sub_prefix_len,
                    ra_preferred_lifetime: static_config.ra_preferred_lifetime,
                    ra_valid_lifetime: static_config.ra_valid_lifetime,
                });
            }
            IPV6RaConfigSource::Pd(pd_config) => {
                let pd_prefix_info: Option<ICMPv6ConfigInfo> = None;
                let pd_prefix_info = Arc::new(ArcSwap::from_pointee(pd_prefix_info));
                let mut ia_config_watch = prefix_map.get_ia_prefix(&pd_config.depend_iface).await;
                runtime.pd_info.insert(pd_config.depend_iface.clone(), pd_prefix_info.clone());

                let change_tx_clone = change_tx.clone();
                let ra_token_clone = ra_token.clone();
                let dhcpv6_token_clone = dhcpv6_token.clone();
                let iface_name_owned = iface_name.to_string();
                let lan_info_clone = lan_info.clone();
                let route_service_clone = route_service.clone();

                let mut expire_time = Box::pin(tokio::time::sleep(Duration::from_secs(0)));
                // Check once immediately
                let ia_prefix = ia_config_watch.borrow().clone();
                if let Some(ia_prefix) = ia_prefix {
                    pd_prefix_info.store(Arc::new(Some(
                        update_current_info(
                            &iface_name_owned,
                            ia_prefix,
                            &pd_config,
                            expire_time.as_mut(),
                            &lan_info_clone,
                            &route_service_clone,
                        )
                        .await,
                    )));
                }

                tokio::spawn(async move {
                    // Wait for both RA and DHCPv6 to finish before stopping
                    let both_done = async {
                        tokio::join!(ra_token_clone.cancelled(), dhcpv6_token_clone.cancelled());
                    };
                    tokio::pin!(both_done);

                    loop {
                        tokio::select! {
                            change_result = ia_config_watch.changed() => {
                                tracing::info!("IA_PREFIX update");
                                if let Err(_) = change_result {
                                    tracing::error!("get change result error. exit loop");
                                    break;
                                }
                                let ia_prefix = ia_config_watch.borrow().clone();
                                if let Some(ia_prefix) = ia_prefix {
                                    pd_prefix_info.store(Arc::new(Some(update_current_info(
                                        &iface_name_owned,
                                        ia_prefix,
                                        &pd_config,
                                        expire_time.as_mut(),
                                        &lan_info_clone,
                                        &route_service_clone,
                                    ).await)));
                                }
                                let _ = change_tx_clone.send(());
                            },
                            _ = &mut both_done => {
                                break;
                            }
                            _ = expire_time.as_mut() => {
                                pd_prefix_info.store(Arc::new(None));
                                let _ = change_tx_clone.send(());
                                tracing::debug!("expire_time active");
                                expire_time.as_mut().set(tokio::time::sleep(Duration::from_secs(u64::MAX)));
                            }
                        }
                    }

                    tracing::info!("iface: {} prefix listen is down", pd_config.depend_iface);
                });
            }
        }
    }

    // Keep the change_tx sender alive until both RA and DHCPv6 tokens are cancelled.
    // Without this, if there are no PD sources (only static), change_tx is dropped
    // when this function returns, causing change_notify.changed() to immediately
    // return Err in the RA main loop → infinite send storm.
    {
        let ra_token_clone = ra_token.clone();
        let dhcpv6_token_clone = dhcpv6_token.clone();
        tokio::spawn(async move {
            let _keep_alive = change_tx;
            tokio::join!(ra_token_clone.cancelled(), dhcpv6_token_clone.cancelled());
        });
    }

    PrefixSetupResult {
        runtime,
        ra_token,
        dhcpv6_token,
        change_notify: change_rx,
        cleanup_ips,
    }
}

/// Clean up prefix resources: remove routes and delete static IPs
pub async fn cleanup_prefix_sources(
    static_ip_infos: Vec<(Ipv6Addr, u8, String)>,
    iface_name: &str,
    route_service: &IpRouteService,
) {
    route_service.remove_ipv6_lan_route(iface_name).await;
    for (ip, prefix, iface_name) in static_ip_infos {
        del_iface_ip(ip, prefix, &iface_name);
    }
}

async fn update_current_info(
    iface_name: &str,
    ia_prefix: LDIAPrefix,
    pd_config: &IPv6RaPdConfig,
    mut expire_time: Pin<&mut tokio::time::Sleep>,
    lan_info: &LanRouteInfo,
    route_service: &IpRouteService,
) -> ICMPv6ConfigInfo {
    expire_time.set(tokio::time::sleep(Duration::from_secs(ia_prefix.valid_lifetime as u64)));
    let (sub_prefix, sub_router) = allocate_subnet(
        ia_prefix.prefix_ip,
        ia_prefix.prefix_len,
        pd_config.prefix_len,
        pd_config.subnet_index as u128,
    );

    let mut lan_info = lan_info.clone();
    lan_info.iface_ip = IpAddr::V6(sub_router);
    lan_info.prefix = pd_config.prefix_len;
    let lan_info_key = LanIPv6RouteKey {
        iface_name: iface_name.to_string(),
        subnet_index: pd_config.subnet_index,
    };
    route_service.insert_ipv6_lan_route(lan_info_key, lan_info).await;

    add_route(sub_prefix, pd_config.prefix_len, iface_name, Some(ia_prefix.valid_lifetime));
    set_iface_ip(
        sub_router,
        pd_config.prefix_len,
        iface_name,
        Some(ia_prefix.valid_lifetime),
        Some(ia_prefix.preferred_lifetime),
    );

    ICMPv6ConfigInfo {
        rt_prefix: ia_prefix.prefix_ip,
        rt_prefix_len: ia_prefix.prefix_len,
        sub_prefix,
        sub_prefix_len: pd_config.prefix_len,
        sub_router,
        ra_preferred_lifetime: pd_config.ra_preferred_lifetime,
        ra_valid_lifetime: pd_config.ra_valid_lifetime,
    }
}

/// Compute subnet network address and router address from a prefix and subnet index
pub fn allocate_subnet(
    pd_ip: Ipv6Addr,
    pd_prefix_len: u8,
    sub_prefix_len: u8,
    subnet_index: u128,
) -> (Ipv6Addr, Ipv6Addr) {
    assert!(sub_prefix_len >= pd_prefix_len, "子网前缀长度必须大于等于原始前缀长度");

    let max_subnets = 1u128 << (sub_prefix_len - pd_prefix_len);
    assert!(subnet_index < max_subnets, "subnet_index 超出可用子网范围");

    let prefix_u128 = u128::from(pd_ip);
    let parent_mask = (!0u128) << (128 - pd_prefix_len);
    let parent_network = prefix_u128 & parent_mask;
    let sub_mask = (!0u128) << (128 - sub_prefix_len);
    let base_network = parent_network & sub_mask;
    let subnet_size = 1u128 << (128 - sub_prefix_len);
    let subnet_network = base_network + (subnet_index * subnet_size);
    let router_address = subnet_network + 1;

    (Ipv6Addr::from(subnet_network), Ipv6Addr::from(router_address))
}

pub fn add_route(ip: Ipv6Addr, prefix: u8, iface_name: &str, valid_lifetime: Option<u32>) {
    let mut args = vec![
        "-6".to_string(),
        "route".to_string(),
        "replace".to_string(),
        format!("{}/{}", ip, prefix),
        "dev".to_string(),
        iface_name.to_string(),
    ];

    if let Some(lifetime) = valid_lifetime {
        args.push("expires".to_string());
        args.push(lifetime.to_string());
    }

    let result = std::process::Command::new("ip").args(&args).output();

    if let Err(e) = result {
        tracing::error!("{e:?}");
    }
}

pub fn del_iface_ip(ip: Ipv6Addr, prefix: u8, iface_name: &str) {
    let args = vec![
        "-6".to_string(),
        "addr".to_string(),
        "del".to_string(),
        format!("{}/{}", ip, prefix),
        "dev".to_string(),
        iface_name.to_string(),
    ];

    let result = std::process::Command::new("ip").args(&args).output();

    if let Err(e) = result {
        tracing::error!("{e:?}");
    }
}

pub fn set_iface_ip(
    ip: Ipv6Addr,
    prefix: u8,
    iface_name: &str,
    valid_lifetime: Option<u32>,
    preferred_lft: Option<u32>,
) {
    let mut args = vec![
        "-6".to_string(),
        "addr".to_string(),
        "replace".to_string(),
        format!("{}/{}", ip, prefix),
        "dev".to_string(),
        iface_name.to_string(),
    ];

    if let Some(valid) = valid_lifetime {
        args.push("valid_lft".to_string());
        args.push(valid.to_string());
    }

    if let Some(preferred) = preferred_lft {
        args.push("preferred_lft".to_string());
        args.push(preferred.to_string());
    }

    let result = std::process::Command::new("ip").args(&args).output();

    if let Err(e) = result {
        tracing::error!("{e:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::allocate_subnet;
    use landscape_common::{config::ra::IPv6RaStaticConfig, ipv6_pd::LDIAPrefix};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::watch;
    use tokio_util::sync::CancellationToken;

    #[test]
    fn test() {
        let ldia_prefix = LDIAPrefix {
            preferred_lifetime: 3600,
            valid_lifetime: 7200,
            prefix_len: 48,
            prefix_ip: "2001:db8::".parse().unwrap(),
            last_update_time: 0.0,
        };
        let sub_prefix_len = 64;
        let subnet_index = 2;
        let (subnet_network, router_addr) = allocate_subnet(
            ldia_prefix.prefix_ip,
            ldia_prefix.prefix_len,
            sub_prefix_len,
            subnet_index,
        );
        println!("子网网络地址: {}/{}", subnet_network, sub_prefix_len);
        println!("路由器地址: {}", router_addr);
    }

    #[test]
    fn test_static_setting() {
        let ldia_prefix = IPv6RaStaticConfig {
            base_prefix: "2001:db8::3".parse().unwrap(),
            sub_prefix_len: 64,
            sub_index: 2,
            ra_preferred_lifetime: 300,
            ra_valid_lifetime: 600,
        };
        let (subnet_network, router_addr) = allocate_subnet(
            ldia_prefix.base_prefix,
            60,
            ldia_prefix.sub_prefix_len,
            ldia_prefix.sub_index as u128,
        );
        println!("子网网络地址: {}/{}", subnet_network, ldia_prefix.sub_prefix_len);
        println!("路由器地址: {}", router_addr);
    }

    #[tokio::test]
    async fn test_change_channel_stays_alive_without_pd_sources() {
        // Simulate setup_prefix_sources with only static sources (no PD):
        // the keep-alive task should hold the sender alive.
        let (change_tx, mut change_rx) = watch::channel(());
        let change_tx = Arc::new(change_tx);
        let ra_token = CancellationToken::new();
        let dhcpv6_token = CancellationToken::new();

        // keep-alive task (mirrors the fix in setup_prefix_sources)
        {
            let ra_clone = ra_token.clone();
            let dhcpv6_clone = dhcpv6_token.clone();
            tokio::spawn(async move {
                let _keep_alive = change_tx;
                tokio::join!(ra_clone.cancelled(), dhcpv6_clone.cancelled());
            });
        }
        // change_tx (original Arc) has been moved into spawn

        change_rx.borrow_and_update();

        // changed() should timeout (no one sending), not return Err
        let result = tokio::time::timeout(Duration::from_millis(100), change_rx.changed()).await;
        assert!(result.is_err(), "should timeout, not get channel Err");

        // Cancel tokens → keep-alive task ends → sender drops
        ra_token.cancel();
        dhcpv6_token.cancel();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Now changed() should return Err (sender dropped)
        let result = change_rx.changed().await;
        assert!(result.is_err(), "sender dropped, should get Err");
    }

    #[tokio::test]
    async fn test_change_channel_receives_pd_notifications() {
        let (change_tx, mut change_rx) = watch::channel(());
        let change_tx = Arc::new(change_tx);
        let tx_clone = change_tx.clone();

        change_rx.borrow_and_update();

        // Simulate PD task sending notification
        tx_clone.send(()).unwrap();

        // changed() should return Ok immediately
        let result = tokio::time::timeout(Duration::from_millis(100), change_rx.changed()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}
