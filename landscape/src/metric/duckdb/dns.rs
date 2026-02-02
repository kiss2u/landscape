use duckdb::Connection;
use landscape_common::metric::connect::SortOrder;
use landscape_common::metric::dns::{DnsHistoryQueryParams, DnsHistoryResponse, DnsMetric, DnsSortKey};

pub enum DnsQuery {
    History(DnsHistoryQueryParams),
}

pub fn handle_query(conn: &Connection, query: DnsQuery) -> DnsHistoryResponse {
    match query {
        DnsQuery::History(params) => query_dns_history(conn, params),
    }
}

pub fn create_dns_table(conn: &Connection) -> duckdb::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS dns_metrics (
            flow_id INTEGER,
            domain TEXT,
            query_type TEXT,
            response_code TEXT,
            report_time BIGINT,
            duration_ms INTEGER,
            src_ip TEXT,
            answers TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_dns_report_time ON dns_metrics (report_time);
        ",
    )
}

pub fn query_dns_history(conn: &Connection, mut params: DnsHistoryQueryParams) -> DnsHistoryResponse {
    let mut where_clauses = Vec::new();
    let mut sql_params: Vec<Box<dyn duckdb::ToSql>> = Vec::new();

    if let Some(start) = params.start_time {
        where_clauses.push(format!("report_time >= ?"));
        sql_params.push(Box::new(start as i64));
    }
    if let Some(end) = params.end_time {
        where_clauses.push(format!("report_time <= ?"));
        sql_params.push(Box::new(end as i64));
    }
    if let Some(domain) = params.domain.as_mut() {
        if !domain.is_empty() {
            // 同样处理一下末尾的点，确保匹配
            if domain.ends_with('.') && domain.len() > 1 {
                domain.pop();
            }
            where_clauses.push(format!("domain LIKE ?"));
            sql_params.push(Box::new(format!("%{}%", domain)));
        }
    }
    if let Some(ip) = params.src_ip {
        if !ip.is_empty() {
            where_clauses.push(format!("src_ip LIKE ?"));
            sql_params.push(Box::new(format!("%{}%", ip)));
        }
    }

    let where_stmt = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // 1. 获取总数
    let count_stmt_str = format!("SELECT COUNT(*) FROM dns_metrics {}", where_stmt);
    let total: usize = {
        let mut stmt = conn.prepare(&count_stmt_str).unwrap();
        let param_refs: Vec<&dyn duckdb::ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        stmt.query_row(&param_refs[..], |row| row.get(0)).unwrap_or(0)
    };

    // 2. 排序
    let sort_col = match params.sort_key.unwrap_or_default() {
        DnsSortKey::Time => "report_time",
        DnsSortKey::Domain => "domain",
        DnsSortKey::Duration => "duration_ms",
    };
    let sort_order_str = match params.sort_order.unwrap_or_default() {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };

    // 3. 分页
    let limit_val = params.limit.unwrap_or(20);
    let offset_val = params.offset.unwrap_or(0);

    let query_stmt_str = format!(
        "
        SELECT 
            flow_id, domain, query_type, response_code, report_time, duration_ms, src_ip, answers
        FROM dns_metrics
        {}
        ORDER BY {} {}
        LIMIT {} OFFSET {}
    ",
        where_stmt, sort_col, sort_order_str, limit_val, offset_val
    );

    let mut stmt = match conn.prepare(&query_stmt_str) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to prepare DNS SQL: {}, error: {}", query_stmt_str, e);
            return DnsHistoryResponse { items: Vec::new(), total };
        }
    };

    let param_refs: Vec<&dyn duckdb::ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(&param_refs[..], |row| {
        let answers_str: String = row.get(7)?;
        let answers: Vec<String> = serde_json::from_str(&answers_str).unwrap_or_default();
        Ok(DnsMetric {
            flow_id: row.get::<_, i64>(0)? as u32,
            domain: row.get(1)?,
            query_type: row.get(2)?,
            response_code: row.get(3)?,
            report_time: row.get::<_, i64>(4)? as u64,
            duration_ms: row.get::<_, i64>(5)? as u32,
            src_ip: row.get::<_, String>(6)?.parse().unwrap_or("0.0.0.0".parse().unwrap()),
            answers,
        })
    });

    let items = match rows {
        Ok(r) => r.filter_map(Result::ok).collect(),
        Err(e) => {
            tracing::error!("Failed to execute DNS query: {}", e);
            Vec::new()
        }
    };

    DnsHistoryResponse { items, total }
}

pub fn cleanup_old_dns_metrics(conn: &Connection, cutoff: u64) {
    let _ = conn.execute(
        "DELETE FROM dns_metrics WHERE report_time < ?1",
        duckdb::params![cutoff as i64],
    );
}
