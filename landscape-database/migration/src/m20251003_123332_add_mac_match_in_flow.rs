use sea_orm_migration::{prelude::*, sea_orm::FromQueryResult};
use uuid::Uuid;

use crate::tables::flow_rule::FlowConfigs;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 查询所有 flow_configs 记录
        let select = Query::select()
            .columns([Alias::new("id"), Alias::new("flow_match_rules")])
            .from(Alias::new("flow_configs"))
            .to_owned();

        let builder = manager.get_database_backend();
        let db = manager.get_connection();
        let rows: Vec<FlowConfigRow> =
            FlowConfigRow::find_by_statement(builder.build(&select)).all(db).await?;

        // 遍历每一行并更新 flow_match_rules
        for row in rows {
            let Ok(old_rules) =
                serde_json::from_str::<Vec<serde_json::Value>>(&row.flow_match_rules)
            else {
                continue;
            };

            let mut new_rules = Vec::new();

            for mut rule in old_rules {
                // 提取 qos
                let qos = rule.get("qos").cloned();

                // 移除 qos 字段
                if let Some(obj) = rule.as_object_mut() {
                    obj.remove("qos");
                }

                // 添加 t: "ip"
                if let Some(obj) = rule.as_object_mut() {
                    obj.insert("t".to_string(), serde_json::json!("ip"));
                }

                // 构造新规则
                let new_rule = serde_json::json!({
                    "qos": qos,
                    "mode": rule
                });

                new_rules.push(new_rule);
            }

            // 更新数据库
            let new_rules_json =
                serde_json::to_string(&new_rules).unwrap_or_else(|_| "[]".to_string());

            let update = Query::update()
                .table(FlowConfigs::Table)
                .values([(FlowConfigs::FlowMatchRules, new_rules_json.into())])
                .and_where(Expr::col(FlowConfigs::Id).eq(row.id))
                .to_owned();

            manager.exec_stmt(update).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 查询所有 flow_configs 记录
        let select = Query::select()
            .columns([Alias::new("id"), Alias::new("flow_match_rules")])
            .from(Alias::new("flow_configs"))
            .to_owned();

        let builder = manager.get_database_backend();
        let db = manager.get_connection();
        let rows: Vec<FlowConfigRow> =
            FlowConfigRow::find_by_statement(builder.build(&select)).all(db).await?;

        // 遍历每一行并还原 flow_match_rules
        for row in rows {
            let Ok(new_rules) =
                serde_json::from_str::<Vec<serde_json::Value>>(&row.flow_match_rules)
            else {
                continue;
            };

            let mut old_rules = Vec::new();

            for rule in new_rules {
                let qos = rule.get("qos").cloned();

                if let Some(mut mode) = rule.get("mode").cloned() {
                    let mode_type = mode.get("t").and_then(|v| v.as_str());

                    // 检查是否包含 mac 类型
                    if mode_type == Some("mac") {
                        continue;
                    }

                    // 移除 t 字段
                    if let Some(obj) = mode.as_object_mut() {
                        obj.remove("t");
                    }

                    // 添加 qos 和缺失字段设为 null
                    if let Some(obj) = mode.as_object_mut() {
                        obj.insert("qos".to_string(), qos.unwrap_or(serde_json::Value::Null));
                        if !obj.contains_key("vlan_id") {
                            obj.insert("vlan_id".to_string(), serde_json::Value::Null);
                        }
                    }

                    old_rules.push(mode);
                }
            }

            let old_rules_json =
                serde_json::to_string(&old_rules).unwrap_or_else(|_| "[]".to_string());

            let update = Query::update()
                .table(FlowConfigs::Table)
                .values([(FlowConfigs::FlowMatchRules, old_rules_json.into())])
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
    flow_match_rules: String,
}
