use duckdb::{params, Connection};
use landscape_common::metric::connect::{
    ConnectGlobalStats, ConnectHistoryQueryParams, ConnectHistoryStatus, ConnectKey,
    ConnectMetricPoint, ConnectSortKey, MetricResolution, SortOrder,
};

struct ConnectHistoryWhereBuilder {
    clauses: Vec<String>,
    params: Vec<Box<dyn duckdb::ToSql>>,
}

impl ConnectHistoryWhereBuilder {
    fn new() -> Self {
        Self { clauses: Vec::new(), params: Vec::new() }
    }

    fn push_param<T>(&mut self, clause: &str, value: T)
    where
        T: duckdb::ToSql + 'static,
    {
        self.clauses.push(clause.to_string());
        self.params.push(Box::new(value));
    }

    fn push_common_filters(mut self, params: &ConnectHistoryQueryParams) -> Self {
        if let Some(start) = params.start_time {
            self.push_param("last_report_time >= ?", start as i64);
        }
        if let Some(end) = params.end_time {
            self.push_param("last_report_time <= ?", end as i64);
        }
        if let Some(ip) = params.src_ip.as_ref().filter(|ip| !ip.is_empty()) {
            self.push_param("src_ip LIKE ?", format!("%{}%", ip));
        }
        if let Some(ip) = params.dst_ip.as_ref().filter(|ip| !ip.is_empty()) {
            self.push_param("dst_ip LIKE ?", format!("%{}%", ip));
        }
        if let Some(flow_id) = params.flow_id {
            self.push_param("flow_id = ?", flow_id as i64);
        }
        self
    }

    fn push_summary_filters(mut self, params: &ConnectHistoryQueryParams) -> Self {
        if let Some(port) = params.port_start {
            self.push_param("src_port = ?", port as i64);
        }
        if let Some(port) = params.port_end {
            self.push_param("dst_port = ?", port as i64);
        }
        if let Some(l3_proto) = params.l3_proto {
            self.push_param("l3_proto = ?", l3_proto as i64);
        }
        if let Some(l4_proto) = params.l4_proto {
            self.push_param("l4_proto = ?", l4_proto as i64);
        }
        if let Some(status) = params.status {
            self.push_param("status = ?", status as i64);
        }
        if let Some(gress) = params.gress {
            self.push_param("gress = ?", gress as i64);
        }
        self
    }

    fn where_stmt(&self) -> String {
        if self.clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", self.clauses.join(" AND "))
        }
    }

    fn param_refs(&self) -> Vec<&dyn duckdb::ToSql> {
        self.params.iter().map(|param| param.as_ref()).collect()
    }
}

fn parse_ip_or_default(ip: String) -> std::net::IpAddr {
    ip.parse().unwrap_or_else(|_| "0.0.0.0".parse().expect("valid fallback ip"))
}

