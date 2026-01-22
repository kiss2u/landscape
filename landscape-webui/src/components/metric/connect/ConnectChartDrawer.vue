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
      `${props.conn.src_ip}:${props.conn.src_port} => ${props.conn.dst_ip}:${props.conn.dst_port}`
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
  })
);

// Divide values by 5 to get per-second rates
const bytesSeries = computed(() => [
  {
    name: "Ingress Bytes/s",
    data: chart.value.map((m) => m.ingress_bytes / 5),
  },
  {
    name: "Egress Bytes/s",
    data: chart.value.map((m) => m.egress_bytes / 5),
  },
]);

const packetsSeries = computed(() => [
  {
    name: "Ingress Packets/s",
    data: chart.value.map((m) => m.ingress_packets / 5),
  },
  {
    name: "Egress Packets/s",
    data: chart.value.map((m) => m.egress_packets / 5),
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
    categories: categories.value, // ✅ 响应式绑定
    tickAmount: 10,
    title: { text: "Time" },
    labels: {
      showDuplicates: false,
      hideOverlappingLabels: true,
    },
  },
  legend: {
    position: "top",
  },
}));

function formatRate(value: number): string {
  if (value === 0) return "0 B/s";
  const units = ["B/s", "KB/s", "MB/s", "GB/s", "TB/s"];
  const k = 1024;
  const i = Math.floor(Math.log(value) / Math.log(k));
  const scaled = value / Math.pow(k, i);
  return `${scaled.toFixed(1)} ${units[i]}`;
}

const bytesOptions = computed<ApexOptions>(() => ({
  ...baseOptions.value,
  yaxis: {
    title: { text: "Bytes/s" },
    labels: {
      formatter: formatRate,
    },
  },
}));

const packetsOptions = computed<ApexOptions>(() => ({
  ...baseOptions.value,
  yaxis: {
    title: { text: "Packets/s" },
    labels: {
      formatter: (value: number) => `${value.toFixed(1)} pkt/s`,
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
