use landscape_common::LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT;

#[tokio::main]
pub async fn main() {
    landscape_ebpf::map_setting::add_expose_port(LANDSCAPE_DEFAULE_DHCP_V6_CLIENT_PORT);
}
