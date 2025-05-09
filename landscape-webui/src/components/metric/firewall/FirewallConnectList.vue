<script setup lang="ts">
import { FrontEndFirewallConnectData } from "@/rust_bindings/common/metric";

interface Props {
  connect_metrics: FrontEndFirewallConnectData[];
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

function get_time(time: number): number {
  return time / 1_000_000;
}
</script>

<template>
  <!-- <n-flex>
    <n-card v-for="conn of connect_metrics" size="small">
      <n-flex align="center">
        <n-time :time="get_time(conn.key.create_time)" />
        {{
          `${conn.key.src_ip}:${conn.key.src_port} => ${conn.key.dst_ip}:${conn.key.dst_port}`
        }}
        <n-tag type="success" :bordered="false">
          {{ conn.key.l3_proto == 0 ? "IPV4" : "IPV6" }}
        </n-tag>

        <n-tag type="info" :bordered="false">
          {{ l4_proto(conn.key.l4_proto) }}
        </n-tag>
      </n-flex>
    </n-card>
  </n-flex> -->
  <n-collapse :accordion="true">
    <n-collapse-item
      v-for="conn of connect_metrics"
      :name="conn.key.create_time"
      :key="conn.key.create_time"
    >
      <template #header>
        <!-- {{ conn }} -->
        <n-flex align="center">
          <n-time :time="get_time(conn.key.create_time)" />

          <n-tag :bordered="false">
            {{ `${conn.key.src_ip}:${conn.key.src_port}` }}
          </n-tag>
          =>
          <n-tag :bordered="false">
            {{ `${conn.key.dst_ip}:${conn.key.dst_port}` }}
          </n-tag>

          <!-- {{
            `${conn.key.src_ip}:${conn.key.src_port} => ${conn.key.dst_ip}:${conn.key.dst_port}`
          }} -->
          <n-tag type="success" :bordered="false">
            {{ conn.key.l3_proto == 0 ? "IPV4" : "IPV6" }}
          </n-tag>

          <n-tag type="info" :bordered="false">
            {{ l4_proto(conn.key.l4_proto) }}
          </n-tag>
          <n-tag type="warning" v-if="conn.key.flow_id !== 0" :bordered="false">
            Flow: {{ conn.key.flow_id }}
          </n-tag>
        </n-flex>
      </template>
      <FirewallConnectListItem :chart="conn.value" />
    </n-collapse-item>
  </n-collapse>
</template>
