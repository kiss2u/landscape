use crate::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectInfo, ConnectKey,
    ConnectMetric, ConnectSortKey, SortOrder,
};
use duckdb::{params, Connection};
use std::path::PathBuf;
use std::thread;
use tokio::sync::{mpsc, oneshot};

/// Database operation messages
pub enum DBMessage {
    InsertConnectInfo(ConnectInfo),
    InsertMetric(ConnectMetric),

    QueryMetricByKey {
        key: ConnectKey,
        resp: oneshot::Sender<Vec<ConnectMetric>>,
    },
    QueryCurrentActiveConnectKeys {
        resp: oneshot::Sender<Vec<ConnectKey>>,
    },

    CollectAndCleanupOldMetrics {
        cutoff: u64,
        resp: oneshot::Sender<Box<Vec<ConnectMetric>>>,
    },
    QueryConnectHistory {
        limit: Option<usize>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        resp: oneshot::Sender<Vec<ConnectHistoryStatus>>,
    },
    QueryConnectHistoryComplex {
        params: ConnectHistoryQueryParams,
        resp: oneshot::Sender<Vec<ConnectHistoryStatus>>,
    },
    QueryGlobalStats {
        resp: oneshot::Sender<ConnectGlobalStats>,
    },
}

#[derive(Clone)]
pub struct DuckMetricStore {
    tx: mpsc::Sender<DBMessage>,
}

pub fn insert_connect_info(conn: &Connection, info: &ConnectInfo) {
    let key = &info.key;

    let event_type_val: u8 = info.event_type.clone().into();

    let stmt = "
    INSERT INTO connect VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
";

    conn.execute(
        stmt,
        duckdb::params![
            key.src_ip.to_string(),
            key.dst_ip.to_string(),
            key.src_port as i64,
            key.dst_port as i64,
            key.l4_proto as i64,
            key.l3_proto as i64,
            key.flow_id as i64,
            key.trace_id as i64,
            key.create_time as i64,
            event_type_val as i64,
            info.report_time as i64
        ],
    )
    .unwrap();
}

pub fn insert_metric(conn: &Connection, metric: &ConnectMetric) {
    let key = &metric.key;
    let stmt = "
        INSERT INTO metrics VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
    ";

    conn.execute(
        stmt,
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
            metric.report_time as i64,
            metric.ingress_bytes as i64,
            metric.ingress_packets as i64,
            metric.egress_bytes as i64,
            metric.egress_packets as i64,
        ],
    )
    .unwrap();
}

