use duckdb::Connection;

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
        CREATE INDEX IF NOT EXISTS idx_dns_report_time ON dns_metrics (report_time);
        CREATE INDEX IF NOT EXISTS idx_dns_domain ON dns_metrics (domain);
        CREATE INDEX IF NOT EXISTS idx_dns_src_ip ON dns_metrics (src_ip);
        CREATE INDEX IF NOT EXISTS idx_dns_status ON dns_metrics (status);
    ";

    conn.execute_batch(sql)
}

pub fn cleanup_old_dns_metrics(conn: &Connection, cutoff: u64) {
    let _ = conn
        .execute("DELETE FROM dns_metrics WHERE report_time < ?1", duckdb::params![cutoff as i64]);
}
