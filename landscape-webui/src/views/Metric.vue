<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useMetricStore } from "@/stores/status_metric";
import { ConnectFilter } from "@/lib/metric.rs";
import { formatRate, formatPackets } from "@/lib/util";
import { useThemeVars } from "naive-ui";

const metricStore = useMetricStore();
const themeVars = useThemeVars();

let timer: any = null;

onMounted(async () => {
  metricStore.SET_ENABLE(true);
  await metricStore.UPDATE_INFO();
  timer = setInterval(async () => {
    await metricStore.UPDATE_INFO();
  }, 5000);
});

onUnmounted(() => {
  metricStore.SET_ENABLE(false);
  if (timer) clearInterval(timer);
});

// 初始化过滤器
const filter = ref(new ConnectFilter());

// 协议类型选项
const protocolOptions = [
  { label: "全部", value: null },
  { label: "TCP", value: 6 },
  { label: "UDP", value: 17 },
  { label: "ICMP", value: 1 },
  { label: "ICMPv6", value: 58 },
];

// IP 类型选项
const ipTypeOptions = [
  { label: "全部", value: null },
  { label: "IPv4", value: 0 },
  { label: "IPv6", value: 1 },
];

// 重置过滤器
const resetFilter = () => {
  filter.value = new ConnectFilter();
};

// 排序状态
const sortKey = ref<"time" | "port" | "speed">("time");
const sortOrder = ref<"asc" | "desc">("desc");

const toggleSort = (key: "time" | "port" | "speed") => {
  if (sortKey.value === key) {
    sortOrder.value = sortOrder.value === "asc" ? "desc" : "asc";
  } else {
    sortKey.value = key;
    sortOrder.value = "desc";
  }
};

// 计算过滤及排序后的连接指标
const filteredConnectMetrics = computed(() => {
  if (!metricStore.firewall_info) return [];

  const filtered = metricStore.firewall_info.filter((item) => {
    const key = item.key;

    // 源IP过滤
    if (filter.value.src_ip && !key.src_ip.includes(filter.value.src_ip)) {
      return false;
    }

    // 目标IP过滤
    if (filter.value.dst_ip && !key.dst_ip.includes(filter.value.dst_ip)) {
      return false;
    }

    // 源端口精准匹配
    if (
      filter.value.port_start !== null &&
      key.src_port !== filter.value.port_start
    ) {
      return false;
    }

    // 目标端口精准匹配
    if (
      filter.value.port_end !== null &&
      key.dst_port !== filter.value.port_end
    ) {
      return false;
    }

    // L3协议过滤
    if (
      filter.value.l3_proto !== null &&
      key.l3_proto !== filter.value.l3_proto
    ) {
      return false;
    }

    // L4协议过滤
    if (
      filter.value.l4_proto !== null &&
      key.l4_proto !== filter.value.l4_proto
    ) {
      return false;
    }

    // Flow ID过滤
    if (filter.value.flow_id !== null && key.flow_id !== filter.value.flow_id) {
      return false;
    }

    return true;
  });

  // 排序逻辑
  return filtered.sort((a, b) => {
    let result = 0;
    if (sortKey.value === "time") {
      result = (a.key.create_time || 0) - (b.key.create_time || 0);
    } else if (sortKey.value === "port") {
      result = (a.key.src_port || 0) - (b.key.src_port || 0);
    } else if (sortKey.value === "speed") {
      const speedA = (a.ingress_bps || 0) + (a.egress_bps || 0);
      const speedB = (b.ingress_bps || 0) + (b.egress_bps || 0);
      result = speedA - speedB;
    }

    return sortOrder.value === "asc" ? result : -result;
  });
});

// 应用过滤器 (计算属性会自动更新，这里只是占位)
const applyFilter = () => {};

// 系统全局汇总 (未过滤)
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

