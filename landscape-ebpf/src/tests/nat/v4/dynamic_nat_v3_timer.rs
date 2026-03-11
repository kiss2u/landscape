use std::mem::MaybeUninit;
use std::net::Ipv4Addr;

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder as _},
    MapCore, MapFlags, ProgramInput,
};

pub(crate) mod test_nat_v3_timer {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/test_nat_v3_timer.skel.rs"));
}

use test_nat_v3_timer::{types, TestNatV3TimerSkelBuilder};

const NAT_MAPPING_INGRESS: u8 = 0;
const NAT_MAPPING_EGRESS: u8 = 1;
const STATE_SHIFT: u64 = 56;
const STATE_ACTIVE: u64 = 1;
const STATE_CLOSED: u64 = 2;
const TIMER_TIMEOUT_2: u64 = 31;
const TIMER_RELEASE: u64 = 40;
const TIMER_RELEASE_PENDING_QUEUE: u64 = 41;
const STEP_DELETE_CT: u32 = 1;
const STEP_RESTART: u32 = 2;

const WAN_IP: Ipv4Addr = Ipv4Addr::new(203, 0, 113, 1);
const LAN_HOST: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 100);
const REMOTE_IP: Ipv4Addr = Ipv4Addr::new(50, 18, 88, 205);
const LAN_PORT: u16 = 56186;
const NAT_PORT: u16 = 40000;
const GENERATION: u16 = 7;

fn as_bytes<T>(value: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts((value as *const T).cast::<u8>(), std::mem::size_of::<T>())
    }
}

fn read_unaligned<T: Copy>(bytes: &[u8]) -> T {
    unsafe { std::ptr::read_unaligned(bytes.as_ptr().cast::<T>()) }
}

fn state_ref(state: u64, refs: u64) -> u64 {
    (state << STATE_SHIFT) | refs
}

fn timer_key() -> types::nat_timer_key_v4 {
    types::nat_timer_key_v4 {
        l4proto: 6,
        _pad: [0; 3],
        pair_ip: types::inet4_pair {
            src_addr: types::inet4_addr { addr: REMOTE_IP.to_bits().to_be() },
            dst_addr: types::inet4_addr { addr: WAN_IP.to_bits().to_be() },
            src_port: 443u16.to_be(),
            dst_port: NAT_PORT.to_be(),
        },
    }
}

fn ingress_key() -> types::nat_mapping_key_v4 {
    types::nat_mapping_key_v4 {
        gress: NAT_MAPPING_INGRESS,
        l4proto: 6,
        from_port: NAT_PORT.to_be(),
        from_addr: WAN_IP.to_bits().to_be(),
    }
}

fn egress_key() -> types::nat_mapping_key_v4 {
    types::nat_mapping_key_v4 {
        gress: NAT_MAPPING_EGRESS,
        l4proto: 6,
        from_port: LAN_PORT.to_be(),
        from_addr: LAN_HOST.to_bits().to_be(),
    }
}

fn mapping_pair() -> (types::nat_mapping_value_v4, types::nat_mapping_value_v4) {
    let egress = types::nat_mapping_value_v4 {
        addr: WAN_IP.to_bits().to_be(),
        trigger_addr: REMOTE_IP.to_bits().to_be(),
        port: NAT_PORT.to_be(),
        trigger_port: 443u16.to_be(),
        is_static: 0,
        is_allow_reuse: 1,
        _pad: [0; 2],
        active_time: 1,
    };
    let ingress = types::nat_mapping_value_v4 {
        addr: LAN_HOST.to_bits().to_be(),
        trigger_addr: REMOTE_IP.to_bits().to_be(),
        port: LAN_PORT.to_be(),
        trigger_port: 443u16.to_be(),
        is_static: 0,
        is_allow_reuse: 1,
        _pad: [0; 2],
        active_time: 1,
    };
    (egress, ingress)
}

