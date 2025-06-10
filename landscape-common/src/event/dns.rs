pub enum DnsEvent {
    RuleUpdated { flow_id: Option<u32> },
    GeositeUpdated,
}

pub enum DstIpEvent {
    GeoIpUpdated,
}
