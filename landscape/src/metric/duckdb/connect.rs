use duckdb::{params, Connection};
use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectKey,
    ConnectMetricPoint, ConnectSortKey, MetricResolution, SortOrder,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub const SUMMARY_INSERT_SQL: &str = "
    INSERT INTO conn_summaries (
        create_time, cpu_id, src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id,
        last_report_time, total_ingress_bytes, total_egress_bytes, total_ingress_pkts, total_egress_pkts, status, create_time_ms, gress
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
    ON CONFLICT (create_time, cpu_id) DO UPDATE SET
        last_report_time = GREATEST(conn_summaries.last_report_time, EXCLUDED.last_report_time),
        total_ingress_bytes = GREATEST(conn_summaries.total_ingress_bytes, EXCLUDED.total_ingress_bytes),
        total_egress_bytes = GREATEST(conn_summaries.total_egress_bytes, EXCLUDED.total_egress_bytes),
        total_ingress_pkts = GREATEST(conn_summaries.total_ingress_pkts, EXCLUDED.total_ingress_pkts),
        total_egress_pkts = GREATEST(conn_summaries.total_egress_pkts, EXCLUDED.total_egress_pkts),
        status = CASE WHEN EXCLUDED.last_report_time >= conn_summaries.last_report_time THEN EXCLUDED.status ELSE conn_summaries.status END
";

pub fn create_summaries_table(conn: &Connection, schema: &str) {
    let prefix = if schema.is_empty() { "".to_string() } else { format!("{}.", schema) };
    let sql = format!(
        "
        CREATE TABLE IF NOT EXISTS {}conn_summaries (
            create_time UBIGINT,
            cpu_id INTEGER,
            src_ip VARCHAR,
            dst_ip VARCHAR,
            src_port INTEGER,
            dst_port INTEGER,
            l4_proto INTEGER,
            l3_proto INTEGER,
            flow_id INTEGER,
            trace_id INTEGER,
            last_report_time UBIGINT,
            total_ingress_bytes UBIGINT,
            total_egress_bytes UBIGINT,
            total_ingress_pkts UBIGINT,
            total_egress_pkts UBIGINT,
            status INTEGER,
            create_time_ms UBIGINT,
            gress INTEGER,
            PRIMARY KEY (create_time, cpu_id)
        );
        CREATE INDEX IF NOT EXISTS idx_conn_summaries_time ON {}conn_summaries (last_report_time);
    ",
        prefix, prefix
    );

    conn.execute_batch(&sql).expect("Failed to create summaries table");
}

pub fn create_metrics_table(conn: &Connection, schema: &str) -> duckdb::Result<()> {
    let prefix = if schema.is_empty() { "".to_string() } else { format!("{}.", schema) };
    let sql = format!(
        "
        CREATE TABLE IF NOT EXISTS {}conn_metrics_1m (
            create_time UBIGINT,
            cpu_id INTEGER,
            report_time BIGINT,
            ingress_bytes BIGINT,
            ingress_packets BIGINT,
            egress_bytes BIGINT,
            egress_packets BIGINT,
            status INTEGER,
            create_time_ms UBIGINT,
            PRIMARY KEY (create_time, cpu_id, report_time)
        );

        CREATE TABLE IF NOT EXISTS {}conn_metrics_1h (
            create_time UBIGINT,
            cpu_id INTEGER,
            report_time BIGINT,
            ingress_bytes BIGINT,
            ingress_packets BIGINT,
            egress_bytes BIGINT,
            egress_packets BIGINT,
            status INTEGER,
            create_time_ms UBIGINT,
            PRIMARY KEY (create_time, cpu_id, report_time)
        );

        CREATE TABLE IF NOT EXISTS {}conn_metrics_1d (
            create_time UBIGINT,
            cpu_id INTEGER,
            report_time BIGINT,
            ingress_bytes BIGINT,
            ingress_packets BIGINT,
            egress_bytes BIGINT,
            egress_packets BIGINT,
            status INTEGER,
            create_time_ms UBIGINT,
            PRIMARY KEY (create_time, cpu_id, report_time)
        );

        CREATE TABLE IF NOT EXISTS {}global_stats (
            total_ingress_bytes BIGINT,
            total_egress_bytes BIGINT,
            total_ingress_pkts BIGINT,
            total_egress_pkts BIGINT,
            total_connect_count BIGINT,
            last_calculate_time UBIGINT
        );

        CREATE INDEX IF NOT EXISTS idx_conn_metrics_1m_time ON {}conn_metrics_1m (report_time);
        CREATE INDEX IF NOT EXISTS idx_conn_metrics_1h_time ON {}conn_metrics_1h (report_time);
        CREATE INDEX IF NOT EXISTS idx_conn_metrics_1d_time ON {}conn_metrics_1d (report_time);
    ",
        prefix, prefix, prefix, prefix, prefix, prefix, prefix
    );

    conn.execute_batch(&sql)
}

pub fn create_live_tables(conn: &Connection) -> duckdb::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS conn_metrics (
            create_time UBIGINT,
            cpu_id INTEGER,
            report_time BIGINT,
            ingress_bytes BIGINT,
            ingress_packets BIGINT,
            egress_bytes BIGINT,
            egress_packets BIGINT,
            status INTEGER,
            create_time_ms UBIGINT
        );
        CREATE INDEX IF NOT EXISTS idx_conn_metrics_time ON conn_metrics (report_time);
        ",
    )
}

