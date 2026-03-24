use duckdb::{params, Connection};

pub fn create_dns_table(conn: &Connection) -> duckdb::Result<()> {
    let sql = "
        CREATE TABLE IF NOT EXISTS dns_metrics (
            flow_id INTEGER,
            domain TEXT,
            query_type TEXT,
            response_code TEXT,
            report_time BIGINT,
            duration_ms INTEGER,
            src_ip TEXT,
            answers TEXT,
            status TEXT
        );
    ";

    conn.execute_batch(sql)
}

#[allow(clippy::too_many_arguments)]
pub fn insert_dns_row(
    conn: &Connection,
    flow_id: u32,
    domain: &str,
    query_type: &str,
    response_code: &str,
    report_time: u64,
    duration_ms: u32,
    src_ip: &str,
    answers_json: &str,
    status_json: &str,
) -> duckdb::Result<usize> {
    conn.execute(
        "INSERT INTO dns_metrics (
            flow_id, domain, query_type, response_code,
            report_time, duration_ms, src_ip, answers, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            flow_id as i64,
            domain,
            query_type,
            response_code,
            report_time as i64,
            duration_ms as i64,
            src_ip,
            answers_json,
            status_json,
        ],
    )
}

pub fn cleanup_old_dns_metrics(conn: &Connection, cutoff: u64) {
    let _ = conn.execute("DELETE FROM dns_metrics WHERE report_time < ?1", params![cutoff as i64]);
}
