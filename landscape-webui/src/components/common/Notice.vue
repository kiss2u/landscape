<script lang="ts" setup>
import { HelpFilled } from "@vicons/carbon";
</script>
<template>
  <n-flex :size="[5, 0]" justify="center" align="center">
    <slot></slot>
    <n-popover trigger="hover">
      <template #trigger>
        <n-button text>
          <template #icon>
            <n-icon><HelpFilled /></n-icon>
          </template>
        </n-button>
      </template>
      <slot name="msg"></slot>
    </n-popover>
  </n-flex>
</template>
