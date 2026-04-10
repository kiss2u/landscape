use sea_orm_migration::{prelude::*, schema::*};

use crate::tables::lan_ipv6::LanIPv6ServiceConfigs;
use crate::tables::lan_ipv6_v2::LanIPv6ServiceConfigsV2;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LanIPv6ServiceConfigsV2::Table)
                    .if_not_exists()
                    .col(string(LanIPv6ServiceConfigsV2::IfaceName).primary_key())
                    .col(boolean(LanIPv6ServiceConfigsV2::Enable))
                    .col(json_null(LanIPv6ServiceConfigsV2::Config))
                    .col(double(LanIPv6ServiceConfigsV2::UpdateAt).default(0.0))
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        let rows = db
            .query_all(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                format!(
                    "SELECT {}, {}, {}, {} FROM {}",
                    LanIPv6ServiceConfigs::IfaceName.to_string(),
                    LanIPv6ServiceConfigs::Enable.to_string(),
                    LanIPv6ServiceConfigs::Config.to_string(),
                    LanIPv6ServiceConfigs::UpdateAt.to_string(),
                    LanIPv6ServiceConfigs::Table.to_string(),
                ),
            ))
            .await?;

        for row in rows {
            let iface_name: String = row.try_get("", "iface_name")?;
            let enable: bool = row.try_get("", "enable")?;
            let config_str: String = row.try_get("", "config")?;
            let update_at: f64 = row.try_get("", "update_at")?;

            if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&config_str) {
                let migrated = migrate_to_v2(config_json);
                let migrated_str = serde_json::to_string(&migrated).unwrap_or_else(|_| "{}".into());
                db.execute_unprepared(&format!(
                    "INSERT OR REPLACE INTO {} ({}, {}, {}, {}) VALUES ('{}', {}, '{}', {})",
                    LanIPv6ServiceConfigsV2::Table.to_string(),
                    LanIPv6ServiceConfigsV2::IfaceName.to_string(),
                    LanIPv6ServiceConfigsV2::Enable.to_string(),
                    LanIPv6ServiceConfigsV2::Config.to_string(),
                    LanIPv6ServiceConfigsV2::UpdateAt.to_string(),
                    iface_name.replace('\'', "''"),
                    if enable { 1 } else { 0 },
                    migrated_str.replace('\'', "''"),
                    update_at,
                ))
                .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(LanIPv6ServiceConfigsV2::Table).to_owned()).await
    }
}

fn migrate_to_v2(mut config: serde_json::Value) -> serde_json::Value {
    let sources = config
        .get("sources")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let mut prefix_groups: Vec<serde_json::Value> = Vec::new();

    for source in sources {
        let Some(source_type) = source.get("t").and_then(|value| value.as_str()) else {
            continue;
        };
        match source_type {
            "ra_static" | "na_static" | "pd_static" => {
                merge_source_into_groups(&mut prefix_groups, source, true);
            }
            "ra_pd" | "na_pd" | "pd_pd" => {
                merge_source_into_groups(&mut prefix_groups, source, false);
            }
            _ => {}
        }
    }

    config
        .as_object_mut()
        .map(|object| object.remove("sources"));
    config["prefix_groups"] = serde_json::Value::Array(prefix_groups);
    config
}

