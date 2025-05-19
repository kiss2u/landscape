use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::flow::FlowWanServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FlowWanServiceConfigs::Table)
                    .if_not_exists()
                    .col(string(FlowWanServiceConfigs::IfaceName).primary_key())
                    .col(boolean(FlowWanServiceConfigs::Enable))
                    .col(double(FlowWanServiceConfigs::UpdateAt).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(FlowWanServiceConfigs::Table).to_owned()).await
    }
}
