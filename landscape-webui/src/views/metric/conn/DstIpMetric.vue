<script setup lang="ts">
import { computed } from "vue";
import { useMetricStore } from "@/stores/status_metric";
import IpStatsList from "@/components/metric/connect/IpStatsList.vue";

const metricStore = useMetricStore();

const stats = computed(() => metricStore.dst_ip_stats);
</script>

<template>
  <IpStatsList
    :stats="stats"
    title="目的 IP 实时统计"
    ip-label="目的 IP 地址"
  />
</template>