fn merge_source_into_groups(
    groups: &mut Vec<serde_json::Value>,
    source: serde_json::Value,
    is_static: bool,
) {
    let (group_key, parent) = if is_static {
        let Some(base_prefix) = source.get("base_prefix").cloned() else {
            return;
        };
        let parent_prefix_len = if source.get("t").and_then(|v| v.as_str()) == Some("pd_static") {
            source
                .get("base_prefix_len")
                .and_then(|v| v.as_u64())
                .unwrap_or(60)
        } else {
            60
        };
        (
            format!("static:{}:{}", base_prefix, parent_prefix_len),
            serde_json::json!({
                "t": "static",
                "base_prefix": base_prefix,
                "parent_prefix_len": parent_prefix_len,
            }),
        )
    } else {
        let Some(depend_iface) = source.get("depend_iface").and_then(|v| v.as_str()) else {
            return;
        };
        let planned_parent_prefix_len = source
            .get("max_source_prefix_len")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);
        (
            format!("pd:{}:{}", depend_iface, planned_parent_prefix_len),
            serde_json::json!({
                "t": "pd",
                "depend_iface": depend_iface,
                "planned_parent_prefix_len": planned_parent_prefix_len,
            }),
        )
    };

    match source.get("t").and_then(|v| v.as_str()) {
        Some("ra_static") | Some("ra_pd") => {
            let idx = find_group_with_empty_slot(groups, &parent, "ra")
                .unwrap_or_else(|| create_group(groups, &group_key, &parent));
            let group = &mut groups[idx];
            group["ra"] = serde_json::json!({
                "pool_index": source.get("pool_index").and_then(|v| v.as_u64()).unwrap_or(0),
                "preferred_lifetime": source.get("preferred_lifetime").and_then(|v| v.as_u64()).unwrap_or(300),
                "valid_lifetime": source.get("valid_lifetime").and_then(|v| v.as_u64()).unwrap_or(600),
            });
        }
        Some("na_static") | Some("na_pd") => {
            let idx = find_group_with_empty_slot(groups, &parent, "na")
                .unwrap_or_else(|| create_group(groups, &group_key, &parent));
            let group = &mut groups[idx];
            group["na"] = serde_json::json!({
                "pool_index": source.get("pool_index").and_then(|v| v.as_u64()).unwrap_or(0),
            });
        }
        Some("pd_static") | Some("pd_pd") => {
            let pool_len = source.get("pool_len").and_then(|v| v.as_u64()).unwrap_or(64);
            let pool_index = source.get("pool_index").and_then(|v| v.as_u64()).unwrap_or(0);
            let idx = find_group_for_pd(groups, &parent, pool_len, pool_index)
                .unwrap_or_else(|| create_group(groups, &group_key, &parent));
            let group = &mut groups[idx];
            if group.get("pd").is_none() || group.get("pd").unwrap().is_null() {
                group["pd"] = serde_json::json!({
                    "pool_len": pool_len,
                    "start_index": pool_index,
                    "end_index": pool_index,
                });
                return;
            }
            let current_pool_len = group["pd"]["pool_len"].as_u64().unwrap_or(pool_len);
            let start_index = group["pd"]["start_index"].as_u64().unwrap_or(pool_index);
            let end_index = group["pd"]["end_index"].as_u64().unwrap_or(pool_index);
            let next_start = start_index.min(pool_index);
            let next_end = end_index.max(pool_index);
            group["pd"] = serde_json::json!({
                "pool_len": current_pool_len,
                "start_index": next_start,
                "end_index": next_end,
            });
        }
        _ => {}
    }
}

fn create_group(
    groups: &mut Vec<serde_json::Value>,
    base_group_key: &str,
    parent: &serde_json::Value,
) -> usize {
    let mut suffix = 0usize;
    loop {
        let group_id = if suffix == 0 {
            base_group_key.to_string()
        } else {
            format!("{}:{}", base_group_key, suffix)
        };
        if groups
            .iter()
            .all(|group| group.get("group_id").and_then(|v| v.as_str()) != Some(group_id.as_str()))
        {
            groups.push(serde_json::json!({
                "group_id": group_id,
                "parent": parent.clone(),
                "ra": null,
                "na": null,
                "pd": null,
            }));
            return groups.len() - 1;
        }
        suffix += 1;
    }
}

fn same_parent(group: &serde_json::Value, parent: &serde_json::Value) -> bool {
    group.get("parent") == Some(parent)
}

fn slot_is_empty(group: &serde_json::Value, slot: &str) -> bool {
    group.get(slot).map(|value| value.is_null()).unwrap_or(true)
}

fn find_group_with_empty_slot(
    groups: &[serde_json::Value],
    parent: &serde_json::Value,
    slot: &str,
) -> Option<usize> {
    groups.iter().position(|group| same_parent(group, parent) && slot_is_empty(group, slot))
}

fn pd_can_merge(group: &serde_json::Value, pool_len: u64, pool_index: u64) -> bool {
    if slot_is_empty(group, "pd") {
        return true;
    }

    let current_pool_len = group["pd"]["pool_len"].as_u64().unwrap_or(pool_len);
    if current_pool_len != pool_len {
        return false;
    }

    let start_index = group["pd"]["start_index"].as_u64().unwrap_or(pool_index);
    let end_index = group["pd"]["end_index"].as_u64().unwrap_or(pool_index);
    let next_start = start_index.min(pool_index);
    let next_end = end_index.max(pool_index);
    next_end - next_start + 1
        == end_index - start_index + 1 + if pool_index < start_index || pool_index > end_index { 1 } else { 0 }
}

fn find_group_for_pd(
    groups: &[serde_json::Value],
    parent: &serde_json::Value,
    pool_len: u64,
    pool_index: u64,
) -> Option<usize> {
    groups
        .iter()
        .position(|group| same_parent(group, parent) && pd_can_merge(group, pool_len, pool_index))
}
