use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use landscape::{dhcp_client::v4::dhcp_v4_client, iface::get_iface_by_name, route::IpRouteService};
use landscape_common::service::{DefaultWatchServiceStatus, ServiceStatus};

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[arg(short, long, default_value = "ens4")]
    pub iface_name: String,
}

/// cargo run --package landscape --bin dhcp_client_test
#[tokio::main]
async fn main() {
    landscape_common::init_tracing!();

    let args = Args::parse();
    tracing::info!("using args is: {:#?}", args);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    let service_status = DefaultWatchServiceStatus::new();

    let status = service_status.clone();

    tokio::spawn(async move {
        if let Some(iface) = get_iface_by_name(&args.iface_name).await {
            if let Some(mac) = iface.mac {
                dhcp_v4_client(
                    iface.index,
                    iface.name,
                    mac,
                    68,
                    status,
                    "TEST-PC".to_string(),
                    false,
                    IpRouteService::new(),
                )
                .await;
            }
        }
    });

    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::new(1, 0)).await;
    }

    service_status.just_change_status(ServiceStatus::Stopping);

    service_status.wait_stop().await;
}
