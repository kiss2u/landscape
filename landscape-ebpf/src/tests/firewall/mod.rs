use std::mem::MaybeUninit;

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder as _},
    ProgramInput,
};

use crate::firewall::firewall_bpf::FirewallSkelBuilder;

mod package;

pub fn test_ingress_and_egress(mut egress_payload: Vec<u8>, mut ingress_payload: Vec<u8>) {
    let mut firewall_open_object = MaybeUninit::zeroed();
    let firewall_builder = FirewallSkelBuilder::default();

    let firewall_open_skel = firewall_builder.open(&mut firewall_open_object).unwrap();

    let repeat = 10_000;

    let skel = firewall_open_skel.load().unwrap();

    let egress_firewall = skel.progs.egress_firewall;
    let ingress_firewall = skel.progs.ingress_firewall;

    let egress_input = ProgramInput {
        data_in: Some(&mut egress_payload),
        context_in: None,
        context_out: None,
        data_out: None,
        repeat,
        ..Default::default()
    };
    let result = egress_firewall.test_run(egress_input).expect("test_run failed");

    assert_eq!(result.return_value as i32, -1);
    println!("time: {}", result.duration.as_nanos());

    let ingress_input = ProgramInput {
        data_in: Some(&mut ingress_payload),
        context_in: None,
        context_out: None,
        data_out: None,
        repeat,
        ..Default::default()
    };
    let result = ingress_firewall.test_run(ingress_input).expect("test_run failed");

    assert_eq!(result.return_value as i32, -1);
    println!("time: {}", result.duration.as_nanos());
}

#[cfg(test)]
pub mod tests {
    use crate::tests::firewall::{
        package::{icmpv6_ping_egress, icmpv6_ping_ingress},
        test_ingress_and_egress,
    };

    #[test]
    fn test() {
        test_ingress_and_egress(icmpv6_ping_egress(), icmpv6_ping_ingress());
    }
}
