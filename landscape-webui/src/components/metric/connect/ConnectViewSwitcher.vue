<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";

const route = useRoute();
const router = useRouter();

const viewMode = computed({
  get: () => {
    const lastPart = route.path.split("/").pop();
    return lastPart || "live";
  },
  set: (val) => {
    router.push(`/metric/conn/${val}`);
  },
});
</script>

<template>
  <n-flex align="center" :wrap="false">
    <n-tabs
      v-model:value="viewMode"
      type="segment"
      size="small"
      style="min-width: 600px"
    >
      <n-tab name="live">活跃连接</n-tab>
      <n-tab name="src">源IP实时</n-tab>
      <n-tab name="dst">目的IP实时</n-tab>
      <n-tab name="history">连接历史</n-tab>
      <n-tab name="history-src">源IP历史</n-tab>
      <n-tab name="history-dst">目的IP历史</n-tab>
    </n-tabs>

    <n-tag
      v-if="['live', 'src', 'dst'].includes(viewMode)"
      :bordered="false"
      type="info"
      size="small"
      round
    >
      <template #icon>
        <div class="pulse-dot"></div>
      </template>
      5s 采样
    </n-tag>
  </n-flex>
</template>

<style scoped>
.pulse-dot {
  width: 8px;
  height: 8px;
  background-color: #00d2ff;
  border-radius: 50%;
  box-shadow: 0 0 0 0 rgba(0, 210, 255, 0.7);
  animation: pulse 1.5s infinite;
  margin-right: 4px;
}

@keyframes pulse {
  0% {
    transform: scale(0.95);
    box-shadow: 0 0 0 0 rgba(0, 210, 255, 0.7);
  }
  70% {
    transform: scale(1);
    box-shadow: 0 0 0 6px rgba(0, 210, 255, 0);
  }
  100% {
    transform: scale(0.95);
    box-shadow: 0 0 0 0 rgba(0, 210, 255, 0);
  }
}
</style>
