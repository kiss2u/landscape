<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { get_connect_metric_info } from "@/api/metric";
import {
  ConnectKey,
  ConnectMetric,
} from "landscape-types/common/metric/connect";
import BaseConnectChart from "./BaseConnectChart.vue";

interface Props {
  conn: ConnectKey;
}

const props = defineProps<Props>();
const chartData = ref<ConnectMetric[]>([]);
const interval = ref<any>(null);

async function fetchData() {
  chartData.value = await get_connect_metric_info(props.conn);
}

onMounted(() => {
  fetchData();
  interval.value = setInterval(fetchData, 5000);
});

onUnmounted(() => {
  if (interval.value) {
    clearInterval(interval.value);
  }
});
</script>

<template>
  <BaseConnectChart :chart-data="chartData" mode="delta" />
</template>