fn put_mapping_pair<T: MapCore>(map: &T) {
    let (egress, ingress) = mapping_pair();
    map.update(as_bytes(&egress_key()), as_bytes(&egress), MapFlags::ANY).unwrap();
    map.update(as_bytes(&ingress_key()), as_bytes(&ingress), MapFlags::ANY).unwrap();
}

fn put_state<T: MapCore>(map: &T, generation: u16, state_ref_: u64) {
    let value = types::nat4_mapping_state_v3 {
        state_ref: state_ref_,
        generation,
        _pad0: 0,
        _pad1: 0,
    };
    map.update(as_bytes(&ingress_key()), as_bytes(&value), MapFlags::ANY).unwrap();
}

fn put_timer<T: MapCore>(map: &T, status: u64, generation_snapshot: u16, is_final_releaser: u8) {
    let value = types::nat_timer_value_v4_v3 {
        server_status: 1,
        client_status: 1,
        status,
        timer: types::bpf_timer::default(),
        client_addr: types::inet4_addr { addr: LAN_HOST.to_bits().to_be() },
        client_port: LAN_PORT.to_be(),
        gress: NAT_MAPPING_EGRESS,
        flow_id: 0,
        create_time: 1,
        ingress_bytes: 0,
        ingress_packets: 0,
        egress_bytes: 0,
        egress_packets: 0,
        cpu_id: 0,
        generation_snapshot,
        is_final_releaser,
        _pad0: 0,
    };
    map.update(as_bytes(&timer_key()), as_bytes(&value), MapFlags::ANY).unwrap();
}

fn put_test_input<T: MapCore>(map: &T, force_queue_push_fail: bool) {
    let value = types::nat4_timer_test_input_v3 {
        key: timer_key(),
        force_queue_push_fail: force_queue_push_fail as u8,
        _pad: [0; 3],
    };
    let key = 0u32;
    map.update(as_bytes(&key), as_bytes(&value), MapFlags::ANY).unwrap();
}

fn get_test_result<T: MapCore>(map: &T) -> types::nat4_timer_test_result_v3 {
    let key = 0u32;
    let bytes = map.lookup(as_bytes(&key), MapFlags::ANY).unwrap().expect("missing test result");
    read_unaligned::<types::nat4_timer_test_result_v3>(&bytes)
}

