<script setup lang="ts">
import { ref, computed, h } from "vue";
import { formatRate, formatPackets } from "@/lib/util";
import { useThemeVars } from "naive-ui";
import type { IpRealtimeStat } from "landscape-types/common/metric/connect";

const props = defineProps<{
  stats: IpRealtimeStat[];
  title: string;
  ipLabel: string;
}>();

const themeVars = useThemeVars();

const sortKey = ref<string>("egress_bps");
const sortOrder = ref<"asc" | "desc">("desc");

const columns = [
  {
    title: props.ipLabel,
    key: "ip",
    sorter: "default",
  },
  {
    title: "活跃连接",
    key: "active_conns",
    sorter: (a: IpRealtimeStat, b: IpRealtimeStat) =>
      a.stats.active_conns - b.stats.active_conns,
    render: (row: IpRealtimeStat) => row.stats.active_conns,
  },
  {
    title: "上传流速",
    key: "egress_bps",
    sorter: (a: IpRealtimeStat, b: IpRealtimeStat) =>
      a.stats.egress_bps - b.stats.egress_bps,
    render: (row: IpRealtimeStat) => {
      return h(
        "span",
        {
          style: { color: themeVars.value.infoColor, fontWeight: "bold" },
        },
        formatRate(row.stats.egress_bps),
      );
    },
  },
  {
    title: "下载流速",
    key: "ingress_bps",
    sorter: (a: IpRealtimeStat, b: IpRealtimeStat) =>
      a.stats.ingress_bps - b.stats.ingress_bps,
    render: (row: IpRealtimeStat) => {
      return h(
        "span",
        {
          style: { color: themeVars.value.successColor, fontWeight: "bold" },
        },
        formatRate(row.stats.ingress_bps),
      );
    },
  },
  {
    title: "上传 PPS",
    key: "egress_pps",
    sorter: (a: IpRealtimeStat, b: IpRealtimeStat) =>
      a.stats.egress_pps - b.stats.egress_pps,
    render: (row: IpRealtimeStat) => formatPackets(row.stats.egress_pps),
  },
  {
    title: "下载 PPS",
    key: "ingress_pps",
    sorter: (a: IpRealtimeStat, b: IpRealtimeStat) =>
      a.stats.ingress_pps - b.stats.ingress_pps,
    render: (row: IpRealtimeStat) => formatPackets(row.stats.ingress_pps),
  },
];

const handleSort = (sorter: any) => {
  if (sorter) {
    sortKey.value = sorter.columnKey;
    sortOrder.value = sorter.order === "ascend" ? "asc" : "desc";
  }
};

const processedData = computed(() => {
  const data = [...props.stats];
  return data.sort((a: any, b: any) => {
    let vA, vB;
    if (sortKey.value === "ip") {
      vA = a.ip;
      vB = b.ip;
    } else {
      vA = a.stats[sortKey.value];
      vB = b.stats[sortKey.value];
    }
    const result = vA > vB ? 1 : vA < vB ? -1 : 0;
    return sortOrder.value === "asc" ? result : -result;
  });
});
</script>

<template>
  <n-flex vertical style="flex: 1; overflow: hidden">
    <n-flex align="center" justify="space-between" style="margin-bottom: 12px">
      <n-h3 style="margin: 0">{{ title }}</n-h3>
      <n-text depth="3"> 共计 {{ stats.length }} 个节点 </n-text>
    </n-flex>

    <n-data-table
      remote
      size="small"
      :columns="columns"
      :data="processedData"
      :pagination="false"
      :max-height="'calc(100vh - 300px)'"
      @update:sorter="handleSort"
    />
  </n-flex>
</template>
