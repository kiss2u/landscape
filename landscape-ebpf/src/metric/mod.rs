use std::time::Duration;

use landscape_common::event::firewall::{FirewallEvent, FirewallMessage, FirewallMetric};
use landscape_common::{event::nat::NatEvent, metric::MetricData};
use tokio::sync::oneshot::{self, error::TryRecvError};

use crate::firewall::firewall_bpf::types::{firewall_conn_event, firewall_conn_metric_event};
use crate::{nat::land_nat::types::nat_conn_event, MAP_PATHS};

pub fn new_metric(mut service_status: oneshot::Receiver<()>, metric_service: MetricData) {
    let nat_conn_map = libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.nat_conn_events).unwrap();
    let firewall_conn_map =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.firewall_conn_events).unwrap();
    let firewall_conn_metric_events =
        libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.firewall_conn_metric_events).unwrap();

    let offset_time = landscape_common::utils::time::get_relative_time_ns().unwrap_or_default();

    let nat_conn_callback = |data: &[u8]| -> i32 {
        let nat_conn_event_value = plain::from_bytes::<nat_conn_event>(data);
        if let Ok(data) = nat_conn_event_value {
            let _event = NatEvent::from(data);
            // println!("event, {:#?}, time: {time}, diff: {}", event, time - data.time);
        }
        // let _ = nat_conn_events_tx.send(Box::new(data.to_vec()));
        0
    };

    let firewall = metric_service.firewall.clone();
    let firewall_callback = |data: &[u8]| -> i32 {
        // let time = landscape_common::utils::time::get_boot_time_ns().unwrap_or_default();
        let firewall_conn_event_value = plain::from_bytes::<firewall_conn_event>(data);
        if let Ok(data) = firewall_conn_event_value {
            let mut event = FirewallEvent::from(data);
            event.create_time += offset_time;
            // println!("event, {:#?}, time: {time}", event);
            firewall.send_firewall_msg(FirewallMessage::Event(event));
        }
        0
    };

    let firewall = metric_service.firewall.clone();
    let firewall_metric_callback = |data: &[u8]| -> i32 {
        // let time = landscape_common::utils::time::get_boot_time_ns().unwrap_or_default();
        let firewall_conn_event_value = plain::from_bytes::<firewall_conn_metric_event>(data);
        if let Ok(data) = firewall_conn_event_value {
            let mut event = FirewallMetric::from(data);
            event.create_time += offset_time;
            event.time += offset_time;
            // println!("FirewallMetric, {:#?}, time: {time}", event);
            firewall.send_firewall_msg(FirewallMessage::Metric(event));
        }
        0
    };

    let mut builder = libbpf_rs::RingBufferBuilder::new();
    builder
        .add(&nat_conn_map, nat_conn_callback)
        .expect("failed to add nat_conn_events ringbuf")
        .add(&firewall_conn_map, firewall_callback)
        .expect("failed to add firewall_conn_events ringbuf")
        .add(&firewall_conn_metric_events, firewall_metric_callback)
        .expect("failed to add firewall_conn_metric_events ringbuf");
    let mgr = builder.build().expect("failed to build");

    'wait_stop: loop {
        let _ = mgr.poll(Duration::from_millis(1000));
        match service_status.try_recv() {
            Ok(_) => break 'wait_stop,
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Closed) => break 'wait_stop,
        }
    }
}
