use duckdb::Statement;
use duckdb::{params, Connection};
use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectInfo, ConnectKey,
    ConnectMetric, ConnectSortKey, SortOrder,
};

pub enum ConnectQuery {
    ByKey(ConnectKey),
    ActiveKeys,
    HistorySummaries(ConnectHistoryQueryParams),
    GlobalStats,
}

pub enum ConnectQueryResult {
    Metrics(Vec<ConnectMetric>),
    Keys(Vec<ConnectKey>),
    HistoryStatuses(Vec<ConnectHistoryStatus>),
    Stats(ConnectGlobalStats),
}

pub fn handle_query(conn: &Connection, query: ConnectQuery) -> ConnectQueryResult {
    match query {
        ConnectQuery::ByKey(ref key) => ConnectQueryResult::Metrics(query_metric_by_key(conn, key)),
        ConnectQuery::ActiveKeys => ConnectQueryResult::Keys(current_active_connect_keys(conn)),
        ConnectQuery::HistorySummaries(params) => {
            ConnectQueryResult::HistoryStatuses(query_historical_summaries_complex(conn, params))
        }
        ConnectQuery::GlobalStats => ConnectQueryResult::Stats(query_global_stats(conn)),
    }
}

pub const SUMMARY_INSERT_SQL: &str = "
    INSERT OR REPLACE INTO connect_summaries (
        src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id, create_time,
        last_report_time, total_ingress_bytes, total_egress_bytes, total_ingress_pkts, total_egress_pkts, status
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
";

pub fn update_summary_by_info(stmt: &mut Statement, info: &ConnectInfo) -> duckdb::Result<usize> {
    let key = &info.key;
    let event_type_val: u8 = info.event_type.clone().into();

    stmt.execute(params![
        key.src_ip.to_string(),
        key.dst_ip.to_string(),
        key.src_port as i64,
        key.dst_port as i64,
        key.l4_proto as i64,
        key.l3_proto as i64,
        key.flow_id as i64,
        key.trace_id as i64,
        key.create_time as i64,
        info.report_time as i64,
        0_i64,
        0_i64,
        0_i64,
        0_i64,
        event_type_val as i64,
    ])
}

pub fn update_summary_by_metric(
    stmt: &mut Statement,
    metric: &ConnectMetric,
) -> duckdb::Result<usize> {
    let key = &metric.key;
    let event_type_val: u8 = metric.status.clone().into();

    stmt.execute(params![
        key.src_ip.to_string(),
        key.dst_ip.to_string(),
        key.src_port as i64,
        key.dst_port as i64,
        key.l4_proto as i64,
        key.l3_proto as i64,
        key.flow_id as i64,
        key.trace_id as i64,
        key.create_time as i64,
        metric.report_time as i64,
        metric.ingress_bytes as i64,
        metric.egress_bytes as i64,
        metric.ingress_packets as i64,
        metric.egress_packets as i64,
        event_type_val as i64,
    ])
}

pub fn create_summaries_table(conn: &Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS connect_summaries (
            src_ip VARCHAR,
            dst_ip VARCHAR,
            src_port INTEGER,
            dst_port INTEGER,
            l4_proto INTEGER,
            l3_proto INTEGER,
            flow_id INTEGER,
            trace_id INTEGER,
            create_time UBIGINT,
            last_report_time UBIGINT,
            total_ingress_bytes UBIGINT,
            total_egress_bytes UBIGINT,
            total_ingress_pkts UBIGINT,
            total_egress_pkts UBIGINT,
            status INTEGER,
            PRIMARY KEY (src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id, create_time)
        )",
        [],
    ).unwrap();
}

pub fn create_metrics_table(conn: &Connection) -> duckdb::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS metrics (
            src_ip TEXT,
            dst_ip TEXT,
            src_port INTEGER,
            dst_port INTEGER,
            l4_proto INTEGER,
            l3_proto INTEGER,
            flow_id INTEGER,
            trace_id INTEGER,
            create_time BIGINT,
            report_time BIGINT,
            ingress_bytes BIGINT,
            ingress_packets BIGINT,
            egress_bytes BIGINT,
            egress_packets BIGINT,
            status INTEGER
        );
        ",
    )
}

