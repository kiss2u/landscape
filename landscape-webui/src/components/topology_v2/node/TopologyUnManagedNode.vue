<script setup lang="ts">
import { manage_iface } from "@/api/iface";
import { NetworkIfaceConfig } from "@/rust_bindings/iface";

const props = defineProps<{
  node: NetworkIfaceConfig;
}>();

async function manage_this_dev() {
  await manage_iface(props.node.iface_name);
}
</script>

<template>
  <n-card
    size="small"
    :title="node.iface_name"
    style="min-width: 220px; max-width: 230px"
  >
    <template #header-extra>
      <n-button @click="manage_this_dev" size="small" secondary strong>
        小
      </n-button>
    </template>
  </n-card>
</template>
