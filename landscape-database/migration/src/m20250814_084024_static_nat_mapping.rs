use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::nat::StaticNatMappingConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StaticNatMappingConfigs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(StaticNatMappingConfigs::Id).uuid().primary_key())
                    .col(ColumnDef::new(StaticNatMappingConfigs::Enable).boolean().default(false))
                    .col(integer(StaticNatMappingConfigs::WanPort))
                    .col(string_null(StaticNatMappingConfigs::Remark))
                    .col(string_null(StaticNatMappingConfigs::WanIfaceName))
                    .col(integer(StaticNatMappingConfigs::LanPort))
                    .col(string(StaticNatMappingConfigs::LanIp))
                    .col(json(StaticNatMappingConfigs::L4Protocol))
                    .col(double(StaticNatMappingConfigs::UpdateAt).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(StaticNatMappingConfigs::Table).to_owned()).await
    }
}
