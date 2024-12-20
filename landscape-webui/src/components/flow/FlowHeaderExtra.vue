<script setup lang="ts">
import { Add } from "@vicons/carbon";
// import { CashOutline as CashIcon } from "@vicons/ionicons5";
import { ref } from "vue";

const show_create_dev = ref<boolean>(false);
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
