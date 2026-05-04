use std::{collections::HashMap, net::Ipv4Addr};

use sea_orm_migration::{prelude::*, schema::*, sea_orm::FromQueryResult};
use uuid::Uuid;

use crate::tables::{enrolled_device::EnrolledDevice, nat::StaticNatMappingConfigs};

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
        let device_ids_by_ipv4 = load_device_ids_by_ipv4(manager).await?;

        for row in rows {
            let target = build_target(&row, &device_ids_by_ipv4);
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

#[derive(FromQueryResult)]
struct DeviceIpv4Row {
    id: Uuid,
    ipv4: Option<String>,
}

async fn load_device_ids_by_ipv4(
    manager: &SchemaManager<'_>,
) -> Result<HashMap<Ipv4Addr, Uuid>, DbErr> {
    let select = Query::select()
        .columns([EnrolledDevice::Id, EnrolledDevice::Ipv4])
        .from(EnrolledDevice::Table)
        .and_where(Expr::col(EnrolledDevice::Ipv4).is_not_null())
        .to_owned();

    let builder = manager.get_database_backend();
    let db = manager.get_connection();
    let rows: Vec<DeviceIpv4Row> =
        DeviceIpv4Row::find_by_statement(builder.build(&select)).all(db).await?;

    Ok(rows.into_iter().filter_map(|row| Some((row.ipv4?.parse().ok()?, row.id))).collect())
}

fn build_target(
    row: &TargetMigrationRow,
    device_ids_by_ipv4: &HashMap<Ipv4Addr, Uuid>,
) -> serde_json::Value {
    if let Some(device_id) =
        row.lan_ipv4.as_deref().and_then(|ipv4| device_id_for_ipv4(ipv4, device_ids_by_ipv4))
    {
        serde_json::json!({
            "t": "device",
            "device_id": device_id.to_string(),
        })
    } else {
        serde_json::json!({
            "t": "address",
            "ipv4": row.lan_ipv4,
            "ipv6": row.lan_ipv6,
        })
    }
}

fn device_id_for_ipv4(ipv4: &str, device_ids_by_ipv4: &HashMap<Ipv4Addr, Uuid>) -> Option<Uuid> {
    let ipv4 = ipv4.parse::<Ipv4Addr>().ok()?;
    if ipv4.is_unspecified() {
        return None;
    }

    device_ids_by_ipv4.get(&ipv4).copied()
}
