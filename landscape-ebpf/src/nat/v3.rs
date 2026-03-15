use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;

use land_nat_v3::*;
use landscape_common::iface::nat::NatConfig;
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    MapCore, TC_EGRESS, TC_INGRESS,
};
use tokio::sync::oneshot;

use crate::MAP_PATHS;
use crate::{landscape::TcHookProxy, NAT_EGRESS_PRIORITY, NAT_INGRESS_PRIORITY};

pub(crate) mod land_nat_v3 {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/land_nat_v3.skel.rs"));
}

fn seed_port_queue<M>(map: &M, start: u16, end: u16)
where
    M: MapCore,
{
    let fd = map.as_fd().as_raw_fd();
    for port in start..=end {
        let value = types::nat4_port_queue_value_v3 { port: port.to_be(), last_generation: 0 };
        let ret = unsafe {
            libbpf_rs::libbpf_sys::bpf_map_update_elem(
                fd,
                std::ptr::null(),
                (&value as *const types::nat4_port_queue_value_v3).cast_mut().cast(),
                0,
            )
        };
        if ret != 0 {
            break;
        }
    }
}

fn seed_runtime_queues<M1, M2, M3>(
    tcp_queue: &M1,
    udp_queue: &M2,
    icmp_queue: &M3,
    config: &NatConfig,
) where
    M1: MapCore,
    M2: MapCore,
    M3: MapCore,
{
    seed_port_queue(tcp_queue, config.tcp_range.start, config.tcp_range.end);
    seed_port_queue(udp_queue, config.udp_range.start, config.udp_range.end);
    seed_port_queue(icmp_queue, config.icmp_in_range.start, config.icmp_in_range.end);
}

pub fn init_nat(
    ifindex: i32,
    has_mac: bool,
    service_status: oneshot::Receiver<()>,
    config: NatConfig,
) {
    let landscape_builder = LandNatV3SkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let mut landscape_open = landscape_builder.open(&mut open_object).unwrap();

    landscape_open.maps.wan_ip_binding.set_pin_path(&MAP_PATHS.wan_ip).unwrap();
    landscape_open.maps.wan_ip_binding.reuse_pinned_map(&MAP_PATHS.wan_ip).unwrap();

    landscape_open.maps.nat6_static_mappings.set_pin_path(&MAP_PATHS.nat6_static_mappings).unwrap();
    landscape_open
        .maps
        .nat6_static_mappings
        .reuse_pinned_map(&MAP_PATHS.nat6_static_mappings)
        .unwrap();

    landscape_open.maps.nat4_st_map.set_pin_path(&MAP_PATHS.nat4_st_map).unwrap();
    landscape_open.maps.nat4_st_map.reuse_pinned_map(&MAP_PATHS.nat4_st_map).unwrap();

    landscape_open.maps.nat4_mappings.set_pin_path(&MAP_PATHS.nat4_mappings).unwrap();
    landscape_open.maps.nat4_mappings.reuse_pinned_map(&MAP_PATHS.nat4_mappings).unwrap();

    landscape_open.maps.nat4_mapping_timer.set_pin_path(&MAP_PATHS.nat4_mapping_timer).unwrap();
    landscape_open.maps.nat4_mapping_timer.reuse_pinned_map(&MAP_PATHS.nat4_mapping_timer).unwrap();

    landscape_open
        .maps
        .nat_conn_metric_events
        .set_pin_path(&MAP_PATHS.nat_conn_metric_events)
        .unwrap();
    landscape_open
        .maps
        .nat_conn_metric_events
        .reuse_pinned_map(&MAP_PATHS.nat_conn_metric_events)
        .unwrap();

    let rodata_data =
        landscape_open.maps.rodata_data.as_deref_mut().expect("`rodata` is not memery mapped");

    rodata_data.tcp_range_start = config.tcp_range.start;
    rodata_data.tcp_range_end = config.tcp_range.end;
    rodata_data.udp_range_start = config.udp_range.start;
    rodata_data.udp_range_end = config.udp_range.end;
    rodata_data.icmp_range_start = config.icmp_in_range.start;
    rodata_data.icmp_range_end = config.icmp_in_range.end;

    if !has_mac {
        rodata_data.current_l3_offset = 0;
    }

    let landscape_skel = landscape_open.load().unwrap();

    seed_runtime_queues(
        &landscape_skel.maps.nat4_tcp_free_ports_v3,
        &landscape_skel.maps.nat4_udp_free_ports_v3,
        &landscape_skel.maps.nat4_icmp_free_ports_v3,
        &config,
    );

    let nat_egress = landscape_skel.progs.egress_nat;
    let nat_ingress = landscape_skel.progs.ingress_nat;

    let mut nat_egress_hook =
        TcHookProxy::new(&nat_egress, ifindex, TC_EGRESS, NAT_EGRESS_PRIORITY);
    let mut nat_ingress_hook =
        TcHookProxy::new(&nat_ingress, ifindex, TC_INGRESS, NAT_INGRESS_PRIORITY);

    nat_egress_hook.attach();
    nat_ingress_hook.attach();
    let _ = service_status.blocking_recv();
    drop(nat_egress_hook);
    drop(nat_ingress_hook);
}
