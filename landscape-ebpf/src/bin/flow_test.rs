use std::{
    net::{IpAddr, Ipv4Addr},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use landscape_common::route::{
    // LanRouteInfo,
    RouteTargetInfo,
};
use tokio::{sync::oneshot, time::sleep};

// cargo run --package landscape-ebpf --bin flow_test
#[tokio::main]
pub async fn main() {
    landscape_common::init_tracing!();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    let ifindex = 4;
    let (tx, rx) = oneshot::channel::<()>();
    let (other_tx, other_rx) = oneshot::channel::<()>();

    // landscape_ebpf::map_setting::route::add_lan_route(LanRouteInfo {
    //     ifindex: 9,
    //     iface_name: "".to_string(),
    //     iface_ip: IpAddr::V4(Ipv4Addr::new(10, 200, 1, 1)),
    //     prefix: 30,
    //     iface_mac: None,
    // });
    std::thread::spawn(move || {
        println!("启动 attach_match_flow 在 ifindex: {:?}", ifindex);
        landscape_ebpf::flow::lan::attach_match_flow(ifindex, true, rx).unwrap();
        println!("向外部线程发送解除阻塞信号");
        let _ = other_tx.send(());
    });

    let route_index = 15;
    let has_mac = false;
    landscape_ebpf::map_setting::route::add_wan_route(
        0,
        RouteTargetInfo {
            ifindex: route_index,
            iface_name: "".to_string(),
            iface_ip: IpAddr::V4(Ipv4Addr::new(10, 200, 1, 1)),
            weight: 0,
            has_mac,
            is_docker: false,
            default_route: false,
            gateway_ip: IpAddr::V4(Ipv4Addr::new(172, 17, 0, 2)),
        },
    );

    let (tx2, rx2) = oneshot::channel::<()>();
    std::thread::spawn(move || {
        println!("启动 wan_route_attach 在 ifindex: {:?}", route_index);
        landscape_ebpf::route::wan::wan_route_attach(route_index, has_mac, rx2).unwrap();
    });

    while running.load(Ordering::SeqCst) {
        sleep(Duration::new(1, 0)).await;
    }

    let _ = tx.send(());
    let _ = tx2.send(());
    let _ = other_rx.await;
}
