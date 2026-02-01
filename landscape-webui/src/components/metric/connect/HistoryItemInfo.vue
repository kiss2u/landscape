<script setup lang="ts">
import {
  ConnectKey,
  ConnectHistoryStatus,
} from "landscape-types/common/metric/connect";
import { useFrontEndStore } from "@/stores/front_end_config";
import { ChartLine, ArrowUp, ArrowDown, Time } from "@vicons/carbon";
import { mask_string } from "@/lib/common";
import { formatSize, formatCount } from "@/lib/util";
import { useThemeVars } from "naive-ui";

const frontEndStore = useFrontEndStore();
const themeVars = useThemeVars();

interface Props {
  history: ConnectHistoryStatus;
  index?: number;
}

const props = defineProps<Props>();

function l4_proto(value: number): string {
  if (value == 6) {
    return "TCP";
  } else if (value == 17) {
    return "UDP";
  } else if (value == 1) {
    return "ICMP";
  }
  return "Unknow";
}

const emit = defineEmits(["show:key"]);
</script>

<template>
  <div
    class="box"
    :style="{
      backgroundColor:
        (index ?? 0) % 2 === 1 ? themeVars.tableColor : 'transparent',
    }"
  >
    <n-card
      size="small"
      :bordered="false"
      style="background: transparent"
      content-style="padding: 4px 12px"
    >
      <n-flex align="center" justify="space-between">
        <n-flex align="center">
          <n-flex align="center" style="width: 160px">
            <n-flex vertical size="small">
              <n-time
                :time="history.key.create_time"
                format="yyyy-MM-dd HH:mm:ss"
              />
              <div style="font-size: 10px; color: #888">
                记录于 {{ new Date(history.key.create_time).getFullYear() }}
              </div>
            </n-flex>
          </n-flex>

          <n-flex style="width: 200px">
            <n-tag type="success" :bordered="false" size="small">
              {{ history.key.l3_proto == 0 ? "IPV4" : "IPV6" }}
            </n-tag>
            <n-tag type="info" :bordered="false" size="small">
              {{ l4_proto(history.key.l4_proto) }}
            </n-tag>
          </n-flex>

          <n-flex
            align="center"
            style="width: 800px; font-variant-numeric: tabular-nums"
          >
            {{
              `${
                frontEndStore.presentation_mode
                  ? mask_string(history.key.src_ip)
                  : history.key.src_ip
              }:${history.key.src_port} => ${
                frontEndStore.presentation_mode
                  ? mask_string(history.key.dst_ip)
                  : history.key.dst_ip
              }:${history.key.dst_port}`
            }}
          </n-flex>

          <!-- 累计总量展示 -->
          <n-flex align="center" :wrap="false" style="gap: 24px">
            <!-- 累计上行 -->
            <n-flex
              align="center"
              :wrap="false"
              size="small"
              style="width: 100px"
            >
              <n-icon :color="themeVars.infoColor" size="20">
                <ArrowUp />
              </n-icon>
              <n-flex vertical :size="[-4, 0]" style="flex: 1">
                <span
                  style="font-size: 13px; font-weight: 600; white-space: nowrap"
                >
                  {{ formatSize(history.total_egress_bytes) }}
                </span>
                <span style="font-size: 10px; color: #999; white-space: nowrap">
                  {{ formatCount(history.total_egress_pkts) }} pkt
                </span>
              </n-flex>
            </n-flex>

            <!-- 累计下行 -->
            <n-flex
              align="center"
              :wrap="false"
              size="small"
              style="width: 100px"
            >
              <n-icon :color="themeVars.successColor" size="20">
                <ArrowDown />
              </n-icon>
              <n-flex vertical :size="[-4, 0]" style="flex: 1">
                <span
                  style="font-size: 13px; font-weight: 600; white-space: nowrap"
                >
                  {{ formatSize(history.total_ingress_bytes) }}
                </span>
                <span style="font-size: 10px; color: #999; white-space: nowrap">
                  {{ formatCount(history.total_ingress_pkts) }} pkt
                </span>
              </n-flex>
            </n-flex>
          </n-flex>
        </n-flex>

        <!-- 右侧区域：操作按钮 -->
        <n-flex align="center" :wrap="false">
          <!-- 图表按钮 -->
          <n-button
            :focusable="false"
            text
            style="font-size: 16px"
            @click="emit('show:key', history.key)"
          >
            <n-icon>
              <ChartLine />
            </n-icon>
          </n-button>
        </n-flex>
      </n-flex>
    </n-card>
  </div>
</template>

<style scoped>
.box {
  border: 2px solid transparent;
  transition: border-color 0.25s ease;
}

.box:hover {
  border-color: #4fa3ff; /* 你想要的亮色 */
}
</style>
