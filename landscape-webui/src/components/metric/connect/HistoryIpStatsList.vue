<script setup lang="ts">
import { h, computed } from "vue";
import { formatSize, formatCount } from "@/lib/util";
import { useThemeVars, NTooltip, NIcon, NButton } from "naive-ui";
import { Search } from "@vicons/carbon";
import type {
  IpHistoryStat,
  ConnectSortKey,
} from "landscape-types/common/metric/connect";

import FlowExhibit from "@/components/flow/FlowExhibit.vue";

const props = defineProps<{
  stats: IpHistoryStat[];
  title: string;
  ipLabel: string;
  sortKey: string;
  sortOrder: "asc" | "desc";
}>();

const emit = defineEmits(["update:sort", "search:ip"]);

const themeVars = useThemeVars();

// 使用 computed 确保当 props.sortKey 或 props.sortOrder 改变时，列定义会更新
const columns = computed(() => [
  {
    title: props.ipLabel,
    key: "ip",
    render: (row: IpHistoryStat) => {
      return h(
        "div",
        { style: { display: "flex", alignItems: "center", gap: "12px" } },
        [
          h("span", { style: { fontWeight: "500" } }, row.ip),
          h(
            NTooltip,
            { trigger: "hover", placement: "right" },
            {
              trigger: () =>
                h(
                  NButton,
                  {
                    text: true,
                    style: {
                      fontSize: "16px",
                      color: themeVars.value.infoColor,
                      opacity: 0.6,
                      display: "flex",
                      transition: "opacity 0.2s",
                    },
                    // 鼠标悬浮时提高不透明度
                    onMouseenter: (e: MouseEvent) => {
                      (e.currentTarget as HTMLElement).style.opacity = "1";
                    },
                    onMouseleave: (e: MouseEvent) => {
                      (e.currentTarget as HTMLElement).style.opacity = "0.6";
                    },
                    onClick: () => emit("search:ip", row.ip),
                  },
                  {
                    icon: () => h(NIcon, { component: Search }),
                  },
                ),
              default: () => "将此 IP 填入搜索框",
            },
          ),
        ],
      );
    },
  },
  {
    title: "所属 Flow",
    key: "flow_id",
    render: (row: IpHistoryStat) => {
      if (row.flow_id === 0) {
        return h(
          "n-tag",
          {
            type: "info",
            bordered: false,
            size: "small",
            style: { opacity: 0.6 },
          },
          { default: () => "默认Flow" },
        );
      }
      return h(FlowExhibit, {
        flow_id: row.flow_id,
      });
    },
  },
  {
    title: "累计连接",
    key: "time",
    sorter: true,
    sortOrder:
      props.sortKey === "time"
        ? props.sortOrder === "asc"
          ? "ascend"
          : "descend"
        : false,
    render: (row: IpHistoryStat) => row.connect_count,
  },
  {
    title: "累计上传",
    key: "egress",
    sorter: true,
    sortOrder:
      props.sortKey === "egress"
        ? props.sortOrder === "asc"
          ? "ascend"
          : "descend"
        : false,
    render: (row: IpHistoryStat) => {
      return h(
        "span",
        {
          style: { color: themeVars.value.infoColor, fontWeight: "bold" },
        },
        formatSize(row.total_egress_bytes),
      );
    },
  },
  {
    title: "累计下载",
    key: "ingress",
    sorter: true,
    sortOrder:
      props.sortKey === "ingress"
        ? props.sortOrder === "asc"
          ? "ascend"
          : "descend"
        : false,
    render: (row: IpHistoryStat) => {
      return h(
        "span",
        {
          style: { color: themeVars.value.successColor, fontWeight: "bold" },
        },
        formatSize(row.total_ingress_bytes),
      );
    },
  },
  {
    title: "上传数据包",
    key: "total_egress_pkts",
    render: (row: IpHistoryStat) => formatCount(row.total_egress_pkts),
  },
  {
    title: "下载数据包",
    key: "total_ingress_pkts",
    render: (row: IpHistoryStat) => formatCount(row.total_ingress_pkts),
  },
]);

const handleSort = (sorter: any) => {
  if (sorter && sorter.order) {
    const key = sorter.columnKey as ConnectSortKey;
    const order = sorter.order === "ascend" ? "asc" : "desc";
    emit("update:sort", { key, order });
  } else {
    // 如果点击取消排序，我们默认回到按上传流量倒序
    emit("update:sort", { key: "egress", order: "desc" });
  }
};
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
      :data="stats"
      :pagination="false"
      :max-height="'calc(100vh - 350px)'"
      @update:sorter="handleSort"
    />
  </n-flex>
</template>
