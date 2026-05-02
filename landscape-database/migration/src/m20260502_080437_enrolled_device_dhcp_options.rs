use sea_orm_migration::prelude::*;

use crate::tables::enrolled_device::EnrolledDevice;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(EnrolledDevice::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(EnrolledDevice::DhcpCustomOptions)
                            .json()
                            .not_null()
                            .default("[]"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(EnrolledDevice::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(EnrolledDevice::DhcpFilterOptions)
                            .json()
                            .not_null()
                            .default("[]"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(EnrolledDevice::Table)
                    .drop_column(EnrolledDevice::DhcpCustomOptions)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(EnrolledDevice::Table)
                    .drop_column(EnrolledDevice::DhcpFilterOptions)
                    .to_owned(),
            )
            .await
    }
}
