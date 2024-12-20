use hickory_server::ServerFuture;
use multi_rule_dns_server::DnsServer;
use protos::geo::GeoSiteListOwned;
use rule::{DNSRuleConfig, DomainConfig};
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::watch;

mod connection;
pub mod multi_rule_dns_server;
pub mod options;
pub mod protos;
pub mod rule;

/// Timeout for TCP connections.
const TCP_TIMEOUT: Duration = Duration::from_secs(10);

const GEO_SITE_FILE_NAME: &'static str = "geosite.dat";

#[derive(Serialize, Debug, PartialEq, Clone)]
#[serde(tag = "t")]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    // 启动中
    Staring,
    // 正在运行
    Running,
    // 正在停止
    Stopping,
    // 停止运行
    Stop { message: Option<String> },
}

fn serialize_status<S>(
    sender: &watch::Sender<ServiceStatus>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    sender.borrow().serialize(serializer)
}

// fn serialize_rules<S>(
//     rules: &Arc<RwLock<Vec<DNSRuleConfig>>>,
//     serializer: S,
// ) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     let read_locl = rules.blocking_read();
//     read_locl.serialize(serializer)
// }

#[derive(Serialize, Debug, Clone)]
pub struct LandscapeDnsService {
    #[serde(serialize_with = "serialize_status")]
    pub status: watch::Sender<ServiceStatus>,
    // 配置的数据路径位置
    // 里面存储的是 rule 的配置
    pub data_path: PathBuf,
    // pub rules_config: Vec<DNSRuleConfig>,
}

impl LandscapeDnsService {
    pub async fn new(data_path: PathBuf) -> Self {
        let (status, _) = watch::channel(ServiceStatus::Stop { message: None });
        // let rules_config = vec![];
        LandscapeDnsService {
            status,
            data_path,
            // rules_config
        }
    }

    pub async fn read_geo_site_file(&self) -> HashMap<String, Vec<DomainConfig>> {
        let mut result = HashMap::new();
        let geo_file_path = self.data_path.join(GEO_SITE_FILE_NAME);

        // 读取文件并解析为 Owned 结构体
        let data = tokio::fs::read(geo_file_path).await.unwrap();
        let list = GeoSiteListOwned::try_from(data).unwrap();

        for entry in list.entry.iter() {
            let domains = entry.domain.iter().map(DomainConfig::from).collect();
            result.insert(entry.country_code.to_string(), domains);
        }
        result
    }

    pub async fn start(&self, udp_port: u16, tcp_port: Option<u16>, dns_rules: Vec<DNSRuleConfig>) {
        let handler = DnsServer::new(dns_rules, self.read_geo_site_file().await);
        let mut server = ServerFuture::new(handler);
        let status_clone = self.status.clone();

        status_clone.send_replace(ServiceStatus::Staring);
        // register UDP listeners
        server.register_socket(
            UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), udp_port))
                .await
                .unwrap(),
        );

        if let Some(tcp_port) = tcp_port {
            server.register_listener(
                TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), tcp_port))
                    .await
                    .unwrap(),
                TCP_TIMEOUT,
            );
        }

        tokio::spawn(async move {
            status_clone.send_replace(ServiceStatus::Running);
            let mut status_rx = status_clone.subscribe();
            let state_end_loop = status_rx.wait_for(|status| status == &ServiceStatus::Stopping);

            let trigger_by_ui = tokio::select! {
                _ = state_end_loop => {
                    true
                },
                result = server.block_until_done() => {
                    let message = if let Err(e) = result {
                        Some(e.to_string())
                    } else {
                        None
                    };
                    status_clone.send_replace(ServiceStatus::Stop { message });
                    false
                }
            };

            if trigger_by_ui {
                if let Err(e) = server.shutdown_gracefully().await {
                    status_clone.send_replace(ServiceStatus::Stop { message: Some(e.to_string()) });
                } else {
                    status_clone.send_replace(ServiceStatus::Stop { message: None });
                }
            }
        });
    }

    pub fn stop(&self) {
        let if_need_stop = |state: &mut ServiceStatus| match state {
            ServiceStatus::Stop { message: _ } => false,
            _ => {
                *state = ServiceStatus::Stopping;
                true
            }
        };
        self.status.send_if_modified(if_need_stop);
    }
}
