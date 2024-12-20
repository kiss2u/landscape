pub mod landscape;
pub mod map_setting;
pub mod nat;
pub mod ns_proxy;
pub mod packet_mark;
pub mod pppoe;
pub mod tproxy;

pub const WAN_IP_MAP_PING_PATH: &'static str = "/sys/fs/bpf/wan_ipv4_binding";
pub const BLOCK_IP_MAP_PING_PATH: &'static str = "/sys/fs/bpf/firewall_block_map";
pub const PACKET_MARK_MAP_PING_PATH: &'static str = "/sys/fs/bpf/packet_mark_map";
pub const REDIRECT_INDEX_MAP_PING_PATH: &'static str = "/sys/fs/bpf/redirect_index_map";

// pppoe -> Fire wall -> nat
const PPPOE_INGRESS_PRIORITY: u32 = 1;
const FIREWALL_INGRESS_PRIORITY: u32 = 2;
const NAT_INGRESS_PRIORITY: u32 = 3;

// Fire wall -> nat -> pppoe
const PPPOE_MTU_FILTER_EGRESS_PRIORITY: u32 = 1;
const FIREWALL_EGRESS_PRIORITY: u32 = 2;
const NAT_EGRESS_PRIORITY: u32 = 3;
const PPPOE_EGRESS_PRIORITY: u32 = 4;
pub fn init_ebpf() {
    std::thread::spawn(|| {
        landscape::test();
    });
}
