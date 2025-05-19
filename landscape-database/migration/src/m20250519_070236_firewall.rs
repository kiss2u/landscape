use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::firewall::FirewallServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FirewallServiceConfigs::Table)
                    .if_not_exists()
                    .col(string(FirewallServiceConfigs::IfaceName).primary_key())
                    .col(boolean(FirewallServiceConfigs::Enable))
                    .col(double(FirewallServiceConfigs::UpdateAt).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(FirewallServiceConfigs::Table).to_owned()).await
    }
}
