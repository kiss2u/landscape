<script setup lang="ts">
import { VueFlow, useVueFlow } from "@vue-flow/core";
import { MiniMap } from "@vue-flow/minimap";
// import { Controls } from '@vue-flow/controls'
import InteractionControls from "@/components/flow/InteractionControls.vue";
import FlowHeaderExtra from "@/components/flow/FlowHeaderExtra.vue";
import FlowNode from "@/components/flow/FlowNode.vue";
import { add_controller } from "@/api/network";

import {
  NButton,
  NInputGroup,
  NSelect,
  NInputGroupLabel,
  SelectOption,
} from "naive-ui";

import { ref } from "vue";
import { useIfaceNodeStore } from "@/stores/iface_node";

// const { onConnect } = useVueFlow();

let ifaceNodeStore = useIfaceNodeStore();

const controlelr_config = ref<any>({});

async function create_connection() {
  let result = await add_controller(controlelr_config.value);
  controlelr_config.value = {};
  if (result) {
    await ifaceNodeStore.UPDATE_INFO();
  }
}
// onConnect((connection) => {
//   console.log(connection);
// });
function handleMasterUpdate(value: string, option: SelectOption) {
  controlelr_config.value.master_ifindex = option.ifindex;
}
function handleIfaceUpdate(value: string, option: SelectOption) {
  controlelr_config.value.link_ifindex = option.ifindex;
}
</script>

<template>
  <n-input-group>
    <n-input-group-label>Bridge</n-input-group-label>
    <n-select
      :style="{ width: '50%' }"
      @update:value="handleMasterUpdate"
      v-model:value="controlelr_config.master_name"
      :options="ifaceNodeStore.bridges"
    />
    <n-input-group-label>eth</n-input-group-label>
    <n-select
      :style="{ width: '50%' }"
      @update:value="handleIfaceUpdate"
      v-model:value="controlelr_config.link_name"
      :options="ifaceNodeStore.eths"
    />
    <n-button type="primary" @click="create_connection" ghost> Add </n-button>
  </n-input-group>

  <!-- {{ net_devs }} -->
  <VueFlow
    :nodes="ifaceNodeStore.nodes"
    :edges="ifaceNodeStore.edges"
    style="min-height: 600px; min-width: 100%"
  >
    <!-- bind your custom node type to a component by using slots, slot names are always `node-<type>` -->
    <!-- <template #node-special="specialNodeProps">
        <SpecialNode v-bind="specialNodeProps" />
      </template> -->

    <!-- bind your custom edge type to a component by using slots, slot names are always `edge-<type>` -->
    <!-- <template #edge-special="specialEdgeProps">
        <SpecialEdge v-bind="specialEdgeProps" />
      </template> -->
    <MiniMap pannable zoomable />
    <!-- <Controls position="top-right">
        <n-button style="font-size: 16px; padding: 5px;" text >
          <n-icon>
            <cash-icon />
          </n-icon>
        </n-button>
    </Controls> -->
    <template #node-netflow="{ data }">
      <!-- {{ nodeProps }} -->
      <FlowNode :node="data" />
    </template>
    <!-- <InteractionControls /> -->

    <FlowHeaderExtra />
  </VueFlow>
</template>

<style>
/* import the necessary styles for Vue Flow to work */
@import "@vue-flow/core/dist/style.css";

/* import the default theme, this is optional but generally recommended */
@import "@vue-flow/core/dist/theme-default.css";

/* import default minimap styles */
@import "@vue-flow/minimap/dist/style.css";
/* @import '@vue-flow/controls/dist/style.css'; */
</style>
