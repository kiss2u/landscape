<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useMetricStore } from "@/stores/status_metric";
import { formatRate, formatPackets, formatSize } from "@/lib/util";
import { get_connect_global_stats } from "@/api/metric";
import type { ConnectGlobalStats } from "landscape-types/common/metric/connect";
import LiveMetric from "./metric/LiveMetric.vue";
import HistoryMetric from "./metric/HistoryMetric.vue";

const metricStore = useMetricStore();

// 视图模式: live (实时) | history (历史)
const viewMode = ref<"live" | "history" | any>("live");
const globalStats = ref<ConnectGlobalStats | null>(null);

let timer: any = null;

onMounted(async () => {
  metricStore.SET_ENABLE(true);
  await metricStore.UPDATE_INFO();

  // 获取一次历史全量统计
  get_connect_global_stats().then((res) => {
    globalStats.value = res;
  });

  timer = setInterval(async () => {
    if (viewMode.value === "live") {
      await metricStore.UPDATE_INFO();
    }
  }, 5000);
});

onUnmounted(() => {
  metricStore.SET_ENABLE(false);
  if (timer) clearInterval(timer);
});

// 系统全局汇总 (未过滤实时指标，用于顶部 Header)
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
</script>

<template>
  <n-flex style="flex: 1; overflow: hidden; margin-bottom: 10px" vertical>
    <!-- 全局页头：包含切换器和系统级实时统计 -->
    <n-flex align="center" justify="space-between" style="padding: 4px 0">
      <n-flex align="center" size="large">
        <n-radio-group v-model:value="viewMode" size="medium" type="button">
          <n-radio-button value="live">实时活跃连接</n-radio-button>
          <n-radio-button value="history">历史连接查询</n-radio-button>
        </n-radio-group>

        <n-tag
          v-if="viewMode === 'live'"
          :bordered="false"
          type="info"
          size="small"
          round
        >
          <template #icon>
            <div class="pulse-dot"></div>
          </template>
          5s 采样中
        </n-tag>

        <n-divider vertical />

        <!-- 动态面板：根据模式切换统计内容 -->
        <n-flex align="center" size="large" v-if="viewMode === 'live'">
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">活跃连接:</span>
            <span style="font-weight: bold; color: #18a058">{{
              systemStats.count
            }}</span>
          </n-flex>
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">系统上行:</span>
            <n-flex align="center" size="small" :wrap="false">
              <span style="font-weight: bold; color: #2080f0">{{
                formatRate(systemStats.egressBps)
              }}</span>
              <span style="font-size: 11px; color: #aaa"
                >({{ formatPackets(systemStats.egressPps) }})</span
              >
            </n-flex>
          </n-flex>
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">系统下行:</span>
            <n-flex align="center" size="small" :wrap="false">
              <span style="font-weight: bold; color: #18a058">{{
                formatRate(systemStats.ingressBps)
              }}</span>
              <span style="font-size: 11px; color: #aaa"
                >({{ formatPackets(systemStats.ingressPps) }})</span
              >
            </n-flex>
          </n-flex>
        </n-flex>

        <n-flex align="center" size="large" v-else-if="globalStats">
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">历史连接总数:</span>
            <span style="font-weight: bold; color: #4fb233">{{
              globalStats.total_connect_count
            }}</span>
          </n-flex>
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">累计总上传:</span>
            <span style="font-weight: bold; color: #2080f0">{{
              formatSize(globalStats.total_egress_bytes)
            }}</span>
          </n-flex>
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">累计总下载:</span>
            <span style="font-weight: bold; color: #18a058">{{
              formatSize(globalStats.total_ingress_bytes)
            }}</span>
          </n-flex>
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">更新于:</span>
            <n-time
              :time="globalStats.last_calculate_time"
              format="yyyy-MM-dd HH:mm"
              style="color: #aaa"
            />
          </n-flex>
        </n-flex>
      </n-flex>
    </n-flex>

    <!-- 子视图切换 -->
    <LiveMetric v-if="viewMode === 'live'" />
    <HistoryMetric v-else />
  </n-flex>
</template>

<style scoped>
.pulse-dot {
  width: 8px;
  height: 8px;
  background-color: #00d2ff;
  border-radius: 50%;
  box-shadow: 0 0 0 0 rgba(0, 210, 255, 0.7);
  animation: pulse 1.5s infinite;
  margin-right: 4px;
}

@keyframes pulse {
  0% {
    transform: scale(0.95);
    box-shadow: 0 0 0 0 rgba(0, 210, 255, 0.7);
  }
  70% {
    transform: scale(1);
    box-shadow: 0 0 0 6px rgba(0, 210, 255, 0);
  }
  100% {
    transform: scale(0.95);
    box-shadow: 0 0 0 0 rgba(0, 210, 255, 0);
  }
}
</style>