pub fn query_metric_by_key(conn: &Connection, key: &ConnectKey) -> Vec<ConnectMetric> {
    let stmt = "
        SELECT 
            report_time,
            ingress_bytes,
            ingress_packets,
            egress_bytes,
            egress_packets,
            status
        FROM metrics
        WHERE src_ip = ?1 AND dst_ip = ?2 AND src_port = ?3 AND dst_port = ?4
            AND l4_proto = ?5 AND l3_proto = ?6 AND flow_id = ?7 AND trace_id = ?8
            AND create_time = ?9
        ORDER BY report_time
    ";

    let mut stmt = conn.prepare(stmt).unwrap();

    let rows = stmt
        .query_map(
            params![
                key.src_ip.to_string(),
                key.dst_ip.to_string(),
                key.src_port as i64,
                key.dst_port as i64,
                key.l4_proto as i64,
                key.l3_proto as i64,
                key.flow_id as i64,
                key.trace_id as i64,
                key.create_time as i64,
            ],
            |row| {
                Ok(ConnectMetric {
                    key: key.clone(),
                    report_time: row.get(0)?,
                    ingress_bytes: row.get(1)?,
                    ingress_packets: row.get(2)?,
                    egress_bytes: row.get(3)?,
                    egress_packets: row.get(4)?,
                    status: row.get::<_, u8>(5)?.into(),
                })
            },
        )
        .unwrap();

    rows.filter_map(Result::ok).collect()
}

pub fn query_historical_summaries_complex(
    conn: &Connection,
    params: ConnectHistoryQueryParams,
) -> Vec<ConnectHistoryStatus> {
    let mut where_clauses = Vec::new();
    if let Some(start) = params.start_time {
        where_clauses.push(format!("last_report_time >= {}", start));
    }
    if let Some(end) = params.end_time {
        where_clauses.push(format!("last_report_time <= {}", end));
    }
    if let Some(ip) = params.src_ip {
        if !ip.is_empty() {
            where_clauses.push(format!("src_ip LIKE '%{}%'", ip));
        }
    }
    if let Some(ip) = params.dst_ip {
        if !ip.is_empty() {
            where_clauses.push(format!("dst_ip LIKE '%{}%'", ip));
        }
    }
    if let Some(p) = params.port_start {
        where_clauses.push(format!("src_port = {}", p));
    }
    if let Some(p) = params.port_end {
        where_clauses.push(format!("dst_port = {}", p));
    }
    if let Some(p) = params.l3_proto {
        where_clauses.push(format!("l3_proto = {}", p));
    }
    if let Some(p) = params.l4_proto {
        where_clauses.push(format!("l4_proto = {}", p));
    }
    if let Some(p) = params.flow_id {
        where_clauses.push(format!("flow_id = {}", p));
    }

    if let Some(s) = params.status {
        where_clauses.push(format!("status = {}", s));
    }

    let where_stmt = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let sort_col = match params.sort_key.unwrap_or_default() {
        ConnectSortKey::Port => "src_port",
        ConnectSortKey::Ingress => "total_ingress_bytes",
        ConnectSortKey::Egress => "total_egress_bytes",
        ConnectSortKey::Time => "create_time",
        ConnectSortKey::Duration => "(last_report_time - create_time)",
    };
    let sort_order_str = match params.sort_order.unwrap_or_default() {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };

    let limit_clause =
        if let Some(l) = params.limit { format!("LIMIT {}", l) } else { String::new() };

    let stmt = format!("
        SELECT 
            src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id, create_time,
            total_ingress_bytes, total_egress_bytes, total_ingress_pkts, total_egress_pkts, last_report_time, status
        FROM connect_summaries
        {}
        ORDER BY {} {}
        {}
    ", where_stmt, sort_col, sort_order_str, limit_clause);

    let mut stmt = match conn.prepare(&stmt) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to prepare SQL: {}, error: {}", stmt, e);
            return Vec::new();
        }
    };

    let rows = stmt.query_map([], |row| {
        let src_ip_str = row.get::<_, String>(0)?;
        let dst_ip_str = row.get::<_, String>(1)?;
        let key = ConnectKey {
            src_ip: src_ip_str.parse().unwrap_or("0.0.0.0".parse().unwrap()),
            dst_ip: dst_ip_str.parse().unwrap_or("0.0.0.0".parse().unwrap()),
            src_port: row.get::<_, i64>(2)? as u16,
            dst_port: row.get::<_, i64>(3)? as u16,
            l4_proto: row.get::<_, i64>(4)? as u8,
            l3_proto: row.get::<_, i64>(5)? as u8,
            flow_id: row.get::<_, i64>(6)? as u8,
            trace_id: row.get::<_, i64>(7)? as u8,
            create_time: row.get::<_, i64>(8)? as u64,
        };
        Ok(ConnectHistoryStatus {
            key,
            total_ingress_bytes: row.get::<_, i64>(9)? as u64,
            total_egress_bytes: row.get::<_, i64>(10)? as u64,
            total_ingress_pkts: row.get::<_, i64>(11)? as u64,
            total_egress_pkts: row.get::<_, i64>(12)? as u64,
            last_report_time: row.get::<_, i64>(13)? as u64,
            status: row.get::<_, i64>(14)? as u8,
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

pub fn current_active_connect_keys(conn: &Connection) -> Vec<ConnectKey> {
    let stmt = "
        SELECT src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id, create_time
        FROM connect_summaries
        WHERE status = 1
    ";

    let mut stmt = match conn.prepare(stmt) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to prepare SQL for active connects: {}", e);
            return Vec::new();
        }
    };

    let rows = stmt.query_map([], |row| {
        let src_ip_str = row.get::<_, String>(0)?;
        let dst_ip_str = row.get::<_, String>(1)?;
        Ok(ConnectKey {
            src_ip: src_ip_str.parse().unwrap_or("0.0.0.0".parse().unwrap()),
            dst_ip: dst_ip_str.parse().unwrap_or("0.0.0.0".parse().unwrap()),
            src_port: row.get::<_, i64>(2)? as u16,
            dst_port: row.get::<_, i64>(3)? as u16,
            l4_proto: row.get::<_, i64>(4)? as u8,
            l3_proto: row.get::<_, i64>(5)? as u8,
            flow_id: row.get::<_, i64>(6)? as u8,
            trace_id: row.get::<_, i64>(7)? as u8,
            create_time: row.get::<_, i64>(8)? as u64,
        })
    });

    match rows {
        Ok(r) => r.filter_map(Result::ok).collect(),
        Err(e) => {
            tracing::error!("Failed to execute active connects query: {}", e);
            Vec::new()
        }
    }
}

pub fn query_global_stats(conn: &Connection) -> ConnectGlobalStats {
    let stmt = "
        SELECT 
            SUM(total_ingress_bytes), 
            SUM(total_egress_bytes), 
            SUM(total_ingress_pkts), 
            SUM(total_egress_pkts), 
            COUNT(*) 
        FROM connect_summaries
    ";

    let mut stmt = match conn.prepare(stmt) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to prepare SQL for global stats: {}", e);
            return ConnectGlobalStats::default();
        }
    };

    let res = stmt.query_row([], |row| {
        Ok(ConnectGlobalStats {
            total_ingress_bytes: row.get::<_, Option<i64>>(0)?.unwrap_or(0) as u64,
            total_egress_bytes: row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
            total_ingress_pkts: row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u64,
            total_egress_pkts: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as u64,
            total_connect_count: row.get::<_, i64>(4)? as u64,
            last_calculate_time: chrono::Utc::now().timestamp_millis() as u64,
        })
    });

    res.unwrap_or_default()
}

