use landscape_common::flow::target::{FlowTargetPair, TargetInterfaceInfo};
use libbpf_rs::{MapCore, MapFlags};

use crate::MAP_PATHS;

use super::share_map::types::flow_target_info;

impl From<TargetInterfaceInfo> for flow_target_info {
    fn from(info: TargetInterfaceInfo) -> Self {
        flow_target_info {
            ifindex: info.ifindex,
            has_mac: std::mem::MaybeUninit::new(info.has_mac),
            is_docker: std::mem::MaybeUninit::new(info.is_docker),
        }
    }
}

pub fn add_flow_target_info(info: FlowTargetPair) {
    let flow_target_map =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.flow_target_map).unwrap();
    let key = unsafe { plain::as_bytes(&info.key) };
    let value: flow_target_info = info.value.into();
    let value = unsafe { plain::as_bytes(&value) };
    if let Err(e) = flow_target_map.update(&key, &value, MapFlags::ANY) {
        tracing::error!("add block ip error:{e:?}");
    }
}
