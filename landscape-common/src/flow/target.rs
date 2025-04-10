pub struct TargetInterfaceInfo {
    pub ifindex: u32,
    pub has_mac: bool,
    pub is_docker: bool,
}

impl TargetInterfaceInfo {
    pub fn new_net_iface(ifindex: u32, has_mac: bool) -> Self {
        TargetInterfaceInfo { ifindex, has_mac, is_docker: false }
    }

    pub fn new_docker(ifindex: u32) -> Self {
        TargetInterfaceInfo { ifindex, has_mac: true, is_docker: true }
    }
}

pub struct FlowTargetPair {
    pub key: u32,
    pub value: TargetInterfaceInfo,
}
