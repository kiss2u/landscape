use std::{mem::MaybeUninit, time::Duration};

use landscape_common::event::firewall::{FirewallEvent, FirewallMetric};
use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    TC_EGRESS, TC_INGRESS,
};

mod firewall_bpf {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/firewall.skel.rs"));
}

use firewall_bpf::{types::firewall_conn_event, types::firewall_conn_metric_event, *};
use tokio::sync::oneshot::{self, error::TryRecvError};

use crate::{
    bpf_error::LdEbpfResult, landscape::TcHookProxy, FIREWALL_EGRESS_PRIORITY,
    FIREWALL_INGRESS_PRIORITY, MAP_PATHS,
};

mod metric;

pub fn new_firewall(
    ifindex: i32,
    has_mac: bool,
    mut service_status: oneshot::Receiver<()>,
) -> LdEbpfResult<()> {
    let mut open_object = MaybeUninit::zeroed();
    let firewall_builder = FirewallSkelBuilder::default();
    let mut open_skel = firewall_builder.open(&mut open_object)?;
    if !has_mac {
        open_skel.maps.rodata_data.current_eth_net_offset = 0;
    }

    open_skel.maps.firewall_block_ip4_map.set_pin_path(&MAP_PATHS.firewall_ipv4_block)?;
    open_skel.maps.firewall_block_ip6_map.set_pin_path(&MAP_PATHS.firewall_ipv6_block)?;
    open_skel.maps.firewall_allow_rules_map.set_pin_path(&MAP_PATHS.firewall_allow_rules_map)?;

    open_skel.maps.firewall_block_ip4_map.reuse_pinned_map(&MAP_PATHS.firewall_ipv4_block)?;
    open_skel.maps.firewall_block_ip6_map.reuse_pinned_map(&MAP_PATHS.firewall_ipv6_block)?;
    open_skel
        .maps
        .firewall_allow_rules_map
        .reuse_pinned_map(&MAP_PATHS.firewall_allow_rules_map)?;

    let skel = open_skel.load()?;

    let callback = |data: &[u8]| -> i32 {
        let time = landscape_common::utils::time::get_boot_time_ns().unwrap_or_default();
        let firewall_conn_event_value = plain::from_bytes::<firewall_conn_event>(data);
        if let Ok(data) = firewall_conn_event_value {
            let event = FirewallEvent::from(data);
            println!("event, {:#?}, time: {time}", event);
        }
        // let _ = firewall_conn_events_tx.send(Box::new(data.to_vec()));
        0
    };

    let metric_callback = |data: &[u8]| -> i32 {
        let time = landscape_common::utils::time::get_boot_time_ns().unwrap_or_default();
        let firewall_conn_event_value = plain::from_bytes::<firewall_conn_metric_event>(data);
        if let Ok(data) = firewall_conn_event_value {
            let event = FirewallMetric::from(data);
            println!("FirewallMetric, {:#?}, time: {time}", event);
        }
        // let _ = firewall_conn_events_tx.send(Box::new(data.to_vec()));
        0
    };
    let mut builder = libbpf_rs::RingBufferBuilder::new();
    builder
        .add(&skel.maps.firewall_conn_events, callback)
        .expect("failed to add firewall_conn_events ringbuf")
        .add(&skel.maps.firewall_conn_metric_events, metric_callback)
        .expect("failed to add firewall_conn_metric_events ringbuf");
    let mgr = builder.build().expect("failed to build");

    let egress_firewall = skel.progs.egress_firewall;
    let ingress_firewall = skel.progs.ingress_firewall;

    let mut egress_firewall_hook =
        TcHookProxy::new(&egress_firewall, ifindex, TC_EGRESS, FIREWALL_EGRESS_PRIORITY);
    let mut ingress_firewall_hook =
        TcHookProxy::new(&ingress_firewall, ifindex, TC_INGRESS, FIREWALL_INGRESS_PRIORITY);

    egress_firewall_hook.attach();
    ingress_firewall_hook.attach();
    'wait_stop: loop {
        let _ = mgr.poll(Duration::from_millis(1000));
        match service_status.try_recv() {
            Ok(_) => break 'wait_stop,
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Closed) => break 'wait_stop,
        }
    }
    // let _ = service_status.blocking_recv();
    drop(egress_firewall_hook);
    drop(ingress_firewall_hook);

    Ok(())
}
