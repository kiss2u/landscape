// cargo run --package landscape-ebpf --bin route_lan
#[tokio::main]
pub async fn main() {
    landscape_common::init_tracing!();

    let ifindex = 4;
    println!("attach_match_flow attach at ifindex: {:?}", ifindex);
    let route_lan = landscape_ebpf::route::lan_v2::route_lan(ifindex, true)
        .expect("failed to start route lan test");

    let _ = tokio::signal::ctrl_c().await;

    drop(route_lan);
}
