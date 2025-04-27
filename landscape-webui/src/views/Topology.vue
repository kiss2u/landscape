<script setup lang="ts">
import NetFlow from "@/components/topology/NetFlow.vue";
</script>
<template>
  <n-flex style="flex: 1">
    <NetFlow
      style="flex: 1; height: 100%; min-width: 100%"
      :fit_padding="1.2"
    ></NetFlow>
  </n-flex>
</template>
