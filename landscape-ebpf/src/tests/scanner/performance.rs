use std::mem::MaybeUninit;

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder as _},
    MapCore, MapFlags, ProgramInput,
};
use zerocopy::FromBytes as _;

use crate::tests::{scanner::PacketOffsetInfo, test_scanner::TestScannerSkelBuilder, TestSkb};

pub fn performance_test(repeat: u32, mut payload: Vec<u8>) {
    let builder = TestScannerSkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let open = builder.open(&mut open_object).unwrap();

    let skel = open.load().unwrap();

    let key = 0 as u32;
    let value = 1 as u64;
    skel.maps
        .test_sync_map
        .update(&key.to_le_bytes(), &value.to_le_bytes(), MapFlags::ANY)
        .unwrap();

    let scanner_baseline = skel.progs.scanner_baseline;
    let direct_read_info = skel.progs.direct_read_info;
    let scanner_without_offset_info = skel.progs.scanner_without_offset_info;
    let scanner_has_offset = skel.progs.scanner_has_offset;

    let mut ctx = TestSkb::default();
    let ctx_size = std::mem::size_of::<TestSkb>();
    let mut ctx_slice =
        unsafe { std::slice::from_raw_parts_mut(&mut ctx as *mut TestSkb as *mut u8, ctx_size) };

    let input = ProgramInput {
        data_in: Some(&mut payload),
        repeat,
        ..Default::default()
    };
    let result = scanner_baseline.test_run(input).expect("test_run failed");

    println!("baseline result = {}", result.return_value as i32);
    println!("baseline duration = {:?}", result.duration);

    let input = ProgramInput {
        data_in: Some(&mut payload),
        repeat,
        ..Default::default()
    };
    let result = direct_read_info.test_run(input).expect("test_run failed");

    println!("direct_read_info result = {}", result.return_value as i32);
    println!("direct_read_info duration = {:?}", result.duration);

    let input = ProgramInput {
        data_in: Some(&mut payload),
        context_out: Some(&mut ctx_slice),
        repeat,
        ..Default::default()
    };
    let result = scanner_without_offset_info.test_run(input).expect("test_run failed");

    println!("scanner_without_offset_info result = {}", result.return_value as i32);
    println!("scanner_without_offset_info duration = {:?}", result.duration);

    let input = ProgramInput {
        data_in: Some(&mut payload),
        context_in: None,
        context_out: Some(ctx_slice),
        repeat,
        ..Default::default()
    };
    let result = scanner_has_offset.test_run(input).expect("test_run failed");

    println!("scanner_has_offset but first result = {}", result.return_value as i32);
    println!("scanner_has_offset but first duration = {:?}", result.duration);

    let offset_info = PacketOffsetInfo::read_from_bytes(&ctx_slice[48..68]).unwrap();
    println!("scanner_has_offset offsetinfo = {:?}", offset_info);

    let input = ProgramInput {
        data_in: Some(&mut payload),
        context_in: Some(ctx_slice),
        repeat,
        ..Default::default()
    };
    let result = scanner_has_offset.test_run(input).expect("test_run failed");

    println!("scanner_has_offset result = {}", result.return_value as i32);
    println!("scanner_has_offset duration = {:?}", result.duration);
}

pub mod tests {
    use crate::tests::scanner::performance::performance_test;

    #[test]
    fn one_time() {
        performance_test(1, crate::tests::dummpy_tcp_pkg());
    }

    #[test]
    fn all_server() {
        performance_test(5, crate::tests::dummpy_tcp_pkg());
    }

    #[test]
    fn test_server_1000() {
        performance_test(
            1_000_000,
            crate::tests::scanner::package::build_icmpv4_error_with_inner_ipv4_eth(),
        );
    }
}
