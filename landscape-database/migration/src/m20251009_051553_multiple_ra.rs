use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::ra::{IPV6RAServiceConfigs, IPV6RAServiceConfigs7_1_0};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop old table
        manager.drop_table(Table::drop().table(IPV6RAServiceConfigs::Table).to_owned()).await?;
        manager
            .create_table(
                Table::create()
                    .table(IPV6RAServiceConfigs7_1_0::Table)
                    .if_not_exists()
                    .col(string(IPV6RAServiceConfigs7_1_0::IfaceName).primary_key())
                    .col(boolean(IPV6RAServiceConfigs7_1_0::Enable))
                    .col(json_null(IPV6RAServiceConfigs7_1_0::Config))
                    .col(double(IPV6RAServiceConfigs7_1_0::UpdateAt).default(0.0))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(IPV6RAServiceConfigs7_1_0::Table).to_owned())
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(IPV6RAServiceConfigs::Table)
                    .if_not_exists()
                    .col(string(IPV6RAServiceConfigs::IfaceName).primary_key())
                    .col(boolean(IPV6RAServiceConfigs::Enable))
                    .col(tiny_unsigned(IPV6RAServiceConfigs::SubnetPrefix))
                    .col(unsigned(IPV6RAServiceConfigs::SubnetIndex))
                    .col(string(IPV6RAServiceConfigs::DependIface))
                    .col(unsigned(IPV6RAServiceConfigs::RaPreferredLifetime))
                    .col(unsigned(IPV6RAServiceConfigs::RaValidLifetime))
                    .col(tiny_unsigned(IPV6RAServiceConfigs::RaFlag))
                    .col(double(IPV6RAServiceConfigs::UpdateAt).default(0.0))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
