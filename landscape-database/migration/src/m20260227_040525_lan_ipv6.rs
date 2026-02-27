use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::lan_ipv6::LanIPv6ServiceConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create new table
        manager
            .create_table(
                Table::create()
                    .table(LanIPv6ServiceConfigs::Table)
                    .if_not_exists()
                    .col(string(LanIPv6ServiceConfigs::IfaceName).primary_key())
                    .col(boolean(LanIPv6ServiceConfigs::Enable))
                    .col(json_null(LanIPv6ServiceConfigs::Config))
                    .col(double(LanIPv6ServiceConfigs::UpdateAt).default(0.0))
                    .to_owned(),
            )
            .await?;

        // 2. Copy all rows from old table to new table
        // The JSON config doesn't need modification — serde(default) handles the new `mode` field
        let db = manager.get_connection();
        db.execute_unprepared(
            "INSERT INTO lan_ipv6_service_configs (iface_name, enable, config, update_at) \
             SELECT iface_name, enable, config, update_at FROM ipv6_ra_service_configs",
        )
        .await?;

        // 3. Don't delete old table — cleaned up later

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(LanIPv6ServiceConfigs::Table).to_owned()).await?;
        Ok(())
    }
}
