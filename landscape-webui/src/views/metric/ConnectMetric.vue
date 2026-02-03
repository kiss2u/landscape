<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useMetricStore } from "@/stores/status_metric";
import { formatRate, formatPackets, formatSize } from "@/lib/util";
import { get_connect_global_stats } from "@/api/metric";
import type { ConnectGlobalStats } from "landscape-types/common/metric/connect";
import { useThemeVars } from "naive-ui";
import { Renew } from "@vicons/carbon";
import { usePreferenceStore } from "@/stores/preference";

const prefStore = usePreferenceStore();
const metricStore = useMetricStore();
const themeVars = useThemeVars();
const route = useRoute();
const router = useRouter();

// 路由与视图模式映射
const viewMode = computed({
  get: () => {
    const lastPart = route.path.split("/").pop();
    return lastPart || "live";
  },
  set: (val) => {
    router.push(`/metric/conn/${val}`);
  },
});

const globalStats = ref<ConnectGlobalStats | null>(null);
let timer: any = null;

onMounted(async () => {
  metricStore.SET_ENABLE(true);
  await metricStore.UPDATE_INFO();

  // 获取一次历史全量统计
  refreshGlobalStats();

  timer = setInterval(async () => {
    // 只有在实时相关的页面才自动刷新 store
    if (["live", "src", "dst"].includes(viewMode.value)) {
      await metricStore.UPDATE_INFO();
    }
  }, 5000);
});

onUnmounted(() => {
  metricStore.SET_ENABLE(false);
  if (timer) clearInterval(timer);
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

const refreshingStats = ref(false);
async function refreshGlobalStats() {
  if (refreshingStats.value) return;
  refreshingStats.value = true;

  try {
    const [stats] = await Promise.all([
      get_connect_global_stats(),
      new Promise((resolve) => setTimeout(resolve, 500)),
    ]);
    globalStats.value = stats;
  } catch (e) {
    console.error(e);
  } finally {
    refreshingStats.value = false;
  }
}
</script>

<template>
  <n-flex style="flex: 1; overflow: hidden; margin-bottom: 10px" vertical>
    <!-- 全局页头：包含切换器和系统级实时统计 -->
    <n-flex align="center" justify="space-between" style="padding: 4px 0">
      <n-flex align="center" size="large" :wrap="true">
        <n-radio-group v-model:value="viewMode" size="medium" type="button">
          <n-radio-button value="live">活跃连接</n-radio-button>
          <n-radio-button value="src">源IP统计</n-radio-button>
          <n-radio-button value="dst">目的IP统计</n-radio-button>
          <n-radio-button value="history">历史查询</n-radio-button>
        </n-radio-group>

        <n-tag
          v-if="['live', 'src', 'dst'].includes(viewMode)"
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

        <!-- 动态面板 -->
        <n-flex align="center" size="large" v-if="viewMode !== 'history'">
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">总活跃连接:</span>
            <span style="font-weight: bold">{{ systemStats.count }}</span>
          </n-flex>
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">总上行:</span>
            <n-flex align="center" size="small" :wrap="false">
              <span
                :style="{ fontWeight: 'bold', color: themeVars.infoColor }"
                >{{ formatRate(systemStats.egressBps) }}</span
              >
            </n-flex>
          </n-flex>
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">总下行:</span>
            <n-flex align="center" size="small" :wrap="false">
              <span
                :style="{ fontWeight: 'bold', color: themeVars.successColor }"
                >{{ formatRate(systemStats.ingressBps) }}</span
              >
            </n-flex>
          </n-flex>
        </n-flex>

        <n-flex align="center" size="large" v-else-if="globalStats">
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">历史连接总数:</span>
            <span style="font-weight: bold">{{
              globalStats.total_connect_count
            }}</span>
          </n-flex>
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">累计总上传:</span>
            <span :style="{ fontWeight: 'bold', color: themeVars.infoColor }">{{
              formatSize(globalStats.total_egress_bytes)
            }}</span>
          </n-flex>
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">累计总下载:</span>
            <span
              :style="{ fontWeight: 'bold', color: themeVars.successColor }"
              >{{ formatSize(globalStats.total_ingress_bytes) }}</span
            >
          </n-flex>
          <n-button
            quaternary
            circle
            size="tiny"
            @click="refreshGlobalStats"
            :loading="refreshingStats"
            style="margin-left: 4px"
          >
            <template #icon>
              <n-icon><Renew /></n-icon>
            </template>
          </n-button>
        </n-flex>
      </n-flex>
    </n-flex>

    <!-- 子路由内容 -->
    <router-view v-slot="{ Component }">
      <keep-alive>
        <component :is="Component" />
      </keep-alive>
    </router-view>
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
