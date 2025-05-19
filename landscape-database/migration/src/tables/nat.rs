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
