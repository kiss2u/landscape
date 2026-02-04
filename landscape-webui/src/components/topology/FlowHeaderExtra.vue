<script setup lang="ts">
import { useIfaceNodeStore } from "@/stores/iface_node";
import { Add, View, ViewOff, Locked, Unlocked } from "@vicons/carbon";
// import { CashOutline as CashIcon } from "@vicons/ionicons5";
import { ref } from "vue";

const show_create_dev = ref<boolean>(false);
const ifaceNodeStore = useIfaceNodeStore();
async function show_down_dev() {
  ifaceNodeStore.HIDE_DOWN(!ifaceNodeStore.hide_down_dev);
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
    <n-float-button @click="show_down_dev">
      <n-icon>
        <View v-if="ifaceNodeStore.hide_down_dev" />
        <ViewOff v-else />
      </n-icon>
    </n-float-button>
    <n-float-button @click="ifaceNodeStore.TOGGLE_VIEW_LOCK">
      <n-icon>
        <Locked v-if="ifaceNodeStore.view_locked" />
        <Unlocked v-else />
      </n-icon>
    </n-float-button>
    <!-- <n-float-button>
      <n-icon><cash-icon /></n-icon>
    </n-float-button>
    <n-float-button>
      <n-icon><cash-icon /></n-icon>
    </n-float-button>
    <n-float-button>
      <n-icon><cash-icon /></n-icon>
    </n-float-button> -->
    <BridgeCreateModal v-model:show="show_create_dev"></BridgeCreateModal>
  </n-float-button-group>
</template>
