use sea_orm_migration::prelude::*;

use crate::tables::geo::GeoSiteConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GeoSiteConfigs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GeoSiteConfigs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(GeoSiteConfigs::UpdateAt).double().not_null())
                    .col(ColumnDef::new(GeoSiteConfigs::Url).string().not_null())
                    .col(ColumnDef::new(GeoSiteConfigs::Name).string().unique_key().not_null())
                    .col(ColumnDef::new(GeoSiteConfigs::Enable).boolean().not_null())
                    .col(ColumnDef::new(GeoSiteConfigs::NextUpdateAt).double().not_null())
                    .col(ColumnDef::new(GeoSiteConfigs::GeoKeys).json().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(GeoSiteConfigs::Table).to_owned()).await
    }
}
