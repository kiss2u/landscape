export default {
  directory: "Config Directory",
  ui_title: "System Preference",
  dns_title: "Global DNS Config",
  metric_title: "Metric Monitoring Config",
  backup_title: "Backup & Export",

  save_ui: "Save Settings",
  save_dns: "Save DNS Config",
  save_metric: "Save Metric Config",

  language: "Language",
  theme: "Theme",
  theme_placeholder: "Light mode is under development",
  timezone: "System Timezone",
  timezone_placeholder: "Select or search, e.g.: Asia/Shanghai",

  cache_capacity: "Cache Capacity",
  cache_capacity_desc: "Maximum records allowed in DNS cache",
  cache_ttl: "Cache TTL (s)",
  cache_ttl_desc: "Maximum retention time for DNS cache records",

  retention_days: "Retention Days",
  retention_days_desc:
    "Retention period for historical connection and traffic metrics (days)",
  flush_interval: "Flush Interval (s)",
  flush_interval_desc: "Interval for flushing metrics to storage",
  batch_size: "Batch Size",
  batch_size_desc: "Maximum number of records per write to storage",
  max_memory: "Max Memory (MB)",
  max_memory_desc: "Maximum memory allowed for metric cache",
  max_threads: "Max Threads",
  max_threads_desc: "Number of background threads for processing metric data",

  backup_desc:
    "You can export all current router configurations (including DNS, firewall, network interfaces, etc.) as an init file for quick recovery or migration.",
  export_init: "Export all current configurations as Init file",

  load_failed: "Failed to load configuration",
  save_success: "Save successful",
  save_failed: "Save failed",
  conflict: "Configuration conflict, please refresh and try again",
};
