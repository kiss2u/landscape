use sea_orm_migration::{prelude::*, sea_orm::FromQueryResult};
use uuid::Uuid;

use crate::tables::flow_rule::FlowConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let select = Query::select()
            .columns([FlowConfigs::Id, FlowConfigs::PacketHandleIfaceName])
            .from(FlowConfigs::Table)
            .to_owned();

        let builder = manager.get_database_backend();
        let db = manager.get_connection();
        let rows: Vec<FlowConfigRow> =
            FlowConfigRow::find_by_statement(builder.build(&select)).all(db).await?;

        for row in rows {
            let Ok(targets) =
                serde_json::from_str::<serde_json::Value>(&row.packet_handle_iface_name)
            else {
                continue;
            };
            let migrated = migrate_flow_targets(targets);
            let migrated_str = serde_json::to_string(&migrated).unwrap_or_else(|_| "[]".into());

            let update = Query::update()
                .table(FlowConfigs::Table)
                .values([(FlowConfigs::PacketHandleIfaceName, migrated_str.into())])
                .and_where(Expr::col(FlowConfigs::Id).eq(row.id))
                .to_owned();
            manager.exec_stmt(update).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let select = Query::select()
            .columns([FlowConfigs::Id, FlowConfigs::PacketHandleIfaceName])
            .from(FlowConfigs::Table)
            .to_owned();

        let builder = manager.get_database_backend();
        let db = manager.get_connection();
        let rows: Vec<FlowConfigRow> =
            FlowConfigRow::find_by_statement(builder.build(&select)).all(db).await?;

        for row in rows {
            let Ok(targets) =
                serde_json::from_str::<serde_json::Value>(&row.packet_handle_iface_name)
            else {
                continue;
            };
            let migrated = rollback_flow_targets(targets);
            let migrated_str = serde_json::to_string(&migrated).unwrap_or_else(|_| "[]".into());

            let update = Query::update()
                .table(FlowConfigs::Table)
                .values([(FlowConfigs::PacketHandleIfaceName, migrated_str.into())])
                .and_where(Expr::col(FlowConfigs::Id).eq(row.id))
                .to_owned();
            manager.exec_stmt(update).await?;
        }

        Ok(())
    }
}

#[derive(FromQueryResult)]
struct FlowConfigRow {
    id: Uuid,
    packet_handle_iface_name: String,
}

fn migrate_flow_targets(value: serde_json::Value) -> serde_json::Value {
    let Some(targets) = value.as_array() else {
        return serde_json::Value::Array(Vec::new());
    };

    let migrated = targets.iter().map(migrate_single_flow_target).collect::<Vec<_>>();

    serde_json::Value::Array(migrated)
}

fn migrate_single_flow_target(value: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "target": value,
        "weight": 1,
    })
}

fn rollback_flow_targets(value: serde_json::Value) -> serde_json::Value {
    let Some(targets) = value.as_array() else {
        return serde_json::Value::Array(Vec::new());
    };

    let rolled_back = targets.iter().map(rollback_single_flow_target).collect::<Vec<_>>();
    serde_json::Value::Array(rolled_back)
}

fn rollback_single_flow_target(value: &serde_json::Value) -> serde_json::Value {
    value.get("target").cloned().unwrap_or(serde_json::Value::Null)
}

#[cfg(test)]
mod tests {
    use super::{migrate_flow_targets, rollback_flow_targets};

    #[test]
    fn migrates_legacy_targets_to_weighted_shape() {
        let migrated = migrate_flow_targets(serde_json::json!([
            { "t": "interface", "name": "wan0" },
            { "t": "netns", "container_name": "ns0" }
        ]));

        assert_eq!(
            migrated,
            serde_json::json!([
                { "target": { "t": "interface", "name": "wan0" }, "weight": 1 },
                { "target": { "t": "netns", "container_name": "ns0" }, "weight": 1 }
            ])
        );
    }

    #[test]
    fn wraps_existing_targets_under_target_key() {
        let migrated = migrate_flow_targets(serde_json::json!([
            { "t": "interface", "name": "wan0" }
        ]));

        assert_eq!(
            migrated,
            serde_json::json!([
                { "target": { "t": "interface", "name": "wan0" }, "weight": 1 }
            ])
        );
    }

    #[test]
    fn rolls_weighted_targets_back_to_legacy_shape() {
        let rolled_back = rollback_flow_targets(serde_json::json!([
            { "target": { "t": "interface", "name": "wan0" }, "weight": 1 },
            { "target": { "t": "netns", "container_name": "ns0" }, "weight": 3 }
        ]));

        assert_eq!(
            rolled_back,
            serde_json::json!([
                { "t": "interface", "name": "wan0" },
                { "t": "netns", "container_name": "ns0" }
            ])
        );
    }
}
