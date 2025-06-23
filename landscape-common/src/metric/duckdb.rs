use crate::metric::connect::{ConnectInfo, ConnectKey, ConnectMetric};
use duckdb::{params, Connection};
use std::path::PathBuf;
use std::thread;
use tokio::sync::{mpsc, oneshot};

/// Database operation messages
pub enum DBMessage {
    InsertConnectInfo(ConnectInfo),
    InsertMetric(ConnectMetric),

    QueryMetricLastMin { key: ConnectKey, since_ms: u64, resp: oneshot::Sender<Vec<ConnectMetric>> },
    QueryCurrentActiveConnectKeys { resp: oneshot::Sender<Vec<ConnectKey>> },

    CollectAndCleanupOldMetrics { cutoff: u64, resp: oneshot::Sender<Box<Vec<ConnectMetric>>> },
    CollectAndCleanupOldInfos { cutoff: u64, resp: oneshot::Sender<Box<Vec<ConnectInfo>>> },
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
               report_time, ingress_bytes, ingress_packets, egress_bytes, egress_packets
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

pub fn query_metric_last_min(
    conn: &Connection,
    key: &ConnectKey,
    since_ms: u64,
) -> Vec<ConnectMetric> {
    let stmt = "
        SELECT report_time, ingress_bytes, ingress_packets, egress_bytes, egress_packets
        FROM metrics
        WHERE src_ip = ?1 AND dst_ip = ?2 AND src_port = ?3 AND dst_port = ?4
            AND l4_proto = ?5 AND l3_proto = ?6 AND flow_id = ?7 AND trace_id = ?8
            AND create_time = ?9 AND report_time >= ?10
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
                since_ms as i64,
            ],
            |row| {
                Ok(ConnectMetric {
                    key: key.clone(),
                    report_time: row.get(0)?,
                    ingress_bytes: row.get(1)?,
                    ingress_packets: row.get(2)?,
                    egress_bytes: row.get(3)?,
                    egress_packets: row.get(4)?,
                })
            },
        )
        .unwrap();

    rows.filter_map(Result::ok).collect()
}

pub fn current_active_connect_keys(conn: &Connection) -> Vec<ConnectKey> {
    let stmt = "
        SELECT * FROM (
            SELECT *,
                   ROW_NUMBER() OVER (PARTITION BY src_ip, dst_ip, src_port, dst_port,
                                             l4_proto, l3_proto, flow_id, trace_id, create_time
                                      ORDER BY report_time DESC) as rn
            FROM connect
        ) WHERE rn = 1 AND event_type = 1
    ";

    let mut stmt = conn.prepare(stmt).unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok(ConnectKey {
                src_ip: row.get::<_, String>(0)?.parse().unwrap(),
                dst_ip: row.get::<_, String>(1)?.parse().unwrap(),
                src_port: row.get::<_, i64>(2)? as u16,
                dst_port: row.get::<_, i64>(3)? as u16,
                l4_proto: row.get::<_, i64>(4)? as u8,
                l3_proto: row.get::<_, i64>(5)? as u8,
                flow_id: row.get::<_, i64>(6)? as u8,
                trace_id: row.get::<_, i64>(7)? as u8,
                create_time: row.get::<_, i64>(8)? as u64,
            })
        })
        .unwrap();

    rows.filter_map(Result::ok).collect()
}

pub fn start_db_thread(mut rx: mpsc::Receiver<DBMessage>) {
    // Create a single-threaded DuckDB connection
    let conn = Connection::open_in_memory().unwrap();

    create_connect_table(&conn).unwrap();
    create_metrics_table(&conn).unwrap();

    while let Some(msg) = rx.blocking_recv() {
        match msg {
            DBMessage::InsertConnectInfo(info) => {
                insert_connect_info(&conn, &info);
            }
            DBMessage::InsertMetric(metric) => {
                insert_metric(&conn, &metric);
            }
            DBMessage::QueryMetricLastMin { key, since_ms, resp } => {
                let result = query_metric_last_min(&conn, &key, since_ms);
                let _ = resp.send(result);
            }
            DBMessage::QueryCurrentActiveConnectKeys { resp } => {
                let result = current_active_connect_keys(&conn);
                let _ = resp.send(result);
            }
            DBMessage::CollectAndCleanupOldMetrics { cutoff, resp } => {
                let result = collect_and_cleanup_old_metrics(&conn, cutoff);
                let _ = resp.send(result);
            }
            DBMessage::CollectAndCleanupOldInfos { cutoff, resp } => {
                let result = collect_and_cleanup_old_infos(&conn, cutoff);
                let _ = resp.send(result);
            }
        }
    }
}

impl DuckMetricStore {
    pub async fn new(_base_path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel::<DBMessage>(1024);
        thread::spawn(move || {
            start_db_thread(rx);
        });

        DuckMetricStore { tx }
    }

    pub async fn insert_connect_info(&self, info: ConnectInfo) {
        let _ = self.tx.send(DBMessage::InsertConnectInfo(info)).await;
    }

    pub async fn insert_metric(&self, metric: ConnectMetric) {
        let _ = self.tx.send(DBMessage::InsertMetric(metric)).await;
    }

    pub async fn query_metric_last_min(&self, key: ConnectKey) -> Vec<ConnectMetric> {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let since_ms = now - 60 * 1000;
        let (resp, rx) = oneshot::channel();
        let _ = self.tx.send(DBMessage::QueryMetricLastMin { key, since_ms, resp }).await;

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

    pub async fn collect_and_cleanup_old_infos(&self, cutoff: u64) -> Box<Vec<ConnectInfo>> {
        let (resp, rx) = oneshot::channel();
        let _ = self.tx.send(DBMessage::CollectAndCleanupOldInfos { cutoff, resp }).await;

        rx.await.unwrap()
    }
}

/// Create `connect` table
fn create_connect_table(conn: &Connection) -> duckdb::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS connect (
            src_ip TEXT,
            dst_ip TEXT,
            src_port INTEGER,
            dst_port INTEGER,
            l4_proto INTEGER,
            l3_proto INTEGER,
            flow_id INTEGER,
            trace_id INTEGER,
            create_time BIGINT,
            event_type INTEGER,
            report_time BIGINT
        );
        ",
    )
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
            egress_packets BIGINT
        );
        ",
    )
}