pub fn collect_and_cleanup_old_infos(conn: &Connection, cutoff: u64) -> Box<Vec<ConnectInfo>> {
    // Fetch expired connect records before deletion
    let stmt = "
        SELECT src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto,
               flow_id, trace_id, create_time, event_type, report_time
        FROM connect
        WHERE event_type != 1 AND report_time < ?1
    ";

    let mut stmt = conn.prepare(stmt).unwrap();

    let infos: Vec<ConnectInfo> = stmt
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

            let event_type = row.get::<_, i64>(9)? as u8;
            let report_time = row.get::<_, i64>(10)? as u64;

            Ok(ConnectInfo { key, event_type: event_type.into(), report_time })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    // Delete expired connect records
    let deleted = conn
        .execute(
            "DELETE FROM connect WHERE event_type != 1 AND report_time < ?1",
            params![cutoff as i64],
        )
        .unwrap();

    tracing::info!(
        "Cleanup complete: deleted {} records from connect table, remaining {} records",
        deleted,
        conn.prepare("SELECT COUNT(*) FROM connect")
            .unwrap()
            .query_row([], |row| row.get::<_, usize>(0))
            .unwrap()
    );

    Box::new(infos)
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

    Box::new(metrics)
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

pub fn start_db_thread(mut rx: mpsc::Receiver<DBMessage>, base_path: PathBuf) {
    // std::fs::create_dir_all(&base_path).expect("Failed to create base directory");

    let db_path = base_path.join("metrics.duckdb");
    let conn = Connection::open(db_path).unwrap();

    create_summaries_table(&conn);
    create_metrics_table(&conn).unwrap();

    // Schema migration: ensure columns exist
    let _ =
        conn.execute("ALTER TABLE connect_summaries ADD COLUMN IF NOT EXISTS status INTEGER", []);
    let _ = conn.execute("ALTER TABLE metrics ADD COLUMN IF NOT EXISTS status INTEGER", []);

    let mut metrics_appender = conn.appender("metrics").unwrap();

    let mut summary_stmt = conn.prepare("
        INSERT OR REPLACE INTO connect_summaries (
            src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id, create_time,
            last_report_time, total_ingress_bytes, total_egress_bytes, total_ingress_pkts, total_egress_pkts, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
    ").unwrap();

    let mut batch_count = 0;
    let flush_interval = std::time::Duration::from_secs(crate::DEFAULT_METRIC_FLUSH_INTERVAL_SECS);
    let mut last_flush = std::time::Instant::now();

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

    rt.block_on(async {
        loop {
            let now = std::time::Instant::now();
            let remaining = flush_interval.saturating_sub(now.duration_since(last_flush));

            let timeout_res = tokio::time::timeout(remaining, rx.recv()).await;

            match timeout_res {
                Ok(Some(msg)) => {
                    match msg {
                        DBMessage::InsertConnectInfo(info) => {
                            let key = &info.key;
                            let event_type_val: u8 = info.event_type.into();

                            // 直接更新汇总表现状
                            let _ = summary_stmt.execute(params![
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
                                0_i64, // 初始流量为0
                                0_i64,
                                0_i64,
                                0_i64,
                                event_type_val as i64,
                            ]);
                            batch_count += 1;
                        }
                        DBMessage::InsertMetric(metric) => {
                            let key = &metric.key;
                            let _ = metrics_appender.append_row(params![
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
                                metric.ingress_packets as i64,
                                metric.egress_bytes as i64,
                                metric.egress_packets as i64,
                                {
                                    let v: u8 = metric.status.clone().into();
                                    v as i64
                                },
                            ]);

                            let _ = summary_stmt.execute(params![
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
                                {
                                    let v: u8 = metric.status.clone().into();
                                    v as i64
                                },
                            ]);

                            batch_count += 1;
                        }
                        DBMessage::QueryMetricByKey { key, resp } => {
                            let result = query_metric_by_key(&conn, &key);
                            let _ = resp.send(result);
                        }
                        DBMessage::QueryCurrentActiveConnectKeys { resp } => {
                            let result = current_active_connect_keys(&conn);
                            let _ = resp.send(result);
                        }
                        DBMessage::CollectAndCleanupOldMetrics { cutoff, resp } => {
                            let _ = metrics_appender.flush();
                            batch_count = 0;
                            last_flush = std::time::Instant::now();
                            let result = collect_and_cleanup_old_metrics(&conn, cutoff);
                            let _ = resp.send(result);
                        }
                        DBMessage::QueryConnectHistory { limit, start_time, end_time, resp } => {
                            let _ = metrics_appender.flush();
                            let result = query_historical_summaries_complex(
                                &conn,
                                ConnectHistoryQueryParams {
                                    limit,
                                    start_time,
                                    end_time,
                                    ..Default::default()
                                },
                            );
                            let _ = resp.send(result);
                        }
                        DBMessage::QueryConnectHistoryComplex { params, resp } => {
                            let _ = metrics_appender.flush();
                            let result = query_historical_summaries_complex(&conn, params);
                            let _ = resp.send(result);
                        }
                        DBMessage::QueryGlobalStats { resp } => {
                            let result = query_global_stats(&conn);
                            let _ = resp.send(result);
                        }
                    }

                    if batch_count >= crate::DEFAULT_METRIC_BATCH_SIZE {
                        let _ = metrics_appender.flush();
                        batch_count = 0;
                        last_flush = std::time::Instant::now();
                    }
                }
                Ok(None) => break, // Channel closed
                Err(_) => {
                    // Timeout reached
                    if batch_count > 0 {
                        let _ = metrics_appender.flush();
                        batch_count = 0;
                    }
                    last_flush = std::time::Instant::now();
                }
            }
        }
    });
}

impl DuckMetricStore {
    pub async fn new(base_path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel::<DBMessage>(1024);
        thread::spawn(move || {
            start_db_thread(rx, base_path);
        });

        DuckMetricStore { tx }
    }

    pub async fn insert_connect_info(&self, info: ConnectInfo) {
        let _ = self.tx.send(DBMessage::InsertConnectInfo(info)).await;
    }

    pub async fn insert_metric(&self, metric: ConnectMetric) {
        let _ = self.tx.send(DBMessage::InsertMetric(metric)).await;
    }

    pub async fn query_metric_by_key(&self, key: ConnectKey) -> Vec<ConnectMetric> {
        let (resp, rx) = oneshot::channel();
        let _ = self.tx.send(DBMessage::QueryMetricByKey { key, resp }).await;

        rx.await.unwrap()
    }

    pub async fn current_active_connect_keys(&self) -> Vec<ConnectKey> {
        let (resp, rx) = oneshot::channel();
        let _ = self.tx.send(DBMessage::QueryCurrentActiveConnectKeys { resp }).await;
        rx.await.unwrap()
    }

    pub async fn collect_and_cleanup_old_metrics(&self, cutoff: u64) -> Box<Vec<ConnectMetric>> {
        let (resp, rx) = oneshot::channel();
        let _ = self.tx.send(DBMessage::CollectAndCleanupOldMetrics { cutoff, resp }).await;

        rx.await.unwrap()
    }

    pub async fn history_summaries(
        &self,
        limit: Option<usize>,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Vec<ConnectHistoryStatus> {
        let (resp, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(DBMessage::QueryConnectHistory { limit, start_time, end_time, resp })
            .await;
        rx.await.unwrap()
    }

    pub async fn history_summaries_complex(
        &self,
        params: ConnectHistoryQueryParams,
    ) -> Vec<ConnectHistoryStatus> {
        let (resp, rx) = oneshot::channel();
        if let Err(e) = self.tx.send(DBMessage::QueryConnectHistoryComplex { params, resp }).await {
            tracing::error!("Failed to send query message: {}", e);
            return Vec::new();
        }
        rx.await.unwrap_or_default()
    }

    pub async fn get_global_stats(&self) -> ConnectGlobalStats {
        let (resp, rx) = oneshot::channel::<ConnectGlobalStats>();
        if let Err(e) = self.tx.send(DBMessage::QueryGlobalStats { resp }).await {
            tracing::error!("Failed to send query global stats message: {}", e);
            return ConnectGlobalStats::default();
        }
        rx.await.unwrap_or_default()
    }
}

/// Create `connect_summaries` table to store the latest state of each connection
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

/// Create `metrics` table
fn create_metrics_table(conn: &Connection) -> duckdb::Result<()> {
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
