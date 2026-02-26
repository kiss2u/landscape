use landscape_common::net::MacAddr;
use std::net::Ipv6Addr;

/// Generate DUID-LL (type 3) from MAC address
pub fn gen_server_duid(mac: &MacAddr) -> Vec<u8> {
    let mut duid = Vec::with_capacity(10);
    // DUID type 3 (DUID-LL): 00 03
    duid.extend_from_slice(&[0x00, 0x03]);
    // Hardware type: Ethernet (1): 00 01
    duid.extend_from_slice(&[0x00, 0x01]);
    // MAC address
    duid.extend_from_slice(&mac.octets());
    duid
}

/// Extract MAC address from client DUID
pub fn extract_mac_from_duid(duid: &[u8]) -> Option<MacAddr> {
    if duid.len() < 4 {
        return None;
    }
    let duid_type = u16::from_be_bytes([duid[0], duid[1]]);
    match duid_type {
        // DUID-LLT (type 1): 2 bytes type + 2 bytes hw type + 4 bytes time + 6 bytes MAC
        1 => {
            if duid.len() >= 14 {
                let mac_bytes: [u8; 6] = duid[8..14].try_into().ok()?;
                Some(MacAddr::from(mac_bytes))
            } else {
                None
            }
        }
        // DUID-LL (type 3): 2 bytes type + 2 bytes hw type + 6 bytes MAC
        3 => {
            if duid.len() >= 10 {
                let mac_bytes: [u8; 6] = duid[4..10].try_into().ok()?;
                Some(MacAddr::from(mac_bytes))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Combine a prefix with a suffix (host part)
pub fn combine_prefix_suffix(prefix: Ipv6Addr, prefix_len: u8, suffix: u64) -> Ipv6Addr {
    let p = u128::from(prefix);
    let mask = if prefix_len >= 128 { !0u128 } else { !0u128 << (128 - prefix_len) };
    Ipv6Addr::from((p & mask) | (suffix as u128))
}

/// Compute a delegated sub-prefix from a base prefix
pub fn compute_delegated_prefix(
    base_prefix: Ipv6Addr,
    base_prefix_len: u8,
    delegate_len: u8,
    sub_index: u32,
) -> Ipv6Addr {
    let base = u128::from(base_prefix);
    let base_mask = if base_prefix_len >= 128 { !0u128 } else { !0u128 << (128 - base_prefix_len) };
    let base_network = base & base_mask;

    let shift_bits = 128 - delegate_len;
    let sub_prefix = base_network | ((sub_index as u128) << shift_bits);
    Ipv6Addr::from(sub_prefix)
}

/// Convert bytes to hex string
pub fn duid_to_hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Hash a DUID to get a seed for pool allocation
pub fn hash_duid(duid: &[u8]) -> u64 {
    let mut hash: u64 = 5381;
    for &byte in duid {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}
