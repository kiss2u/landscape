<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useMetricStore } from "@/stores/status_metric";
import { formatRate } from "@/lib/util";
import { useThemeVars } from "naive-ui";
import IpStatsList from "@/components/metric/connect/IpStatsList.vue";
import ConnectViewSwitcher from "@/components/metric/connect/ConnectViewSwitcher.vue";
import FlowSelect from "@/components/flow/FlowSelect.vue";

const metricStore = useMetricStore();
const themeVars = useThemeVars();

const ipFilter = ref("");
const flowFilter = ref<number | null>(null);

const stats = computed(() => {
  if (!ipFilter.value && flowFilter.value === null) {
    return metricStore.dst_ip_stats;
  }

  const connections = metricStore.firewall_info || [];
  const aggregatedMap = new Map<string, any>();

  connections.forEach((conn) => {
    if (
      ipFilter.value &&
      !conn.dst_ip.toLowerCase().includes(ipFilter.value.toLowerCase())
    )
      return;
    if (flowFilter.value !== null && conn.flow_id !== flowFilter.value) return;

    const key = `${conn.dst_ip}_${conn.flow_id}`;
    if (!aggregatedMap.has(key)) {
      aggregatedMap.set(key, {
        ip: conn.dst_ip,
        flow_id: conn.flow_id,
        stats: {
          active_conns: 0,
          ingress_bps: 0,
          egress_bps: 0,
          ingress_pps: 0,
          egress_pps: 0,
        },
      });
    }

    const item = aggregatedMap.get(key);
    item.stats.active_conns += 1;
    item.stats.ingress_bps += conn.ingress_bps || 0;
    item.stats.egress_bps += conn.egress_bps || 0;
    item.stats.ingress_pps += conn.ingress_pps || 0;
    item.stats.egress_pps += conn.egress_pps || 0;
  });

  return Array.from(aggregatedMap.values());
});

// 系统全局汇总
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

    <!-- 过滤器工具栏 -->
    <n-flex
      align="center"
      :wrap="true"
      style="margin-bottom: 12px"
      size="small"
    >
      <n-input
        v-model:value="ipFilter"
        placeholder="搜索目的 IP"
        clearable
        style="width: 200px"
      />
      <FlowSelect v-model="flowFilter" width="150px" />
      <n-button @click="metricStore.UPDATE_INFO()" :loading="false"
        >刷新采样</n-button
      >
    </n-flex>

    <IpStatsList
      :stats="stats"
      title="目的 IP 实时统计"
      ip-label="目的 IP 地址"
      @search:ip="(ip) => (ipFilter = ip)"
    />
  </n-flex>
</template>
