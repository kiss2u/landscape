//! PPPoE Integration Tests
//!
//! These tests create two network namespaces connected by a veth pair,
//! run pppoe-server in the server namespace or a scripted raw-socket server,
//! and exercise the native PPPoE client inside the client namespace, keeping
//! the host namespace untouched.
//!
//! # Prerequisites
//!
//! - Root privileges (for netns, veth, raw sockets)
//! - `pppoe-server` and `pppd` installed for standard-server tests
//!
//! # Running
//!
//! ```sh
//! RUST_TEST_THREADS=1 cargo test --package landscape --test pppoe_integration -- --nocapture
//! ```
//!
//! # Expected warnings
//!
//! libbpf may emit messages like "Parent Qdisc doesn't exists" or
//! "Cannot find specified filter chain" during tests. These are expected:
//! the PPPoE client tries to attach eBPF TC programs to the test veth
//! interface which has no qdisc configured. The warnings do not affect
//! the PPPoE negotiation or the test outcome.

#[path = "pppoe_integration/mod.rs"]
mod pppoe_integration;
