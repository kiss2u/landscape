use sea_orm_migration::prelude::*;

use crate::tables::geo::GeoIpConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GeoIpConfigs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GeoIpConfigs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(GeoIpConfigs::UpdateAt).double().not_null())
                    .col(ColumnDef::new(GeoIpConfigs::Url).string().not_null())
                    .col(ColumnDef::new(GeoIpConfigs::Name).string().unique_key().not_null())
                    .col(ColumnDef::new(GeoIpConfigs::Enable).boolean().not_null())
                    .col(ColumnDef::new(GeoIpConfigs::NextUpdateAt).double().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(GeoIpConfigs::Table).to_owned()).await
    }
}
