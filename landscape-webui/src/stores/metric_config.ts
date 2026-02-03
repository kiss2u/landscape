import { defineStore } from "pinia";
import { ref } from "vue";
import type { LandscapeMetricConfig } from "landscape-types/common/config";
import { get_metric_config_edit, update_metric_config } from "@/api/sys/config";

export const useMetricConfigStore = defineStore("metric_config", () => {
  const retentionDays = ref<number | undefined>(undefined);
  const batchSize = ref<number | undefined>(undefined);
  const flushIntervalSecs = ref<number | undefined>(undefined);
  const maxMemory = ref<number | undefined>(undefined);
  const maxThreads = ref<number | undefined>(undefined);
  const expectedHash = ref<string>("");

  async function loadMetricConfig() {
    const { metric, hash } = await get_metric_config_edit();
    retentionDays.value = metric.retention_days
      ? Number(metric.retention_days)
      : undefined;
    batchSize.value = metric.batch_size;
    flushIntervalSecs.value = metric.flush_interval_secs
      ? Number(metric.flush_interval_secs)
      : undefined;
    maxMemory.value = metric.max_memory;
    maxThreads.value = metric.max_threads;
    expectedHash.value = hash;
  }

  async function saveMetricConfig() {
    const new_metric: LandscapeMetricConfig = {
      retention_days:
        retentionDays.value !== undefined
          ? BigInt(retentionDays.value)
          : undefined,
      batch_size: batchSize.value || undefined,
      flush_interval_secs:
        flushIntervalSecs.value !== undefined
          ? BigInt(flushIntervalSecs.value)
          : undefined,
      max_memory: maxMemory.value || undefined,
      max_threads: maxThreads.value || undefined,
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
    retentionDays,
    batchSize,
    flushIntervalSecs,
    maxMemory,
    maxThreads,
    expectedHash,
    loadMetricConfig,
    saveMetricConfig,
  };
});
