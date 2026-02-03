<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import { useMetricStore } from "@/stores/status_metric";
import { formatRate, formatPackets } from "@/lib/util";
import { useThemeVars } from "naive-ui";
import IpStatsList from "@/components/metric/connect/IpStatsList.vue";
import ConnectViewSwitcher from "@/components/metric/connect/ConnectViewSwitcher.vue";

const metricStore = useMetricStore();
const themeVars = useThemeVars();

const stats = computed(() => metricStore.src_ip_stats);

// 系统全局汇总 (Duplicate logic from LiveMetric for consistent header)
const systemStats = computed(() => {
  const stats = {
    ingressBps: 0,
    egressBps: 0,
    ingressPps: 0,
    egressPps: 0,
    count: 0,
  };
  if (metricStore.firewall_info) {
    metricStore.firewall_info.forEach((item) => {
      stats.ingressBps += item.ingress_bps || 0;
      stats.egressBps += item.egress_bps || 0;
      stats.ingressPps += item.ingress_pps || 0;
      stats.egressPps += item.egress_pps || 0;
      stats.count++;
    });
  }
  return stats;
});

onMounted(async () => {
  metricStore.SET_ENABLE(true);
  await metricStore.UPDATE_INFO();

  onUnmounted(() => {
    metricStore.SET_ENABLE(false);
  });
});
</script>

<template>
  <n-flex vertical style="flex: 1; overflow: hidden">
    <!-- 系统全局活跃连接统计 -->
    <n-card
      size="small"
      :bordered="false"
      style="margin-bottom: 12px; background-color: #f9f9f910"
    >
      <n-flex align="center" justify="space-between">
        <ConnectViewSwitcher />

        <n-flex align="center" size="large">
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">总活跃连接:</span>
            <span style="font-weight: bold">{{ systemStats.count }}</span>
          </n-flex>
          <n-divider vertical />
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">总上行:</span>
            <span :style="{ fontWeight: 'bold', color: themeVars.infoColor }">{{
              formatRate(systemStats.egressBps)
            }}</span>
          </n-flex>
          <n-divider vertical />
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">总下行:</span>
            <span
              :style="{ fontWeight: 'bold', color: themeVars.successColor }"
              >{{ formatRate(systemStats.ingressBps) }}</span
            >
          </n-flex>
        </n-flex>
      </n-flex>
    </n-card>

    <IpStatsList :stats="stats" title="源 IP 实时统计" ip-label="源 IP 地址" />
  </n-flex>
</template>
