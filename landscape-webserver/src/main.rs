use std::net::SocketAddr;

use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    routing::{get, post},
    Router,
};

use colored::Colorize;
use config_service::{
    dns_rule::get_dns_rule_config_paths, dst_ip_rule::get_dst_ip_rule_config_paths,
    firewall_rule::get_firewall_rule_config_paths, flow_rule::get_flow_rule_config_paths,
    geo_site::get_geo_site_config_paths,
};
use landscape::{
    boot::{boot_check, log::init_logger},
    config_service::{
        dns_rule::DNSRuleService, dst_ip_rule::DstIpRuleService,
        firewall_rule::FirewallRuleService, flow_rule::FlowRuleService,
        geo_site_service::GeoSiteService,
    },
    sys_service::dns_service::LandscapeDnsService,
};
use landscape_common::{
    args::{DATABASE_ARGS, LAND_ARGS, LAND_HOME_PATH, LAND_LOG_ARGS, LAND_WEB_ARGS},
    error::LdResult,
};
use landscape_database::provider::LandscapeDBServiceProvider;
use serde::{Deserialize, Serialize};
use sys_service::dns_service::get_dns_paths;
use tokio::sync::mpsc;
use tower_http::{services::ServeDir, trace::TraceLayer};

mod auth;
mod config_service;
mod docker;
mod dump;
mod error;
mod iface;
mod metric;
mod service;
mod sys_service;
mod sysinfo;

use service::{
    dhcp_v4::get_dhcp_v4_service_paths, firewall::get_firewall_service_paths,
    flow_wan::get_iface_flow_wan_paths, mss_clamp::get_mss_clamp_service_paths,
};
use service::{icmp_ra::get_iface_icmpv6ra_paths, nat::get_iface_nat_paths};
use service::{ipconfig::get_iface_ipconfig_paths, ipvpd::get_iface_pdclient_paths};
use service::{pppd::get_iface_pppd_paths, wifi::get_wifi_service_paths};
use tracing::{error, info};

const DNS_EVENT_CHANNEL_SIZE: usize = 128;

#[derive(Clone)]
pub struct LandscapeApp {
    pub dns_service: LandscapeDnsService,
    pub dns_rule_service: DNSRuleService,
    pub flow_rule_service: FlowRuleService,
    pub geo_site_service: GeoSiteService,
    pub fire_wall_rule_service: FirewallRuleService,
    pub dst_ip_rule_service: DstIpRuleService,
}

#[derive(Clone, Serialize, Deserialize)]
struct SimpleResult {
    success: bool,
}

