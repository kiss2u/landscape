use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::nat::NatServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NatServiceConfigs::Table)
                    .if_not_exists()
                    .col(string(NatServiceConfigs::IfaceName).primary_key())
                    .col(small_unsigned(NatServiceConfigs::TcpRangeStart))
                    .col(small_unsigned(NatServiceConfigs::TcpRangeEnd))
                    .col(small_unsigned(NatServiceConfigs::UdpRangeStart))
                    .col(small_unsigned(NatServiceConfigs::UdpRangeEnd))
                    .col(small_unsigned(NatServiceConfigs::IcmpInRangeStart))
                    .col(small_unsigned(NatServiceConfigs::IcmpInRangeEnd))
                    .col(boolean(NatServiceConfigs::Enable))
                    .col(double(NatServiceConfigs::UpdateAt).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(NatServiceConfigs::Table).to_owned()).await
    }
}
