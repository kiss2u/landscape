<script setup lang="ts">
import { get_connect_metric_info } from "@/api/metric";
import { useMetricStore } from "@/stores/status_metric";
import { useFrontEndStore } from "@/stores/front_end_config";
import {
  ConnectKey,
  ConnectMetric,
} from "landscape-types/common/metric/connect";
import { ApexOptions } from "apexcharts";
import { computed, ref } from "vue";
import VueApexCharts from "vue3-apexcharts";

const metricStore = useMetricStore();
const frontEndStore = useFrontEndStore();

interface Props {
  conn: ConnectKey | null;
}

const props = defineProps<Props>();

const chart = ref<ConnectMetric[]>([]);

const show = defineModel("show");

const title = computed(() => {
  if (props.conn == null) {
    return "";
  } else {
    return frontEndStore.MASK_INFO(
      `${props.conn.src_ip}:${props.conn.src_port} => ${props.conn.dst_ip}:${props.conn.dst_port}`,
    );
  }
});
const interval_number = ref();
async function start_fetch_data() {
  if (props.conn !== null && !interval_number.value) {
    metricStore.SET_ENABLE(false);
    chart.value = await get_connect_metric_info(props.conn);

    interval_number.value = setInterval(async () => {
      if (props.conn !== null) {
        chart.value = await get_connect_metric_info(props.conn);
      }
    }, 5000);
  }
}
async function stop_fetch_data() {
  if (props.conn !== null) {
    metricStore.SET_ENABLE(true);
    clearInterval(interval_number.value);
    interval_number.value = null;
    chart.value = [];
  }
}

const categories = computed(() =>
  chart.value.map((m) => {
    const d = new Date(m.report_time);
    return `${d.getHours()}:${d.getMinutes().toString().padStart(2, "0")}:${d
      .getSeconds()
      .toString()
      .padStart(2, "0")}`;
  }),
);

// Divide values by 5 to get per-second rates
// Use raw cumulative values from the database
const bytesSeries = computed(() => [
  {
    name: "入站总量 (Ingress)",
    data: chart.value.map((m) => m.ingress_bytes),
  },
  {
    name: "出站总量 (Egress)",
    data: chart.value.map((m) => m.egress_bytes),
  },
]);

const packetsSeries = computed(() => [
  {
    name: "入站封包 (Ingress)",
    data: chart.value.map((m) => m.ingress_packets),
  },
  {
    name: "出站封包 (Egress)",
    data: chart.value.map((m) => m.egress_packets),
  },
]);

const baseOptions = computed<ApexOptions>(() => ({
  chart: {
    id: "network-traffic",
    background: "transparent",
    toolbar: { show: false },
  },
  theme: {
    mode: "dark",
  },
  stroke: {
    curve: "smooth",
  },
  tooltip: {
    shared: true,
    intersect: false,
  },
  xaxis: {
    categories: categories.value,
    tickAmount: 10,
    title: { text: "时间" },
    labels: {
      showDuplicates: false,
      hideOverlappingLabels: true,
    },
  },
  legend: {
    position: "top",
  },
}));

function formatVolume(value: number): string {
  if (value === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const k = 1024;
  const i = Math.floor(Math.log(value) / Math.log(k));
  const scaled = value / Math.pow(k, i);
  return `${scaled.toFixed(1)} ${units[i]}`;
}

const bytesOptions = computed<ApexOptions>(() => ({
  ...baseOptions.value,
  yaxis: {
    title: { text: "数据总量" },
    labels: {
      formatter: formatVolume,
    },
  },
}));

const packetsOptions = computed<ApexOptions>(() => ({
  ...baseOptions.value,
  yaxis: {
    title: { text: "封包总量" },
    labels: {
      formatter: (value: number) => `${Math.round(value)} pkt`,
    },
  },
}));
</script>

<template>
  <n-drawer
    v-model:show="show"
    width="80%"
    placement="right"
    @after-enter="start_fetch_data"
    @after-leave="stop_fetch_data"
  >
    <n-drawer-content closable :title="title">
      <n-flex vertical>
        <VueApexCharts
          style="flex: 1"
          type="line"
          height="300"
          :options="bytesOptions"
          :series="bytesSeries"
        />
        <VueApexCharts
          style="flex: 1"
          type="line"
          height="300"
          :options="packetsOptions"
          :series="packetsSeries"
        />
      </n-flex>
    </n-drawer-content>
  </n-drawer>
</template>