fn run_step(
    skel: &test_nat_v3_timer::TestNatV3TimerSkel<'_>,
    force_queue_push_fail: bool,
) -> types::nat4_timer_test_result_v3 {
    put_test_input(&skel.maps.nat4_timer_test_input_v3, force_queue_push_fail);
    let mut data = vec![0u8; 64];
    let input = ProgramInput { data_in: Some(&mut data), ..Default::default() };
    let result = skel.progs.nat_v4_timer_step_test.test_run(input).expect("test_run failed");
    assert_eq!(result.return_value as i32, 0);
    get_test_result(&skel.maps.nat4_timer_test_result_v3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::nat::NAT_V3_TEST_LOCK;

    #[test]
    fn release_generation_mismatch_deletes_only_ct() {
        let _guard = NAT_V3_TEST_LOCK.lock().unwrap();
        let builder = TestNatV3TimerSkelBuilder::default();
        let mut open_object = MaybeUninit::uninit();
        let open = builder.open(&mut open_object).unwrap();
        let skel = open.load().unwrap();
        put_mapping_pair(&skel.maps.nat4_mappings);
        put_state(&skel.maps.nat4_dynamic_state_v3, GENERATION + 1, state_ref(STATE_ACTIVE, 1));
        put_timer(&skel.maps.nat4_mapping_timer_v3, TIMER_RELEASE, GENERATION, 0);

        let result = run_step(&skel, false);

        assert_eq!(result.action, STEP_DELETE_CT);
        assert_eq!(result.timer_exists, 0);
        assert_eq!(result.ingress_mapping_exists, 1);
        assert_eq!(result.egress_mapping_exists, 1);
        assert_eq!(result.state_exists, 1);
        assert_eq!(result.generation, GENERATION + 1);
        assert_eq!(result.state_ref, state_ref(STATE_ACTIVE, 1));
    }

    #[test]
    fn release_active_two_decrements_ref() {
        let _guard = NAT_V3_TEST_LOCK.lock().unwrap();
        let builder = TestNatV3TimerSkelBuilder::default();
        let mut open_object = MaybeUninit::uninit();
        let open = builder.open(&mut open_object).unwrap();
        let skel = open.load().unwrap();
        put_mapping_pair(&skel.maps.nat4_mappings);
        put_state(&skel.maps.nat4_dynamic_state_v3, GENERATION, state_ref(STATE_ACTIVE, 2));
        put_timer(&skel.maps.nat4_mapping_timer_v3, TIMER_RELEASE, GENERATION, 0);

        let result = run_step(&skel, false);

        assert_eq!(result.action, STEP_DELETE_CT);
        assert_eq!(result.timer_exists, 0);
        assert_eq!(result.ingress_mapping_exists, 1);
        assert_eq!(result.egress_mapping_exists, 1);
        assert_eq!(result.state_exists, 1);
        assert_eq!(result.state_ref, state_ref(STATE_ACTIVE, 1));
    }

    #[test]
    fn timeout2_transitions_to_release_and_closes_last() {
        let _guard = NAT_V3_TEST_LOCK.lock().unwrap();
        let builder = TestNatV3TimerSkelBuilder::default();
        let mut open_object = MaybeUninit::uninit();
        let open = builder.open(&mut open_object).unwrap();
        let skel = open.load().unwrap();
        put_mapping_pair(&skel.maps.nat4_mappings);
        put_state(&skel.maps.nat4_dynamic_state_v3, GENERATION, state_ref(STATE_ACTIVE, 1));
        put_timer(&skel.maps.nat4_mapping_timer_v3, TIMER_TIMEOUT_2, GENERATION, 0);

        let result = run_step(&skel, false);

        assert_eq!(result.action, STEP_RESTART);
        assert_eq!(result.timer_exists, 1);
        assert_eq!(u64::from(result.status), TIMER_RELEASE);
        assert_eq!(result.state_exists, 1);
        assert_eq!(result.state_ref, state_ref(STATE_CLOSED, 1));
    }

    #[test]
    fn release_closed_queue_fail_enters_pending() {
        let _guard = NAT_V3_TEST_LOCK.lock().unwrap();
        let builder = TestNatV3TimerSkelBuilder::default();
        let mut open_object = MaybeUninit::uninit();
        let open = builder.open(&mut open_object).unwrap();
        let skel = open.load().unwrap();
        put_mapping_pair(&skel.maps.nat4_mappings);
        put_state(&skel.maps.nat4_dynamic_state_v3, GENERATION, state_ref(STATE_CLOSED, 1));
        put_timer(&skel.maps.nat4_mapping_timer_v3, TIMER_RELEASE, GENERATION, 1);

        let result = run_step(&skel, true);

        assert_eq!(result.action, STEP_RESTART);
        assert_eq!(result.queue_push_ret, -1);
        assert_eq!(result.timer_exists, 1);
        assert_eq!(u64::from(result.status), TIMER_RELEASE_PENDING_QUEUE);
        assert_eq!(result.ingress_mapping_exists, 0);
        assert_eq!(result.egress_mapping_exists, 0);
        assert_eq!(result.state_exists, 0);
    }

    #[test]
    fn pending_queue_retry_success_deletes_ct() {
        let _guard = NAT_V3_TEST_LOCK.lock().unwrap();
        let builder = TestNatV3TimerSkelBuilder::default();
        let mut open_object = MaybeUninit::uninit();
        let open = builder.open(&mut open_object).unwrap();
        let skel = open.load().unwrap();
        put_timer(&skel.maps.nat4_mapping_timer_v3, TIMER_RELEASE_PENDING_QUEUE, GENERATION, 1);

        let result = run_step(&skel, false);

        assert_eq!(result.action, STEP_DELETE_CT);
        assert_eq!(result.queue_push_ret, 0);
        assert_eq!(result.timer_exists, 0);
    }
}
