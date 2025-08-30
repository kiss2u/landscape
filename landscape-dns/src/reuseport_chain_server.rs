use std::{
    collections::HashMap,
    net::{Ipv6Addr, SocketAddr, SocketAddrV6},
    sync::Arc,
};

use hickory_server::ServerFuture;
use landscape_common::{config::dns::DNSRuntimeRule, service::DefaultWatchServiceStatus};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::{
    convert_record_type, server::request::LandscapeDnsRequestHandle, CheckDnsReq, CheckDnsResult,
};

#[derive(Clone)]
pub struct LandscapeReusePortChainDnsServer {
    pub status: DefaultWatchServiceStatus,
    flow_dns_server: Arc<Mutex<HashMap<u32, (LandscapeDnsRequestHandle, CancellationToken)>>>,
    pub addr: SocketAddr,
}

impl LandscapeReusePortChainDnsServer {
    pub fn new(listen_port: u16) -> Self {
        crate::check_resolver_conf();
        let status = DefaultWatchServiceStatus::new();
        Self {
            status,
            flow_dns_server: Arc::new(Mutex::new(HashMap::new())),
            addr: SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, listen_port, 0, 0)),
        }
    }

    pub fn get_status(&self) -> &DefaultWatchServiceStatus {
        &self.status
    }

    pub async fn refresh_flow_server(&self, flow_id: u32, dns_rules: Vec<DNSRuntimeRule>) {
        {
            let mut lock = self.flow_dns_server.lock().await;
            if let Some((old_handler, _)) = lock.get_mut(&flow_id) {
                old_handler.renew_rules(dns_rules);
                return;
            }
        }

        let handler = LandscapeDnsRequestHandle::new(dns_rules, flow_id);
        let token = start_dns_server(flow_id, self.addr, handler.clone()).await;

        {
            let mut lock = self.flow_dns_server.lock().await;
            lock.insert(flow_id, (handler, token));
        }
    }

    pub async fn check_domain(&self, req: CheckDnsReq) -> CheckDnsResult {
        let handler = {
            let flow_server = self.flow_dns_server.lock().await;
            if let Some((handler, _)) = flow_server.get(&req.flow_id) {
                Some(handler.clone())
            } else {
                None
            }
        };

        if let Some(handler) = handler {
            handler.check_domain(&req.get_domain(), convert_record_type(req.record_type)).await
        } else {
            CheckDnsResult::default()
        }
    }
}

pub async fn start_dns_server(
    flow_id: u32,
    addr: SocketAddr,
    handler: LandscapeDnsRequestHandle,
) -> CancellationToken {
    let Ok((udp, sock_fd)) = crate::reuseport_server::listener::create_udp_socket(addr).await
    else {
        tracing::error!("[flow: {flow_id}]: create udp socket error");
        let result = CancellationToken::new();
        result.cancel();
        return result;
    };

    landscape_ebpf::map_setting::dns::setting_dns_sock_map(sock_fd, flow_id);
    landscape_ebpf::dns_dispatcher::attach_reuseport_ebpf(sock_fd).unwrap();
    let mut server = ServerFuture::new(handler);
    server.register_socket(udp);

    let token = server.shutdown_token().clone();

    tokio::spawn(async move {
        if let Err(e) = server.block_until_done().await {
            tracing::error!("[flow: {flow_id}]: server down, error: {e:?}");
        } else {
            tracing::info!("[flow: {flow_id}]: server down");
        }
    });

    token
}
