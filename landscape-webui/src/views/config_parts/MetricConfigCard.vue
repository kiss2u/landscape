<script setup lang="ts">
import { useMetricConfigStore } from "@/stores/metric_config";
import { useMessage } from "naive-ui";

const metricStore = useMetricConfigStore();
const message = useMessage();

async function handleSaveMetric() {
  try {
    await metricStore.saveMetricConfig();
    message.success("指标配置保存成功");
  } catch (e: any) {
    if (e.response?.status === 409) {
      message.error("指标配置冲突，请刷新后重试");
    } else {
      message.error("保存失败: " + e.message);
    }
  }
}
</script>

<template>
  <n-card title="指标监控配置" segmented id="metric-config">
    <template #header-extra>
      <n-button type="primary" @click="handleSaveMetric">
        保存指标配置
      </n-button>
    </template>
    <n-form label-placement="left" label-width="120">
      <n-form-item label="数据保存天数">
        <n-input-number
          v-model:value="metricStore.retentionDays"
          :min="1"
          :max="365"
          placeholder="7"
          style="width: 200px"
        />
        <template #feedback> 历史连接和流量指标数据的保存期限（天） </template>
      </n-form-item>
      <n-form-item label="刷新间隔 (秒)">
        <n-input-number
          v-model:value="metricStore.flushIntervalSecs"
          :min="1"
          :max="3600"
          placeholder="5"
          style="width: 200px"
        />
        <template #feedback> 指标刷新到存储的间隔时间 </template>
      </n-form-item>
      <n-form-item label="批处理大小">
        <n-input-number
          v-model:value="metricStore.batchSize"
          :min="100"
          :max="10000"
          placeholder="2000"
          style="width: 200px"
        />
        <template #feedback> 每次写入存储的最大记录数 </template>
      </n-form-item>
      <n-form-item label="最大内存限制 (MB)">
        <n-input-number
          v-model:value="metricStore.maxMemory"
          :min="32"
          :max="4096"
          placeholder="128"
          style="width: 200px"
        />
        <template #feedback> 指标缓存允许占用的最大内存 </template>
      </n-form-item>
      <n-form-item label="并发处理线程">
        <n-input-number
          v-model:value="metricStore.maxThreads"
          :min="1"
          :max="16"
          placeholder="1"
          style="width: 200px"
        />
        <template #feedback> 用于处理指标数据的后台线程数 </template>
      </n-form-item>
    </n-form>
  </n-card>
</template>
