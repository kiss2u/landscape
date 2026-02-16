use duckdb::Statement;
use duckdb::{params, Connection};
use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectKey, ConnectMetric,
    ConnectMetricPoint, ConnectSortKey, MetricResolution, SortOrder,
};
use std::path::PathBuf;

pub const SUMMARY_INSERT_SQL: &str = "
    INSERT OR REPLACE INTO conn_summaries (
        create_time, cpu_id, src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id,
        last_report_time, total_ingress_bytes, total_egress_bytes, total_ingress_pkts, total_egress_pkts, status, create_time_ms
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
";

pub const METRICS_1M_INSERT_SQL: &str = "
    INSERT OR REPLACE INTO conn_metrics_1m (
        create_time, cpu_id, report_time,
        ingress_bytes, ingress_packets, egress_bytes, egress_packets, status, create_time_ms
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
";

pub const METRICS_1H_INSERT_SQL: &str = "
    INSERT OR REPLACE INTO conn_metrics_1h (
        create_time, cpu_id, report_time,
        ingress_bytes, ingress_packets, egress_bytes, egress_packets, status, create_time_ms
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
";

pub const METRICS_1D_INSERT_SQL: &str = "
    INSERT OR REPLACE INTO conn_metrics_1d (
        create_time, cpu_id, report_time,
        ingress_bytes, ingress_packets, egress_bytes, egress_packets, status, create_time_ms
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
";

pub const LIVE_METRIC_INSERT_SQL: &str = "
    INSERT INTO conn_realtime (
        create_time, cpu_id, src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id,
        ingress_bytes, egress_bytes, ingress_packets, egress_packets, ingress_bps, egress_bps, ingress_pps, egress_pps,
        last_report_time, status, create_time_ms
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)
    ON CONFLICT (create_time, cpu_id) DO UPDATE SET
        ingress_bps = CASE
            WHEN (EXCLUDED.last_report_time - last_report_time) > 0
            THEN (EXCLUDED.ingress_bytes - ingress_bytes) * 8000 / (EXCLUDED.last_report_time - last_report_time)
            ELSE ingress_bps
        END,
        egress_bps = CASE
            WHEN (EXCLUDED.last_report_time - last_report_time) > 0
            THEN (EXCLUDED.egress_bytes - egress_bytes) * 8000 / (EXCLUDED.last_report_time - last_report_time)
            ELSE egress_bps
        END,
        ingress_pps = CASE
            WHEN (EXCLUDED.last_report_time - last_report_time) > 0
            THEN (EXCLUDED.ingress_packets - ingress_packets) * 1000 / (EXCLUDED.last_report_time - last_report_time)
            ELSE ingress_pps
        END,
        egress_pps = CASE
            WHEN (EXCLUDED.last_report_time - last_report_time) > 0
            THEN (EXCLUDED.egress_packets - egress_packets) * 1000 / (EXCLUDED.last_report_time - last_report_time)
            ELSE egress_pps
        END,
        ingress_bytes = EXCLUDED.ingress_bytes,
        egress_bytes = EXCLUDED.egress_bytes,
        ingress_packets = EXCLUDED.ingress_packets,
        egress_packets = EXCLUDED.egress_packets,
        last_report_time = EXCLUDED.last_report_time,
        status = EXCLUDED.status
";

pub fn update_summary_by_metric(
    stmt: &mut Statement,
    metric: &ConnectMetric,
) -> duckdb::Result<usize> {
    let key = &metric.key;
    let event_type_val: u8 = metric.status.clone().into();

    stmt.execute(params![
        key.create_time as i64,
        key.cpu_id as i64,
        metric.src_ip.to_string(),
        metric.dst_ip.to_string(),
        metric.src_port as i64,
        metric.dst_port as i64,
        metric.l4_proto as i64,
        metric.l3_proto as i64,
        metric.flow_id as i64,
        metric.trace_id as i64,
        metric.report_time as i64,
        metric.ingress_bytes as i64,
        metric.egress_bytes as i64,
        metric.ingress_packets as i64,
        metric.egress_packets as i64,
        event_type_val as i64,
        metric.create_time_ms as i64,
    ])
}

pub fn update_live_metric(stmt: &mut Statement, metric: &ConnectMetric) -> duckdb::Result<usize> {
    let key = &metric.key;
    let event_type_val: u8 = metric.status.clone().into();

    stmt.execute(params![
        key.create_time as i64,
        key.cpu_id as i64,
        metric.src_ip.to_string(),
        metric.dst_ip.to_string(),
        metric.src_port as i64,
        metric.dst_port as i64,
        metric.l4_proto as i64,
        metric.l3_proto as i64,
        metric.flow_id as i64,
        metric.trace_id as i64,
        metric.ingress_bytes as i64,
        metric.egress_bytes as i64,
        metric.ingress_packets as i64,
        metric.egress_packets as i64,
        0i64,
        0i64,
        0i64,
        0i64,
        metric.report_time as i64,
        event_type_val as i64,
        metric.create_time_ms as i64,
    ])
}

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
    ",
        prefix, prefix, prefix, prefix
    );

    conn.execute_batch(&sql)
}

pub fn create_live_tables(conn: &Connection) -> duckdb::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS conn_realtime (
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
            ingress_bytes UBIGINT,
            egress_bytes UBIGINT,
            ingress_packets UBIGINT,
            egress_packets UBIGINT,
            ingress_bps UBIGINT,
            egress_bps UBIGINT,
            ingress_pps UBIGINT,
            egress_pps UBIGINT,
            last_report_time UBIGINT,
            status INTEGER,
            create_time_ms UBIGINT,
            PRIMARY KEY (create_time, cpu_id)
        );

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

        CREATE TABLE IF NOT EXISTS conn_summaries (
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
            PRIMARY KEY (create_time, cpu_id)
        );
        ",
    )
}

