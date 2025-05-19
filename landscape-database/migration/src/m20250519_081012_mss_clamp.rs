use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::mss_clamp::MssClampServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MssClampServiceConfigs::Table)
                    .if_not_exists()
                    .col(string(MssClampServiceConfigs::IfaceName).primary_key())
                    .col(boolean(MssClampServiceConfigs::Enable))
                    .col(small_unsigned(MssClampServiceConfigs::ClampSize))
                    .col(double(MssClampServiceConfigs::UpdateAt).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(MssClampServiceConfigs::Table).to_owned()).await
    }
}
