use sea_orm_migration::prelude::*;

use crate::tables::pppd::PPPDServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PPPDServiceConfigs::Table)
                    .add_column(ColumnDef::new(PPPDServiceConfigs::Ac).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PPPDServiceConfigs::Table)
                    .drop_column(PPPDServiceConfigs::Ac)
                    .to_owned(),
            )
            .await
    }
}
