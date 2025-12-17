use sea_orm_migration::prelude::*;

use crate::tables::route::RouteLanServiceConfigsV2;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(RouteLanServiceConfigsV2::Table)
                    .add_column(
                        ColumnDef::new(RouteLanServiceConfigsV2::StaticRoutes).json().null(),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(RouteLanServiceConfigsV2::Table)
                    .drop_column(RouteLanServiceConfigsV2::StaticRoutes)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
