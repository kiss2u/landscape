use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum NatServiceConfigs {
    Table,
    IfaceName,
    Enable,
    TcpRangeStart,
    TcpRangeEnd,
    UdpRangeStart,
    UdpRangeEnd,
    IcmpInRangeStart,
    IcmpInRangeEnd,
    UpdateAt,
}

#[derive(DeriveIden)]
pub enum StaticNatMappingConfigs {
    #[sea_orm(iden = "static_nat_mapping_configs")]
    Table,
    Id,
    Enable,
    Remark,
    WanPort,
    WanIfaceName,
    LanPort,
    #[sea_orm(iden = "lan_ipv4")]
    LanIpv4,
    #[sea_orm(iden = "lan_ipv6")]
    LanIpv6,
    #[sea_orm(iden = "ipv4_l4_protocol")]
    Ipv4L4Protocol,
    #[sea_orm(iden = "ipv6_l4_protocol")]
    Ipv6L4Protocol,
    UpdateAt,
}
