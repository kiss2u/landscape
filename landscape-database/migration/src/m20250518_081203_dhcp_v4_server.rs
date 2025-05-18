use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::dhcp_v4_server::DHCPv4ServerConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DHCPv4ServerConfigs::Table)
                    .if_not_exists()
                    .col(string(DHCPv4ServerConfigs::IfaceName).primary_key())
                    .col(boolean(DHCPv4ServerConfigs::Enable))
                    .col(string(DHCPv4ServerConfigs::IpRangeStart))
                    .col(string_null(DHCPv4ServerConfigs::IpRangeEnd))
                    .col(string(DHCPv4ServerConfigs::ServerIpAddr))
                    .col(tiny_unsigned(DHCPv4ServerConfigs::NetworkMask))
                    .col(unsigned_null(DHCPv4ServerConfigs::AddressLeaseTime))
                    .col(json_null(DHCPv4ServerConfigs::MacBindingRecords))
                    .col(double(DHCPv4ServerConfigs::UpdateAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(DHCPv4ServerConfigs::Table).to_owned()).await
    }
}
