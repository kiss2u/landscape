use std::mem::MaybeUninit;

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder as _},
    ProgramInput,
};

use crate::nat::{land_nat::LandNatSkelBuilder, v2::land_nat_v2::LandNatV2SkelBuilder};

mod ipv6_egress;
mod ipv6_ingress;
mod package;

pub fn test_nat(mut payload: Vec<u8>) {
    let landscape_builder = LandNatSkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let landscape_open = landscape_builder.open(&mut open_object).unwrap();

    let landscape_skel = landscape_open.load().unwrap();

    let test_nat_read = landscape_skel.progs.test_nat_read;
    let repeat = 1000;

    let input = ProgramInput {
        data_in: Some(&mut payload),
        context_in: None,
        context_out: None,
        data_out: None,
        repeat,
        ..Default::default()
    };
    let result = test_nat_read.test_run(input).expect("test_run failed");

    println!("return_value = {}", result.return_value as i32);
    println!("duration = {:?}", result.duration);
}

pub fn test_nat_v2(mut payload: Vec<u8>) {
    let landscape_builder = LandNatV2SkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let landscape_open = landscape_builder.open(&mut open_object).unwrap();

    let landscape_skel = landscape_open.load().unwrap();

    let test_nat_read = landscape_skel.progs.test_nat_read;
    let repeat = 1000;

    let input = ProgramInput {
        data_in: Some(&mut payload),
        context_in: None,
        context_out: None,
        data_out: None,
        repeat,
        ..Default::default()
    };
    let result = test_nat_read.test_run(input).expect("test_run failed");

    println!("return_value = {}", result.return_value as i32);
    println!("duration = {:?}", result.duration);
}

#[cfg(test)]
pub mod tests {
    use crate::tests::nat::{test_nat, test_nat_v2};

    // cargo test --package landscape-ebpf --lib -- tests::test_nat::tests::test --exact --show-output
    #[test]
    fn test() {
        test_nat(crate::tests::dummpy_tcp_pkg());
        test_nat_v2(crate::tests::dummpy_tcp_pkg());
    }
}
