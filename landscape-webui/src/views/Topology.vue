<script setup lang="ts">
import LandscapeTopology from "@/components/topology_v2/LandscapeTopology.vue";
</script>
<template>
  <n-flex style="flex: 1">
    <LandscapeTopology
      style="flex: 1; height: 100%; min-width: 100%"
      :fit_padding="1.2"
    ></LandscapeTopology>
  </n-flex>
</template>
