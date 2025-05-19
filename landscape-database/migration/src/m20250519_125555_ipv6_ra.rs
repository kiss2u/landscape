use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::ra::IPV6RAServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(IPV6RAServiceConfigs::Table)
                    .if_not_exists()
                    .col(string(IPV6RAServiceConfigs::IfaceName).primary_key())
                    .col(boolean(IPV6RAServiceConfigs::Enable))
                    .col(tiny_unsigned(IPV6RAServiceConfigs::SubnetPrefix))
                    // .col(big_unsigned(IPV6RAServiceConfigs::SubnetIndex))
                    .col(unsigned(IPV6RAServiceConfigs::SubnetIndex))
                    .col(string(IPV6RAServiceConfigs::DependIface))
                    .col(unsigned(IPV6RAServiceConfigs::RaPreferredLifetime))
                    .col(unsigned(IPV6RAServiceConfigs::RaValidLifetime))
                    .col(tiny_unsigned(IPV6RAServiceConfigs::RaFlag))
                    .col(double(IPV6RAServiceConfigs::UpdateAt).default(0.0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(IPV6RAServiceConfigs::Table).to_owned()).await
    }
}
