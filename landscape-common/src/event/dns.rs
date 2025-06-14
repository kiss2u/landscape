pub enum DnsEvent {
    RuleUpdated { flow_id: Option<u32> },
    GeositeUpdated,
    FlowUpdated,
}

pub enum DstIpEvent {
    GeoIpUpdated,
}
