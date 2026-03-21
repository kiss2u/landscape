export default {
  directory: "Config Directory",
  ui_title: "System Preference",
  dns_title: "Global DNS Config",
  metric_title: "Metric Monitoring Config",
  backup_title: "Backup & Export",

  save_ui: "Save Settings",
  save_dns: "Save DNS Config",
  save_metric: "Save Metric Config",

  metric_enabled: "Enable Metric Service",
  metric_enabled_desc:
    "When disabled, new connection and DNS metric data will no longer be collected",

  language: "Language",
  theme: "Theme",
  theme_placeholder: "Light mode is under development",
  timezone: "System Timezone",
  timezone_placeholder: "Select or search, e.g.: Asia/Shanghai",

  cache_capacity: "Cache Capacity",
  cache_capacity_desc: "Maximum records allowed in DNS cache",
  cache_ttl: "Cache TTL (s)",
  cache_ttl_desc: "Maximum retention time for DNS cache records",
  cache_negative_ttl: "Negative Cache TTL (s)",
  cache_negative_ttl_desc:
    "Retention time for negative (NXDOMAIN/NODATA) DNS records",

  conn_retention_mins: "Raw Data Retention (Mins)",
  conn_retention_mins_desc:
    "Retention period for raw connection metrics (seconds interval) in minutes",
  connect_second_window_mins: "Second Window (Mins)",
  connect_second_window_mins_desc:
    "In-memory retention window for second-level connection charts",
  conn_retention_minute_days: "Aggregated Data (Minute)",
  conn_retention_minute_days_desc:
    "Retention period for minute aggregated connection metrics in days",
  conn_retention_hour_days: "Aggregated Data (Hour)",
  conn_retention_hour_days_desc:
    "Retention period for hourly aggregated connection metrics in days",
  conn_retention_day_days: "Aggregated Data (Day)",
  conn_retention_day_days_desc:
    "Retention period for daily aggregated connection metrics in days",
  dns_retention_days: "DNS Retention (Days)",
  dns_retention_days_desc:
    "Retention period for DNS query logs and metrics in days",

  performance_settings: "Performance & Storage Settings",
  write_batch_size: "Write Batch Size",
  write_batch_size_desc: "Maximum record count for metrics flush to disk",
  write_flush_interval: "Commit Interval (s)",
  write_flush_interval_desc: "Maximum seconds before forcing a database commit",
  db_max_memory: "DB Max Memory (MB)",
  db_max_memory_desc: "Maximum memory allowed for DuckDB (metrics storage)",
  db_max_threads: "DB Max Threads",
  db_max_threads_desc: "Maximum background threads for database operations",

  maintenance_settings: "Maintenance & Aggregation Tasks",
  cleanup_interval: "Cleanup Interval (s)",
  cleanup_interval_desc: "Interval between history cleanup tasks",
  cleanup_budget: "Cleanup Budget (ms)",
  cleanup_budget_desc: "Maximum milliseconds allowed per cleanup task",
  cleanup_slice_window: "Cleanup Slice Window (s)",
  cleanup_slice_window_desc:
    "Time granularity for each cleanup transaction slice",
  aggregate_interval: "Aggregate Interval (s)",
  aggregate_interval_desc:
    "Interval between global statistics aggregation tasks",

  backup_desc:
    "You can export all current router configurations (including DNS, firewall, network interfaces, etc.) as an init file for quick recovery or migration.",
  export_init: "Export all current configurations as Init file",

  load_failed: "Failed to load configuration",
  save_success: "Save successful",
  save_failed: "Save failed",
  conflict: "Configuration conflict, please refresh and try again",

  chinese_simplified: "Simplified Chinese",
  dark_mode: "Dark",
  light_mode: "Light",
  welcome: "Welcome, {username}",
};
