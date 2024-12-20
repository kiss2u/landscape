<script setup lang="ts">
import { useFetchIntervalStore } from "@/stores/fetch_interval";

import { SpinnerIos20Regular } from "@vicons/fluent";
// import { Spinner } from "@vicons/fa";

import { PauseFilled, Renew } from "@vicons/carbon";
const fetchIntervalStore = useFetchIntervalStore();
</script>

<template>
  <n-flex style="max-height: 20px; overflow: hidden">
    <n-icon
      @click="fetchIntervalStore.enable_interval = false"
      v-if="fetchIntervalStore.enable_interval"
      class="element"
      size="20"
    >
      <SpinnerIos20Regular />
    </n-icon>
    <n-icon @click="fetchIntervalStore.enable_interval = true" v-else size="20">
      <PauseFilled />
    </n-icon>
  </n-flex>
</template>

<style scoped>
.element {
  animation: rotateAnimation 5s linear infinite;
  /* animation: rotateAnimation 5s cubic-bezier(0.25, 0.1, 0.25, 1) infinite; */
}

@keyframes rotateAnimation {
  from {
    transform: rotate(0deg); /* 初始角度为 0 度 */
  }
  to {
    transform: rotate(360deg); /* 最终角度为 360 度 */
  }
}
</style>
