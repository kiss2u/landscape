use landscape_common::net::MacAddr;

#[derive(Debug)]
pub struct DHCPv6NACache {
    pub suffix: u64,
    pub hostname: Option<String>,
    pub mac: Option<MacAddr>,
    pub duid_hex: String,
    pub relative_offer_time: u64,
    pub valid_time: u32,
    pub preferred_time: u32,
    pub is_static: bool,
}

#[derive(Debug)]
pub struct DHCPv6PDCache {
    pub sub_index: u32,
    pub duid_hex: String,
    pub relative_offer_time: u64,
    pub valid_time: u32,
    pub preferred_time: u32,
}
