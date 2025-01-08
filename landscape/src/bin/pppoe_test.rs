use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use clap::Parser;
use landscape::service::ServiceStatus;
use tokio::sync::oneshot;

#[derive(Parser, Clone, Debug)]
pub struct CmdArgs {
    #[clap(long, short, default_value_t = 5)]
    ifindex: u32,
    #[clap(long, short, default_value = "user")]
    username: String,
    #[clap(long, short, default_value = "pass")]
    pass: String,
}

// tcpdump -vv -i ens6 ether proto 0x8863 or ether proto 0x8864
// cargo run --package landscape --bin pppoe_test
// cargo build --package landscape --bin pppoe_test --target aarch64-unknown-linux-gnu
#[tokio::main]
async fn main() {
    let params = CmdArgs::parse();
    let (service_status, _) = tokio::sync::watch::channel(ServiceStatus::Staring);
    let all_interfaces = pnet::datalink::interfaces();
    let target_interface = all_interfaces.iter().find(|e| e.index == params.ifindex);
    let Some(interface) = target_interface else {
        return;
    };
    let iface_name = interface.name.clone();
    let iface_mac = interface.mac.unwrap().octets().into();

    let (notice, notice_rx) = oneshot::channel();
    let service_status_clone = service_status.clone();
    tokio::spawn(async move {
        landscape::pppoe_client::pppoe_client_v2::create_pppoe_client(
            params.ifindex,
            iface_name,
            iface_mac,
            params.username,
            params.pass,
            service_status_clone,
        )
        .await;

        println!("内部结束， 发送通知");
        if let Err(e) = notice.send(()) {
            println!("发送错误: {e:?}");
        }
    });

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::new(1, 0)).await;
    }
    println!("开始断连");

    service_status.send_replace(ServiceStatus::Stopping);

    println!("开始等待结束");
    if let Err(e) = notice_rx.await {
        println!("等待过程出错: {e:?}");
    }
    println!("结束退出");
}
