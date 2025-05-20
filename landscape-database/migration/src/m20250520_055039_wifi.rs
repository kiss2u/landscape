use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::wifi::WifiServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WifiServiceConfigs::Table)
                    .if_not_exists()
                    .col(string(WifiServiceConfigs::IfaceName).primary_key())
                    .col(boolean(WifiServiceConfigs::Enable))
                    .col(text(WifiServiceConfigs::Config))
                    .col(double(WifiServiceConfigs::UpdateAt).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(WifiServiceConfigs::Table).to_owned()).await
    }
}
