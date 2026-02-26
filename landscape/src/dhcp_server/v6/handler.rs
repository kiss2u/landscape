use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

use arc_swap::ArcSwap;
use landscape_common::dhcp::v6_server::config::DHCPv6ServerConfig;
use landscape_common::dhcp::v6_server::status::DHCPv6OfferInfo;
use landscape_common::net::MacAddr;
use landscape_common::net_proto::udp::dhcp::DhcpV6MessageType;
use landscape_common::service::WatchService;
use landscape_common::LANDSCAPE_DEFAULE_DHCP_V6_SERVER_PORT;

use dhcproto::v6::{self, IAAddr, IAPrefix, Status, StatusCode, IANA, IAPD};
use dhcproto::{Decodable, Decoder, Encodable, Encoder};

use socket2::{Domain, Protocol, Type};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

use crate::icmp::v6::ICMPv6ConfigInfo;

use super::server::DHCPv6Server;
use super::utils::{
    combine_prefix_suffix, compute_delegated_prefix, extract_mac_from_duid, gen_server_duid,
};

const LEASE_EXPIRE_INTERVAL: u64 = 60 * 10;
static DHCPV6_MULTICAST: Ipv6Addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0x1, 0x2);

/// Main DHCPv6 server function
#[tracing::instrument(skip(
    dhcpv6_config,
    ra_pd_runtime_sources,
    ra_static_infos,
    service_status,
    assigned_info,
    static_bindings
))]
pub async fn dhcp_v6_server(
    link_ifindex: u32,
    iface_name: String,
    mac: MacAddr,
    dhcpv6_config: DHCPv6ServerConfig,
    ra_pd_runtime_sources: Vec<Arc<ArcSwap<Option<ICMPv6ConfigInfo>>>>,
    ra_static_infos: Vec<ICMPv6ConfigInfo>,
    service_status: WatchService,
    assigned_info: Arc<RwLock<DHCPv6OfferInfo>>,
    static_bindings: HashMap<MacAddr, Ipv6Addr>,
) {
    let server_duid = gen_server_duid(&mac);

    let mut dhcp_server = DHCPv6Server::init(&dhcpv6_config, static_bindings);
    dhcp_server.server_duid = server_duid.clone();

    let socket_addr =
        SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), LANDSCAPE_DEFAULE_DHCP_V6_SERVER_PORT);

    let socket2 = socket2::Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP)).unwrap();

    socket2.set_only_v6(true).unwrap();
    socket2.set_reuse_address(true).unwrap();
    socket2.set_reuse_port(true).unwrap();

    socket2.bind(&socket_addr.into()).unwrap();
    socket2.set_nonblocking(true).unwrap();

    if let Err(e) = socket2.bind_device(Some(iface_name.as_bytes())) {
        tracing::error!("DHCPv6 bind_device error: {e:?}");
        return;
    }

    socket2.join_multicast_v6(&DHCPV6_MULTICAST, link_ifindex).unwrap();

    let socket = UdpSocket::from_std(socket2.into()).unwrap();
    let send_socket = Arc::new(socket);
    let recv_socket = send_socket.clone();

    let (message_tx, mut message_rx) = tokio::sync::mpsc::channel::<(Vec<u8>, SocketAddr)>(1024);

    // Receive loop
    tokio::spawn(async move {
        let mut buf = vec![0u8; 65535];
        loop {
            tokio::select! {
                result = recv_socket.recv_from(&mut buf) => {
                    match result {
                        Ok((len, addr)) => {
                            tracing::debug!("DHCPv6 received {} bytes from {}", len, addr);
                            let message = buf[..len].to_vec();
                            if let Err(e) = message_tx.try_send((message, addr)) {
                                tracing::error!("DHCPv6 channel send error: {:?}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("DHCPv6 recv error: {:?}", e);
                        }
                    }
                },
                _ = message_tx.closed() => {
                    break;
                }
            }
        }
        tracing::info!("DHCPv6 recv loop down");
    });

    tracing::info!("DHCPv6 Server Running on {iface_name}");

    let mut service_status_subscribe = service_status.subscribe();
    let timeout_timer = tokio::time::sleep(tokio::time::Duration::from_secs(LEASE_EXPIRE_INTERVAL));
    tokio::pin!(timeout_timer);

    loop {
        tokio::select! {
            message = message_rx.recv() => {
                match message {
                    Some((msg_bytes, msg_addr)) => {
                        let need_update = handle_dhcpv6_message(
                            &mut dhcp_server,
                            &send_socket,
                            &server_duid,
                            (msg_bytes, msg_addr),
                            &ra_pd_runtime_sources,
                            &ra_static_infos,
                        ).await;
                        if need_update {
                            update_assigned_info(
                                assigned_info.clone(),
                                dhcp_server.get_offered_info(&ra_pd_runtime_sources, &ra_static_infos),
                            ).await;
                        }
                    },
                    None => {
                        tracing::error!("DHCPv6 message channel closed");
                        break;
                    }
                }
            }
            _ = &mut timeout_timer => {
                dhcp_server.clean_expired_na();
                dhcp_server.clean_expired_pd();
                timeout_timer.as_mut().reset(
                    tokio::time::Instant::now() + tokio::time::Duration::from_secs(LEASE_EXPIRE_INTERVAL)
                );
                update_assigned_info(
                    assigned_info.clone(),
                    dhcp_server.get_offered_info(&ra_pd_runtime_sources, &ra_static_infos),
                ).await;
            }
            change_result = service_status_subscribe.changed() => {
                if let Err(_) = change_result {
                    tracing::error!("DHCPv6 service status channel error");
                    break;
                }
                if service_status.is_exit() {
                    break;
                }
            }
        }
    }

    tracing::info!("DHCPv6 Server Stop on {iface_name}");
}

async fn update_assigned_info(assigned_info: Arc<RwLock<DHCPv6OfferInfo>>, info: DHCPv6OfferInfo) {
    match tokio::time::timeout(tokio::time::Duration::from_secs(5), assigned_info.write()).await {
        Ok(mut write_lock) => {
            *write_lock = info;
        }
        Err(_) => {
            tracing::error!("DHCPv6 failed to acquire write lock");
        }
    }
}

async fn handle_dhcpv6_message(
    server: &mut DHCPv6Server,
    send_socket: &Arc<UdpSocket>,
    server_duid: &[u8],
    (msg_bytes, msg_addr): (Vec<u8>, SocketAddr),
    runtime_sources: &[Arc<ArcSwap<Option<ICMPv6ConfigInfo>>>],
    static_infos: &[ICMPv6ConfigInfo],
) -> bool {
    let msg = match v6::Message::decode(&mut Decoder::new(&msg_bytes)) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("DHCPv6 decode error: {e:?}");
            return false;
        }
    };

    // Extract client ID
    let client_duid = match msg.opts().get(v6::OptionCode::ClientId) {
        Some(v6::DhcpOption::ClientId(duid)) => duid.clone(),
        _ => {
            tracing::warn!("DHCPv6 message without ClientId");
            return false;
        }
    };

    let mac = extract_mac_from_duid(&client_duid);

    // Extract IANA ID if present
    let iana_id = msg.opts().get(v6::OptionCode::IANA).and_then(|opt| {
        if let v6::DhcpOption::IANA(iana) = opt {
            Some(iana.id)
        } else {
            None
        }
    });

    // Extract IAPD ID if present
    let iapd_id = msg.opts().get(v6::OptionCode::IAPD).and_then(|opt| {
        if let v6::DhcpOption::IAPD(iapd) = opt {
            Some(iapd.id)
        } else {
            None
        }
    });

    match msg.msg_type() {
        DhcpV6MessageType::Solicit => {
            tracing::info!(
                "DHCPv6 SOLICIT from {:?}, IANA ID: {:?}, IAPD ID: {:?}",
                mac,
                iana_id,
                iapd_id
            );
            tracing::info!(
                "DHCPv6 Server config - NA: {:?}, PD: {:?}",
                server.na_config.is_some(),
                server.pd_config.is_some()
            );
            tracing::info!(
                "DHCPv6 static_infos count: {}, runtime_sources count: {}",
                static_infos.len(),
                runtime_sources.len()
            );

            // Allocate addresses/prefixes
            if let Some(_) = &server.na_config {
                if iana_id.is_some() {
                    server.offer_na_suffix(&client_duid, mac, None);
                }
            }
            if let Some(_) = &server.pd_config {
                if iapd_id.is_some() {
                    server.offer_pd_index(&client_duid);
                }
            }

            // Build ADVERTISE
            let mut reply = v6::Message::new(DhcpV6MessageType::Advertise);
            reply.set_xid(msg.xid());
            reply.opts_mut().insert(v6::DhcpOption::ClientId(client_duid.clone()));
            reply.opts_mut().insert(v6::DhcpOption::ServerId(server_duid.to_vec()));
            reply.opts_mut().insert(v6::DhcpOption::Preference(255));

            if let Some(iana_id) = iana_id {
                if server.na_config.is_some() {
                    let na_prefixes =
                        server.get_qualifying_na_prefixes(runtime_sources, static_infos);
                    tracing::info!(
                        "DHCPv6 IANA - qualifying prefixes count: {}",
                        na_prefixes.len()
                    );
                    for (prefix, len) in &na_prefixes {
                        tracing::info!("  - Prefix: {}/{}", prefix, len);
                    }
                    if !na_prefixes.is_empty() {
                        let iana = build_iana_options(server, &client_duid, iana_id, &na_prefixes);
                        reply.opts_mut().insert(v6::DhcpOption::IANA(iana));
                    } else {
                        tracing::warn!("DHCPv6 IANA: No qualifying prefixes available!");
                    }
                }
            }

            if let Some(iapd_id) = iapd_id {
                if server.pd_config.is_some() {
                    let pd_prefixes =
                        server.get_qualifying_pd_prefixes(runtime_sources, static_infos);
                    if !pd_prefixes.is_empty() {
                        let iapd = build_iapd_options(server, &client_duid, iapd_id, &pd_prefixes);
                        reply.opts_mut().insert(v6::DhcpOption::IAPD(iapd));
                    }
                }
            }

            if !server.dns_servers.is_empty() {
                reply
                    .opts_mut()
                    .insert(v6::DhcpOption::DomainNameServers(server.dns_servers.clone()));
            }

            send_dhcpv6_reply(&reply, send_socket, msg_addr).await;
            return true;
        }

        DhcpV6MessageType::Request | DhcpV6MessageType::Renew | DhcpV6MessageType::Rebind => {
            // Verify server ID for Request and Renew (not Rebind)
            if msg.msg_type() != DhcpV6MessageType::Rebind {
                match msg.opts().get(v6::OptionCode::ServerId) {
                    Some(v6::DhcpOption::ServerId(sid)) if sid == server_duid => {}
                    _ => {
                        tracing::debug!("DHCPv6 message not for us (wrong ServerId)");
                        return false;
                    }
                }
            }

            tracing::debug!("DHCPv6 {:?} from {:?}", msg.msg_type(), mac);

            // Confirm/allocate
            if server.na_config.is_some() && iana_id.is_some() {
                if !server.confirm_na(&client_duid) {
                    // New client during REBIND or first REQUEST
                    server.offer_na_suffix(&client_duid, mac, None);
                    server.confirm_na(&client_duid);
                }
            }
            if server.pd_config.is_some() && iapd_id.is_some() {
                if !server.confirm_pd(&client_duid) {
                    server.offer_pd_index(&client_duid);
                    server.confirm_pd(&client_duid);
                }
            }

            // Build REPLY
            let mut reply = v6::Message::new(DhcpV6MessageType::Reply);
            reply.set_xid(msg.xid());
            reply.opts_mut().insert(v6::DhcpOption::ClientId(client_duid.clone()));
            reply.opts_mut().insert(v6::DhcpOption::ServerId(server_duid.to_vec()));

            if let Some(iana_id) = iana_id {
                if server.na_config.is_some() {
                    let na_prefixes =
                        server.get_qualifying_na_prefixes(runtime_sources, static_infos);
                    if !na_prefixes.is_empty() {
                        let iana = build_iana_options(server, &client_duid, iana_id, &na_prefixes);
                        reply.opts_mut().insert(v6::DhcpOption::IANA(iana));
                    }
                }
            }

            if let Some(iapd_id) = iapd_id {
                if server.pd_config.is_some() {
                    let pd_prefixes =
                        server.get_qualifying_pd_prefixes(runtime_sources, static_infos);
                    if !pd_prefixes.is_empty() {
                        let iapd = build_iapd_options(server, &client_duid, iapd_id, &pd_prefixes);
                        reply.opts_mut().insert(v6::DhcpOption::IAPD(iapd));
                    }
                }
            }

            if !server.dns_servers.is_empty() {
                reply
                    .opts_mut()
                    .insert(v6::DhcpOption::DomainNameServers(server.dns_servers.clone()));
            }

            send_dhcpv6_reply(&reply, send_socket, msg_addr).await;
            return true;
        }

        DhcpV6MessageType::Release => {
            match msg.opts().get(v6::OptionCode::ServerId) {
                Some(v6::DhcpOption::ServerId(sid)) if sid == server_duid => {}
                _ => return false,
            }

            tracing::info!("DHCPv6 RELEASE from {:?}", mac);
            server.release_na(&client_duid);
            server.release_pd(&client_duid);

            let mut reply = v6::Message::new(DhcpV6MessageType::Reply);
            reply.set_xid(msg.xid());
            reply.opts_mut().insert(v6::DhcpOption::ClientId(client_duid));
            reply.opts_mut().insert(v6::DhcpOption::ServerId(server_duid.to_vec()));
            reply.opts_mut().insert(v6::DhcpOption::StatusCode(StatusCode {
                status: Status::Success,
                msg: String::new(),
            }));

            send_dhcpv6_reply(&reply, send_socket, msg_addr).await;
            return true;
        }

        DhcpV6MessageType::Decline => {
            tracing::info!("DHCPv6 DECLINE from {:?}", mac);
            // Mark declined, remove from offered
            server.release_na(&client_duid);
            return true;
        }

        DhcpV6MessageType::Confirm => {
            let mut reply = v6::Message::new(DhcpV6MessageType::Reply);
            reply.set_xid(msg.xid());
            reply.opts_mut().insert(v6::DhcpOption::ClientId(client_duid));
            reply.opts_mut().insert(v6::DhcpOption::ServerId(server_duid.to_vec()));
            reply.opts_mut().insert(v6::DhcpOption::StatusCode(StatusCode {
                status: Status::Success,
                msg: String::new(),
            }));

            send_dhcpv6_reply(&reply, send_socket, msg_addr).await;
            return false;
        }

        DhcpV6MessageType::InformationRequest => {
            let mut reply = v6::Message::new(DhcpV6MessageType::Reply);
            reply.set_xid(msg.xid());
            reply.opts_mut().insert(v6::DhcpOption::ClientId(client_duid));
            reply.opts_mut().insert(v6::DhcpOption::ServerId(server_duid.to_vec()));

            if !server.dns_servers.is_empty() {
                reply
                    .opts_mut()
                    .insert(v6::DhcpOption::DomainNameServers(server.dns_servers.clone()));
            }

            send_dhcpv6_reply(&reply, send_socket, msg_addr).await;
            return false;
        }

        other => {
            tracing::debug!("DHCPv6 ignoring message type: {:?}", other);
            return false;
        }
    }
}

