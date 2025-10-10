use arc_swap::ArcSwap;
use dhcproto::{Decodable, Decoder, Encodable, Encoder};
use landscape_common::config::ra::{IPV6RAConfig, IPV6RaConfigSource, IPv6RaPdConfig, RouterFlags};
use landscape_common::error::LdResult;
use landscape_common::ipv6_pd::{IAPrefixMap, LDIAPrefix};
use landscape_common::route::{LanIPv6RouteKey, LanRouteInfo};
use landscape_common::service::{DefaultWatchServiceStatus, ServiceStatus};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::dump::icmp::v6::options::{Icmpv6Message, RouterAdvertisement};
use crate::iface::ip::addresses_by_iface_name;
use crate::route::IpRouteService;
use landscape_common::net::MacAddr;
use landscape_common::net_proto::icmpv6::options::{
    IcmpV6Option, IcmpV6Options, PrefixInformation, RouteInformation,
};
use socket2::{Domain, Protocol, Socket, Type};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::u64;

static ICMPV6_MULTICAST_ROUTER: Ipv6Addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0x2);
static ICMPV6_MULTICAST: Ipv6Addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0x1);

pub struct ICMPv6ConfigInfo {
    pub rt_prefix: Ipv6Addr,
    pub rt_prefix_len: u8,

    pub sub_router: Ipv6Addr,
    pub sub_prefix: Ipv6Addr,
    pub sub_prefix_len: u8,

    pub ra_preferred_lifetime: u32,
    pub ra_valid_lifetime: u32,
}

#[derive(Default)]
pub struct RaIPRuntimeSource {
    static_info: Vec<ICMPv6ConfigInfo>,
    pd_info: HashMap<String, Arc<ArcSwap<Option<ICMPv6ConfigInfo>>>>,
}

