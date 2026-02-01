<script setup lang="ts">
import { ref, computed, reactive, onMounted, watch } from "vue";
import { ConnectFilter } from "@/lib/metric.rs";
import { get_connect_history } from "@/api/metric";
import { formatSize, formatCount } from "@/lib/util";
import { useThemeVars } from "naive-ui";
import HistoryItemInfo from "@/components/metric/connect/HistoryItemInfo.vue";
import { ConnectKey } from "landscape-types/common/metric/connect";

const themeVars = useThemeVars();

// 1. 声明所有基础响应式状态 (State)
const historicalData = ref<any[]>([]);
const timeRange = ref<number | null>(300); // 默认 5 分钟 (300秒)
const queryLimit = ref<number | null>(100); // 默认限制 100 条
const historyFilter = reactive(new ConnectFilter());
const sortKey = ref<"time" | "port" | "ingress" | "egress">("time");
const sortOrder = ref<"asc" | "desc">("desc");

// 图表展示状态
const showChart = ref(false);
const showChartKey = ref<ConnectKey | null>(null);
const loading = ref(false);

const showChartDrawer = (key: ConnectKey) => {
  showChartKey.value = key;
  showChart.value = true;
};

// 2. 声明常量选项 (Options)
const protocolOptions = [
  { label: "全部", value: null },
  { label: "TCP", value: 6 },
  { label: "UDP", value: 17 },
  { label: "ICMP", value: 1 },
  { label: "ICMPv6", value: 58 },
];

const timeRangeOptions = [
  { label: "近 5 分钟", value: 300 },
  { label: "近 15 分钟", value: 900 },
  { label: "近 1 小时", value: 3600 },
  { label: "近 6 小时", value: 21600 },
  { label: "近 24 小时", value: 86400 },
  { label: "近 3 天", value: 259200 },
  { label: "不限时间", value: null },
];

const limitOptions = [
  { label: "限制 100 条", value: 100 },
  { label: "限制 500 条", value: 500 },
  { label: "限制 1000 条", value: 1000 },
  { label: "限制 5000 条", value: 5000 },
  { label: "不限制数量", value: null },
];

// 3. 声明数据获取逻辑 (Actions)
const fetchHistory = async () => {
  loading.value = true;
  try {
    let startTime: number | undefined;
    if (timeRange.value !== null) {
      startTime = Math.floor((Date.now() - timeRange.value * 1000) / 1000);
    }

    historicalData.value = await get_connect_history({
      start_time: startTime,
      limit: queryLimit.value || undefined,
      src_ip: historyFilter.src_ip || undefined,
      dst_ip: historyFilter.dst_ip || undefined,
      port_start: historyFilter.port_start || undefined,
      port_end: historyFilter.port_end || undefined,
      l3_proto: historyFilter.l3_proto || undefined,
      l4_proto: historyFilter.l4_proto || undefined,
      flow_id: historyFilter.flow_id || undefined,
      sort_key: sortKey.value,
      sort_order: sortOrder.value,
    });
  } finally {
    loading.value = false;
  }
};

const resetHistoryFilter = () => {
  Object.assign(historyFilter, new ConnectFilter());
  fetchHistory();
};

const toggleSort = (key: "time" | "port" | "ingress" | "egress") => {
  if (sortKey.value === key) {
    sortOrder.value = sortOrder.value === "asc" ? "desc" : "asc";
  } else {
    sortKey.value = key;
    sortOrder.value = "desc";
  }
};

// 4. 计算属性 (Computed)
const filteredHistory = computed(() => {
  return historicalData.value || [];
});

const historyTotalStats = computed(() => {
  const stats = {
    totalIngressBytes: 0,
    totalEgressBytes: 0,
    totalIngressPkts: 0,
    totalEgressPkts: 0,
    count: 0,
  };
  if (filteredHistory.value) {
    filteredHistory.value.forEach((item) => {
      stats.totalIngressBytes += item.total_ingress_bytes || 0;
      stats.totalEgressBytes += item.total_egress_bytes || 0;
      stats.totalIngressPkts += item.total_ingress_pkts || 0;
      stats.totalEgressPkts += item.total_egress_pkts || 0;
      stats.count++;
    });
  }
  return stats;
});

// 5. 监听器与生命周期 (Watchers & Lifecycle)
watch([timeRange, queryLimit, sortKey, sortOrder], () => {
  fetchHistory();
});

watch(
  historyFilter,
  () => {
    fetchHistory();
  },
  { deep: true },
);

onMounted(() => {
  fetchHistory();
});
</script>

