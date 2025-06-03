use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::pppd::PPPDServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PPPDServiceConfigs::Table)
                    .if_not_exists()
                    .col(string(PPPDServiceConfigs::IfaceName).primary_key())
                    .col(string(PPPDServiceConfigs::AttachIfaceName))
                    .col(boolean(PPPDServiceConfigs::Enable))
                    .col(boolean(PPPDServiceConfigs::DefaultRoute))
                    .col(string(PPPDServiceConfigs::PeerId))
                    .col(string(PPPDServiceConfigs::Password))
                    .col(double(PPPDServiceConfigs::UpdateAt).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(PPPDServiceConfigs::Table).to_owned()).await
    }
}
