use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::route::RouteWanServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RouteWanServiceConfigs::Table)
                    .if_not_exists()
                    .col(string(RouteWanServiceConfigs::IfaceName).primary_key())
                    .col(boolean(RouteWanServiceConfigs::Enable))
                    .col(double(RouteWanServiceConfigs::UpdateAt).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(RouteWanServiceConfigs::Table).to_owned()).await
    }
}
