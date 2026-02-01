use landscape_common::metric::connect::SortOrder;
use landscape_common::metric::dns::{DnsHistoryQueryParams, DnsMetric, DnsSortKey};
use duckdb::Connection;

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
        ",
    )
}

pub fn query_dns_history(conn: &Connection, params: DnsHistoryQueryParams) -> Vec<DnsMetric> {
    let mut where_clauses = Vec::new();
    if let Some(start) = params.start_time {
        where_clauses.push(format!("report_time >= {}", start));
    }
    if let Some(end) = params.end_time {
        where_clauses.push(format!("report_time <= {}", end));
    }
    if let Some(domain) = params.domain {
        if !domain.is_empty() {
            where_clauses.push(format!("domain LIKE '%{}%'", domain));
        }
    }
    if let Some(ip) = params.src_ip {
        if !ip.is_empty() {
            where_clauses.push(format!("src_ip LIKE '%{}%'", ip));
        }
    }

    let where_stmt = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let sort_col = match params.sort_key.unwrap_or_default() {
        DnsSortKey::Time => "report_time",
        DnsSortKey::Domain => "domain",
        DnsSortKey::Duration => "duration_ms",
    };
    let sort_order_str = match params.sort_order.unwrap_or_default() {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };

    let limit_clause =
        if let Some(l) = params.limit { format!("LIMIT {}", l) } else { String::new() };

    let stmt = format!(
        "
        SELECT 
            flow_id, domain, query_type, response_code, report_time, duration_ms, src_ip, answers
        FROM dns_metrics
        {}
        ORDER BY {} {}
        {}
    ",
        where_stmt, sort_col, sort_order_str, limit_clause
    );

    let mut stmt = match conn.prepare(&stmt) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to prepare SQL: {}, error: {}", stmt, e);
            return Vec::new();
        }
    };

    let rows = stmt.query_map([], |row| {
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

    match rows {
        Ok(r) => r.filter_map(Result::ok).collect(),
        Err(e) => {
            tracing::error!("Failed to execute query: {}", e);
            Vec::new()
        }
    }
}

pub fn cleanup_old_dns_metrics(conn: &Connection, cutoff: u64) {
    let deleted_count = conn
        .execute("DELETE FROM dns_metrics WHERE report_time < ?1", duckdb::params![cutoff as i64])
        .unwrap_or(0);

    tracing::info!("DNS Cleanup complete: deleted {} dns metric records", deleted_count);
}
