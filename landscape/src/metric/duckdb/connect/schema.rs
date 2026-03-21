use duckdb::{params, Connection};
use landscape_common::metric::connect::ConnectMetric;

use crate::metric::duckdb::ingest::clean_ip_string;

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

pub fn upsert_metric_bucket(
    conn: &Connection,
    table: &str,
    metric: &ConnectMetric,
    bucket_report_time: u64,
) -> duckdb::Result<usize> {
    let sql = format!(
        "
        INSERT INTO {table} (
            create_time, cpu_id, report_time,
            ingress_bytes, ingress_packets, egress_bytes, egress_packets,
            status, create_time_ms
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT (create_time, cpu_id, report_time) DO UPDATE SET
            ingress_bytes = GREATEST({table}.ingress_bytes, EXCLUDED.ingress_bytes),
            ingress_packets = GREATEST({table}.ingress_packets, EXCLUDED.ingress_packets),
            egress_bytes = GREATEST({table}.egress_bytes, EXCLUDED.egress_bytes),
            egress_packets = GREATEST({table}.egress_packets, EXCLUDED.egress_packets),
            status = GREATEST({table}.status, EXCLUDED.status),
            create_time_ms = GREATEST({table}.create_time_ms, EXCLUDED.create_time_ms)
    "
    );

    let status: u8 = metric.status.clone().into();
    conn.execute(
        &sql,
        params![
            metric.key.create_time as i64,
            metric.key.cpu_id as i64,
            bucket_report_time as i64,
            metric.ingress_bytes as i64,
            metric.ingress_packets as i64,
            metric.egress_bytes as i64,
            metric.egress_packets as i64,
            status as i64,
            metric.create_time_ms as i64,
        ],
    )
}

pub fn upsert_summary(conn: &Connection, metric: &ConnectMetric) -> duckdb::Result<usize> {
    let status: u8 = metric.status.clone().into();
    conn.execute(
        SUMMARY_INSERT_SQL,
        params![
            metric.key.create_time as i64,
            metric.key.cpu_id as i64,
            clean_ip_string(&metric.src_ip),
            clean_ip_string(&metric.dst_ip),
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
            status as i64,
            metric.create_time_ms as i64,
            metric.gress as i64,
        ],
    )
}

pub fn create_summaries_table(conn: &Connection) -> duckdb::Result<()> {
    let sql = "
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
            gress INTEGER,
            PRIMARY KEY (create_time, cpu_id)
        );
        CREATE INDEX IF NOT EXISTS idx_conn_summaries_time ON conn_summaries (last_report_time);
    ";

    conn.execute_batch(sql)
}

pub fn create_metrics_table(conn: &Connection) -> duckdb::Result<()> {
    let sql = "
        CREATE TABLE IF NOT EXISTS conn_metrics_1m (
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

        CREATE TABLE IF NOT EXISTS conn_metrics_1h (
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

        CREATE TABLE IF NOT EXISTS conn_metrics_1d (
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

        CREATE INDEX IF NOT EXISTS idx_conn_metrics_1m_time ON conn_metrics_1m (report_time);
        CREATE INDEX IF NOT EXISTS idx_conn_metrics_1h_time ON conn_metrics_1h (report_time);
        CREATE INDEX IF NOT EXISTS idx_conn_metrics_1d_time ON conn_metrics_1d (report_time);
    ";

    conn.execute_batch(sql)
}
