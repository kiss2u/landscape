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
  <n-card size="small">
    <template #header>
      <n-time :time="conn.create_time" />
    </template>

    <template #header-extra>
      <n-button text style="font-size: 16px" @click="emit('show:key', conn)">
        <n-icon>
          <ChartLine />
        </n-icon>
      </n-button>
    </template>

    {{
      `${
        frontEndStore.presentation_mode ? mask_string(conn.src_ip) : conn.src_ip
      }:${conn.src_port} => ${
        frontEndStore.presentation_mode ? mask_string(conn.dst_ip) : conn.dst_ip
      }:${conn.dst_port}`
    }}

    <template #action>
      <n-flex>
        <n-tag type="success" :bordered="false">
          {{ conn.l3_proto == 0 ? "IPV4" : "IPV6" }}
        </n-tag>
        <n-tag type="info" :bordered="false">
          {{ l4_proto(conn.l4_proto) }}
        </n-tag>
      </n-flex>
    </template>
  </n-card>
</template>
