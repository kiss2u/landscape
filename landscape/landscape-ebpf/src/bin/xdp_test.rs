use landscape_ebpf::landscape;

#[tokio::main]
async fn main() {
    landscape::xdp_test().await
}
