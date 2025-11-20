<script setup lang="ts">
import { ConnectKey } from "@/rust_bindings/common/metric/connect";
import { useFrontEndStore } from "@/stores/front_end_config";
import { ChartLine } from "@vicons/carbon";
import { mask_string } from "@/lib/common";

const frontEndStore = useFrontEndStore();

interface Props {
  conn: ConnectKey;
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
  <n-flex class="box" style="margin: 5px 0px">
    <n-card size="small">
      <n-flex align="center" justify="space-between">
        <n-flex>
          <n-flex align="center"><n-time :time="conn.create_time" /></n-flex>

          <n-flex>
            <n-tag type="success" :bordered="false">
              {{ conn.l3_proto == 0 ? "IPV4" : "IPV6" }}
            </n-tag>
            <n-tag type="info" :bordered="false">
              {{ l4_proto(conn.l4_proto) }}
            </n-tag>

            <n-tag v-if="conn.flow_id != 0" type="info" :bordered="false">
              FLOW: {{ conn.flow_id }}
            </n-tag>
          </n-flex>

          <n-flex align="center">
            {{
              `${
                frontEndStore.presentation_mode
                  ? mask_string(conn.src_ip)
                  : conn.src_ip
              }:${conn.src_port} => ${
                frontEndStore.presentation_mode
                  ? mask_string(conn.dst_ip)
                  : conn.dst_ip
              }:${conn.dst_port}`
            }}
          </n-flex>
        </n-flex>

        <n-flex>
          <n-button
            :focusable="false"
            text
            style="font-size: 16px"
            @click="emit('show:key', conn)"
          >
            <n-icon>
              <ChartLine />
            </n-icon>
          </n-button>
        </n-flex>
      </n-flex>
    </n-card>
  </n-flex>
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
