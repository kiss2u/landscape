use landscape_common::flow::target::{FlowTargetPair, TargetInterfaceInfo};

// cargo run --package landscape-ebpf --bin add_flow_target
pub fn main() {
    landscape_common::init_tracing!();
    landscape_ebpf::setting_libbpf_log();

    landscape_ebpf::map_setting::flow_target::add_flow_target_info(FlowTargetPair {
        key: 9,
        value: TargetInterfaceInfo::new_docker(34),
    });
}
