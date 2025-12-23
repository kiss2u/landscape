use tokio::sync::oneshot;

// cargo run --package landscape-ebpf --bin route_lan
#[tokio::main]
pub async fn main() {
    landscape_common::init_tracing!();

    let ifindex = 4;
    let (tx, rx) = oneshot::channel::<()>();
    let (other_tx, other_rx) = oneshot::channel::<()>();

    std::thread::spawn(move || {
        println!("attach_match_flow attach at ifindex: {:?}", ifindex);
        landscape_ebpf::route::lan_v2::route_lan(ifindex, true, rx).unwrap();
        let _ = other_tx.send(());
    });

    let _ = tokio::signal::ctrl_c().await;

    let _ = tx.send(());
    let _ = other_rx.await;
}