pub fn collect_and_cleanup_old_metrics(conn: &Connection, cutoff: u64) -> Box<Vec<ConnectMetric>> {
    // Fetch expired metric records
    let stmt = "
        SELECT src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id, create_time,
               report_time, ingress_bytes, ingress_packets, egress_bytes, egress_packets, status
        FROM metrics
        WHERE report_time < ?1
    ";

    let mut stmt = conn.prepare(stmt).unwrap();

    let metrics = stmt
        .query_map([cutoff as i64], |row| {
            let key = ConnectKey {
                src_ip: row.get::<_, String>(0)?.parse().unwrap(),
                dst_ip: row.get::<_, String>(1)?.parse().unwrap(),
                src_port: row.get::<_, i64>(2)? as u16,
                dst_port: row.get::<_, i64>(3)? as u16,
                l4_proto: row.get::<_, i64>(4)? as u8,
                l3_proto: row.get::<_, i64>(5)? as u8,
                flow_id: row.get::<_, i64>(6)? as u8,
                trace_id: row.get::<_, i64>(7)? as u8,
                create_time: row.get::<_, i64>(8)? as u64,
            };

            Ok(ConnectMetric {
                key,
                report_time: row.get(9)?,
                ingress_bytes: row.get(10)?,
                ingress_packets: row.get(11)?,
                egress_bytes: row.get(12)?,
                egress_packets: row.get(13)?,
                status: row.get::<_, u8>(14)?.into(),
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    // Delete expired metric records
    let deleted_metrics =
        conn.execute("DELETE FROM metrics WHERE report_time < ?1", params![cutoff as i64]).unwrap();

    let size = conn
        .prepare("SELECT COUNT(*) FROM metrics")
        .unwrap()
        .query_row([], |row| row.get::<_, usize>(0))
        .unwrap();
    tracing::info!(
        "Cleanup complete: deleted {} metric records, remaining: {}",
        deleted_metrics,
        size
    );

    let deleted_summaries = conn
        .execute(
            "DELETE FROM connect_summaries WHERE last_report_time < ?1",
            params![cutoff as i64],
        )
        .unwrap_or(0);
    tracing::info!(
        "Cleanup summaries complete: deleted {} connect_summaries records",
        deleted_summaries
    );

    Box::new(metrics)
}
