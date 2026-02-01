<script setup lang="ts">
import { ref, computed, reactive } from "vue";
import { useMetricStore } from "@/stores/status_metric";
import { ConnectFilter } from "@/lib/metric.rs";
import { formatRate, formatPackets } from "@/lib/util";
import { useThemeVars } from "naive-ui";
import ConnectVirtualList from "@/components/metric/connect/ConnectVirtualList.vue";

const metricStore = useMetricStore();
const themeVars = useThemeVars();

// 实时过滤器状态
const liveFilter = reactive(new ConnectFilter());

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

// 排序状态
const sortKey = ref<"time" | "port" | "ingress" | "egress">("time");
const sortOrder = ref<"asc" | "desc">("desc");

const resetLiveFilter = () => {
  Object.assign(liveFilter, new ConnectFilter());
};

const toggleSort = (key: "time" | "port" | "ingress" | "egress") => {
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
    if (liveFilter.src_ip && !key.src_ip.includes(liveFilter.src_ip)) return false;
    if (liveFilter.dst_ip && !key.dst_ip.includes(liveFilter.dst_ip)) return false;
    if (liveFilter.port_start !== null && key.src_port !== liveFilter.port_start) return false;
    if (liveFilter.port_end !== null && key.dst_port !== liveFilter.port_end) return false;
    if (liveFilter.l3_proto !== null && key.l3_proto !== liveFilter.l3_proto) return false;
    if (liveFilter.l4_proto !== null && key.l4_proto !== liveFilter.l4_proto) return false;
    if (liveFilter.flow_id !== null && key.flow_id !== liveFilter.flow_id) return false;
    return true;
  });

  return filtered.sort((a, b) => {
    let result = 0;
    if (sortKey.value === "time") {
      result = (a.key.create_time || 0) - (b.key.create_time || 0);
    } else if (sortKey.value === "port") {
      result = (a.key.src_port || 0) - (b.key.src_port || 0);
    } else if (sortKey.value === "ingress") {
      result = (a.ingress_bps || 0) - (b.ingress_bps || 0);
    } else if (sortKey.value === "egress") {
      result = (a.egress_bps || 0) - (b.egress_bps || 0);
    }
    return sortOrder.value === "asc" ? result : -result;
  });
});

// 过滤后的数据汇总
const totalStats = computed(() => {
  const stats = { ingressBps: 0, egressBps: 0, ingressPps: 0, egressPps: 0, count: 0 };
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
  <div style="display: contents">
    <!-- 实时模式专用工具栏 -->
    <n-flex align="center" :wrap="true" style="margin-bottom: 12px">
      <n-input v-model:value="liveFilter.src_ip" placeholder="源IP" clearable style="width: 170px" />
      <n-input v-model:value="liveFilter.dst_ip" placeholder="目标IP" clearable style="width: 170px" />
      <n-input-group style="width: 220px">
        <n-input-number v-model:value="liveFilter.port_start" placeholder="源端口" :show-button="false" clearable />
        <n-input-group-label>=></n-input-group-label>
        <n-input-number v-model:value="liveFilter.port_end" placeholder="目的" :show-button="false" clearable />
      </n-input-group>
      <n-select v-model:value="liveFilter.l4_proto" placeholder="传输协议" :options="protocolOptions" clearable style="width: 130px" />
      <n-select v-model:value="liveFilter.l3_proto" placeholder="IP类型" :options="ipTypeOptions" clearable style="width: 110px" />
      <n-input-number v-model:value="liveFilter.flow_id" placeholder="Flow" :min="1" :max="255" clearable style="width: 100px" />
      
      <n-button-group>
        <n-button @click="metricStore.UPDATE_INFO()" type="primary">刷新采样</n-button>
        <n-button @click="resetLiveFilter">重置</n-button>
      </n-button-group>

      <n-divider vertical />

      <n-button-group>
        <n-button :type="sortKey === 'time' ? 'primary' : 'default'" @click="toggleSort('time')">
          创建时间 {{ sortKey === 'time' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
        </n-button>
        <n-button :type="sortKey === 'port' ? 'primary' : 'default'" @click="toggleSort('port')">
          端口 {{ sortKey === 'port' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
        </n-button>
        <n-button :type="sortKey === 'ingress' ? 'primary' : 'default'" @click="toggleSort('ingress')">
          下载流速 {{ sortKey === 'ingress' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
        </n-button>
        <n-button :type="sortKey === 'egress' ? 'primary' : 'default'" @click="toggleSort('egress')">
          上传流速 {{ sortKey === 'egress' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
        </n-button>
      </n-button-group>
    </n-flex>

    <n-grid x-gap="12" :cols="5" style="margin-bottom: 12px">
      <n-gi>
        <n-card size="small" :bordered="false" style="background-color: #f9f9f910">
          <n-statistic label="过滤活跃连接" :value="totalStats.count" />
        </n-card>
      </n-gi>
      <n-gi>
        <n-card size="small" :bordered="false" style="background-color: #f9f9f910">
          <n-statistic label="实时总上行">
            <span :style="{ color: themeVars.infoColor, fontWeight: 'bold' }">
              {{ formatRate(totalStats.egressBps) }}
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card size="small" :bordered="false" style="background-color: #f9f9f910">
          <n-statistic label="实时总下行">
            <span :style="{ color: themeVars.successColor, fontWeight: 'bold' }">
              {{ formatRate(totalStats.ingressBps) }}
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card size="small" :bordered="false" style="background-color: #f9f9f910">
          <n-statistic label="入站流量 PPS">
            <span style="color: #888">
              {{ formatPackets(totalStats.ingressPps) }}
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card size="small" :bordered="false" style="background-color: #f9f9f910">
          <n-statistic label="出站流量 PPS">
            <span style="color: #888">
              {{ formatPackets(totalStats.egressPps) }}
            </span>
          </n-statistic>
        </n-card>
      </n-gi>
    </n-grid>

    <ConnectVirtualList
      v-if="filteredConnectMetrics"
      :connect_metrics="filteredConnectMetrics"
    />
  </div>
</template>
