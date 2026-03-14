import { defineStore } from "pinia";
import { ref } from "vue";
import type { LandscapeMetricConfig } from "@landscape-router/types/api/schemas";
import { get_metric_config_edit, update_metric_config } from "@/api/sys/config";

export const useMetricConfigStore = defineStore("metric_config", () => {
  const enabled = ref<boolean>(true);
  const rawRetentionMinutes = ref<number | undefined>(undefined);
  const rollup1mRetentionDays = ref<number | undefined>(undefined);
  const rollup1hRetentionDays = ref<number | undefined>(undefined);
  const rollup1dRetentionDays = ref<number | undefined>(undefined);
  const dnsRetentionDays = ref<number | undefined>(undefined);
  const writeBatchSize = ref<number | undefined>(undefined);
  const writeFlushIntervalSecs = ref<number | undefined>(undefined);
  const dbMaxMemoryMb = ref<number | undefined>(undefined);
  const dbMaxThreads = ref<number | undefined>(undefined);
  const cleanupIntervalSecs = ref<number | undefined>(undefined);
  const cleanupTimeBudgetMs = ref<number | undefined>(undefined);
  const cleanupSliceWindowSecs = ref<number | undefined>(undefined);
  const aggregateIntervalSecs = ref<number | undefined>(undefined);
  const expectedHash = ref<string>("");

  async function loadMetricConfig() {
    const { metric, hash } = await get_metric_config_edit();
    enabled.value = metric.enable ?? true;
    rawRetentionMinutes.value = metric.raw_retention_minutes ?? undefined;
    rollup1mRetentionDays.value = metric.rollup_1m_retention_days ?? undefined;
    rollup1hRetentionDays.value = metric.rollup_1h_retention_days ?? undefined;
    rollup1dRetentionDays.value = metric.rollup_1d_retention_days ?? undefined;
    dnsRetentionDays.value = metric.dns_retention_days ?? undefined;
    writeBatchSize.value = metric.write_batch_size ?? undefined;
    writeFlushIntervalSecs.value =
      metric.write_flush_interval_secs ?? undefined;
    dbMaxMemoryMb.value = metric.db_max_memory_mb ?? undefined;
    dbMaxThreads.value = metric.db_max_threads ?? undefined;
    cleanupIntervalSecs.value = metric.cleanup_interval_secs ?? undefined;
    cleanupTimeBudgetMs.value = metric.cleanup_time_budget_ms ?? undefined;
    cleanupSliceWindowSecs.value =
      metric.cleanup_slice_window_secs ?? undefined;
    aggregateIntervalSecs.value = metric.aggregate_interval_secs ?? undefined;
    expectedHash.value = hash;
  }

  async function saveMetricConfig() {
    const new_metric: LandscapeMetricConfig = {
      enable: enabled.value,
      raw_retention_minutes: rawRetentionMinutes.value,
      rollup_1m_retention_days: rollup1mRetentionDays.value,
      rollup_1h_retention_days: rollup1hRetentionDays.value,
      rollup_1d_retention_days: rollup1dRetentionDays.value,
      dns_retention_days: dnsRetentionDays.value,
      write_batch_size: writeBatchSize.value,
      write_flush_interval_secs: writeFlushIntervalSecs.value,
      db_max_memory_mb: dbMaxMemoryMb.value,
      db_max_threads: dbMaxThreads.value,
      cleanup_interval_secs: cleanupIntervalSecs.value,
      cleanup_time_budget_ms: cleanupTimeBudgetMs.value,
      cleanup_slice_window_secs: cleanupSliceWindowSecs.value,
      aggregate_interval_secs: aggregateIntervalSecs.value,
    };
    await update_metric_config({
      new_metric,
      expected_hash: expectedHash.value,
    });

    // Refresh hash after save
    const { hash } = await get_metric_config_edit();
    expectedHash.value = hash;
  }

  return {
    enabled,
    rawRetentionMinutes,
    rollup1mRetentionDays,
    rollup1hRetentionDays,
    rollup1dRetentionDays,
    dnsRetentionDays,
    writeBatchSize,
    writeFlushIntervalSecs,
    dbMaxMemoryMb,
    dbMaxThreads,
    cleanupIntervalSecs,
    cleanupTimeBudgetMs,
    cleanupSliceWindowSecs,
    aggregateIntervalSecs,
    expectedHash,
    loadMetricConfig,
    saveMetricConfig,
  };
});