#[tracing::instrument(skip(config, mac_addr, service_status, lan_info, route_service, prefix_map))]
pub async fn icmp_ra_server(
    config: IPV6RAConfig,
    // RA 通告要发送的 网卡 MAC 信息
    mac_addr: MacAddr,
    // RA 通告要发送的 网卡名称
    iface_name: String,
    service_status: DefaultWatchServiceStatus,
    lan_info: LanRouteInfo,
    route_service: IpRouteService,
    prefix_map: IAPrefixMap,
) -> LdResult<()> {
    let IPV6RAConfig { ad_interval, ra_flag, source } = config;

    let mut ctx = RaIPRuntimeSource::default();
    // TODO: ip link set ens5 addrgenmode none
    // OR
    // # 禁用IPv6路由器请求
    // sudo sysctl -w net.ipv6.conf.ens5.router_solicitations=0
    // # 对所有接口禁用
    // sudo sysctl -w net.ipv6.conf.all.router_solicitations=0
    // sudo sysctl -w net.ipv6.conf.default.router_solicitations=0

    let ipv6_forwarding_path = format!("/proc/sys/net/ipv6/conf/{}/forwarding", iface_name);
    std::fs::write(&ipv6_forwarding_path, "1")
        .expect(&format!("set {} ipv6 forwarding error", iface_name));

    service_status.just_change_status(ServiceStatus::Staring);
    //  sysctl -w net.ipv6.conf.all.forwarding=1
    let socket = Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::ICMPV6))?;
    socket.set_nonblocking(true)?;
    //
    // socket.set_multicast_loop_v6(false)?;
    // 设置 IPv6 单播 Hop Limit 为 255
    socket.set_unicast_hops_v6(255)?;

    // 如果您发送多播消息，还需要设置多播 Hop Limit
    socket.set_multicast_hops_v6(255)?;
    socket.bind_device(Some(iface_name.as_bytes()))?;

    let setting_result = crate::set_iface_ip_no_limit(
        &iface_name,
        std::net::IpAddr::V6(mac_addr.to_ipv6_link_local()),
        64,
    )
    .await;

    if !setting_result {
        tracing::error!("setting unicast_link_local error");
    }

    let address = addresses_by_iface_name(iface_name.to_string()).await;
    let mut link_ipv6_addr = None;
    let mut link_ifindex = 0;
    for addr in address.iter() {
        match addr.address {
            std::net::IpAddr::V4(_) => continue,
            std::net::IpAddr::V6(ipv6_addr) => {
                if ipv6_addr.is_unicast_link_local() {
                    link_ipv6_addr = Some(ipv6_addr);
                    link_ifindex = addr.ifindex;
                }
            }
        }
    }

    let Some(ipaddr) = link_ipv6_addr else {
        tracing::error!("can not find unicast_link_local");
        service_status.just_change_status(ServiceStatus::Stop);
        return Ok(());
    };
    tracing::info!("address {:?}", ipaddr);
    tracing::info!("link_ifindex {:?}", link_ifindex);

    socket.join_multicast_v6(&ICMPV6_MULTICAST_ROUTER, link_ifindex).unwrap();

    let udp_socket = UdpSocket::from_std(socket.into()).unwrap();
    let send_socket = Arc::new(udp_socket);

    let recive_socket_raw = send_socket.clone();

    let (message_tx, mut message_rx) = tokio::sync::mpsc::channel::<(Vec<u8>, SocketAddr)>(1024);

    // let data = [133, 0, 199, 38, 0, 0, 0, 1];
    // let addr = SocketAddrV6::new(ICMPV6_MULTICAST, 0, 0, 4);
    // send_socket.send_to(&data, addr).await.unwrap();
    // 接收数据
    tokio::spawn(async move {
        // 超时重发定时器

        let mut buf = vec![0u8; 65535];

        loop {
            tokio::select! {
                result = recive_socket_raw.recv_from(&mut buf) => {
                    // 接收数据包
                    match result {
                        Ok((len, addr)) => {
                            let message = buf[..len].to_vec();
                            if let Err(e) = message_tx.try_send((message, addr)) {
                                tracing::error!("Error sending message to channel: {:?}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error receiving data: {:?}", e);
                        }
                    }
                },
                _ = message_tx.closed() => {
                    tracing::error!("message_tx closed");
                    break;
                }
            }
        }

        tracing::info!("ICMP recv loop down");
    });

    let mut static_ip_infos = vec![];
    service_status.just_change_status(ServiceStatus::Running);

    let cancle_token = CancellationToken::new();
    let (trigger_tx, mut trigger_rx) = mpsc::channel::<()>(1024);
    for source in source {
        match source {
            IPV6RaConfigSource::Static(static_config) => {
                let rt_prefix_len = 56;
                let (sub_prefix, sub_router) = allocate_subnet(
                    static_config.base_prefix,
                    rt_prefix_len,
                    static_config.sub_prefix_len,
                    static_config.sub_index as u128,
                );
                set_iface_ip(sub_router, static_config.sub_prefix_len, &iface_name, None, None);
                static_ip_infos.push((
                    sub_router,
                    static_config.sub_prefix_len,
                    iface_name.clone(),
                ));
                let mut lan_info = lan_info.clone();
                lan_info.iface_ip = IpAddr::V6(sub_router.clone());
                lan_info.prefix = static_config.sub_prefix_len;
                let lan_info_key = LanIPv6RouteKey {
                    iface_name: iface_name.to_string(),
                    subnet_index: static_config.sub_index,
                };
                route_service.insert_ipv6_lan_route(lan_info_key, lan_info).await;
                ctx.static_info.push(ICMPv6ConfigInfo {
                    rt_prefix: static_config.base_prefix,
                    rt_prefix_len,
                    sub_router,
                    sub_prefix,
                    sub_prefix_len: static_config.sub_prefix_len,
                    ra_preferred_lifetime: static_config.ra_preferred_lifetime,
                    ra_valid_lifetime: static_config.ra_valid_lifetime,
                });
            }
            IPV6RaConfigSource::Pd(ipv6_ra_pd_config) => {
                let pd_prefix_info: Option<ICMPv6ConfigInfo> = None;
                let pd_prefix_info = Arc::new(ArcSwap::from_pointee(pd_prefix_info));
                let token = cancle_token.child_token();
                let mut ia_config_watch =
                    prefix_map.get_ia_prefix(&ipv6_ra_pd_config.depend_iface).await;
                ctx.pd_info.insert(ipv6_ra_pd_config.depend_iface.clone(), pd_prefix_info.clone());
                let trigger_tx_clone = trigger_tx.clone();

                let iface_name_clone = iface_name.clone();
                let lan_info_cloen = lan_info.clone();
                let route_service_clone = route_service.clone();

                let mut expire_time = Box::pin(tokio::time::sleep(Duration::from_secs(0)));
                // 立即检查一次
                let ia_prefix = ia_config_watch.borrow().clone();
                if let Some(ia_prefix) = ia_prefix {
                    pd_prefix_info.store(Arc::new(Some(
                        update_current_info(
                            &iface_name_clone,
                            ia_prefix,
                            &ipv6_ra_pd_config,
                            expire_time.as_mut(),
                            &lan_info_cloen,
                            &route_service_clone,
                        )
                        .await,
                    )));
                }

                tokio::spawn(async move {
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
                                        &iface_name_clone,
                                        ia_prefix,
                                        &ipv6_ra_pd_config,
                                        expire_time.as_mut(),
                                        &lan_info_cloen,
                                        &route_service_clone,
                                    ).await)));
                                }
                                // 立即进行通告
                                let _ = trigger_tx_clone.send(()).await;
                            },
                            _ = token.cancelled() => {
                                break;
                            }
                            _ = expire_time.as_mut() => {
                               pd_prefix_info.store(Arc::new(None));
                                let _ = trigger_tx_clone.send(()).await;
                                tracing::debug!("expire_time active");
                                expire_time.as_mut().set(tokio::time::sleep(Duration::from_secs(u64::MAX)));
                            }
                        }
                    }

                    tracing::info!(
                        "iface: {} prefix listen is down",
                        ipv6_ra_pd_config.depend_iface
                    );
                });
            }
        }
    }

    // let mut ia_config_watch = prefix_map.get_ia_prefix(&depend_iface).await;
    tracing::debug!("ICMP get IPv6 Prefix Watch");
    // let mut current_config_info: Option<ICMPv6ConfigInfo> = None;
    // let mut count_down = LdCountdown::new(Duration::from_secs(0));

    // let mut expire_time = Box::pin(tokio::time::sleep(Duration::from_secs(0)));
    // init
    // let ia_prefix = ia_config_watch.borrow().clone();
    // if let Some(ia_prefix) = ia_prefix {
    //     current_config_info = Some(
    //         update_current_info(
    //             &iface_name,
    //             ia_prefix,
    //             subnet_prefix,
    //             subnet_index,
    //             expire_time.as_mut(),
    //             &lan_info,
    //             &route_service,
    //         )
    //         .await,
    //     );
    // }

    // tracing::info!("ICMP v6 RA Server Running, RA interval: {ra_preferred_lifetime:?}s");
    let mut interval = Box::pin(tokio::time::interval(Duration::from_secs(ad_interval as u64)));

    let mut service_status_subscribe = service_status.subscribe();
    loop {
        tokio::select! {
            _ = interval.tick() => {
                interval_msg(
                    &mac_addr,
                    &send_socket,
                    &ctx,
                    ra_flag
                ).await;
            }
            _ = trigger_rx.recv() => {
                interval_msg(
                    &mac_addr,
                    &send_socket,
                    &ctx,
                    ra_flag
                ).await;
            },
            // 发送时间为 0 的
            // _ = expire_time.as_mut() => {
            //     current_config_info = None;
            //     tracing::debug!("expire_time active");
            //     expire_time.as_mut().set(tokio::time::sleep(Duration::from_secs(u64::MAX)));
            // }
            message_result = message_rx.recv() => {
                // 处理接收到的数据包
                match message_result {
                    Some(data) => {
                        // handle RS
                        handle_rs_msg(
                            &mac_addr,
                            data,
                            &send_socket,
                            &ctx,
                            ra_flag
                        ).await;
                    }
                    // message_rx close
                    None => break
                }
            },
            // IA_PREFIX change
            // change_result = ia_config_watch.changed() => {
            //     tracing::info!("IA_PREFIX update");
            //     if let Err(_) = change_result {
            //         tracing::error!("get change result error. exit loop");
            //         break;
            //     }
            //     let ia_prefix = ia_config_watch.borrow().clone();
            //     if let Some(ia_prefix) = ia_prefix {
            //         current_config_info = Some(
            //             update_current_info(
            //                 &iface_name,
            //                 ia_prefix,
            //                 subnet_prefix,
            //                 subnet_index,
            //                 expire_time.as_mut(),
            //                 &lan_info,
            //                 &route_service,
            //             ).await
            //         );
            //     }
            //     // 立即进行通告
            //     interval.reset_immediately();
            // }
            // status change
            change_result = service_status_subscribe.changed() => {
                tracing::debug!("ICMP v6 RA Service change");
                if let Err(_) = change_result {
                    tracing::error!("get change result error. exit loop");
                    break;
                }

                if service_status.is_exit() {
                    service_status.just_change_status(ServiceStatus::Stop);
                    tracing::info!("release send and stop");
                    break;
                }
            }
        }
    }

    route_service.remove_ipv6_lan_route(&iface_name).await;
    for (ip, prefix, iface_name) in static_ip_infos {
        del_iface_ip(ip, prefix, &iface_name)
    }

    std::fs::write(&ipv6_forwarding_path, "0")
        .expect(&format!("set {} ipv6 forwarding error", iface_name));
    tracing::info!("ICMP v6 RA Server Stop: {:#?}", service_status);
    if !service_status.is_stop() {
        service_status.just_change_status(ServiceStatus::Stop);
    }
    Ok(())
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
    lan_info.iface_ip = IpAddr::V6(sub_router.clone());
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
async fn interval_msg(
    my_mac_addr: &MacAddr,
    send_socket: &UdpSocket,
    ctx: &RaIPRuntimeSource,
    ra_flag: RouterFlags,
) {
    build_and_send_ra(
        my_mac_addr,
        send_socket,
        SocketAddr::new(IpAddr::V6(ICMPV6_MULTICAST), 0),
        ctx,
        ra_flag,
    )
    .await;
}

