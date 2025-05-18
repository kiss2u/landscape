use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum DHCPv4ServerConfigs {
    #[sea_orm(iden = "dhcpv4_server_configs")]
    Table,
    IfaceName,
    Enable,
    IpRangeStart,
    IpRangeEnd,
    ServerIpAddr,
    NetworkMask,
    AddressLeaseTime,
    MacBindingRecords,
    UpdateAt,
}