<template>
  <n-flex vertical style="flex: 1; overflow: hidden">
    <!-- 历史模式专用工具栏 -->
    <n-flex align="center" :wrap="true" style="margin-bottom: 12px">
      <n-input
        v-model:value="historyFilter.src_ip"
        placeholder="源IP"
        clearable
        :disabled="loading"
        style="width: 170px"
      />
      <n-input
        v-model:value="historyFilter.dst_ip"
        placeholder="目标IP"
        clearable
        :disabled="loading"
        style="width: 170px"
      />
      <n-input-group style="width: 220px">
        <n-input-number
          v-model:value="historyFilter.port_start"
          placeholder="源端口"
          :show-button="false"
          :disabled="loading"
          clearable
        />
        <n-input-group-label>=></n-input-group-label>
        <n-input-number
          v-model:value="historyFilter.port_end"
          placeholder="目的"
          :show-button="false"
          :disabled="loading"
          clearable
        />
      </n-input-group>
      <n-select
        v-model:value="historyFilter.l4_proto"
        placeholder="传输协议"
        :options="protocolOptions"
        :disabled="loading"
        clearable
        style="width: 130px"
      />
      <n-input-number
        v-model:value="historyFilter.flow_id"
        placeholder="Flow"
        :min="1"
        :max="255"
        :disabled="loading"
        clearable
        style="width: 100px"
      />

      <n-divider vertical />

      <n-select
        v-model:value="timeRange"
        :options="timeRangeOptions"
        :disabled="loading"
        style="width: 140px"
      />
      <n-select
        v-model:value="queryLimit"
        :options="limitOptions"
        :disabled="loading"
        style="width: 140px"
      />

      <n-button-group>
        <n-button @click="fetchHistory" type="primary" :loading="loading">查询</n-button>
        <n-button @click="resetHistoryFilter" :disabled="loading">重置</n-button>
      </n-button-group>

      <n-divider vertical />

      <n-button-group>
        <n-button
          :type="sortKey === 'time' ? 'primary' : 'default'"
          :disabled="loading"
          @click="toggleSort('time')"
        >
          发起时间
          {{ sortKey === "time" ? (sortOrder === "asc" ? "↑" : "↓") : "" }}
        </n-button>
        <n-button
          :type="sortKey === 'port' ? 'primary' : 'default'"
          :disabled="loading"
          @click="toggleSort('port')"
        >
          端口 {{ sortKey === "port" ? (sortOrder === "asc" ? "↑" : "↓") : "" }}
        </n-button>
        <n-button
          :type="sortKey === 'egress' ? 'primary' : 'default'"
          :disabled="loading"
          @click="toggleSort('egress')"
        >
          上传量排序
          {{ sortKey === "egress" ? (sortOrder === "asc" ? "↑" : "↓") : "" }}
        </n-button>
        <n-button
          :type="sortKey === 'ingress' ? 'primary' : 'default'"
          :disabled="loading"
          @click="toggleSort('ingress')"
        >
          下载量排序
          {{ sortKey === "ingress" ? (sortOrder === "asc" ? "↑" : "↓") : "" }}
        </n-button>
      </n-button-group>
    </n-flex>

    <n-grid x-gap="12" :cols="5" style="margin-bottom: 12px">
      <n-gi>
        <n-card
          size="small"
          :bordered="false"
          style="background-color: #f9f9f910"
        >
          <n-statistic label="过滤结果总数" :value="historyTotalStats.count" />
        </n-card>
      </n-gi>
      <n-gi>
        <n-card
          size="small"
          :bordered="false"
          style="background-color: #f9f9f910"
        >
          <n-statistic label="过滤结果总上行">
            <span :style="{ color: themeVars.infoColor, fontWeight: 'bold' }">
              {{ formatSize(historyTotalStats.totalEgressBytes) }}
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card
          size="small"
          :bordered="false"
          style="background-color: #f9f9f910"
        >
          <n-statistic label="过滤结果总下行">
            <span
              :style="{ color: themeVars.successColor, fontWeight: 'bold' }"
            >
              {{ formatSize(historyTotalStats.totalIngressBytes) }}
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card
          size="small"
          :bordered="false"
          style="background-color: #f9f9f910"
        >
          <n-statistic label="过滤结果总入站">
            <span style="color: #888">
              {{ formatCount(historyTotalStats.totalIngressPkts) }} pkt
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card
          size="small"
          :bordered="false"
          style="background-color: #f9f9f910"
        >
          <n-statistic label="过滤结果总出站">
            <span style="color: #888">
              {{ formatCount(historyTotalStats.totalEgressPkts) }} pkt
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
    </n-grid>

    <n-virtual-list
      style="flex: 1"
      :item-size="40"
      :items="filteredHistory"
    >
      <template #default="{ item, index }">
        <HistoryItemInfo
          :history="item"
          :index="index"
          @show:key="showChartDrawer"
        />
      </template>
    </n-virtual-list>
    <ConnectChartDrawer v-model:show="showChart" :conn="showChartKey" mode="cumulative" />
  </n-flex>
</template>