#[tokio::main]
async fn main() -> LdResult<()> {
    if let Err(e) = init_logger() {
        panic!("init log error: {e:?}");
    }
    banner();

    let home_path = LAND_HOME_PATH.clone();

    let db_store_provider = LandscapeDBServiceProvider::new(DATABASE_ARGS.clone()).await;

    let need_init_config = boot_check(&home_path)?;
    db_store_provider.truncate_and_fit_from(need_init_config.clone()).await;

    // 初始化 App

    let (dns_service_tx, dns_service_rx) = mpsc::channel(DNS_EVENT_CHANNEL_SIZE);
    let geo_site_service =
        GeoSiteService::new(db_store_provider.clone(), dns_service_tx.clone()).await;
    let dns_rule_service =
        DNSRuleService::new(db_store_provider.clone(), dns_service_tx.clone()).await;
    let flow_rule_service = FlowRuleService::new(db_store_provider.clone()).await;
    let dns_service = LandscapeDnsService::new(
        dns_service_rx,
        dns_rule_service.clone(),
        flow_rule_service.clone(),
        geo_site_service.clone(),
    )
    .await;
    let fire_wall_rule_service = FirewallRuleService::new(db_store_provider.clone()).await;
    let dst_ip_rule_service = DstIpRuleService::new(db_store_provider.clone()).await;

    let landscape_app_status = LandscapeApp {
        dns_service,
        dns_rule_service,
        flow_rule_service,
        geo_site_service,
        fire_wall_rule_service,
        dst_ip_rule_service,
    };
    // 初始化结束

    let dev_obs = landscape::observer::dev_observer().await;

    let home_log_str = format!("{}", home_path.display()).bright_green();
    if !LAND_ARGS.log_output_in_terminal {
        let log_path = format!("{}", LAND_LOG_ARGS.log_path.display()).green();
        println!("Log Folder path: {}", log_path);
        println!("All Config Home path: {home_log_str}");
    }
    info!("config path: {home_log_str}");
    info!("init config: {need_init_config:#?}");

    // need iproute2
    if let Err(e) =
        std::process::Command::new("iptables").args(["-A", "FORWARD", "-j", "ACCEPT"]).output()
    {
        error!("iptables cmd exec err: {e:#?}");
    }

    // need procps
    if let Err(e) =
        std::process::Command::new("sysctl").args(["-w", "net.ipv4.ip_forward=1"]).output()
    {
        error!("sysctl cmd exec err: {e:#?}");
    }

    if let Err(e) =
        std::process::Command::new("sysctl").args(["-w", "net.ipv6.conf.all.forwarding=1"]).output()
    {
        error!("sysctl cmd exec err: {e:#?}");
    }

    if let Err(e) = std::process::Command::new("sysctl")
        .args(["-w", "net.ipv6.conf.default.forwarding=1"])
        .output()
    {
        error!("sysctl cmd exec err: {e:#?}");
    }

    let addr = SocketAddr::from((LAND_WEB_ARGS.address, LAND_WEB_ARGS.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // let _ = landscape_dns::connection::set_socket_mark(
    //     std::os::fd::AsFd::as_fd(&listener).as_raw_fd(),
    //     landscape_common::mark::PacketMark::LandscapeSys.into(),
    // )
    // .unwrap();

    let service = handle_404.into_service();

    let web_root_str = format!("{}", LAND_WEB_ARGS.web_root.display()).green();
    let listen_at_str = format!("{:?}:{:?}", LAND_ARGS.address, LAND_ARGS.port).green();
    if !LAND_ARGS.log_output_in_terminal {
        println!("Web Root path: {}", web_root_str);
        println!("Listen   on: {}", listen_at_str);
    }

    info!("Web Root path: {}", web_root_str);
    info!("Listen   on: {}", listen_at_str);
    let serve_dir = ServeDir::new(&LAND_WEB_ARGS.web_root).not_found_service(service);

    auth::output_sys_token().await;
    let source_route = Router::new()
        .nest("/docker", docker::get_docker_paths(home_path.clone()).await)
        .nest("/iface", iface::get_network_paths(db_store_provider.clone()).await)
        .nest("/metric", metric::get_metric_service_paths().await)
        .nest("/sys_service", get_dns_paths().await.with_state(landscape_app_status.clone()))
        .nest(
            "/config",
            Router::new()
                .merge(get_dns_rule_config_paths().await)
                .merge(get_firewall_rule_config_paths().await)
                .merge(get_flow_rule_config_paths().await)
                .merge(get_geo_site_config_paths().await)
                .merge(get_dst_ip_rule_config_paths().await)
                .with_state(landscape_app_status.clone()),
        )
        .nest(
            "/services",
            Router::new()
                .merge(
                    get_mss_clamp_service_paths(db_store_provider.clone(), dev_obs.resubscribe())
                        .await,
                )
                .merge(
                    get_firewall_service_paths(db_store_provider.clone(), dev_obs.resubscribe())
                        .await,
                )
                .merge(
                    get_iface_ipconfig_paths(db_store_provider.clone(), dev_obs.resubscribe())
                        .await,
                )
                .merge(
                    get_dhcp_v4_service_paths(db_store_provider.clone(), dev_obs.resubscribe())
                        .await,
                )
                .merge(get_wifi_service_paths(db_store_provider.clone()).await)
                .merge(get_iface_pppd_paths(db_store_provider.clone()).await)
                .merge(
                    get_iface_pdclient_paths(db_store_provider.clone(), dev_obs.resubscribe())
                        .await,
                )
                .merge(
                    get_iface_icmpv6ra_paths(db_store_provider.clone(), dev_obs.resubscribe())
                        .await,
                )
                .merge(get_iface_nat_paths(db_store_provider.clone(), dev_obs.resubscribe()).await)
                .merge(get_iface_flow_wan_paths(db_store_provider.clone(), dev_obs).await),
        )
        .nest("/sysinfo", sysinfo::get_sys_info_route())
        .route_layer(axum::middleware::from_fn(auth::auth_middleware));

    let auth_route = Router::new().route("/login", post(auth::login_handler));
    let api_route = Router::new()
        // 资源路由
        .nest("/src", source_route)
        // 认证路由
        .nest("/auth", auth_route);
    let app = Router::new()
        .nest("/api", api_route)
        .nest("/sock", dump::get_tump_router())
        .route("/foo", get(|| async { "Hi from /foo" }))
        .fallback_service(serve_dir);

    axum::serve(listener, app.layer(TraceLayer::new_for_http())).await.unwrap();
    Ok(())
}

/// NOT Found
async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

fn banner() {
    let banner = r#"
██╗      █████╗ ███╗   ██╗██████╗ ███████╗ ██████╗ █████╗ ██████╗ ███████╗
██║     ██╔══██╗████╗  ██║██╔══██╗██╔════╝██╔════╝██╔══██╗██╔══██╗██╔════╝
██║     ███████║██╔██╗ ██║██║  ██║███████╗██║     ███████║██████╔╝█████╗  
██║     ██╔══██║██║╚██╗██║██║  ██║╚════██║██║     ██╔══██║██╔═══╝ ██╔══╝  
███████╗██║  ██║██║ ╚████║██████╔╝███████║╚██████╗██║  ██║██║     ███████╗
╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝ ╚═════╝╚═╝  ╚═╝╚═╝     ╚══════╝
                                                                          
██████╗  ██████╗ ██╗   ██╗████████╗███████╗██████╗                        
██╔══██╗██╔═══██╗██║   ██║╚══██╔══╝██╔════╝██╔══██╗                       
██████╔╝██║   ██║██║   ██║   ██║   █████╗  ██████╔╝                       
██╔══██╗██║   ██║██║   ██║   ██║   ██╔══╝  ██╔══██╗                       
██║  ██║╚██████╔╝╚██████╔╝   ██║   ███████╗██║  ██║                       
╚═╝  ╚═╝ ╚═════╝  ╚═════╝    ╚═╝   ╚══════╝╚═╝  ╚═╝                       
    "#;
    let args = LAND_ARGS.clone();
    let banner = banner.bright_blue().bold();
    let args_str = format!("{args:#?}").green();
    info!("{}", banner);
    info!("Using Args: {}", args_str);
    if !args.log_output_in_terminal {
        // 当日志不在 terminal 直接展示时, 仅输出一些信息
        println!("{}", banner);
        println!("Using Args: {}", args_str);
    }
}