pub fn query_metric_by_key(
    conn: &Connection,
    key: &ConnectKey,
    _resolution: MetricResolution,
    _history_db_path: Option<&PathBuf>,
) -> Vec<ConnectMetricPoint> {
    let table = match _resolution {
        MetricResolution::Second => "conn_metrics",
        MetricResolution::Minute => "conn_metrics_1m",
        MetricResolution::Hour => "conn_metrics_1h",
        MetricResolution::Day => "conn_metrics_1d",
    };

    let stmt_str = format!(
        "
        SELECT
            report_time,
            ingress_bytes,
            ingress_packets,
            egress_bytes,
            egress_packets,
            status
        FROM {}
        WHERE create_time = ?1 AND cpu_id = ?2
        ORDER BY report_time
    ",
        table
    );

    let mut stmt = match conn.prepare(&stmt_str) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(
                "Failed to prepare query_metric_by_key SQL: {}, error: {}",
                stmt_str,
                e
            );
            return Vec::new();
        }
    };

    let rows = stmt.query_map(params![key.create_time as i64, key.cpu_id as i64,], |row| {
        Ok(ConnectMetricPoint {
            report_time: row.get(0)?,
            ingress_bytes: row.get(1)?,
            ingress_packets: row.get(2)?,
            egress_bytes: row.get(3)?,
            egress_packets: row.get(4)?,
            status: row.get::<_, u8>(5)?.into(),
        })
    });

    match rows {
        Ok(r) => r.filter_map(Result::ok).collect(),
        Err(e) => {
            tracing::error!("Failed to execute query_metric_by_key: {}", e);
            Vec::new()
        }
    }
}

