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
import { useThemeVars } from "naive-ui";

const metricStore = useMetricStore();
const frontEndStore = useFrontEndStore();
const themeVars = useThemeVars();

interface Props {
  conn: ConnectKey | null;
  mode?: "cumulative" | "delta"; // cumulative: 显示累计总量, delta: 显示增量
}

const props = withDefaults(defineProps<Props>(), {
  mode: "delta", // 默认显示增量（兼容实时连接）
});

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

// 数据降采样：当数据点过多时，抽样显示以提高性能和可读性
function downsampleData(data: number[], maxPoints: number = 100): { data: number[]; indices: number[] } {
  if (data.length <= maxPoints) {
    return { data, indices: data.map((_, i) => i) };
  }
  
  const step = Math.ceil(data.length / maxPoints);
  const sampledData: number[] = [];
  const sampledIndices: number[] = [];
  
  for (let i = 0; i < data.length; i += step) {
    sampledData.push(data[i]);
    sampledIndices.push(i);
  }
  
  // 确保包含最后一个点
  if (sampledIndices[sampledIndices.length - 1] !== data.length - 1) {
    sampledData.push(data[data.length - 1]);
    sampledIndices.push(data.length - 1);
  }
  
  return { data: sampledData, indices: sampledIndices };
}

// 降采样后的数据和索引
const sampledIndices = computed(() => {
  const ingressData = chart.value.map((m) => m.ingress_bytes);
  return downsampleData(ingressData).indices;
});

const categories = computed(() =>
  sampledIndices.value.map((idx) => {
    const m = chart.value[idx];
    const d = new Date(m.report_time);
    return `${d.getHours()}:${d.getMinutes().toString().padStart(2, "0")}:${d
      .getSeconds()
      .toString()
      .padStart(2, "0")}`;
  }),
);

// 计算增量数据（用于实时连接）
function calculateDeltas(values: number[]): number[] {
  if (values.length === 0) return [];
  const deltas = [0]; // 第一个点的增量为0
  for (let i = 1; i < values.length; i++) {
    deltas.push(Math.max(0, values[i] - values[i - 1]));
  }
  return deltas;
}


// 根据模式返回累计值或增量值
const bytesSeries = computed(() => {
  const ingressData = chart.value.map((m) => m.ingress_bytes);
  const egressData = chart.value.map((m) => m.egress_bytes);

  // 先处理模式（累计或增量）
  const processedIngress = props.mode === "cumulative" ? ingressData : calculateDeltas(ingressData);
  const processedEgress = props.mode === "cumulative" ? egressData : calculateDeltas(egressData);

  // 然后降采样
  const sampledIngress = sampledIndices.value.map((idx) => processedIngress[idx]);
  const sampledEgress = sampledIndices.value.map((idx) => processedEgress[idx]);

  return [
    {
      name: props.mode === "cumulative" ? "入站总量 (Ingress)" : "入站增量 (Ingress)",
      data: sampledIngress,
    },
    {
      name: props.mode === "cumulative" ? "出站总量 (Egress)" : "出站增量 (Egress)",
      data: sampledEgress,
    },
  ];
});

const packetsSeries = computed(() => {
  const ingressData = chart.value.map((m) => m.ingress_packets);
  const egressData = chart.value.map((m) => m.egress_packets);

  // 先处理模式（累计或增量）
  const processedIngress = props.mode === "cumulative" ? ingressData : calculateDeltas(ingressData);
  const processedEgress = props.mode === "cumulative" ? egressData : calculateDeltas(egressData);

  // 然后降采样
  const sampledIngress = sampledIndices.value.map((idx) => processedIngress[idx]);
  const sampledEgress = sampledIndices.value.map((idx) => processedEgress[idx]);

  return [
    {
      name: props.mode === "cumulative" ? "入站封包 (Ingress)" : "入站封包增量 (Ingress)",
      data: sampledIngress,
    },
    {
      name: props.mode === "cumulative" ? "出站封包 (Egress)" : "出站封包增量 (Egress)",
      data: sampledEgress,
    },
  ];
});

const baseOptions = computed<ApexOptions>(() => ({
  chart: {
    id: "network-traffic",
    background: "transparent",
    toolbar: {
      show: true,
      tools: {
        download: false,
        selection: true,
        zoom: true,
        zoomin: true,
        zoomout: true,
        pan: true,
        reset: true,
      },
      autoSelected: "zoom",
    },
    zoom: {
      enabled: true,
      type: "x",
      autoScaleYaxis: true,
    },
    selection: {
      enabled: true,
      type: "x",
      fill: {
        color: themeVars.value.primaryColor,
        opacity: 0.1,
      },
      stroke: {
        width: 1,
        color: themeVars.value.primaryColor,
        opacity: 0.4,
      },
    },
  },
  theme: {
    mode: "dark",
  },
  colors: [themeVars.value.successColor, themeVars.value.infoColor], // 入站(绿色), 出站(蓝色)
  stroke: {
    curve: "smooth",
    width: 2,
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
