use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::route::RouteLanServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RouteLanServiceConfigs::Table)
                    .if_not_exists()
                    .col(string(RouteLanServiceConfigs::IfaceName).primary_key())
                    .col(boolean(RouteLanServiceConfigs::Enable))
                    .col(double(RouteLanServiceConfigs::UpdateAt).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(RouteLanServiceConfigs::Table).to_owned()).await
    }
}
