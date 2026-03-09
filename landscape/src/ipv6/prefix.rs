use arc_swap::ArcSwap;
use landscape_common::ipv6::lan::{LanIPv6SourceConfig, SourceServiceKind};
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

/// Parent prefix block available for IA_PD delegation.
/// Derived from PdStatic (static) or PdPd (upstream PD watch).
/// Completely independent from NA prefix info.
#[derive(Clone)]
pub struct PdDelegationParent {
    pub prefix: Ipv6Addr, // sub-block network address (computed from pool_index/pool_len)
    pub prefix_len: u8,   // = pool_len, the parent block length for delegation
}

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
    // PD delegation sources (independent from NA path)
    pub pd_delegation_static: Vec<PdDelegationParent>,
    pub pd_delegation_dynamic: Vec<Arc<ArcSwap<Option<PdDelegationParent>>>>,
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

/// Runtime config for a PD source, extracted from LanIPv6SourceConfig
struct PdSourceConfig {
    depend_iface: String,
    pool_index: u32,
    sub_prefix_len: u8,
    preferred_lifetime: u32,
    valid_lifetime: u32,
}

/// Set up prefix sources (static + PD) and return runtime data, tokens, and change notifications.
///
/// Accepts the new flat `LanIPv6SourceConfig` sources, optionally filtered by service kind.
/// - Ra* sources: used for RA prefix setup (static IP + route, or PD watch)
/// - Na* sources: used for DHCPv6 IA_NA prefix setup (same mechanics)
/// - Pd* sources: NOT handled here (IA_PD pool management is in DHCPv6 server)
///
/// If `filter_kind` is Some, only sources matching that kind are processed.
/// For backward compat with the service layer, RA uses Ra sources, DHCPv6 NA uses Na sources.
pub async fn setup_prefix_sources(
    sources: &[LanIPv6SourceConfig],
    filter_kinds: &[SourceServiceKind],
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
        pd_delegation_static: vec![],
        pd_delegation_dynamic: vec![],
        relative_boot_time: Instant::now(),
    };
    let mut cleanup_ips = vec![];

    for src in sources {
        if !filter_kinds.contains(&src.service_kind()) {
            continue;
        }

        // Extract common parameters from the source variant
        let (base_prefix_opt, pd_config_opt) = match src {
            LanIPv6SourceConfig::RaStatic {
                base_prefix,
                pool_index,
                preferred_lifetime,
                valid_lifetime,
            } => (Some((*base_prefix, *pool_index, *preferred_lifetime, *valid_lifetime)), None),
            LanIPv6SourceConfig::NaStatic { base_prefix, pool_index } => {
                (Some((*base_prefix, *pool_index, 0, 0)), None)
            }
            LanIPv6SourceConfig::RaPd {
                depend_iface,
                pool_index,
                preferred_lifetime,
                valid_lifetime,
            } => (
                None,
                Some(PdSourceConfig {
                    depend_iface: depend_iface.clone(),
                    pool_index: *pool_index,
                    sub_prefix_len: 64,
                    preferred_lifetime: *preferred_lifetime,
                    valid_lifetime: *valid_lifetime,
                }),
            ),
            LanIPv6SourceConfig::NaPd { depend_iface, pool_index } => (
                None,
                Some(PdSourceConfig {
                    depend_iface: depend_iface.clone(),
                    pool_index: *pool_index,
                    sub_prefix_len: 64,
                    preferred_lifetime: 0,
                    valid_lifetime: 0,
                }),
            ),
            // PdStatic: compute sub-block and store in pd_delegation_static
            LanIPv6SourceConfig::PdStatic {
                base_prefix,
                base_prefix_len,
                pool_index,
                pool_len,
            } => {
                let (sub_block, _) =
                    allocate_subnet(*base_prefix, *base_prefix_len, *pool_len, *pool_index as u128);
                runtime
                    .pd_delegation_static
                    .push(PdDelegationParent { prefix: sub_block, prefix_len: *pool_len });
                continue;
            }
            // PdPd: watch upstream PD and store in pd_delegation_dynamic
            LanIPv6SourceConfig::PdPd {
                depend_iface,
                max_source_prefix_len,
                pool_index,
                pool_len,
            } => {
                let pd_prefix_info: Option<PdDelegationParent> = None;
                let pd_prefix_info = Arc::new(ArcSwap::from_pointee(pd_prefix_info));
                let mut ia_config_watch = prefix_map.get_ia_prefix(depend_iface).await;
                runtime.pd_delegation_dynamic.push(pd_prefix_info.clone());

                let dhcpv6_token_clone = dhcpv6_token.clone();
                let max_source_prefix_len = *max_source_prefix_len;
                let pool_index = *pool_index;
                let pool_len = *pool_len;
                let depend_iface_owned = depend_iface.clone();

                let mut expire_time = Box::pin(tokio::time::sleep(Duration::from_secs(0)));
                // Check once immediately
                let ia_prefix = ia_config_watch.borrow().clone();
                if let Some(ia_prefix) = ia_prefix {
                    if ia_prefix.prefix_len <= max_source_prefix_len {
                        let (sub_block, _) = allocate_subnet(
                            ia_prefix.prefix_ip,
                            ia_prefix.prefix_len,
                            pool_len,
                            pool_index as u128,
                        );
                        pd_prefix_info.store(Arc::new(Some(PdDelegationParent {
                            prefix: sub_block,
                            prefix_len: pool_len,
                        })));
                        expire_time.as_mut().set(tokio::time::sleep(Duration::from_secs(
                            ia_prefix.valid_lifetime as u64,
                        )));
                    }
                }

                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            change_result = ia_config_watch.changed() => {
                                tracing::info!("PdPd IA_PREFIX update for {}", depend_iface_owned);
                                if change_result.is_err() {
                                    tracing::error!("PdPd change result error. exit loop");
                                    break;
                                }
                                let ia_prefix = ia_config_watch.borrow().clone();
                                if let Some(ia_prefix) = ia_prefix {
                                    if ia_prefix.prefix_len <= max_source_prefix_len {
                                        let (sub_block, _) = allocate_subnet(
                                            ia_prefix.prefix_ip,
                                            ia_prefix.prefix_len,
                                            pool_len,
                                            pool_index as u128,
                                        );
                                        pd_prefix_info.store(Arc::new(Some(PdDelegationParent {
                                            prefix: sub_block,
                                            prefix_len: pool_len,
                                        })));
                                        expire_time.as_mut().set(tokio::time::sleep(Duration::from_secs(ia_prefix.valid_lifetime as u64)));
                                    } else {
                                        pd_prefix_info.store(Arc::new(None));
                                    }
                                } else {
                                    pd_prefix_info.store(Arc::new(None));
                                }
                            },
                            _ = dhcpv6_token_clone.cancelled() => {
                                break;
                            }
                            _ = expire_time.as_mut() => {
                                pd_prefix_info.store(Arc::new(None));
                                tracing::debug!("PdPd expire_time active for {}", depend_iface_owned);
                                expire_time.as_mut().set(tokio::time::sleep(Duration::from_secs(u64::MAX)));
                            }
                        }
                    }
                    tracing::info!("PdPd prefix listen for {} is down", depend_iface_owned);
                });
                continue;
            }
        };

        if let Some((base_prefix, pool_index, pref_lt, valid_lt)) = base_prefix_opt {
            // Static source handling
            let rt_prefix_len = 56;
            let sub_prefix_len = 64u8;
            let (sub_prefix, sub_router) =
                allocate_subnet(base_prefix, rt_prefix_len, sub_prefix_len, pool_index as u128);
            set_iface_ip(sub_router, sub_prefix_len, iface_name, None, None);
            cleanup_ips.push((sub_router, sub_prefix_len, iface_name.to_string()));
            let mut li = lan_info.clone();
            li.iface_ip = IpAddr::V6(sub_router);
            li.prefix = sub_prefix_len;
            let lan_info_key = LanIPv6RouteKey {
                iface_name: iface_name.to_string(),
                subnet_index: pool_index,
            };
            route_service.insert_ipv6_lan_route(lan_info_key, li).await;
            runtime.static_info.push(ICMPv6ConfigInfo {
                rt_prefix: base_prefix,
                rt_prefix_len,
                sub_router,
                sub_prefix,
                sub_prefix_len,
                ra_preferred_lifetime: pref_lt,
                ra_valid_lifetime: valid_lt,
            });
        }

        if let Some(pd_config) = pd_config_opt {
            // PD source handling

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

    // Keep the change_tx sender alive until both RA and DHCPv6 tokens are cancelled.
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
    pd_config: &PdSourceConfig,
    mut expire_time: Pin<&mut tokio::time::Sleep>,
    lan_info: &LanRouteInfo,
    route_service: &IpRouteService,
) -> ICMPv6ConfigInfo {
    expire_time.set(tokio::time::sleep(Duration::from_secs(ia_prefix.valid_lifetime as u64)));
    let (sub_prefix, sub_router) = allocate_subnet(
        ia_prefix.prefix_ip,
        ia_prefix.prefix_len,
        pd_config.sub_prefix_len,
        pd_config.pool_index as u128,
    );

    let mut lan_info = lan_info.clone();
    lan_info.iface_ip = IpAddr::V6(sub_router);
    lan_info.prefix = pd_config.sub_prefix_len;
    let lan_info_key = LanIPv6RouteKey {
        iface_name: iface_name.to_string(),
        subnet_index: pd_config.pool_index,
    };
    route_service.insert_ipv6_lan_route(lan_info_key, lan_info).await;

    add_route(sub_prefix, pd_config.sub_prefix_len, iface_name, Some(ia_prefix.valid_lifetime));
    set_iface_ip(
        sub_router,
        pd_config.sub_prefix_len,
        iface_name,
        Some(ia_prefix.valid_lifetime),
        Some(ia_prefix.preferred_lifetime),
    );

    ICMPv6ConfigInfo {
        rt_prefix: ia_prefix.prefix_ip,
        rt_prefix_len: ia_prefix.prefix_len,
        sub_prefix,
        sub_prefix_len: pd_config.sub_prefix_len,
        sub_router,
        ra_preferred_lifetime: pd_config.preferred_lifetime,
        ra_valid_lifetime: pd_config.valid_lifetime,
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

/// Add a route with a gateway (via), used for PD delegation routes.
/// e.g. `ip -6 route replace <prefix>/<len> via <gateway> dev <iface> expires <lifetime>`
pub fn add_route_via(
    prefix: Ipv6Addr,
    prefix_len: u8,
    via: Ipv6Addr,
    iface_name: &str,
    valid_lifetime: Option<u32>,
) {
    let mut args = vec![
        "-6".to_string(),
        "route".to_string(),
        "replace".to_string(),
        format!("{}/{}", prefix, prefix_len),
        "via".to_string(),
        via.to_string(),
        "dev".to_string(),
        iface_name.to_string(),
    ];

    if let Some(lifetime) = valid_lifetime {
        args.push("expires".to_string());
        args.push(lifetime.to_string());
    }

    tracing::info!("Adding PD route: ip {}", args.join(" "));
    let result = std::process::Command::new("ip").args(&args).output();

    if let Err(e) = result {
        tracing::error!("add_route_via error: {e:?}");
    }
}

/// Delete a route, used to clean up PD delegation routes on release/expiry.
pub fn del_route(prefix: Ipv6Addr, prefix_len: u8, iface_name: &str) {
    let args = vec![
        "-6".to_string(),
        "route".to_string(),
        "del".to_string(),
        format!("{}/{}", prefix, prefix_len),
        "dev".to_string(),
        iface_name.to_string(),
    ];

    tracing::debug!("Deleting PD route: ip {}", args.join(" "));
    let result = std::process::Command::new("ip").args(&args).output();

    if let Err(e) = result {
        tracing::error!("del_route error: {e:?}");
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
    use landscape_common::ipv6_pd::LDIAPrefix;
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
        let (subnet_network, router_addr) =
            allocate_subnet("2001:db8::3".parse().unwrap(), 60, 64, 2);
        println!("子网网络地址: {}/{}", subnet_network, 64);
        println!("路由器地址: {}", router_addr);
    }

    #[tokio::test]
    async fn test_change_channel_stays_alive_without_pd_sources() {
        let (change_tx, mut change_rx) = watch::channel(());
        let change_tx = Arc::new(change_tx);
        let ra_token = CancellationToken::new();
        let dhcpv6_token = CancellationToken::new();

        {
            let ra_clone = ra_token.clone();
            let dhcpv6_clone = dhcpv6_token.clone();
            tokio::spawn(async move {
                let _keep_alive = change_tx;
                tokio::join!(ra_clone.cancelled(), dhcpv6_clone.cancelled());
            });
        }

        change_rx.borrow_and_update();

        let result = tokio::time::timeout(Duration::from_millis(100), change_rx.changed()).await;
        assert!(result.is_err(), "should timeout, not get channel Err");

        ra_token.cancel();
        dhcpv6_token.cancel();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let result = change_rx.changed().await;
        assert!(result.is_err(), "sender dropped, should get Err");
    }

    #[tokio::test]
    async fn test_change_channel_receives_pd_notifications() {
        let (change_tx, mut change_rx) = watch::channel(());
        let change_tx = Arc::new(change_tx);
        let tx_clone = change_tx.clone();

        change_rx.borrow_and_update();

        tx_clone.send(()).unwrap();

        let result = tokio::time::timeout(Duration::from_millis(100), change_rx.changed()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}
