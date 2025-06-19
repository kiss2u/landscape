<script setup lang="ts">
import { ConnectKey } from "@/rust_bindings/common/metric/connect";

interface Props {
  connect_metrics: ConnectKey[];
}

const props = defineProps<Props>();
</script>

<template>
  <n-grid x-gap="12" y-gap="12" cols="2 400:2 600:5">
    <n-grid-item v-for="conn of connect_metrics" :key="conn">
      <ConnectCardInfo :conn="conn" />
    </n-grid-item>
  </n-grid>
</template>
