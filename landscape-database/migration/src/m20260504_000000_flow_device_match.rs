use std::collections::HashMap;

use sea_orm_migration::{prelude::*, sea_orm::FromQueryResult};
use uuid::Uuid;

use crate::tables::{enrolled_device::EnrolledDevice, flow_rule::FlowConfigs};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let device_ids_by_mac = load_device_ids_by_mac(manager).await?;
        let rows = load_flow_config_rows(manager).await?;

        for row in rows {
            let Ok(rules) = serde_json::from_str::<serde_json::Value>(&row.flow_match_rules) else {
                continue;
            };

            let (rules, changed) = migrate_rules(rules, &device_ids_by_mac);
            if !changed {
                continue;
            }

            update_flow_match_rules(manager, row.id, rules).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let device_macs_by_id = load_device_macs_by_id(manager).await?;
        let rows = load_flow_config_rows(manager).await?;

        for row in rows {
            let Ok(rules) = serde_json::from_str::<serde_json::Value>(&row.flow_match_rules) else {
                continue;
            };

            let (rules, changed) = rollback_rules(rules, &device_macs_by_id);
            if !changed {
                continue;
            }

            update_flow_match_rules(manager, row.id, rules).await?;
        }

        Ok(())
    }
}

#[derive(FromQueryResult)]
struct FlowConfigRow {
    id: Uuid,
    flow_match_rules: String,
}

#[derive(FromQueryResult)]
struct DeviceMacRow {
    id: Uuid,
    mac: String,
}

async fn load_flow_config_rows(manager: &SchemaManager<'_>) -> Result<Vec<FlowConfigRow>, DbErr> {
    let select = Query::select()
        .columns([FlowConfigs::Id, FlowConfigs::FlowMatchRules])
        .from(FlowConfigs::Table)
        .to_owned();

    let builder = manager.get_database_backend();
    let db = manager.get_connection();
    FlowConfigRow::find_by_statement(builder.build(&select)).all(db).await
}

async fn update_flow_match_rules(
    manager: &SchemaManager<'_>,
    id: Uuid,
    rules: serde_json::Value,
) -> Result<(), DbErr> {
    let rules_str = serde_json::to_string(&rules).unwrap_or_else(|_| "[]".into());
    let update = Query::update()
        .table(FlowConfigs::Table)
        .values([(FlowConfigs::FlowMatchRules, rules_str.into())])
        .and_where(Expr::col(FlowConfigs::Id).eq(id))
        .to_owned();
    manager.exec_stmt(update).await
}

async fn load_device_mac_rows(manager: &SchemaManager<'_>) -> Result<Vec<DeviceMacRow>, DbErr> {
    let select = Query::select()
        .columns([EnrolledDevice::Id, EnrolledDevice::Mac])
        .from(EnrolledDevice::Table)
        .to_owned();

    let builder = manager.get_database_backend();
    let db = manager.get_connection();
    DeviceMacRow::find_by_statement(builder.build(&select)).all(db).await
}

async fn load_device_ids_by_mac(
    manager: &SchemaManager<'_>,
) -> Result<HashMap<String, Uuid>, DbErr> {
    Ok(load_device_mac_rows(manager)
        .await?
        .into_iter()
        .map(|row| (normalize_mac(&row.mac), row.id))
        .collect())
}

async fn load_device_macs_by_id(
    manager: &SchemaManager<'_>,
) -> Result<HashMap<Uuid, String>, DbErr> {
    Ok(load_device_mac_rows(manager).await?.into_iter().map(|row| (row.id, row.mac)).collect())
}

fn migrate_rules(
    rules: serde_json::Value,
    device_ids_by_mac: &HashMap<String, Uuid>,
) -> (serde_json::Value, bool) {
    let Some(rules) = rules.as_array() else {
        return (rules, false);
    };

    let mut changed = false;
    let rules = rules
        .iter()
        .map(|rule| {
            let mut rule = rule.clone();
            changed |= migrate_rule(&mut rule, device_ids_by_mac);
            rule
        })
        .collect();

    (serde_json::Value::Array(rules), changed)
}

fn rollback_rules(
    rules: serde_json::Value,
    device_macs_by_id: &HashMap<Uuid, String>,
) -> (serde_json::Value, bool) {
    let Some(rules) = rules.as_array() else {
        return (rules, false);
    };

    let mut changed = false;
    let rules = rules
        .iter()
        .map(|rule| {
            let mut rule = rule.clone();
            changed |= rollback_rule(&mut rule, device_macs_by_id);
            rule
        })
        .collect();

    (serde_json::Value::Array(rules), changed)
}

fn migrate_rule(rule: &mut serde_json::Value, device_ids_by_mac: &HashMap<String, Uuid>) -> bool {
    let Some(rule) = rule.as_object_mut() else {
        return false;
    };
    let Some(mode) = rule.get_mut("mode") else {
        return false;
    };
    let Some(device_id) = device_id_for_mac_mode(mode, device_ids_by_mac) else {
        return false;
    };

    *mode = serde_json::json!({
        "t": "device",
        "device_id": device_id.to_string(),
    });
    true
}

fn rollback_rule(rule: &mut serde_json::Value, device_macs_by_id: &HashMap<Uuid, String>) -> bool {
    let Some(rule) = rule.as_object_mut() else {
        return false;
    };
    let Some(mode) = rule.get_mut("mode") else {
        return false;
    };
    let Some(mac_addr) = mac_for_device_mode(mode, device_macs_by_id) else {
        return false;
    };

    *mode = serde_json::json!({
        "t": "mac",
        "mac_addr": mac_addr,
    });
    true
}

fn device_id_for_mac_mode(
    mode: &serde_json::Value,
    device_ids_by_mac: &HashMap<String, Uuid>,
) -> Option<Uuid> {
    let mode = mode.as_object()?;
    if mode.get("t").and_then(|value| value.as_str()) != Some("mac") {
        return None;
    }

    let mac_addr = mode.get("mac_addr").and_then(|value| value.as_str())?;
    device_ids_by_mac.get(&normalize_mac(mac_addr)).copied()
}

fn mac_for_device_mode(
    mode: &serde_json::Value,
    device_macs_by_id: &HashMap<Uuid, String>,
) -> Option<String> {
    let mode = mode.as_object()?;
    if mode.get("t").and_then(|value| value.as_str()) != Some("device") {
        return None;
    }

    let device_id = mode.get("device_id").and_then(|value| value.as_str())?;
    device_macs_by_id.get(&Uuid::parse_str(device_id).ok()?).cloned()
}

fn normalize_mac(mac: &str) -> String {
    mac.to_ascii_lowercase()
}
