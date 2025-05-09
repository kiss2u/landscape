<script setup lang="ts">
import { SingleConnectMetric } from "@/rust_bindings/common/metric";
import { ApexOptions } from "apexcharts";
import { computed, ref } from "vue";
import VueApexCharts from "vue3-apexcharts";

interface Props {
  chart: SingleConnectMetric;
}

const props = defineProps<Props>();

const show = computed(() => props.chart.metrics.length > 1);
const categories = computed(() =>
  props.chart.metrics.map((m) => {
    const d = new Date(m.time / 1_000_000);
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
    data: props.chart.metrics.map((m) => m.ingress_bytes / 5),
  },
  {
    name: "Egress Bytes/s",
    data: props.chart.metrics.map((m) => m.egress_bytes / 5),
  },
]);
const packetsSeries = computed(() => [
  {
    name: "Ingress Packets/s",
    data: props.chart.metrics.map((m) => m.ingress_packets / 5),
  },
  {
    name: "Egress Packets/s",
    data: props.chart.metrics.map((m) => m.egress_packets / 5),
  },
]);

const baseOptions: ApexOptions = {
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
    title: { text: "Time" },
    labels: {
      showDuplicates: false,
      hideOverlappingLabels: true,
    },
  },
  legend: {
    position: "top",
  },
};

function formatRate(value: number): string {
  if (value === 0) return "0 B/s";
  const units = ["B/s", "KB/s", "MB/s", "GB/s", "TB/s"];
  const k = 1024;
  const i = Math.floor(Math.log(value) / Math.log(k));
  const scaled = value / Math.pow(k, i);
  return `${scaled.toFixed(1)} ${units[i]}`;
}

const bytesOptions: ApexOptions = {
  ...baseOptions,
  yaxis: {
    title: { text: "Bytes/s" },
    labels: {
      formatter: formatRate,
    },
  },
};

const packetsOptions: ApexOptions = {
  ...baseOptions,
  yaxis: {
    title: { text: "Packets/s" },
    labels: {
      formatter: (value: number) => `${value.toFixed(1)} pkt/s`,
    },
  },
};
</script>

<template>
  <n-flex v-if="show">
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
</template>
