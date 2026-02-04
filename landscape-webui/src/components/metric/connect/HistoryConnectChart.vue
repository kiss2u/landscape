<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { get_connect_metric_info } from "@/api/metric";
import {
  ConnectKey,
  ConnectMetric,
  MetricResolution,
} from "landscape-types/common/metric/connect";
import BaseConnectChart from "./BaseConnectChart.vue";

interface Props {
  conn: ConnectKey;
}

const props = defineProps<Props>();
const chartData = ref<ConnectMetric[]>([]);
const resolution = ref<MetricResolution>("second");
const loading = ref(false);

const resolutionOptions = [
  { label: "秒级 (实时)", value: "second" },
  { label: "小时级 (30天保留)", value: "hour" },
  { label: "天级 (180天保留)", value: "day" },
];

async function fetchData() {
  loading.value = true;
  try {
    chartData.value = await get_connect_metric_info(
      props.conn,
      resolution.value,
    );
  } finally {
    loading.value = false;
  }
}

watch(resolution, fetchData);

onMounted(fetchData);
</script>

<template>
  <n-flex vertical>
    <n-flex justify="end">
      <n-radio-group v-model:value="resolution" size="small">
        <n-radio-button
          v-for="opt in resolutionOptions"
          :key="opt.value"
          :value="opt.value"
        >
          {{ opt.label }}
        </n-radio-button>
      </n-radio-group>
    </n-flex>
    <n-spin :show="loading">
      <BaseConnectChart :chart-data="chartData" mode="cumulative" />
    </n-spin>
  </n-flex>
</template>
