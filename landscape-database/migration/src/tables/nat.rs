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
    LanIp,
    #[sea_orm(iden = "l4_protocol")]
    L4Protocol,
    UpdateAt,
}
