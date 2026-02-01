<script setup lang="ts">
import { ref } from "vue";
import {
  ConnectKey,
  ConnectRealtimeStatus,
} from "landscape-types/common/metric/connect";

interface Props {
  connect_metrics: ConnectRealtimeStatus[];
}
const props = defineProps<Props>();

const show_chart = ref(false);
const show_chart_key = ref<ConnectKey | null>(null);
async function show_chart_drawer(key: ConnectKey) {
  show_chart_key.value = key;
  show_chart.value = true;
}
</script>

<template>
  <n-virtual-list class="list" :item-size="40" :items="props.connect_metrics">
    <template #default="{ item, index }">
      <ConnectItemInfo
        @show:key="show_chart_drawer"
        :conn="item"
        :index="index"
      />
    </template>
  </n-virtual-list>

  <ConnectChartDrawer v-model:show="show_chart" :conn="show_chart_key" />
</template>

<style scoped>
.list {
  height: 100%;
}
</style>
