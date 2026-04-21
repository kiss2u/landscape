use landscape_common::config::FlowId;
use landscape_common::error::LdError;
use landscape_common::flow::config::FlowConfig;
use landscape_common::flow::{FlowEntryMatchMode, FlowTarget};
use migration::Expr;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::flow_rule::entity::Column;
use crate::DBId;

use super::entity::{FlowConfigActiveModel, FlowConfigEntity, FlowConfigModel};

#[derive(Clone)]
pub struct FlowConfigRepository {
    db: DatabaseConnection,
}

impl FlowConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_flow_id(&self, flow_id: FlowId) -> Result<Option<FlowConfig>, LdError> {
        let result =
            FlowConfigEntity::find().filter(Column::FlowId.eq(flow_id)).one(&self.db).await?;

        Ok(result.map(From::from))
    }

    /// 查询是否有其他 flow config（排除 exclude_id）包含相同的入口匹配规则
    pub async fn find_conflict_by_entry_mode(
        &self,
        exclude_id: DBId,
        mode: &FlowEntryMatchMode,
    ) -> Result<Option<FlowConfig>, LdError> {
        let (condition_sql, params) = match mode {
            FlowEntryMatchMode::Mac { mac_addr } => (
                "json_extract(json_each.value, '$.mode.t') = 'mac' AND json_extract(json_each.value, '$.mode.mac_addr') = ?",
                vec![sea_orm::Value::String(Some(Box::new(mac_addr.to_string())))],
            ),
            FlowEntryMatchMode::Ip { ip, prefix_len } => (
                "json_extract(json_each.value, '$.mode.t') = 'ip' AND json_extract(json_each.value, '$.mode.ip') = ? AND json_extract(json_each.value, '$.mode.prefix_len') = ?",
                vec![
                    sea_orm::Value::String(Some(Box::new(ip.to_string()))),
                    sea_orm::Value::Int(Some(*prefix_len as i32)),
                ],
            ),
        };

        let full_sql = format!(
            "EXISTS (
            SELECT 1 FROM json_each(flow_match_rules)
            WHERE {}
        )",
            condition_sql
        );

        let expr = Expr::cust_with_values(&full_sql, params);

        let result = FlowConfigEntity::find()
            .filter(Column::Id.ne(exclude_id))
            .filter(expr)
            .one(&self.db)
            .await?;

        Ok(result.map(From::from))
    }

    pub async fn find_by_target(&self, t: FlowTarget) -> Result<Vec<FlowConfig>, LdError> {
        // 构造条件 SQL 和参数
        let (condition_sql, param_value) = match t {
            FlowTarget::Interface { name } => (
                "json_extract(json_each.value, '$.target.t') = 'interface' AND json_extract(json_each.value, '$.target.name') = ?",
                name,
            ),
            FlowTarget::Netns { container_name } => (
                "json_extract(json_each.value, '$.target.t') = 'netns' AND json_extract(json_each.value, '$.target.container_name') = ?",
                container_name,
            ),
        };

        let full_sql = format!(
            "EXISTS (
            SELECT 1 FROM json_each(packet_handle_iface_name)
            WHERE {}
        )",
            condition_sql
        );

        let expr = Expr::cust_with_values(
            &full_sql,
            vec![sea_orm::Value::String(Some(Box::new(param_value)))],
        );

        // 查询执行
        let result = FlowConfigEntity::find().filter(expr).all(&self.db).await?;

        Ok(result.into_iter().map(From::from).collect())
    }
}

crate::impl_repository!(
    FlowConfigRepository,
    FlowConfigModel,
    FlowConfigEntity,
    FlowConfigActiveModel,
    FlowConfig,
    DBId
);

crate::impl_flow_store!(FlowConfigRepository, FlowConfigModel, FlowConfigEntity);
