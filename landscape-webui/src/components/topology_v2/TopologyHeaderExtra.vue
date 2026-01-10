<script setup lang="ts">
import { useTopologyStore } from "@/stores/topology";
import { Add, View, ViewOff } from "@vicons/carbon";
import { PageFit16Filled } from "@vicons/fluent";
import { Docker } from "@vicons/fa";
import HideDocker from "@/components/icon/HideDocker.vue";
import { useVueFlow } from "@vue-flow/core";
import { Refresh } from "@vicons/ionicons5";
// import { CashOutline as CashIcon } from "@vicons/ionicons5";
import { nextTick, ref } from "vue";

const show_create_dev = ref<boolean>(false);

const { fitView } = useVueFlow();

let ifaceNodeStore = useTopologyStore();

// 重绘拓扑（清除缓存）
async function redrawTopology() {
  ifaceNodeStore.clear_position_cache();
  await ifaceNodeStore.UPDATE_INFO();
  window.$message.success('拓扑已重新绘制');
  setTimeout(() => {
    fitView({ padding: 0.3 });
  }, 100);
}
async function show_down_dev() {
  ifaceNodeStore.UPDATE_HIDE(!ifaceNodeStore.hide_down_dev);
  await ifaceNodeStore.UPDATE_INFO();
  await nextTick();
  await fit_view();
}

async function show_docker_dev() {
  ifaceNodeStore.UPDATE_DOCKER_HIDE(!ifaceNodeStore.hide_docker_dev);
  await ifaceNodeStore.UPDATE_INFO();
  await fit_view();
}

async function fit_view() {
  await fitView({ padding: 0.3 });
}
</script>

<template>
  <n-float-button-group
    style="z-index: 5"
    shape="square"
    :right="20"
    :top="20"
    position="absolute"
  >
    <n-float-button @click="show_create_dev = true">
      <n-icon><Add /></n-icon>
    </n-float-button>
    <n-float-button @click="fit_view">
      <n-icon><PageFit16Filled /></n-icon>
    </n-float-button>
    <n-float-button @click="show_docker_dev">
      <n-icon>
        <HideDocker v-if="ifaceNodeStore.hide_docker_dev" />
        <Docker v-else />
      </n-icon>
    </n-float-button>
    <n-float-button @click="show_down_dev">
      <n-icon>
        <ViewOff v-if="ifaceNodeStore.hide_down_dev" />
        <View v-else />
      </n-icon>
    </n-float-button>
    <n-float-button @click="redrawTopology">
      <n-icon><Refresh /></n-icon>
    </n-float-button>
    <!--<n-float-button>
      <n-icon><cash-icon /></n-icon>
    </n-float-button>
    <n-float-button>
      <n-icon><cash-icon /></n-icon>
    </n-float-button> -->
    <BridgeCreateModal v-model:show="show_create_dev"></BridgeCreateModal>
  </n-float-button-group>
</template>
