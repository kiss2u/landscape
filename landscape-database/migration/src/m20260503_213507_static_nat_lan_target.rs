use sea_orm_migration::{prelude::*, schema::*, sea_orm::FromQueryResult};
use uuid::Uuid;

use crate::tables::nat::StaticNatMappingConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(StaticNatMappingConfigs::Table)
                    .add_column_if_not_exists(json_null(StaticNatMappingConfigs::LanTarget))
                    .to_owned(),
            )
            .await?;

        let select = Query::select()
            .columns([
                StaticNatMappingConfigs::Id,
                StaticNatMappingConfigs::LanIpv4,
                StaticNatMappingConfigs::LanIpv6,
            ])
            .from(StaticNatMappingConfigs::Table)
            .and_where(Expr::col(StaticNatMappingConfigs::LanTarget).is_null())
            .to_owned();

        let builder = manager.get_database_backend();
        let db = manager.get_connection();
        let rows: Vec<TargetMigrationRow> =
            TargetMigrationRow::find_by_statement(builder.build(&select)).all(db).await?;

        for row in rows {
            let target = serde_json::json!({
                "t": "address",
                "ipv4": row.lan_ipv4,
                "ipv6": row.lan_ipv6,
            });
            let target_str = serde_json::to_string(&target).unwrap_or_else(|_| "null".into());

            let update = Query::update()
                .table(StaticNatMappingConfigs::Table)
                .values([(StaticNatMappingConfigs::LanTarget, target_str.into())])
                .and_where(Expr::col(StaticNatMappingConfigs::Id).eq(row.id))
                .to_owned();
            manager.exec_stmt(update).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(StaticNatMappingConfigs::Table)
                    .drop_column(StaticNatMappingConfigs::LanTarget)
                    .to_owned(),
            )
            .await
    }
}

#[derive(FromQueryResult)]
struct TargetMigrationRow {
    id: Uuid,
    lan_ipv4: Option<String>,
    lan_ipv6: Option<String>,
}
