use landscape_common::service::ServiceStatus;
use landscape_common::service::WatchService;
use pnet::datalink::NetworkInterface;
use std::env;
use std::time::Duration;

fn env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn parse_u16_env(key: &str, default: u16) -> Result<u16, String> {
    match env::var(key) {
        Ok(value) => value.parse::<u16>().map_err(|_| format!("invalid {key}: {value}")),
        Err(_) => Ok(default),
    }
}

fn parse_u64_env(key: &str, default: u64) -> Result<u64, String> {
    match env::var(key) {
        Ok(value) => value.parse::<u64>().map_err(|_| format!("invalid {key}: {value}")),
        Err(_) => Ok(default),
    }
}

fn resolve_interface() -> Result<NetworkInterface, String> {
    let iface_name = env::var("PPPOE_TEST_IFACE_NAME").ok();
    let ifindex = env::var("PPPOE_TEST_IFINDEX").ok();
    let all_interfaces = pnet::datalink::interfaces();

    if let Some(iface_name) = iface_name {
        return all_interfaces
            .into_iter()
            .find(|iface| iface.name == iface_name)
            .ok_or_else(|| format!("interface not found: {iface_name}"));
    }

    if let Some(ifindex) = ifindex {
        let ifindex =
            ifindex.parse::<u32>().map_err(|_| format!("invalid PPPOE_TEST_IFINDEX: {ifindex}"))?;
        return all_interfaces
            .into_iter()
            .find(|iface| iface.index == ifindex)
            .ok_or_else(|| format!("interface not found by ifindex: {ifindex}"));
    }

    Err("missing PPPOE_TEST_IFACE_NAME or PPPOE_TEST_IFINDEX".to_string())
}

#[tokio::main]
async fn main() {
    let username = env_or_default("PPPOE_TEST_USERNAME", "pppoe-user");
    let password = env_or_default("PPPOE_TEST_PASSWORD", "pppoe-pass");
    let mtu = match parse_u16_env("PPPOE_TEST_MTU", 1492) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };
    let timeout_secs = match parse_u64_env("PPPOE_TEST_TIMEOUT_SECS", 30) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };

    let interface = match resolve_interface() {
        Ok(interface) => interface,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };
    let Some(interface_mac) = interface.mac else {
        eprintln!("interface {} has no MAC address", interface.name);
        std::process::exit(2);
    };

    let service_status = WatchService::new();
    let iface_name = interface.name.clone();
    let iface_index = interface.index;
    let iface_mac = interface_mac.octets().into();
    let service_status_for_task = service_status.clone();

    println!(
        "开始测试 PPPoE，iface={} ifindex={} mtu={} timeout={}s",
        iface_name, iface_index, mtu, timeout_secs
    );

    tokio::spawn(async move {
        landscape::pppoe_client::create_pppoe_client(
            landscape::pppoe_client::PPPoEClientConfig::new(
                iface_index,
                iface_name,
                iface_mac,
                username,
                password,
                true,
                mtu,
            ),
            service_status_for_task,
            None,
        )
        .await;
    });

    let mut status_rx = service_status.subscribe();
    let timeout = tokio::time::sleep(Duration::from_secs(timeout_secs));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!("收到 Ctrl-C，测试终止");
                std::process::exit(130);
            }
            _ = &mut timeout => {
                eprintln!("等待 PPPoE 连接超时");
                std::process::exit(124);
            }
            change_result = status_rx.changed() => {
                if change_result.is_err() {
                    eprintln!("状态通道关闭");
                    std::process::exit(1);
                }

                let current_status = status_rx.borrow().clone();
                println!("PPPoE 状态变更: {current_status:?}");
                match current_status {
                    ServiceStatus::Running => std::process::exit(0),
                    ServiceStatus::Failed => std::process::exit(1),
                    ServiceStatus::Staring | ServiceStatus::Stopping | ServiceStatus::Stop => {}
                }
            }
        }
    }
}
