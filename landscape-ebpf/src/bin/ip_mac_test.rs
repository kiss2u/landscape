use tokio::sync::oneshot;

// cargo run --package landscape-ebpf --bin ip_mac_test
#[tokio::main]
pub async fn main() {
    landscape_common::init_tracing!();

    let (tx, rx) = oneshot::channel::<()>();
    let (other_tx, other_rx) = oneshot::channel::<()>();

    std::thread::spawn(move || {
        landscape_ebpf::base::ip_mac::neigh_update(rx).unwrap();
        let _ = other_tx.send(());
    });

    let _ = tokio::signal::ctrl_c().await;

    let _ = tx.send(());
    let _ = other_rx.await;
}