async fn handle_rs_msg(
    my_mac_addr: &MacAddr,
    (msg, target_addr): (Vec<u8>, SocketAddr),
    send_socket: &UdpSocket,
    ctx: &RaIPRuntimeSource,
    ra_flag: RouterFlags,
) {
    let icmp_v6_msg = Icmpv6Message::decode(&mut Decoder::new(&msg));
    let icmp_v6_msg = match icmp_v6_msg {
        Ok(msg) => msg,
        Err(e) => {
            tracing::error!("decode msg error: {e:?}");
            return;
        }
    };

    let target_ip = match target_addr {
        SocketAddr::V4(socket_addr_v4) => {
            tracing::debug!("not ipv6 msg ignore: {socket_addr_v4:?}");
            return;
        }
        SocketAddr::V6(socket_addr_v6) => {
            // println!("scope_id {:?}", socket_addr_v6.scope_id());
            socket_addr_v6.ip().to_owned()
        }
    };

    match icmp_v6_msg {
        Icmpv6Message::RouterSolicitation(router_solicitation) => {
            tracing::debug!("router_solicitation: {router_solicitation:?}");
            tracing::debug!("target_ip: {target_ip:?}");
            build_and_send_ra(my_mac_addr, send_socket, target_addr, ctx, ra_flag).await;
        }
        Icmpv6Message::RouterAdvertisement(_) => {}
        Icmpv6Message::Unassigned(msg_type, _) => {
            tracing::error!("recv not handle Icmpv6Message msg_type: {msg_type:?}");
        }
    }
}

