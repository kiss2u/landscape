use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::dhcp_v6_client::DHCPv6ClientConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DHCPv6ClientConfigs::Table)
                    .if_not_exists()
                    .col(string(DHCPv6ClientConfigs::IfaceName).primary_key())
                    .col(boolean(DHCPv6ClientConfigs::Enable))
                    .col(string(DHCPv6ClientConfigs::Mac))
                    .col(double(DHCPv6ClientConfigs::UpdateAt).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(DHCPv6ClientConfigs::Table).to_owned()).await
    }
}