/// Build IA_NA options for a reply message
fn build_iana_options(
    server: &DHCPv6Server,
    client_duid: &[u8],
    iana_id: u32,
    qualifying_prefixes: &[(Ipv6Addr, u8)],
) -> v6::IANA {
    let na_config = server.na_config.as_ref().unwrap();
    let mut iana_opts = v6::DhcpOptions::new();

    if let Some(cache) = server.na_offered.get(client_duid) {
        for (prefix, prefix_len) in qualifying_prefixes {
            let addr = combine_prefix_suffix(*prefix, *prefix_len, cache.suffix);
            iana_opts.insert(v6::DhcpOption::IAAddr(IAAddr {
                addr,
                preferred_life: cache.preferred_time,
                valid_life: cache.valid_time,
                opts: v6::DhcpOptions::new(),
            }));
        }
    }

    iana_opts.insert(v6::DhcpOption::StatusCode(StatusCode {
        status: Status::Success,
        msg: String::new(),
    }));

    IANA {
        id: iana_id,
        t1: na_config.preferred_lifetime / 2,
        t2: (na_config.preferred_lifetime * 4) / 5,
        opts: iana_opts,
    }
}

/// Build IA_PD options for a reply message
fn build_iapd_options(
    server: &DHCPv6Server,
    client_duid: &[u8],
    iapd_id: u32,
    qualifying_prefixes: &[(Ipv6Addr, u8)],
) -> v6::IAPD {
    let pd_config = server.pd_config.as_ref().unwrap();
    let mut iapd_opts = v6::DhcpOptions::new();

    if let Some(cache) = server.pd_offered.get(client_duid) {
        for (base_prefix, base_prefix_len) in qualifying_prefixes {
            let delegated = compute_delegated_prefix(
                *base_prefix,
                *base_prefix_len,
                pd_config.delegate_prefix_len,
                cache.sub_index,
            );
            iapd_opts.insert(v6::DhcpOption::IAPrefix(IAPrefix {
                preferred_lifetime: cache.preferred_time,
                valid_lifetime: cache.valid_time,
                prefix_len: pd_config.delegate_prefix_len,
                prefix_ip: delegated,
                opts: v6::DhcpOptions::new(),
            }));
        }
    }

    iapd_opts.insert(v6::DhcpOption::StatusCode(StatusCode {
        status: Status::Success,
        msg: String::new(),
    }));

    IAPD {
        id: iapd_id,
        t1: pd_config.preferred_lifetime / 2,
        t2: (pd_config.preferred_lifetime * 4) / 5,
        opts: iapd_opts,
    }
}

async fn send_dhcpv6_reply(msg: &v6::Message, send_socket: &UdpSocket, target: SocketAddr) {
    let mut buf = Vec::new();
    let mut e = Encoder::new(&mut buf);
    if let Err(e) = msg.encode(&mut e) {
        tracing::error!("DHCPv6 encode error: {e:?}");
        return;
    }
    match send_socket.send_to(&buf, &target).await {
        Ok(len) => {
            tracing::debug!("DHCPv6 sent {} bytes to {}", len, target);
        }
        Err(e) => {
            tracing::error!("DHCPv6 send error: {:?}", e);
        }
    }
}
