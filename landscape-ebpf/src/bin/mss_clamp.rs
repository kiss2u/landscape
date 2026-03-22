// cargo run --package landscape-ebpf --bin mss_clamp
// cargo build --package landscape-ebpf --bin mss_clamp --target aarch64-unknown-linux-gnu
#[tokio::main]
pub async fn main() {
    landscape_common::init_tracing!();
    landscape_ebpf::setting_libbpf_log();

    let ifindex = 2;
    println!("Starting run_mss_clamp on ifindex: {:?}", ifindex);
    let mss_clamp = landscape_ebpf::mss_clamp::run_mss_clamp(ifindex, 1492, true).unwrap();

    let _ = tokio::signal::ctrl_c().await;

    drop(mss_clamp);
}