pub fn query_metric_by_key(
    conn: &Connection,
    key: &ConnectKey,
    resolution: MetricResolution,
) -> Vec<ConnectMetricPoint> {
    let table = match resolution {
        MetricResolution::Second => return Vec::new(),
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
        Ok(stmt) => stmt,
        Err(error) => {
            tracing::error!(
                "Failed to prepare query_metric_by_key SQL: {}, error: {}",
                stmt_str,
                error
            );
            return Vec::new();
        }
    };

    let rows = stmt.query_map(params![key.create_time as i64, key.cpu_id as i64], |row| {
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
        Ok(rows) => rows.filter_map(Result::ok).collect(),
        Err(error) => {
            tracing::error!("Failed to execute query_metric_by_key: {}", error);
            Vec::new()
        }
    }
}

pub fn query_historical_summaries_complex(
    conn: &Connection,
    params: ConnectHistoryQueryParams,
) -> Vec<ConnectHistoryStatus> {
    let builder = ConnectHistoryWhereBuilder::new()
        .push_common_filters(&params)
        .push_summary_filters(&params);
    let where_stmt = builder.where_stmt();
    let param_refs = builder.param_refs();

    let sort_col = match params.sort_key.unwrap_or_default() {
        ConnectSortKey::Port => "src_port",
        ConnectSortKey::Ingress => "total_ingress_bytes",
        ConnectSortKey::Egress => "total_egress_bytes",
        ConnectSortKey::Time => "last_report_time",
        ConnectSortKey::Duration => {
            "(CAST(last_report_time AS BIGINT) - CAST(create_time_ms AS BIGINT))"
        }
    };
    let sort_order = match params.sort_order.unwrap_or_default() {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };
    let limit_clause =
        if let Some(limit) = params.limit { format!("LIMIT {}", limit) } else { String::new() };

    let stmt_str = format!(
        "
        SELECT
            create_time, cpu_id, src_ip, dst_ip, src_port, dst_port, l4_proto, l3_proto, flow_id, trace_id,
            total_ingress_bytes, total_egress_bytes, total_ingress_pkts, total_egress_pkts, last_report_time, status, create_time_ms, gress
        FROM conn_summaries
        {}
        ORDER BY {} {}
        {}
    ",
        where_stmt, sort_col, sort_order, limit_clause
    );

    let mut stmt = match conn.prepare(&stmt_str) {
        Ok(stmt) => stmt,
        Err(error) => {
            tracing::error!("Failed to prepare SQL: {}, error: {}", stmt_str, error);
            return Vec::new();
        }
    };

    let rows = stmt.query_map(&param_refs[..], |row| {
        Ok(ConnectHistoryStatus {
            key: ConnectKey {
                create_time: row.get::<_, i64>(0)? as u64,
                cpu_id: row.get::<_, i64>(1)? as u32,
            },
            src_ip: parse_ip_or_default(row.get::<_, String>(2)?),
            dst_ip: parse_ip_or_default(row.get::<_, String>(3)?),
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
            create_time_ms: row.get::<_, i64>(16)? as u64,
            gress: row.get::<_, Option<i64>>(17)?.unwrap_or(0) as u8,
        })
    });

    match rows {
        Ok(rows) => rows.filter_map(Result::ok).collect(),
        Err(error) => {
            tracing::error!("Failed to execute query: {}", error);
            Vec::new()
        }
    }
}

pub fn query_global_stats(conn: &Connection) -> duckdb::Result<ConnectGlobalStats> {
    let stmt = "
        SELECT
            COALESCE(SUM(total_ingress_bytes), 0),
            COALESCE(SUM(total_egress_bytes), 0),
            COALESCE(SUM(total_ingress_pkts), 0),
            COALESCE(SUM(total_egress_pkts), 0),
            COUNT(*)
        FROM conn_summaries
    ";

    let mut stmt = conn.prepare(stmt)?;

    stmt.query_row([], |row| {
        Ok(ConnectGlobalStats {
            total_ingress_bytes: row.get::<_, i64>(0)? as u64,
            total_egress_bytes: row.get::<_, i64>(1)? as u64,
            total_ingress_pkts: row.get::<_, i64>(2)? as u64,
            total_egress_pkts: row.get::<_, i64>(3)? as u64,
            total_connect_count: row.get::<_, i64>(4)? as u64,
            last_calculate_time: landscape_common::utils::time::get_current_time_ms()
                .unwrap_or_default(),
        })
    })
}

pub fn query_connection_ip_history(
    conn: &Connection,
    params: ConnectHistoryQueryParams,
    is_src: bool,
) -> Vec<landscape_common::metric::connect::IpHistoryStat> {
    let builder = ConnectHistoryWhereBuilder::new().push_common_filters(&params);
    let where_stmt = builder.where_stmt();
    let param_refs = builder.param_refs();
    let column = if is_src { "src_ip" } else { "dst_ip" };

    let sort_col = match params.sort_key.unwrap_or(ConnectSortKey::Ingress) {
        ConnectSortKey::Ingress => "2",
        ConnectSortKey::Egress => "3",
        _ => "2",
    };
    let sort_order = match params.sort_order.unwrap_or(SortOrder::Desc) {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };
    let limit = params.limit.unwrap_or(10);

    let stmt_str = format!(
        "
        SELECT
            {},
            SUM(total_ingress_bytes), SUM(total_egress_bytes),
            SUM(total_ingress_pkts), SUM(total_egress_pkts),
            COUNT(*)
        FROM conn_summaries
        {}
        GROUP BY 1
        ORDER BY {} {}
        LIMIT {}
    ",
        column, where_stmt, sort_col, sort_order, limit
    );

    let mut stmt = match conn.prepare(&stmt_str) {
        Ok(stmt) => stmt,
        Err(error) => {
            tracing::error!("Failed to prepare IP history SQL: {}, error: {}", stmt_str, error);
            return Vec::new();
        }
    };

    let rows = stmt.query_map(&param_refs[..], |row| {
        Ok(landscape_common::metric::connect::IpHistoryStat {
            ip: parse_ip_or_default(row.get::<_, String>(0)?),
            flow_id: 0,
            total_ingress_bytes: row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
            total_egress_bytes: row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u64,
            total_ingress_pkts: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as u64,
            total_egress_pkts: row.get::<_, Option<i64>>(4)?.unwrap_or(0) as u64,
            connect_count: row.get::<_, i64>(5)? as u32,
        })
    });

    match rows {
        Ok(rows) => rows.filter_map(Result::ok).collect(),
        Err(error) => {
            tracing::error!("Failed to execute IP history query: {}", error);
            Vec::new()
        }
    }
}
