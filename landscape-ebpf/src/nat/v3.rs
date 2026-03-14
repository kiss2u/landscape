use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;

use land_nat_v3::*;
use landscape_common::iface::nat::NatConfig;
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    MapCore, MapFlags, OpenMapMut, TC_EGRESS, TC_INGRESS,
};
use tokio::sync::oneshot;

use crate::map_setting::{probe_pinned_map_reuse, PinnedMapReuseStatus};
use crate::{LandscapeMapPath, MAP_PATHS};
use crate::{landscape::TcHookProxy, NAT_EGRESS_PRIORITY, NAT_INGRESS_PRIORITY};

pub(crate) mod land_nat_v3 {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/land_nat_v3.skel.rs"));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NatV3RuntimeBundleMode {
    ReuseAll,
    RecreateAll,
}

fn nat_v3_runtime_bundle_mode(
    statuses: [PinnedMapReuseStatus; 6],
) -> NatV3RuntimeBundleMode {
    if statuses.into_iter().all(|status| status == PinnedMapReuseStatus::Compatible) {
        NatV3RuntimeBundleMode::ReuseAll
    } else {
        NatV3RuntimeBundleMode::RecreateAll
    }
}

fn recreate_pinned_map(map: &mut OpenMapMut<'_>, path: &std::path::Path) {
    map.set_pin_path(path).unwrap();
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

fn reuse_pinned_map(map: &mut OpenMapMut<'_>, path: &std::path::Path) {
    map.set_pin_path(path).unwrap();
    map.reuse_pinned_map(path)
        .unwrap_or_else(|e| panic!("failed to reuse NAT v3 pinned map {}: {e}", path.display()));
}

pub(crate) fn configure_runtime_bundle_maps(
    nat4_mappings: &mut OpenMapMut<'_>,
    dynamic_state: &mut OpenMapMut<'_>,
    timer_map: &mut OpenMapMut<'_>,
    tcp_queue: &mut OpenMapMut<'_>,
    udp_queue: &mut OpenMapMut<'_>,
    icmp_queue: &mut OpenMapMut<'_>,
    paths: &LandscapeMapPath,
) -> NatV3RuntimeBundleMode {
    let mode = nat_v3_runtime_bundle_mode([
        probe_pinned_map_reuse(nat4_mappings, &paths.nat4_mappings),
        probe_pinned_map_reuse(dynamic_state, &paths.nat4_dynamic_state_v3),
        probe_pinned_map_reuse(timer_map, &paths.nat4_mapping_timer_v3),
        probe_pinned_map_reuse(tcp_queue, &paths.nat4_tcp_free_ports_v3),
        probe_pinned_map_reuse(udp_queue, &paths.nat4_udp_free_ports_v3),
        probe_pinned_map_reuse(icmp_queue, &paths.nat4_icmp_free_ports_v3),
    ]);

    match mode {
        NatV3RuntimeBundleMode::ReuseAll => {
            reuse_pinned_map(nat4_mappings, &paths.nat4_mappings);
            reuse_pinned_map(dynamic_state, &paths.nat4_dynamic_state_v3);
            reuse_pinned_map(timer_map, &paths.nat4_mapping_timer_v3);
            reuse_pinned_map(tcp_queue, &paths.nat4_tcp_free_ports_v3);
            reuse_pinned_map(udp_queue, &paths.nat4_udp_free_ports_v3);
            reuse_pinned_map(icmp_queue, &paths.nat4_icmp_free_ports_v3);
        }
        NatV3RuntimeBundleMode::RecreateAll => {
            recreate_pinned_map(nat4_mappings, &paths.nat4_mappings);
            recreate_pinned_map(dynamic_state, &paths.nat4_dynamic_state_v3);
            recreate_pinned_map(timer_map, &paths.nat4_mapping_timer_v3);
            recreate_pinned_map(tcp_queue, &paths.nat4_tcp_free_ports_v3);
            recreate_pinned_map(udp_queue, &paths.nat4_udp_free_ports_v3);
            recreate_pinned_map(icmp_queue, &paths.nat4_icmp_free_ports_v3);
        }
    }

    mode
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

fn seed_runtime_queues<M1, M2, M3>(tcp_queue: &M1, udp_queue: &M2, icmp_queue: &M3, config: &NatConfig)
where
    M1: MapCore,
    M2: MapCore,
    M3: MapCore,
{
    seed_port_queue(tcp_queue, config.tcp_range.start, config.tcp_range.end);
    seed_port_queue(udp_queue, config.udp_range.start, config.udp_range.end);
    seed_port_queue(icmp_queue, config.icmp_in_range.start, config.icmp_in_range.end);
}

#[cfg_attr(not(test), allow(dead_code))]
fn clear_all_map_entries<M>(map: &M)
where
    M: MapCore,
{
    let keys: Vec<Vec<u8>> = map.keys().collect();
    for key in keys {
        if let Err(e) = map.delete(&key) {
            tracing::warn!("failed to delete map entry during NAT v3 reset: {e}");
        }
    }
}

fn clear_port_queue<M>(map: &M)
where
    M: MapCore,
{
    let key: [u8; 0] = [];
    loop {
        match map.lookup_and_delete(&key) {
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(e) => {
                tracing::warn!("failed to clear NAT v3 queue: {e}");
                break;
            }
        }
    }
}

fn map_has_entries<M>(map: &M) -> bool
where
    M: MapCore,
{
    map.keys().next().is_some()
}

fn queue_has_entries<M>(map: &M) -> bool
where
    M: MapCore,
{
    let key: [u8; 0] = [];
    matches!(map.lookup(&key, MapFlags::ANY), Ok(Some(_)))
}

pub(crate) fn runtime_bundle_needs_queue_seed<M1, M2, M3, M4, M5>(
    dynamic_state: &M1,
    timer_map: &M2,
    tcp_queue: &M3,
    udp_queue: &M4,
    icmp_queue: &M5,
) -> bool
where
    M1: MapCore,
    M2: MapCore,
    M3: MapCore,
    M4: MapCore,
    M5: MapCore,
{
    !map_has_entries(dynamic_state)
        && !map_has_entries(timer_map)
        && !queue_has_entries(tcp_queue)
        && !queue_has_entries(udp_queue)
        && !queue_has_entries(icmp_queue)
}

#[cfg_attr(not(test), allow(dead_code))]
fn clear_dynamic_nat4_mappings<M>(map: &M)
where
    M: MapCore,
{
    let keys: Vec<Vec<u8>> = map.keys().collect();
    for key in keys {
        let value = match map.lookup(&key, MapFlags::ANY) {
            Ok(Some(value)) => value,
            Ok(None) => continue,
            Err(e) => {
                tracing::warn!("failed to inspect NAT v3 mapping during reset: {e}");
                continue;
            }
        };
        let value = unsafe {
            std::ptr::read_unaligned(value.as_ptr().cast::<types::nat_mapping_value_v4>())
        };
        if value.is_static == 0 {
            if let Err(e) = map.delete(&key) {
                tracing::warn!("failed to delete dynamic NAT v3 mapping during reset: {e}");
            }
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn reset_dynamic_nat_v3_runtime<M1, M2, M3, M4, M5, M6>(
    nat4_mappings: &M1,
    dynamic_state: &M2,
    timer_map: &M3,
    tcp_queue: &M4,
    udp_queue: &M5,
    icmp_queue: &M6,
    config: &NatConfig,
) where
    M1: MapCore,
    M2: MapCore,
    M3: MapCore,
    M4: MapCore,
    M5: MapCore,
    M6: MapCore,
{
    clear_all_map_entries(timer_map);
    clear_all_map_entries(dynamic_state);
    clear_dynamic_nat4_mappings(nat4_mappings);

    clear_port_queue(tcp_queue);
    clear_port_queue(udp_queue);
    clear_port_queue(icmp_queue);

    seed_runtime_queues(tcp_queue, udp_queue, icmp_queue, config);
}

pub(crate) fn finalize_runtime_bundle_after_load<M1, M2, M3, M4, M5>(
    mode: NatV3RuntimeBundleMode,
    dynamic_state: &M1,
    timer_map: &M2,
    tcp_queue: &M3,
    udp_queue: &M4,
    icmp_queue: &M5,
    config: &NatConfig,
) where
    M1: MapCore,
    M2: MapCore,
    M3: MapCore,
    M4: MapCore,
    M5: MapCore,
{
    if mode == NatV3RuntimeBundleMode::RecreateAll
        || runtime_bundle_needs_queue_seed(dynamic_state, timer_map, tcp_queue, udp_queue, icmp_queue)
    {
        clear_port_queue(tcp_queue);
        clear_port_queue(udp_queue);
        clear_port_queue(icmp_queue);
        seed_runtime_queues(tcp_queue, udp_queue, icmp_queue, config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_mode_requires_all_maps_to_be_compatible() {
        assert_eq!(
            nat_v3_runtime_bundle_mode([
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Compatible,
            ]),
            NatV3RuntimeBundleMode::ReuseAll
        );

        assert_eq!(
            nat_v3_runtime_bundle_mode([
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Missing,
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Compatible,
            ]),
            NatV3RuntimeBundleMode::RecreateAll
        );

        assert_eq!(
            nat_v3_runtime_bundle_mode([
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Incompatible,
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Compatible,
                PinnedMapReuseStatus::Compatible,
            ]),
            NatV3RuntimeBundleMode::RecreateAll
        );
    }
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

    let runtime_mode = configure_runtime_bundle_maps(
        &mut landscape_open.maps.nat4_mappings,
        &mut landscape_open.maps.nat4_dynamic_state_v3,
        &mut landscape_open.maps.nat4_mapping_timer_v3,
        &mut landscape_open.maps.nat4_tcp_free_ports_v3,
        &mut landscape_open.maps.nat4_udp_free_ports_v3,
        &mut landscape_open.maps.nat4_icmp_free_ports_v3,
        &MAP_PATHS,
    );

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

    finalize_runtime_bundle_after_load(
        runtime_mode,
        &landscape_skel.maps.nat4_dynamic_state_v3,
        &landscape_skel.maps.nat4_mapping_timer_v3,
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