pub fn query_historical_summaries_complex(
    conn: &Connection,
    params: ConnectHistoryQueryParams,
    _history_db_path: Option<&PathBuf>,
) -> Vec<ConnectHistoryStatus> {
    let now = landscape_common::utils::time::get_current_time_ms().unwrap_or_default();

    // Always use conn_summaries (no history prefix needed with new architecture)
    let table_name = "conn_summaries";

    if let Some(start) = params.start_time {
        tracing::debug!(
            "History Query - StartTime: {}, Now: {}, Diff: {}ms",
            start,
            now,
            now.saturating_sub(start)
        );
    }

    let mut where_clauses = Vec::new();
    let mut sql_params: Vec<Box<dyn duckdb::ToSql>> = Vec::new();

    if let Some(start) = params.start_time {
        where_clauses.push(format!("last_report_time >= {}", start));
    }
    if let Some(end) = params.end_time {
        where_clauses.push(format!("last_report_time <= {}", end));
    }
    if let Some(ip) = params.src_ip {
        if !ip.is_empty() {
            where_clauses.push("src_ip LIKE ?".to_string());
            sql_params.push(Box::new(format!("%{}%", ip)));
        }
    }
    if let Some(ip) = params.dst_ip {
        if !ip.is_empty() {
            where_clauses.push("dst_ip LIKE ?".to_string());
            sql_params.push(Box::new(format!("%{}%", ip)));
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
    if let Some(g) = params.gress {
        where_clauses.push(format!("gress = {}", g));
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
        ConnectSortKey::Time => "last_report_time",
        ConnectSortKey::Duration => {
            "(CAST(last_report_time AS BIGINT) - CAST(create_time_ms AS BIGINT))"
        }
    };
    let sort_order_str = match params.sort_order.unwrap_or_default() {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };

    let limit_clause =
        if let Some(l) = params.limit { format!("LIMIT {}", l) } else { String::new() };

    let stmt_str = format!("
        SELECT
            create_time, cpu_id, src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id,
            total_ingress_bytes, total_egress_bytes, total_ingress_pkts, total_egress_pkts, last_report_time, status, create_time_ms, gress
        FROM {}
        {}
        ORDER BY {} {}
        {}
    ", table_name, where_stmt, sort_col, sort_order_str, limit_clause);

    let mut stmt = match conn.prepare(&stmt_str) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to prepare SQL: {}, error: {}", stmt_str, e);
            return Vec::new();
        }
    };

    let param_refs: Vec<&dyn duckdb::ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(&param_refs[..], |row| {
        let create_time_ms: u64 = row.get::<_, i64>(16)? as u64;
        let key = ConnectKey {
            create_time: row.get::<_, i64>(0)? as u64,
            cpu_id: row.get::<_, i64>(1)? as u32,
        };
        Ok(ConnectHistoryStatus {
            key,
            src_ip: row.get::<_, String>(2)?.parse().unwrap_or("0.0.0.0".parse().unwrap()),
            dst_ip: row.get::<_, String>(3)?.parse().unwrap_or("0.0.0.0".parse().unwrap()),
            src_port: row.get::<_, i64>(4)? as u16,
            dst_port: row.get::<_, i64>(5)? as u16,
            l4_proto: row.get::<_, i64>(6)? as u8,
            l3_proto: row.get::<_, i64>(7)? as u8,
            flow_id: row.get::<_, i64>(8)? as u8,
            trace_id: row.get::<_, i64>(9)? as u8,
            total_ingress_bytes: row.get::<_, i64>(10)? as u64,
            total_egress_bytes: row.get::<_, i64>(11)? as u64,
            total_ingress_pkts: row.get::<_, i64>(12)? as u64,
            total_egress_pkts: row.get::<_, i64>(13)? as u64,
            last_report_time: row.get::<_, i64>(14)? as u64,
            status: row.get::<_, i64>(15)? as u8,
            create_time_ms,
            gress: row.get::<_, Option<i64>>(17)?.unwrap_or(0) as u8,
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

pub fn query_global_stats(conn: &Connection) -> ConnectGlobalStats {
    let stmt = "
        SELECT
            total_ingress_bytes,
            total_egress_bytes,
            total_ingress_pkts,
            total_egress_pkts,
            total_connect_count,
            last_calculate_time
        FROM global_stats
        LIMIT 1
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
            total_ingress_bytes: row.get::<_, i64>(0)? as u64,
            total_egress_bytes: row.get::<_, i64>(1)? as u64,
            total_ingress_pkts: row.get::<_, i64>(2)? as u64,
            total_egress_pkts: row.get::<_, i64>(3)? as u64,
            total_connect_count: row.get::<_, i64>(4)? as u64,
            last_calculate_time: row.get::<_, i64>(5)? as u64,
        })
    });

    res.unwrap_or_default()
}

pub fn aggregate_global_stats(conn: &Connection) -> duckdb::Result<()> {
    conn.execute_batch(
        "
        DELETE FROM global_stats;
        INSERT INTO global_stats
        SELECT
            SUM(max_ingress_bytes),
            SUM(max_egress_bytes),
            SUM(max_ingress_pkts),
            SUM(max_egress_pkts),
            COUNT(*),
            EXTRACT(EPOCH FROM now()) * 1000
        FROM (
            SELECT
                MAX(ingress_bytes) as max_ingress_bytes,
                MAX(egress_bytes) as max_egress_bytes,
                MAX(ingress_packets) as max_ingress_pkts,
                MAX(egress_packets) as max_egress_pkts
            FROM conn_metrics_1d
            GROUP BY create_time, cpu_id
        );
    ",
    )
}

pub fn perform_inner_db_rollup(conn: &Connection) -> duckdb::Result<()> {
    // 1. Aggregate raw metrics (5s) into 1 minute buckets
    conn.execute_batch(
        "
        INSERT INTO conn_metrics_1m (
            create_time, cpu_id, report_time,
            ingress_bytes, ingress_packets, egress_bytes, egress_packets, 
            status, create_time_ms
        )
        SELECT 
            create_time, cpu_id, (report_time // 60000) * 60000 as bucket_time,
            MAX(ingress_bytes), MAX(ingress_packets), MAX(egress_bytes), MAX(egress_packets),
            MAX(status), MAX(create_time_ms)
        FROM conn_metrics
        WHERE report_time >= (EXTRACT(EPOCH FROM now()) * 1000 - 600000)
        GROUP BY 1, 2, 3
        ON CONFLICT (create_time, cpu_id, report_time) DO UPDATE SET
            ingress_bytes = GREATEST(conn_metrics_1m.ingress_bytes, EXCLUDED.ingress_bytes),
            ingress_packets = GREATEST(conn_metrics_1m.ingress_packets, EXCLUDED.ingress_packets),
            egress_bytes = GREATEST(conn_metrics_1m.egress_bytes, EXCLUDED.egress_bytes),
            egress_packets = GREATEST(conn_metrics_1m.egress_packets, EXCLUDED.egress_packets),
            status = GREATEST(conn_metrics_1m.status, EXCLUDED.status);

        -- 2. Aggregate 1m into 1h
        INSERT INTO conn_metrics_1h (
            create_time, cpu_id, report_time,
            ingress_bytes, ingress_packets, egress_bytes, egress_packets, 
            status, create_time_ms
        )
        SELECT 
            create_time, cpu_id, (report_time // 3600000) * 3600000 as bucket_time,
            MAX(ingress_bytes), MAX(ingress_packets), MAX(egress_bytes), MAX(egress_packets),
            MAX(status), MAX(create_time_ms)
        FROM conn_metrics_1m
        WHERE report_time >= (EXTRACT(EPOCH FROM now()) * 1000 - 7200000)
        GROUP BY 1, 2, 3
        ON CONFLICT (create_time, cpu_id, report_time) DO UPDATE SET
            ingress_bytes = GREATEST(conn_metrics_1h.ingress_bytes, EXCLUDED.ingress_bytes),
            ingress_packets = GREATEST(conn_metrics_1h.ingress_packets, EXCLUDED.ingress_packets),
            egress_bytes = GREATEST(conn_metrics_1h.egress_bytes, EXCLUDED.egress_bytes),
            egress_packets = GREATEST(conn_metrics_1h.egress_packets, EXCLUDED.egress_packets),
            status = GREATEST(conn_metrics_1h.status, EXCLUDED.status);

        -- 3. Aggregate 1h into 1d
        INSERT INTO conn_metrics_1d (
            create_time, cpu_id, report_time,
            ingress_bytes, ingress_packets, egress_bytes, egress_packets, 
            status, create_time_ms
        )
        SELECT 
            create_time, cpu_id, (report_time // 86400000) * 86400000 as bucket_time,
            MAX(ingress_bytes), MAX(ingress_packets), MAX(egress_bytes), MAX(egress_packets),
            MAX(status), MAX(create_time_ms)
        FROM conn_metrics_1h
        WHERE report_time >= (EXTRACT(EPOCH FROM now()) * 1000 - 172800000)
        GROUP BY 1, 2, 3
        ON CONFLICT (create_time, cpu_id, report_time) DO UPDATE SET
            ingress_bytes = GREATEST(conn_metrics_1d.ingress_bytes, EXCLUDED.ingress_bytes),
            ingress_packets = GREATEST(conn_metrics_1d.ingress_packets, EXCLUDED.ingress_packets),
            egress_bytes = GREATEST(conn_metrics_1d.egress_bytes, EXCLUDED.egress_bytes),
            egress_packets = GREATEST(conn_metrics_1d.egress_packets, EXCLUDED.egress_packets),
            status = GREATEST(conn_metrics_1d.status, EXCLUDED.status);
    ",
    )?;
    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct CleanupStats {
    pub deleted_raw: usize,
    pub deleted_1m: usize,
    pub deleted_1h: usize,
    pub deleted_1d: usize,
    pub deleted_summaries: usize,
    pub budget_hit: bool,
    pub elapsed_ms: u128,
}

fn query_next_timestamp_before(
    conn: &Connection,
    table: &str,
    time_column: &str,
    lower_bound_inclusive: u64,
    cutoff_exclusive: u64,
) -> Option<u64> {
    let sql = format!(
        "SELECT MIN({time_column}) FROM {table} WHERE {time_column} >= ?1 AND {time_column} < ?2"
    );
    conn.query_row(&sql, params![lower_bound_inclusive as i64, cutoff_exclusive as i64], |row| {
        row.get::<_, Option<i64>>(0)
    })
    .ok()
    .flatten()
    .map(|ts| ts.max(0) as u64)
}

fn delete_table_in_slices(
    conn: &Connection,
    table: &str,
    time_column: &str,
    cutoff_exclusive: u64,
    slice_window_ms: u64,
    deadline: Instant,
) -> (usize, bool) {
    let mut deleted_total = 0usize;
    let mut cursor = query_next_timestamp_before(conn, table, time_column, 0, cutoff_exclusive);

    while let Some(slice_start) = cursor {
        if Instant::now() >= deadline {
            return (deleted_total, true);
        }

        let slice_end = (slice_start.saturating_add(slice_window_ms)).min(cutoff_exclusive);
        let sql = format!("DELETE FROM {table} WHERE {time_column} >= ?1 AND {time_column} < ?2");
        let deleted =
            conn.execute(&sql, params![slice_start as i64, slice_end as i64]).unwrap_or_else(|e| {
                tracing::error!("Failed to delete slice from {}: {}", table, e);
                0
            });
        deleted_total += deleted;

        cursor = query_next_timestamp_before(conn, table, time_column, slice_end, cutoff_exclusive);
    }

    (deleted_total, false)
}

pub fn cleanup_old_metrics_only(
    conn_mem: &Connection,
    conn_disk: &Connection,
    cutoff_raw: u64,
    cutoff_1m: u64,
    cutoff_1h: u64,
    cutoff_1d: u64,
    cleanup_time_budget_ms: u64,
    cleanup_slice_window_secs: u64,
) -> CleanupStats {
    let start = Instant::now();
    let budget_ms = cleanup_time_budget_ms.max(1);
    let deadline = start + Duration::from_millis(budget_ms);
    let slice_window_ms = cleanup_slice_window_secs.max(1) * 1000;

    let mut stats = CleanupStats::default();

    let (deleted_raw, budget_hit) = delete_table_in_slices(
        conn_mem,
        "conn_metrics",
        "report_time",
        cutoff_raw,
        slice_window_ms,
        deadline,
    );
    stats.deleted_raw = deleted_raw;
    stats.budget_hit = budget_hit;
    if stats.budget_hit {
        stats.elapsed_ms = start.elapsed().as_millis();
        return stats;
    }

    let (deleted_1m, budget_hit) = delete_table_in_slices(
        conn_disk,
        "conn_metrics_1m",
        "report_time",
        cutoff_1m,
        slice_window_ms,
        deadline,
    );
    stats.deleted_1m = deleted_1m;
    stats.budget_hit = budget_hit;
    if stats.budget_hit {
        stats.elapsed_ms = start.elapsed().as_millis();
        return stats;
    }

    let (deleted_1h, budget_hit) = delete_table_in_slices(
        conn_disk,
        "conn_metrics_1h",
        "report_time",
        cutoff_1h,
        slice_window_ms,
        deadline,
    );
    stats.deleted_1h = deleted_1h;
    stats.budget_hit = budget_hit;
    if stats.budget_hit {
        stats.elapsed_ms = start.elapsed().as_millis();
        return stats;
    }

    let (deleted_1d, budget_hit) = delete_table_in_slices(
        conn_disk,
        "conn_metrics_1d",
        "report_time",
        cutoff_1d,
        slice_window_ms,
        deadline,
    );
    stats.deleted_1d = deleted_1d;
    stats.budget_hit = budget_hit;
    if stats.budget_hit {
        stats.elapsed_ms = start.elapsed().as_millis();
        return stats;
    }

    let (deleted_summaries, budget_hit) = delete_table_in_slices(
        conn_disk,
        "conn_summaries",
        "last_report_time",
        cutoff_1d,
        slice_window_ms,
        deadline,
    );
    stats.deleted_summaries = deleted_summaries;
    stats.budget_hit = budget_hit;
    stats.elapsed_ms = start.elapsed().as_millis();

    stats
}

pub fn query_connection_ip_history(
    conn: &Connection,
    params: ConnectHistoryQueryParams,
    is_src: bool,
    _history_db_path: Option<&PathBuf>,
) -> Vec<landscape_common::metric::connect::IpHistoryStat> {
    // Always use conn_summaries (no history prefix needed with unified architecture)
    let table_name = "conn_summaries";

    let mut where_clauses = Vec::new();
    let mut sql_params: Vec<Box<dyn duckdb::ToSql>> = Vec::new();
    let col = if is_src { "src_ip" } else { "dst_ip" };
    if let Some(start) = params.start_time {
        where_clauses.push(format!("last_report_time >= {}", start));
    }
    if let Some(end) = params.end_time {
        where_clauses.push(format!("last_report_time <= {}", end));
    }
    if let Some(p) = params.flow_id {
        where_clauses.push(format!("flow_id = {}", p));
    }
    if let Some(ip) = params.src_ip {
        if !ip.is_empty() {
            where_clauses.push("src_ip LIKE ?".to_string());
            sql_params.push(Box::new(format!("%{}%", ip)));
        }
    }
    if let Some(ip) = params.dst_ip {
        if !ip.is_empty() {
            where_clauses.push("dst_ip LIKE ?".to_string());
            sql_params.push(Box::new(format!("%{}%", ip)));
        }
    }

    let where_stmt = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let sort_col = match params.sort_key.unwrap_or(ConnectSortKey::Ingress) {
        ConnectSortKey::Ingress => "2",
        ConnectSortKey::Egress => "3",
        _ => "2",
    };
    let sort_order_str = match params.sort_order.unwrap_or(SortOrder::Desc) {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };
    let limit_val = params.limit.unwrap_or(10);

    let stmt_str = format!(
        "
        SELECT
            {},
            SUM(total_ingress_bytes), SUM(total_egress_bytes),
            SUM(total_ingress_pkts), SUM(total_egress_pkts),
            COUNT(*)
        FROM {}
        {}
        GROUP BY 1
        ORDER BY {} {}
        LIMIT {}
    ",
        col, table_name, where_stmt, sort_col, sort_order_str, limit_val
    );

    let mut stmt = match conn.prepare(&stmt_str) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to prepare IP history SQL: {}, error: {}", stmt_str, e);
            return Vec::new();
        }
    };

    let param_refs: Vec<&dyn duckdb::ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(&param_refs[..], |row| {
        Ok(landscape_common::metric::connect::IpHistoryStat {
            ip: row.get::<_, String>(0)?.parse().unwrap_or("0.0.0.0".parse().unwrap()),
            flow_id: 0,
            total_ingress_bytes: row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
            total_egress_bytes: row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u64,
            total_ingress_pkts: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as u64,
            total_egress_pkts: row.get::<_, Option<i64>>(4)?.unwrap_or(0) as u64,
            connect_count: row.get::<_, i64>(5)? as u32,
        })
    });

    match rows {
        Ok(r) => r.filter_map(Result::ok).collect(),
        Err(e) => {
            tracing::error!("Failed to execute IP history query: {}", e);
            Vec::new()
        }
    }
}