pub fn query_metric_by_key(
    conn: &Connection,
    key: &ConnectKey,
    resolution: MetricResolution,
    _history_db_path: Option<&PathBuf>,
) -> Vec<ConnectMetricPoint> {
    let table = match resolution {
        MetricResolution::Second => "conn_metrics", // Raw metrics from disk
        MetricResolution::Minute => "conn_metrics_1m", // 1-minute aggregation
        MetricResolution::Hour => "conn_metrics_1h", // 1-hour aggregation
        MetricResolution::Day => "conn_metrics_1d", // 1-day aggregation
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
            total_ingress_bytes, total_egress_bytes, total_ingress_pkts, total_egress_pkts, last_report_time, status, create_time_ms
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

    let rows = stmt.query_map([], |row| {
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
        SELECT create_time, cpu_id
        FROM conn_summaries
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
        Ok(ConnectKey {
            create_time: row.get::<_, i64>(0)? as u64,
            cpu_id: row.get::<_, i64>(1)? as u32,
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

pub fn perform_batch_rollup(conn_mem: &Connection, conn_disk: &Connection) -> duckdb::Result<()> {
    // Aggregate metrics in memory and sync to disk using INSERT OR REPLACE
    // Only sync aggregated data (1m/1h/1d), not raw conn_metrics (to reduce disk I/O)

    // 1m aggregation
    let sql_1m_query = "
        SELECT
            create_time, cpu_id, (report_time // 60000) * 60000 as report_time,
            MAX(ingress_bytes) as ingress_bytes, MAX(ingress_packets) as ingress_packets,
            MAX(egress_bytes) as egress_bytes, MAX(egress_packets) as egress_packets,
            MAX(status) as status, MAX(create_time_ms) as create_time_ms
        FROM conn_metrics
        GROUP BY 1, 2, 3";

    let mut stmt_1m = conn_mem.prepare(sql_1m_query)?;
    let rows_1m: Vec<_> = stmt_1m
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?, // create_time
                row.get::<_, i32>(1)?, // cpu_id
                row.get::<_, i64>(2)?, // report_time
                row.get::<_, i64>(3)?, // ingress_bytes
                row.get::<_, i64>(4)?, // ingress_packets
                row.get::<_, i64>(5)?, // egress_bytes
                row.get::<_, i64>(6)?, // egress_packets
                row.get::<_, i32>(7)?, // status
                row.get::<_, i64>(8)?, // create_time_ms
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut count_1m = 0;
    let sql_1m_insert = "INSERT OR REPLACE INTO conn_metrics_1m VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
    for row in rows_1m {
        conn_disk.execute(
            sql_1m_insert,
            params![row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8],
        )?;
        count_1m += 1;
    }

    // 1h aggregation
    let sql_1h_query = "
        SELECT
            create_time, cpu_id, (report_time // 3600000) * 3600000 as report_time,
            MAX(ingress_bytes) as ingress_bytes, MAX(ingress_packets) as ingress_packets,
            MAX(egress_bytes) as egress_bytes, MAX(egress_packets) as egress_packets,
            MAX(status) as status, MAX(create_time_ms) as create_time_ms
        FROM conn_metrics
        GROUP BY 1, 2, 3";

    let mut stmt_1h = conn_mem.prepare(sql_1h_query)?;
    let rows_1h: Vec<_> = stmt_1h
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i32>(7)?,
                row.get::<_, i64>(8)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut count_1h = 0;
    let sql_1h_insert = "INSERT OR REPLACE INTO conn_metrics_1h VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
    for row in rows_1h {
        conn_disk.execute(
            sql_1h_insert,
            params![row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8],
        )?;
        count_1h += 1;
    }

    // 1d aggregation
    let sql_1d_query = "
        SELECT
            create_time, cpu_id, (report_time // 86400000) * 86400000 as report_time,
            MAX(ingress_bytes) as ingress_bytes, MAX(ingress_packets) as ingress_packets,
            MAX(egress_bytes) as egress_bytes, MAX(egress_packets) as egress_packets,
            MAX(status) as status, MAX(create_time_ms) as create_time_ms
        FROM conn_metrics
        GROUP BY 1, 2, 3";

    let mut stmt_1d = conn_mem.prepare(sql_1d_query)?;
    let rows_1d: Vec<_> = stmt_1d
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i32>(7)?,
                row.get::<_, i64>(8)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut count_1d = 0;
    let sql_1d_insert = "INSERT OR REPLACE INTO conn_metrics_1d VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
    for row in rows_1d {
        conn_disk.execute(
            sql_1d_insert,
            params![row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8],
        )?;
        count_1d += 1;
    }

    // Step 2: Sync summaries from memory to disk
    let sql_summaries_query = "SELECT create_time, cpu_id, src_ip, dst_ip, src_port, dst_port,
        l4_proto, l3_proto, flow_id, trace_id, create_time_ms FROM conn_summaries";

    let mut stmt_summaries = conn_mem.prepare(sql_summaries_query)?;
    let mut rows_summaries = stmt_summaries.query([])?;

    // Prepare statement to query disk metrics for each summary
    let sql_metrics_lookup = "SELECT MAX(report_time) as last_report_time,
        MAX(ingress_bytes) as total_ingress_bytes, MAX(egress_bytes) as total_egress_bytes,
        MAX(ingress_packets) as total_ingress_pkts, MAX(egress_packets) as total_egress_pkts,
        MAX(status) as status
        FROM conn_metrics_1m WHERE create_time = ?1 AND cpu_id = ?2";

    let mut stmt_metrics = conn_disk.prepare(sql_metrics_lookup)?;

    // Use INSERT ... ON CONFLICT to update summaries
    let sql_summary_upsert = "INSERT INTO conn_summaries (
        create_time, cpu_id, src_ip, dst_ip, src_port, dst_port,
        l4_proto, l3_proto, flow_id, trace_id,
        last_report_time, total_ingress_bytes, total_egress_bytes,
        total_ingress_pkts, total_egress_pkts, status, create_time_ms
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
    ON CONFLICT (create_time, cpu_id) DO UPDATE SET
        last_report_time = GREATEST(conn_summaries.last_report_time, EXCLUDED.last_report_time),
        total_ingress_bytes = GREATEST(conn_summaries.total_ingress_bytes, EXCLUDED.total_ingress_bytes),
        total_egress_bytes = GREATEST(conn_summaries.total_egress_bytes, EXCLUDED.total_egress_bytes),
        total_ingress_pkts = GREATEST(conn_summaries.total_ingress_pkts, EXCLUDED.total_ingress_pkts),
        total_egress_pkts = GREATEST(conn_summaries.total_egress_pkts, EXCLUDED.total_egress_pkts),
        create_time_ms = LEAST(conn_summaries.create_time_ms, EXCLUDED.create_time_ms),
        status = EXCLUDED.status";

    let mut stmt_upsert = conn_disk.prepare(sql_summary_upsert)?;
    let mut summary_count = 0;

    while let Some(row) = rows_summaries.next()? {
        let create_time: i64 = row.get(0)?;
        let cpu_id: i32 = row.get(1)?;
        let src_ip: String = row.get(2)?;
        let dst_ip: String = row.get(3)?;
        let src_port: i32 = row.get(4)?;
        let dst_port: i32 = row.get(5)?;
        let l4_proto: i32 = row.get(6)?;
        let l3_proto: i32 = row.get(7)?;
        let flow_id: i64 = row.get(8)?;
        let trace_id: i64 = row.get(9)?;
        let create_time_ms: i64 = row.get(10)?;

        // Look up metrics from disk
        let mut metrics_rows = stmt_metrics.query(params![create_time, cpu_id])?;
        if let Some(metrics_row) = metrics_rows.next()? {
            let last_report_time: i64 = metrics_row.get(0).unwrap_or(0);
            let total_ingress_bytes: i64 = metrics_row.get(1).unwrap_or(0);
            let total_egress_bytes: i64 = metrics_row.get(2).unwrap_or(0);
            let total_ingress_pkts: i64 = metrics_row.get(3).unwrap_or(0);
            let total_egress_pkts: i64 = metrics_row.get(4).unwrap_or(0);
            let status: i32 = metrics_row.get(5).unwrap_or(0);

            stmt_upsert.execute(params![
                create_time,
                cpu_id,
                src_ip,
                dst_ip,
                src_port,
                dst_port,
                l4_proto,
                l3_proto,
                flow_id,
                trace_id,
                last_report_time,
                total_ingress_bytes,
                total_egress_bytes,
                total_ingress_pkts,
                total_egress_pkts,
                status,
                create_time_ms
            ])?;
            summary_count += 1;
        }
    }

    tracing::info!(
        "Batch rollup: 1m:{} 1h:{} 1d:{} sum:{}",
        count_1m,
        count_1h,
        count_1d,
        summary_count
    );

    // No CHECKPOINT here - rely on auto-checkpoint mechanism
    Ok(())
}

pub fn collect_and_cleanup_old_metrics(
    conn_mem: &Connection,
    conn_disk: &Connection,
    cutoff_raw: u64,
    cutoff_1m: u64,
    cutoff_1h: u64,
    cutoff_1d: u64,
) -> Box<Vec<ConnectMetric>> {
    // Fetch expired metric records from memory (for return value)
    // Join with memory summaries to get full metric info
    let stmt = "
        SELECT
            s.create_time, s.cpu_id, s.src_ip, s.dst_ip, s.src_port, s.dst_port, s.l4_proto, s.l3_proto, s.flow_id, s.trace_id,
            m.report_time, m.ingress_bytes, m.ingress_packets, m.egress_bytes, m.egress_packets, m.status, s.create_time_ms
        FROM conn_metrics m
        JOIN conn_summaries s ON m.create_time = s.create_time AND m.cpu_id = s.cpu_id
        WHERE m.report_time < ?1
    ";

    let mut stmt = match conn_mem.prepare(stmt) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to prepare cleanup SELECT SQL: {}, error: {}", stmt, e);
            return Box::new(Vec::new());
        }
    };

    let metrics_iter = stmt.query_map([cutoff_raw as i64], |row| {
        let key = ConnectKey {
            create_time: row.get::<_, i64>(0)? as u64,
            cpu_id: row.get::<_, i64>(1)? as u32,
        };

        Ok(ConnectMetric {
            key,
            src_ip: row.get::<_, String>(2)?.parse().unwrap_or("0.0.0.0".parse().unwrap()),
            dst_ip: row.get::<_, String>(3)?.parse().unwrap_or("0.0.0.0".parse().unwrap()),
            src_port: row.get::<_, i64>(4)? as u16,
            dst_port: row.get::<_, i64>(5)? as u16,
            l4_proto: row.get::<_, i64>(6)? as u8,
            l3_proto: row.get::<_, i64>(7)? as u8,
            flow_id: row.get::<_, i64>(8)? as u8,
            trace_id: row.get::<_, i64>(9)? as u8,
            report_time: row.get(10)?,
            create_time_ms: row.get(16)?,
            ingress_bytes: row.get(11)?,
            ingress_packets: row.get(12)?,
            egress_bytes: row.get(13)?,
            egress_packets: row.get(14)?,
            status: row.get::<_, u8>(15)?.into(),
        })
    });

    let metrics = match metrics_iter {
        Ok(r) => r.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(e) => {
            tracing::error!("Failed to execute cleanup SELECT: {}", e);
            Vec::new()
        }
    };

    // Delete expired metric records from memory
    let deleted_metrics = conn_mem
        .execute("DELETE FROM conn_metrics WHERE report_time < ?1", params![cutoff_raw as i64])
        .unwrap_or_else(|e| {
            tracing::error!("Failed to delete expired raw metrics from memory: {}", e);
            0
        });

    // Delete expired memory summaries (more aggressively, same as raw metrics)
    let deleted_memory_summaries = conn_mem
        .execute(
            "DELETE FROM conn_summaries WHERE last_report_time < ?1",
            params![cutoff_raw as i64],
        )
        .unwrap_or(0);

    let size = match conn_mem.prepare("SELECT COUNT(*) FROM conn_metrics") {
        Ok(mut stmt) => stmt.query_row([], |row| row.get::<_, usize>(0)).unwrap_or(0),
        Err(e) => {
            tracing::error!("Failed to prepare count query: {}", e);
            0
        }
    };

    tracing::info!(
        "Cleanup memory complete: deleted {} metric records, {} summaries, remaining: {}",
        deleted_metrics,
        deleted_memory_summaries,
        size
    );

    // Delete expired metric records from disk
    let _ = conn_disk
        .execute("DELETE FROM conn_metrics_1m WHERE report_time < ?1", params![cutoff_1m as i64])
        .map_err(|e| tracing::error!("Failed to delete expired 1m metrics: {}", e));

    let _ = conn_disk
        .execute("DELETE FROM conn_metrics_1h WHERE report_time < ?1", params![cutoff_1h as i64])
        .map_err(|e| tracing::error!("Failed to delete expired 1h metrics: {}", e));

    let _ = conn_disk
        .execute("DELETE FROM conn_metrics_1d WHERE report_time < ?1", params![cutoff_1d as i64])
        .map_err(|e| tracing::error!("Failed to delete expired 1d metrics: {}", e));

    let deleted_disk_summaries = conn_disk
        .execute(
            "DELETE FROM conn_summaries WHERE last_report_time < ?1",
            params![cutoff_1d as i64],
        )
        .unwrap_or(0);

    tracing::info!("Cleanup disk complete: deleted {} summaries", deleted_disk_summaries);

    Box::new(metrics)
}

pub fn query_connection_ip_history(
    conn: &Connection,
    params: ConnectHistoryQueryParams,
    is_src: bool,
    history_db_path: Option<&PathBuf>,
) -> Vec<landscape_common::metric::connect::IpHistoryStat> {
    let table_prefix = if history_db_path.is_some() { "history." } else { "" };
    let table_name = format!("{}conn_summaries", table_prefix);

    let mut where_clauses = Vec::new();
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
            where_clauses.push(format!("src_ip LIKE '%{}%'", ip));
        }
    }
    if let Some(ip) = params.dst_ip {
        if !ip.is_empty() {
            where_clauses.push(format!("dst_ip LIKE '%{}%'", ip));
        }
    }

    let where_stmt = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

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
        ORDER BY 2 DESC
        LIMIT 10
    ",
        col, table_name, where_stmt
    );

    let mut stmt = match conn.prepare(&stmt_str) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to prepare IP history SQL: {}, error: {}", stmt_str, e);
            return Vec::new();
        }
    };

    let rows = stmt.query_map([], |row| {
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