// 过滤后的数据汇总
const totalStats = computed(() => {
  const stats = {
    ingressBps: 0,
    egressBps: 0,
    ingressPps: 0,
    egressPps: 0,
    count: 0,
  };

  if (filteredConnectMetrics.value) {
    filteredConnectMetrics.value.forEach((item) => {
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
    <n-flex align="center" justify="space-between" style="padding: 4px 0">
      <n-flex align="center" size="large">
        <n-flex align="center" size="small">
          <span style="font-weight: bold; font-size: 18px">实时连接看板</span>
          <n-tag :bordered="false" type="info" size="small" round>
            <template #icon>
              <div class="pulse-dot"></div>
            </template>
            5s 采样中
          </n-tag>
        </n-flex>
        <n-divider vertical />
        <n-flex align="center" size="large">
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">连接:</span>
            <span style="font-weight: bold; color: #18a058">{{ systemStats.count }}</span>
          </n-flex>
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">总上行:</span>
            <n-flex align="center" size="small" :wrap="false">
              <span style="font-weight: bold; color: #2080f0">{{ formatRate(systemStats.egressBps) }}</span>
              <span style="font-size: 11px; color: #aaa">({{ formatPackets(systemStats.egressPps) }})</span>
            </n-flex>
          </n-flex>
          <n-flex align="center" size="small">
            <span style="color: #888; font-size: 13px">总下行:</span>
            <n-flex align="center" size="small" :wrap="false">
              <span style="font-weight: bold; color: #18a058">{{ formatRate(systemStats.ingressBps) }}</span>
              <span style="font-size: 11px; color: #aaa">({{ formatPackets(systemStats.ingressPps) }})</span>
            </n-flex>
          </n-flex>
        </n-flex>
      </n-flex>
    </n-flex>

    <n-flex align="center" :wrap="true" style="margin-bottom: 8px">
      <!-- 源 IP -->
      <n-input
        v-model:value="filter.src_ip"
        placeholder="源IP"
        clearable
        style="width: 180px"
      />

      <!-- 目标 IP -->
      <n-input
        v-model:value="filter.dst_ip"
        placeholder="目标IP"
        clearable
        style="width: 180px"
      />

      <n-input-group style="width: 240px">
        <n-input-number
          v-model:value="filter.port_start"
          placeholder="源端口"
          :show-button="false"
          :min="1"
          :max="65535"
          clearable
        />
        <n-input-group-label>=></n-input-group-label>
        <n-input-number
          v-model:value="filter.port_end"
          placeholder="目标端口"
          :show-button="false"
          :min="1"
          :max="65535"
          clearable
        />
      </n-input-group>

      <!-- L4 协议类型 -->
      <n-select
        v-model:value="filter.l4_proto"
        placeholder="传输协议"
        :options="protocolOptions"
        clearable
        style="width: 140px"
      />

      <!-- L3 协议类型 -->
      <n-select
        v-model:value="filter.l3_proto"
        placeholder="IP协议"
        :options="ipTypeOptions"
        clearable
        style="width: 120px"
      />
      <!-- flow id -->
      <n-input-number
        v-model:value="filter.flow_id"
        placeholder="Flow ID"
        :min="1"
        :max="255"
        clearable
        style="width: 120px"
      />

      <n-button @click="applyFilter" type="primary">过滤</n-button>
      <n-button @click="resetFilter">重置</n-button>

      <n-divider vertical />

      <n-button-group>
        <n-button
          :type="sortKey === 'time' ? 'primary' : 'default'"
          @click="toggleSort('time')"
        >
          时间 {{ sortKey === 'time' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
        </n-button>
        <n-button
          :type="sortKey === 'port' ? 'primary' : 'default'"
          @click="toggleSort('port')"
        >
          端口 {{ sortKey === 'port' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
        </n-button>
        <n-button
          :type="sortKey === 'speed' ? 'primary' : 'default'"
          @click="toggleSort('speed')"
        >
          速度 {{ sortKey === 'speed' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
        </n-button>
      </n-button-group>
    </n-flex>

    <n-grid x-gap="12" :cols="5">
      <n-gi>
        <n-card size="small" :bordered="false" style="background-color: #f9f9f910">
          <n-statistic label="过滤后连接数" :value="totalStats.count" />
        </n-card>
      </n-gi>
      <n-gi>
        <n-card size="small" :bordered="false" style="background-color: #f9f9f910">
          <n-statistic label="过滤总上行">
            <span :style="{ color: themeVars.infoColor, fontWeight: 'bold' }">
              {{ formatRate(totalStats.egressBps) }}
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card size="small" :bordered="false" style="background-color: #f9f9f910">
          <n-statistic label="过滤总下行">
            <span :style="{ color: themeVars.successColor, fontWeight: 'bold' }">
              {{ formatRate(totalStats.ingressBps) }}
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card size="small" :bordered="false" style="background-color: #f9f9f910">
          <n-statistic label="过滤出站 PPS">
            <span style="color: #888">
              {{ formatPackets(totalStats.egressPps) }}
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card size="small" :bordered="false" style="background-color: #f9f9f910">
          <n-statistic label="过滤入站 PPS">
            <span style="color: #888">
              {{ formatPackets(totalStats.ingressPps) }}
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
    </n-grid>

    <ConnectVirtualList
      v-if="filteredConnectMetrics"
      :connect_metrics="filteredConnectMetrics"
    />
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
