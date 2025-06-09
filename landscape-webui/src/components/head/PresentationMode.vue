<script setup lang="ts">
import { useFrontEndStore } from "@/stores/front_end_config";
import { IosEye, IosEyeOff } from "@vicons/ionicons4";

const frontEndStore = useFrontEndStore();
</script>
<template>
  <n-popover trigger="hover">
    <template #trigger>
      <n-button
        v-if="frontEndStore.presentation_mode"
        text
        style="font-size: 24px"
        @click="
          frontEndStore.presentation_mode = !frontEndStore.presentation_mode
        "
      >
        <n-icon>
          <IosEyeOff />
        </n-icon>
      </n-button>
      <n-button
        v-else
        text
        style="font-size: 24px"
        @click="
          frontEndStore.presentation_mode = !frontEndStore.presentation_mode
        "
      >
        <n-icon>
          <IosEye />
        </n-icon>
      </n-button>
    </template>
    <span>
      切换当前
      <n-button
        style="cursor: default; pointer-events: none"
        @click.stop="() => {}"
        text
        target="_blank"
        type="primary"
      >
        隐私模式
      </n-button>
      <br />
      将会隐藏大部分的 IP MAC 等敏感信息
    </span>
  </n-popover>
</template>
