use std::time::Duration;

use landscape::{dhcp_client::dhcp_client, macaddr::MacAddr, service::ServiceStatus};
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    let mac_addr = MacAddr::new(00, 160, 152, 30, 211, 86);

    // let mac_addr = MacAddr::new(0xbe, 0x25, 0x85, 0x83, 0x00, 0x0d);

    let (status, _) = watch::channel(ServiceStatus::Staring);

    let service_status = status.clone();
    tokio::spawn(async move {
        dhcp_client(5, "ens4".into(), mac_addr, 68, service_status, "TEST-PC".to_string()).await;
    });

    tokio::time::sleep(Duration::from_secs(30)).await;

    let timeout_timer = tokio::time::sleep(tokio::time::Duration::from_secs(300));
    timeout_timer.await;
    status.send_replace(ServiceStatus::Stopping);

    let _ = status.subscribe().wait_for(|s| matches!(s, ServiceStatus::Stop { .. })).await;
}
