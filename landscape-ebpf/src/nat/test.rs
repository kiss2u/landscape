use std::mem::MaybeUninit;

use libbpf_rs::skel::{OpenSkel, SkelBuilder as _};

pub(crate) mod test_nat {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/test_nat.skel.rs"));
}

use test_nat::*;

pub fn test() {
    let landscape_builder = TestNatSkelBuilder::default();
    let mut open_object = MaybeUninit::uninit();
    let landscape_open = landscape_builder.open(&mut open_object).unwrap();

    let _ = landscape_open.load().unwrap();
}