async fn send_data(msg: &Icmpv6Message, send_socket: &UdpSocket, target_sock: SocketAddr) {
    let mut buf = Vec::new();
    let mut e = Encoder::new(&mut buf);
    if let Err(e) = msg.encode(&mut e) {
        tracing::error!("msg encode error: {e:?}");
        return;
    }
    match send_socket.send_to(&buf, &target_sock).await {
        Ok(len) => {
            tracing::debug!("send icmpv6 fram: {msg:?},  len: {len:?}");
        }
        Err(e) => {
            tracing::error!("error: {:?}", e);
        }
    }
}

/// 根据传入的前缀、目标子网前缀长度以及子网索引，返回对应子网的网络地址和一个路由器地址
fn allocate_subnet(
    pd_ip: Ipv6Addr,
    pd_prefix_len: u8,
    sub_prefix_len: u8,
    subnet_index: u128,
) -> (Ipv6Addr, Ipv6Addr) {
    // 子网前缀长度必须大于等于原始前缀长度
    assert!(sub_prefix_len >= pd_prefix_len, "子网前缀长度必须大于等于原始前缀长度");

    // 计算可划分的子网总数
    let max_subnets = 1u128 << (sub_prefix_len - pd_prefix_len);
    assert!(subnet_index < max_subnets, "subnet_index 超出可用子网范围");

    // 将 IPv6 地址转换为 u128 类型进行位运算
    let prefix_u128 = u128::from(pd_ip);

    // 计算父网络地址（假设 prefix_ip 已经对齐到 prefix_len）
    let parent_mask = (!0u128) << (128 - pd_prefix_len);
    let parent_network = prefix_u128 & parent_mask;

    // 计算子网掩码：前 sub_prefix_len 位为 1，其余为 0
    let sub_mask = (!0u128) << (128 - sub_prefix_len);

    // 基础子网地址，对齐到子网前缀边界
    let base_network = parent_network & sub_mask;

    // 每个子网的地址块大小
    let subnet_size = 1u128 << (128 - sub_prefix_len);

    // 根据子网索引计算目标子网的网络地址
    let subnet_network = base_network + (subnet_index * subnet_size);

    // 选择该子网的第一个地址作为路由器地址
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

async fn build_and_send_ra(
    my_mac_addr: &MacAddr,
    send_socket: &UdpSocket,
    target_addr: SocketAddr,
    ctx: &RaIPRuntimeSource,
    ra_flag: RouterFlags,
) {
    let mut is_empty = true;
    let mut opts = IcmpV6Options::new();
    opts.insert(IcmpV6Option::SourceLinkLayerAddress(my_mac_addr.octets().to_vec()));
    for static_prefix in ctx.static_info.iter() {
        is_empty = false;
        opts.insert(IcmpV6Option::PrefixInformation(PrefixInformation::new(
            static_prefix.sub_prefix_len,
            600,
            300,
            static_prefix.sub_prefix,
        )));

        opts.insert(IcmpV6Option::RouteInformation(RouteInformation::new(
            static_prefix.rt_prefix_len,
            static_prefix.rt_prefix,
        )));
        opts.insert(IcmpV6Option::RecursiveDNSServer((60_000, static_prefix.sub_router)));
    }

    for (_, pd_prefix) in ctx.pd_info.iter() {
        let pd_prefix = pd_prefix.load();
        let Some(pd_prefix) = pd_prefix.as_ref() else {
            continue;
        };
        is_empty = false;
        opts.insert(IcmpV6Option::PrefixInformation(PrefixInformation::new(
            pd_prefix.sub_prefix_len,
            600,
            300,
            pd_prefix.sub_prefix,
        )));

        opts.insert(IcmpV6Option::RouteInformation(RouteInformation::new(
            pd_prefix.rt_prefix_len,
            pd_prefix.rt_prefix,
        )));
        opts.insert(IcmpV6Option::RecursiveDNSServer((60_000, pd_prefix.sub_router)));
    }
    // opts.insert(IcmpV6Option::PrefixInformation(PrefixInformation::new(
    //     current_info.subnet_prefix,
    //     ra_valid_lifetime,
    //     ra_preferred_lifetime,
    //     current_info.subnet,
    // )));
    // opts.insert(IcmpV6Option::RouteInformation(RouteInformation::new(
    //     current_info.watch_ia_prefix.prefix_len,
    //     current_info.watch_ia_prefix.prefix_ip,
    // )));
    // opts.insert(IcmpV6Option::RecursiveDNSServer((60_000, current_info.subnet_router)));
    opts.insert(IcmpV6Option::MTU(1500));
    opts.insert(IcmpV6Option::AdvertisementInterval(60_000));

    let msg = Icmpv6Message::RouterAdvertisement(RouterAdvertisement::new(ra_flag.into(), opts));

    if !is_empty {
        send_data(&msg, send_socket, target_addr).await;
    } else {
        tracing::error!("current config_info is None, can not handle message");
    }
}

#[cfg(test)]
mod tests {
    use crate::icmp::v6::allocate_subnet;
    use landscape_common::{config::ra::IPv6RaStaticConfig, ipv6_pd::LDIAPrefix};

    #[test]
    fn test() {
        // 示例：假设原始前缀为 2001:db8::/48，我们希望划分出 /64 的子网，并选择第 2 个子网（索引从 0 开始）
        let ldia_prefix = LDIAPrefix {
            preferred_lifetime: 3600,
            valid_lifetime: 7200,
            prefix_len: 48,
            prefix_ip: "2001:db8::".parse().unwrap(),
            last_update_time: 0.0,
        };
        let sub_prefix_len = 64;
        let subnet_index = 2; // 0 表示第一个子网，1 表示第二个子网，以此类推
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
        // 示例：假设原始前缀为 2001:db8::/48，我们希望划分出 /64 的子网，并选择第 2 个子网（索引从 0 开始）
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
}
